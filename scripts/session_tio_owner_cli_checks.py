#!/usr/bin/env python3
"""Compare attached-session terminal settings and teardown with the baseline."""

import fcntl
import os
import pathlib
import pty
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="session-tio-owner-", dir=root / "target") as tmp:
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        master, slave = pty.openpty()
        attrs = termios.tcgetattr(slave)
        attrs[6][termios.VINTR] = b"\x06"  # ^F, copied from the client tty.
        attrs[6][termios.VEOF] = b"\x1d"  # ^], also copied to the pane tty.
        termios.tcsetattr(slave, termios.TCSANOW, attrs)
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None
        try:
            client = subprocess.Popen(
                base + ["new-session", "-s", "terminal", "stty -a; sleep 30"],
                env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 8
            while True:
                try:
                    first = run("capture-pane", "-p", "-t", "terminal:0.0")
                except AssertionError:
                    first = b""
                if b"intr = ^F;" in first and b"eof = ^];" in first:
                    break
                assert time.monotonic() < deadline, ("first pane did not receive tty settings", first)
                time.sleep(0.05)

            run("new-window", "-d", "-t", "terminal", "stty -a; sleep 30")
            deadline = time.monotonic() + 8
            while True:
                second = run("capture-pane", "-p", "-t", "terminal:1.0")
                if b"intr = ^F;" in second and b"eof = ^];" in second:
                    break
                assert time.monotonic() < deadline, ("second pane did not retain tty settings", second)
                time.sleep(0.05)

            run("new-session", "-d", "-s", "idle", "sleep 30")
            run("kill-session", "-t", "terminal")
            client.wait(timeout=8)
            remaining = run("list-sessions", "-F", "#{session_name}")
            assert remaining == b"idle\n", remaining
            return first, second, remaining
        finally:
            if client is not None and client.poll() is None:
                client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("session termios owner CLI checks passed")
