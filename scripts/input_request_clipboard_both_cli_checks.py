#!/usr/bin/env python3
"""Compare a pane OSC 52 query round trip in get-clipboard=both mode."""

import base64
import fcntl
import os
import pathlib
import pty
import select
import shlex
import struct
import subprocess
import tempfile
import termios
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
ENV = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
TERMINAL_QUERY = b"\x1b]52;;?\x07"
PAYLOAD = b"A\0B\xff"
CASES = (
    ("binary", b"\x1b]52;c;" + base64.b64encode(PAYLOAD) + b"\x07",
     b"\x1b]52;c;" + base64.b64encode(PAYLOAD) + b"\x07", PAYLOAD, 2),
    ("invalid", b"\x1b]52;c;!!!\x1b\\", b"", b"prior", 1),
    ("empty", b"\x1b]52;c;\x07", b"", b"prior", 1),
)


def pane_command(reply_path):
    # Raw pane input lets the application collect the OSC reply byte for byte.
    # A short deadline also records the absence of a reply for invalid input.
    code = f"""import os, select, time, tty
tty.setraw(0)
os.write(1, b'\\x1b]52;c;?\\x07')
response = bytearray()
deadline = time.monotonic() + 1.5
while time.monotonic() < deadline:
    ready, _, _ = select.select([0], [], [], 0.05)
    if ready:
        response.extend(os.read(0, 4096))
    if response.endswith(b'\\x07') or response.endswith(b'\\x1b\\\\'):
        break
open({str(reply_path)!r}, 'wb').write(response)
"""
    return "python3 -c " + shlex.quote(code)


def trace(binary, case, terminal_reply, expected_reply, expected_buffer, expected_count):
    with tempfile.TemporaryDirectory(prefix=f"input-clipboard-both-{case}-", dir=ROOT / "target") as tmp:
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
            run("set-option", "-s", "get-clipboard", "both")
            run("set-option", "-s", "set-clipboard", "on")
            run("set-buffer", "--", "prior")
            attached = subprocess.Popen(
                base + ["attach-session", "-t", "clip"], env=ENV,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_name}").strip():
                assert attached.poll() is None, (case, "attached client exited")
                assert time.monotonic() < deadline, (case, "attached client missing")
                time.sleep(0.02)

            while select.select([master], [], [], 0)[0]:
                os.read(master, 65536)
            reply_path = pathlib.Path(tmp) / "pane-reply"
            run("new-window", "-t", "clip:", "-n", "query", pane_command(reply_path))

            terminal_output = bytearray()
            deadline = time.monotonic() + 5
            while TERMINAL_QUERY not in terminal_output and time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    terminal_output.extend(os.read(master, 65536))
            assert TERMINAL_QUERY in terminal_output, (
                case, "terminal query missing", terminal_output[-300:]
            )

            os.write(master, terminal_reply)
            deadline = time.monotonic() + 5
            while not reply_path.exists() and time.monotonic() < deadline:
                time.sleep(0.02)
            assert reply_path.exists(), (case, "pane did not finish query")
            pane_reply = reply_path.read_bytes()
            buffers = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            buffer = run("show-buffer")
            result = pane_reply, buffer, len(buffers)
            expected = expected_reply, expected_buffer, expected_count
            assert result == expected, (case, result, expected)
            return result
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
            subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=5)


for case_args in CASES:
    reference = trace(BASELINE, *case_args)
    observed = trace(CANDIDATE, *case_args)
    assert observed == reference, (case_args[0], observed, reference)

print("input request clipboard both CLI checks passed")
