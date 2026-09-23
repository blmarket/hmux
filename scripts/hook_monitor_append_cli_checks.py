#!/usr/bin/env python3
"""Check monitor-hook value append across a private server."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="hook-monitor-append-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"hook", b"sleep", b"60")
        monitor = b"@watch::#{session_name}"
        run(b"set-hook", b"-g", b"-B", monitor, b"left")
        assert run(b"show-options", b"-gv", b"@watch") == b"left\n"

        run(b"set-hook", b"-ga", b"-B", monitor, b"right")
        assert run(b"show-options", b"-gv", b"@watch") == b"leftright\n"

        run(b"set-hook", b"-ga", b"-B", monitor, b"\xff")
        assert run(b"show-options", b"-gv", b"@watch") == b"leftright\xff\n"

        run(b"set-hook", b"-g", b"-B", monitor, b"")
        run(b"set-hook", b"-ga", b"-B", monitor, b"tail")
        assert run(b"show-options", b"-gv", b"@watch") == b"tail\n"

        invalid = subprocess.run(
            base + [b"set-hook", b"-g", b"-B", b"@invalid"],
            env=env, capture_output=True, timeout=20,
        )
        assert invalid.returncode == 1
        assert invalid.stderr == b"invalid subscription: @invalid\n"

        # The unsubscribe path also accepts a bare hook name.
        run(b"set-hook", b"-g", b"-u", b"-B", b"@watch")
        assert b"@watch" not in run(b"show-hooks", b"-g", b"-B", b"-F", b"#{option_name}")
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)

print("hook monitor append CLI checks passed")
