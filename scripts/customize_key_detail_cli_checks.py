#!/usr/bin/env python3
"""Compare command, note, and repeat rows in an attached customize-mode tree."""

import fcntl
import os
import pathlib
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
csi = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")
detail_row = re.compile(rb"^\(M-[a-z]\).*\b(?:Command|Note|Repeat):")


def details(binary_path):
    with tempfile.TemporaryDirectory(prefix="customize-key-details-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 35, 130, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "keys", "sleep 30")
            run("bind-key", "-T", "ownertable", "F11", "display-message", "empty-note")
            run(
                "bind-key", "-r", "-N", "owner-note-\u00e9", "-T", "ownertable",
                "F12", "display-message", "owner-command-\u00e9",
            )
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
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)

            run("customize-mode", "-t", "keys:0.0", "-f", "#{m:F1[12],#{key}}")
            # The isolated table sorts two rows before the built-in prefix/root tables.
            run("send-keys", "-t", "keys:0.0", "G", "Up", "Up", "Right")
            run("send-keys", "-t", "keys:0.0", "Down", "Right")
            run("send-keys", "-t", "keys:0.0", "Down", "Down", "Down", "Down", "Right")

            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                cleaned = csi.sub(b"", output).replace(b"\x1b(B", b"")
                if b"Repeat: on" in cleaned:
                    found = [line for line in cleaned.splitlines() if detail_row.match(line)]
                    if len(found) >= 6:
                        return found[-6:]
            raise AssertionError(f"missing key detail rows: {output[-3000:]!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                try:
                    client.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    client.kill()
                    client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = details(binary)
expected_suffixes = [
    b"Command: display-message empty-note",
    b"Note:",
    b"Repeat: off",
    "Command: display-message owner-command-\u00e9".encode(),
    "Note: owner-note-\u00e9".encode(),
    b"Repeat: on",
]
assert all(line.endswith(suffix) for line, suffix in zip(actual, expected_suffixes)), actual
if baseline is not None:
    expected = details(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)

print("customize key-detail CLI checks passed")
