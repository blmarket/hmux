#!/usr/bin/env python3
"""Check pane-text format search through a private server."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-search-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    def search(expression):
        return run("display-message", "-p", "-t", "search:0.0", expression)

    try:
        run("new-session", "-d", "-s", "search",
            "printf 'prefix Needle suffix\\nsecond line\\n'; sleep 30")
        deadline = time.monotonic() + 5
        while b"second line" not in run("capture-pane", "-p", "-t", "search:0.0"):
            assert time.monotonic() < deadline, "pane output did not appear"
            time.sleep(0.05)

        assert search("#{C:Needle}") == b"1\n"
        assert search("#{C:second}") == b"2\n"
        assert search("#{C:needle}") == b"0\n"
        assert search("#{C/i:needle}") == b"1\n"
        assert search("#{C/r:^prefix Needle}") == b"1\n"
        assert search("#{C:Absent}") == b"0\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)

print("format search CLI checks passed")
