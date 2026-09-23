#!/usr/bin/env python3
"""Grow an OSC parser buffer, return to ground, and reuse it in a live pane."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
long_title = "title-" + "x" * 200
short_title = "reset-ok"

with tempfile.TemporaryDirectory(prefix="input-buffer-", dir=root / "target") as tmp:
    go = pathlib.Path(tmp) / "go"
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def wait_for_title(expected):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            actual = run("display-message", "-p", "-t", "ibuf:0", "#{pane_title}")
            if actual == expected.encode() + b"\n":
                return
            time.sleep(0.05)
        raise AssertionError((expected, actual))

    pane = (
        f"printf '\\033]2;{long_title}\\007'; "
        f"while [ ! -e '{go}' ]; do sleep 0.05; done; "
        f"printf '\\033]2;{short_title}\\007'; sleep 30"
    )
    try:
        run("new-session", "-d", "-s", "ibuf", pane)
        wait_for_title(long_title)
        go.touch()
        wait_for_title(short_title)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("input buffer owner CLI checks passed")
