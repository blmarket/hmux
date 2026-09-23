#!/usr/bin/env python3
"""Exercise #() command-name ownership through live format expansion."""

import fcntl
import os
import pathlib
import pty
import shlex
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="format-job-name-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    attached = None
    master = None
    try:
        run("new-session", "-d", "-s", "jobname", "sleep", "30")
        # The closing parenthesis in the shell string exercises balanced scanning.
        marker = pathlib.Path(tmp) / "command-output"
        expression = f"#(printf 'job(one)' > {shlex.quote(str(marker))})"
        run("set-option", "-g", "status-right", expression)
        # Keep the client connected while the status job runs asynchronously.
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        attached = subprocess.Popen(
            base + ["attach-session", "-t", "jobname"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            start_new_session=True,
        )
        os.close(slave)
        deadline = time.monotonic() + 5
        while (not marker.exists() or marker.read_bytes() != b"job(one)") and time.monotonic() < deadline:
            time.sleep(0.05)
        assert marker.exists() and marker.read_bytes() == b"job(one)"
        assert attached.poll() is None
        trace = run("display-message", "-p", "-v", "-t", "jobname", expression)
        assert b"found #(): printf 'job(one)'" in trace, trace
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)
        if attached is not None:
            attached.wait(timeout=5)
        if master is not None:
            os.close(master)

print("format job-name CLI checks passed")
