#!/usr/bin/env python3
"""Check live pane command and cwd lookups through Linux /proc paths."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="osdep-proc-", dir=root / "target") as tmp:
    directory = pathlib.Path(tmp)
    base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "proc", "-c", str(directory), "exec sleep 60")
        expected = f"sleep|{directory}\n".encode()
        deadline = time.monotonic() + 5
        while True:
            actual = run("display-message", "-p", "#{pane_current_command}|#{pane_current_path}")
            if actual == expected:
                break
            assert time.monotonic() < deadline, (expected, actual)
            time.sleep(0.05)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("osdep proc CLI checks passed")
