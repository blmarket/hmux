#!/usr/bin/env python3
"""Compare parsed view-mode text against the pinned baseline."""

import os
import pathlib
import pty
import subprocess
import tempfile
import time
import fcntl
import struct
import termios


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="copy-vadd-owner-", dir=root / "target") as tmp:
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args):
            process = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert process.returncode == 0, (args, process.returncode, process.stderr)
            return process.stdout

        try:
            run("new-session", "-d", "-x", "40", "-y", "8", "-s", "copy-owner", "sleep 30")
            run(
                "run-shell", "-t", "copy-owner:0.0",
                r"printf 'first\n\033[31mred-é\033[0m\nlast-\377\n'",
            )
            deadline = time.monotonic() + 5
            while run("display-message", "-p", "#{pane_in_mode}|#{pane_mode}") != b"1|view-mode\n":
                assert time.monotonic() < deadline, "run-shell did not open view mode"
                time.sleep(0.02)

            lines = []
            for index in range(3):
                if index:
                    run("send-keys", "-X", "cursor-down")
                run("send-keys", "-X", "select-line")
                run("send-keys", "-X", "copy-selection")
                lines.append(run("show-buffer"))
            return lines
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


def embedded_nul_trace(binary):
    """Drive show-buffer from an attached client, preserving its binary input."""
    with tempfile.TemporaryDirectory(prefix="copy-vadd-nul-", dir=root / "target") as tmp:
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        buffer_file = pathlib.Path(tmp) / "binary-buffer"
        buffer_file.write_bytes(b"A\x00B")

        def run(*args):
            process = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert process.returncode == 0, (args, process.returncode, process.stderr)
            return process.stdout

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None
        try:
            run("new-session", "-d", "-s", "binary", "sleep 30")
            run("load-buffer", str(buffer_file))
            assert run("show-buffer") == b"A\x00B"
            run("bind-key", "-n", "C-g", "show-buffer")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "binary"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.02)

            os.write(master, b"\x07")  # Bound C-g invokes show-buffer in the attached client.
            deadline = time.monotonic() + 5
            while run("display-message", "-p", "#{pane_in_mode}|#{pane_mode}") != b"1|view-mode\n":
                assert time.monotonic() < deadline, "show-buffer did not open view mode"
                time.sleep(0.02)
            run("send-keys", "-X", "select-line")
            run("send-keys", "-X", "copy-selection")
            return run("show-buffer")
        finally:
            if client is not None:
                client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


expected = [b"first\n", "red-é\n".encode(), "last-�\n".encode()]
actual = trace(candidate)
reference = trace(baseline)
assert reference == expected, (reference, expected)
assert actual == reference, (actual, reference)
assert embedded_nul_trace(baseline) == b"A\n"
assert embedded_nul_trace(candidate) == b"A\n"
print("window copy vadd owner CLI checks passed")
