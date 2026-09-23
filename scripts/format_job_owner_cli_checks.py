#!/usr/bin/env python3
"""Exercise format-job output ownership and command replacement in a live client."""

import fcntl
import os
import pathlib
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="format-job-owner-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)

    def wait_for_status(master, expected):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            readable, _, _ = select.select([master], [], [], 0.1)
            if readable:
                output.extend(os.read(master, 65536))
            if expected in output:
                return
        raise AssertionError((expected, bytes(output)))

    attached = None
    master = None
    try:
        run("new-session", "-d", "-s", "jobs", "sleep", "30")
        run("set-option", "-g", "status-interval", "1")
        run("set-option", "-g", "@job_value", "first")
        run("set-option", "-g", "status-right", "#(printf '#{@job_value}')")

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        attached = subprocess.Popen(
            base + ["attach-session", "-t", "jobs"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            start_new_session=True,
        )
        os.close(slave)

        wait_for_status(master, b"first")
        assert attached.poll() is None
        run("set-option", "-g", "@job_value", "second")
        wait_for_status(master, b"second")
        assert attached.poll() is None
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)
        if attached is not None:
            attached.wait(timeout=5)
        if master is not None:
            os.close(master)

print("format job owner CLI checks passed")
