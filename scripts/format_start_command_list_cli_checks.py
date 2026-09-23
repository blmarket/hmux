#!/usr/bin/env python3
"""Check pane start-command lists against live argv, including raw bytes."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-start-command-", dir=root / "target") as tmp:
    base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def pane_command(target):
        return run(b"display-message", b"-p", b"-t", target, b"#{pane_start_command_list}")

    try:
        run(b"new-session", b"-d", b"-s", b"start")
        assert pane_command(b"start:0") == b"\n"  # No explicit command: argc = 0.
        assert run(b"list-sessions", b"-F", b"<#{pane_start_command_list}>") == b"<>\n"

        run(b"new-window", b"-d", b"-t", b"start:", b"sleep", b"60")
        assert pane_command(b"start:1") == b"'sleep' '60'\n"

        run(b"new-window", b"-d", b"-t", b"start:", b"/bin/sh", b"-c", b"exec sleep 60", b"o'hara")
        assert pane_command(b"start:2") == b"'/bin/sh' '-c' 'exec sleep 60' 'o'\\''hara'\n"

        run(b"new-window", b"-d", b"-t", b"start:", b"/bin/sh", b"-c", b"exec sleep 60", b"", b"\xff")
        assert pane_command(b"start:3") == b"'/bin/sh' '-c' 'exec sleep 60' '' '\xff'\n"

        long_arg = b"a" * 4096
        run(b"new-window", b"-d", b"-t", b"start:", b"/bin/sh", b"-c", b"exec sleep 60", long_arg)
        assert pane_command(b"start:4") == b"'/bin/sh' '-c' 'exec sleep 60' '" + long_arg + b"'\n"
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)

print("start-command list CLI checks passed")
