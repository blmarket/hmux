#!/usr/bin/env python3
"""Check pane_tabs for default, cleared, and custom terminal tab stops."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="pane-tabs-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def expect_tabs(expected):
        deadline = time.monotonic() + 5
        while True:
            actual = run("display-message", "-p", "#{pane_tabs}")
            if actual == expected:
                return
            assert time.monotonic() < deadline, (expected, actual)
            time.sleep(0.05)

    def expect_cursor(expected):
        deadline = time.monotonic() + 5
        while True:
            actual = run("display-message", "-p", "#{cursor_x}")
            if actual == str(expected).encode() + b"\n":
                return
            assert time.monotonic() < deadline, (expected, actual)
            time.sleep(0.05)

    try:
        run("new-session", "-d", "-x", "40", "-y", "8", "-s", "tabs",
            "stty raw -echo; printf READY; cat")
        deadline = time.monotonic() + 5
        while b"READY" not in run("capture-pane", "-p"):
            assert time.monotonic() < deadline, "pane did not start"
            time.sleep(0.05)

        expect_tabs(b"8,16,24,32\n")
        run("send-keys", "-l", "\x1b[3g")
        expect_tabs(b"\n")
        run("send-keys", "-l", "\x1b[5G\x1bH\x1b[13G\x1bH")
        expect_tabs(b"4,12\n")
        run("send-keys", "-l", "\x1b[1G\t")
        expect_cursor(4)
        run("send-keys", "-l", "\t")
        expect_cursor(12)
        run("send-keys", "-l", "\x1b[Z")
        expect_cursor(4)
        run("send-keys", "-l", "\x1b[g")
        expect_tabs(b"12\n")
        run("resize-window", "-x", "17", "-y", "8")
        expect_tabs(b"8,16\n")
        run("resize-window", "-x", "9", "-y", "8")
        expect_tabs(b"8\n")
        run("send-keys", "-l", "\x1b[3g")
        expect_tabs(b"\n")
        run("send-keys", "-l", "\x1bc")
        expect_tabs(b"8\n")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("pane tabs CLI checks passed")
