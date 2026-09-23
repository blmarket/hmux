#!/usr/bin/env python3
"""Compare generated session names for existing and new groups."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="session-prefix-owner-", dir=root / "target") as tmp:
        base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def sessions():
            return run(b"list-sessions", b"-F", b"#{session_name}|#{session_group}")

        try:
            run(b"new-session", b"-d", b"-s", b"base", b"sleep 30")
            plain = sessions()
            run(b"new-session", b"-d", b"-t", b"base")
            existing = sessions()
            run(b"new-session", b"-d", b"-t", "g name:é".encode())
            escaped = sessions()
            invalid = subprocess.run(
                base + [b"new-session", b"-d", b"-t", b"bad\xff"],
                env=env, capture_output=True, timeout=20,
            )
            assert invalid.returncode != 0, invalid.stdout
            assert b"invalid session group name" in invalid.stderr, invalid.stderr
            assert plain != existing != escaped, (plain, existing, escaped)
            return plain, existing, escaped, invalid.returncode, invalid.stderr
        finally:
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


reference = trace(baseline)
observed = trace(candidate)
assert observed == reference, (observed, reference)
print("new session prefix owner CLI checks passed")
