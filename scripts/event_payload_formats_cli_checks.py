#!/usr/bin/env python3
"""Exercise event payload format keys and names through live hooks."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="event-payload-formats-", dir=root / "target") as tmp:
    base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"alpha", b"sleep", b"60")
        run(b"rename-window", b"-t", b"alpha:0", b"initial")
        run(
            b"set-hook", b"-g", b"window-renamed",
            b'set-option -gF @window-event "#{hook_window_name}|#{hook_old_name}|#{hook_new_name}"',
        )
        run(b"rename-window", b"-t", b"alpha:0", "écho".encode())
        assert run(b"show-options", b"-gqv", b"@window-event") == "écho|initial|écho\n".encode()

        run(
            b"set-hook", b"-g", b"session-created",
            b'set-option -gF @session-event "#{hook_session_name}"',
        )
        run(b"new-session", b"-d", b"-s", b"beta", b"sleep", b"60")
        assert run(b"show-options", b"-gqv", b"@session-event") == b"beta\n"
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)

print("event payload format CLI checks passed")
