#!/usr/bin/env python3
"""Exercise terminal OSC 52 query replies through an attached PTY client."""

import base64
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


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
ENV = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
QUERY = b"\x1b]52;;?\x07"
PAYLOAD = b"A\0B\xff"


def trace(binary, case, reply, expected, expected_count):
    with tempfile.TemporaryDirectory(prefix=f"tty-clipboard-{case}-", dir=ROOT / "target") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def command(*args):
            return subprocess.run(base + list(args), env=ENV, capture_output=True, timeout=20)

        def run(*args):
            result = command(*args)
            assert result.returncode == 0, (case, args, result.returncode, result.stderr)
            return result.stdout

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        attached = None
        try:
            run("new-session", "-d", "-s", "clip", "sleep 30")
            run("set-buffer", "--", "prior")
            attached = subprocess.Popen(
                base + ["attach-session", "-t", "clip"],
                env=ENV, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None

            deadline = time.monotonic() + 5
            while not (target := run("list-clients", "-F", "#{client_name}").strip()):
                assert attached.poll() is None, (case, "attached client exited")
                assert time.monotonic() < deadline, (case, "attached client missing")
                time.sleep(0.02)

            while select.select([master], [], [], 0)[0]:
                os.read(master, 65536)
            run("refresh-client", "-l", "-t", os.fsdecode(target))
            terminal_output = bytearray()
            deadline = time.monotonic() + 5
            while QUERY not in terminal_output and time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    terminal_output.extend(os.read(master, 65536))
            assert QUERY in terminal_output, (case, "OSC 52 query missing", terminal_output[-300:])

            os.write(master, reply)
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                buffers = run("list-buffers", "-F", "#{buffer_name}").splitlines()
                actual = run("show-buffer")
                if actual == expected and len(buffers) == expected_count:
                    break
                time.sleep(0.02)
            else:
                raise AssertionError((case, actual, buffers, expected, expected_count))

            # Give malformed and empty replies time to pass through the tty parser;
            # they must not create a second buffer after the initial observation.
            time.sleep(0.3)
            actual = run("show-buffer")
            buffers = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            assert actual == expected and len(buffers) == expected_count, (case, actual, buffers)
            return actual, len(buffers)
        finally:
            if attached is not None:
                if attached.poll() is None:
                    attached.terminate()
                try:
                    attached.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    attached.kill()
                    attached.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)
            try:
                subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=5)
            except subprocess.TimeoutExpired:
                pass


CASES = (
    ("binary", b"\x1b]52;c;" + base64.b64encode(PAYLOAD) + b"\x07", PAYLOAD, 2),
    ("invalid", b"\x1b]52;c;!!!\x1b\\", b"prior", 1),
    ("empty", b"\x1b]52;c;\x07", b"prior", 1),
)
for case_args in CASES:
    reference = trace(BASELINE, *case_args)
    observed = trace(CANDIDATE, *case_args)
    assert observed == reference, (case_args[0], observed, reference)

print("tty clipboard query owner CLI checks passed")
