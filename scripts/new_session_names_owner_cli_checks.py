#!/usr/bin/env python3
"""Compare new-session's owned window and session names with pinned tmux."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = os.environb | {
    b"TERM": b"xterm-256color",
    b"LC_ALL": b"C.UTF-8",
    b"TMUX": b"",
    b"SHELL": b"/bin/sh",
}


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="new-session-names-", dir=root / "target") as tmp:
        base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]

        def call(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            return result.returncode, result.stdout, result.stderr

        name = "séss ion".encode()
        try:
            created = call(b"new-session", b"-d", b"-s", name, b"-n", b"win name", b"sleep 60")
            assert created == (0, b"", b""), created
            session = call(b"list-sessions", b"-F", b"#{session_name}")
            window = call(b"list-windows", b"-t", name, b"-F", b"#{window_name}")
            assert session == (0, name + b"\n", b""), session
            assert window == (0, b"win name\n", b""), window

            duplicate = call(b"new-session", b"-d", b"-s", name, b"-n", b"ignored", b"sleep 60")
            attach = call(b"new-session", b"-d", b"-A", b"-s", name, b"-n", b"ignored")
            invalid_window = call(b"new-session", b"-d", b"-s", b"invalid-window", b"-n", b"\xff")
            invalid_session = call(b"new-session", b"-d", b"-s", b"\xff", b"-n", b"valid")
            assert duplicate[0] != 0 and b"duplicate session" in duplicate[2], duplicate
            assert attach[0] != 0 and b"open terminal failed" in attach[2], attach
            assert invalid_window[0] != 0 and b"invalid window name" in invalid_window[2], invalid_window
            assert invalid_session[0] != 0 and b"invalid session name" in invalid_session[2], invalid_session
            assert call(b"list-sessions", b"-F", b"#{session_name}") == session
            assert call(b"list-windows", b"-t", name, b"-F", b"#{window_name}") == window
            return created, session, window, duplicate, attach, invalid_window, invalid_session
        finally:
            call(b"kill-server")


assert trace(candidate) == trace(baseline)
print("new-session names owner CLI checks passed")
