#!/usr/bin/env python3
"""Check the custom key-table title rendered by customize-mode."""

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
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
table = "ownership-\u00e9"

with tempfile.TemporaryDirectory(prefix="customize-key-title-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "keys", "sleep 30")
        run("bind-key", "-T", table, "F12", "display-message", "ownership")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "keys"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None

        run("customize-mode", "-t", "keys:0.0", "-f", "#{m:F12,#{key}}")
        output = bytearray()
        deadline = time.monotonic() + 5
        needle = f"Key Table - {table}".encode()
        while time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                output.extend(os.read(master, 65536))
            if needle in output:
                break
        else:
            raise AssertionError(f"missing key-table title: {output[-3000:]!r}")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("customize key-table-title CLI checks passed")
