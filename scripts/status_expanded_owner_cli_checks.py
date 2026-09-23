#!/usr/bin/env python3
"""Compare unchanged and replaced status expansions on an attached client."""

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
    with tempfile.TemporaryDirectory(prefix="status-expanded-owner-") as directory:
        base = [str(binary), "-S", str(pathlib.Path(directory) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def read_until(master, marker):
            output = bytearray()
            deadline = time.monotonic() + 6
            while marker not in output:
                assert time.monotonic() < deadline, (marker, bytes(output[-200:]))
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    output.extend(os.read(master, 65536))
            return bytes(output)

        def unchanged_redraws(master, marker):
            # Two status timer ticks must retain the same expansion without
            # sending the status text again to the terminal.
            output = bytearray()
            # An option change may queue a second immediate full redraw.
            settle = time.monotonic() + 0.6
            while time.monotonic() < settle:
                ready, _, _ = select.select([master], [], [], min(0.1, settle - time.monotonic()))
                if ready:
                    os.read(master, 65536)
            deadline = time.monotonic() + 2.5
            while time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], min(0.1, deadline - time.monotonic()))
                if ready:
                    output.extend(os.read(master, 65536))
            assert marker not in output, bytes(output[-200:])

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None
        try:
            run("new-session", "-d", "-x", "80", "-y", "24", "-s", "statusowner", "sleep 30")
            run("set-option", "-g", "status-interval", "1")
            run("set-option", "-g", "status-format[0]", "OWNER-STATUS-#{@owner_status}")
            run("set-option", "-g", "@owner_status", "FIRST")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "statusowner"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None

            read_until(master, b"OWNER-STATUS-FIRST")
            unchanged_redraws(master, b"OWNER-STATUS-FIRST")
            run("set-option", "-g", "@owner_status", "SECOND")
            read_until(master, b"OWNER-STATUS-SECOND")
            run("set-option", "-g", "@owner_status", "THIRD")
            read_until(master, b"OWNER-STATUS-THIRD")

            ttys = run("list-clients", "-F", "#{client_tty}").splitlines()
            assert len(ttys) == 1, ttys
            run("detach-client", "-t", ttys[0].decode())
            assert client.wait(timeout=6) == 0
            return run("list-clients", "-F", "#{client_tty}")
        finally:
            if client is not None and client.poll() is None:
                client.terminate()
                client.wait(timeout=6)
            if slave is not None:
                os.close(slave)
            os.close(master)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected == b"", (actual, expected)
print("status expanded owner CLI checks passed")
