#!/usr/bin/env python3
"""Check pane_private_modes across terminal mode changes on a live server."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="pane-modes-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def expect_modes(expected):
        deadline = time.monotonic() + 5
        while True:
            actual = run("display-message", "-p", "#{pane_private_modes}")
            if actual == expected:
                return
            assert time.monotonic() < deadline, (expected, actual)
            time.sleep(0.05)

    try:
        run("new-session", "-d", "-s", "modes", "stty raw -echo; printf READY; cat")
        deadline = time.monotonic() + 5
        while b"READY" not in run("capture-pane", "-p", "-t", "modes:0.0"):
            assert time.monotonic() < deadline, "pane did not start"
            time.sleep(0.05)

        expect_modes(b"7,25\n")
        run("send-keys", "-l", "-t", "modes:0.0", "\x1b[?7;25l")
        expect_modes(b"\n")
        run("send-keys", "-l", "-t", "modes:0.0", "\x1b[?1;6;1000;1006h")
        expect_modes(b"1,6,1000,1006\n")
        run("send-keys", "-l", "-t", "modes:0.0", "\x1b[?12h")
        expect_modes(b"1,6,12,1000,1006\n")
        run("send-keys", "-l", "-t", "modes:0.0", "\x1b[?1;6;12;1000;1006l")
        expect_modes(b"\n")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("pane private modes CLI checks passed")
