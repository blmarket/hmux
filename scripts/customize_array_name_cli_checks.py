#!/usr/bin/env python3
"""Exercise array-option names in the attached customize-mode tree."""

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

with tempfile.TemporaryDirectory(prefix="customize-array-name-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def wait_for_terminal(needle):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
            if ready:
                output.extend(os.read(master, 65536))
                if needle in output:
                    return
        raise AssertionError(f"did not see {needle!r}: {output[-5000:]!r}")

    try:
        run("new-session", "-d", "-s", "opts", "sleep 30")
        run("set-option", "-g", "status-format[7]", "array-marker")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "opts"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None
        run(
            "customize-mode",
            "-t", "opts:0.0",
            "-f", "#{m:*status-format*,#{option_name}}",
            "-F", "#{option_name}:#{option_value}",
        )
        run("send-keys", "-t", "opts:0.0", "Down", "Right")
        wait_for_terminal(b"status-format")
        run("send-keys", "-t", "opts:0.0", "Down", "Right")
        wait_for_terminal(b"status-format[7]:array-marker")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("customize array-name CLI checks passed")
