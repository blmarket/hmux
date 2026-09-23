#!/usr/bin/env python3
"""Render and replace pane-border text on an attached PTY client."""

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
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="pane-border-owner-") as directory:
        base = [str(binary), "-S", str(pathlib.Path(directory) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def read_until(master, marker):
            output = bytearray()
            deadline = time.monotonic() + 5
            while marker not in output:
                assert time.monotonic() < deadline, (marker, bytes(output[-160:]))
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    output.extend(os.read(master, 65536))
            return marker in output

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None
        try:
            run("new-session", "-d", "-x", "80", "-y", "24", "-s", "border", "sleep 60")
            run("split-window", "-d", "-t", "border:0.0", "sleep 60")
            run("set-window-option", "-t", "border:0", "pane-border-status", "top")
            run("set-option", "-p", "-t", "border:0.0", "pane-border-format", "OWNER-PANE-A")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "border"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            first = read_until(master, b"OWNER-PANE-A")
            run("set-option", "-p", "-t", "border:0.0", "pane-border-format", "OWNER-PANE-B")
            run("refresh-client", "-S")
            second = read_until(master, b"OWNER-PANE-B")
            run("kill-pane", "-t", "border:0.0")
            return first, second
        finally:
            if client is not None:
                client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=15)


reference = trace(baseline)
assert reference == (True, True), reference
assert trace(candidate) == reference
print("pane border expansion owner CLI checks passed")
