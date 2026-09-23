#!/usr/bin/env python3
"""Exercise retained scalar option strings on a private server."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="options-string-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def reject(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode != 0, (args, result.stdout, result.stderr)
        return result.stderr

    try:
        run(b"new-session", b"-d", b"-s", b"opts", b"sleep", b"60")
        run(b"set-option", b"-g", b"@bytes", b"\xff")
        assert run(b"show-options", b"-gv", b"@bytes") == b"\xff\n"
        run(b"set-option", b"-ga", b"@bytes", b"\x80")
        assert run(b"show-options", b"-gv", b"@bytes") == b"\xff\x80\n"
        run(b"set-option", b"-g", b"@bytes", b"")
        assert run(b"show-options", b"-gv", b"@bytes") == b"\n"
        run(b"set-option", b"-g", b"status-left", b"left")
        run(b"set-option", b"-ga", b"status-left", b"right")
        assert run(b"show-options", b"-gv", b"status-left") == b"leftright\n"
        run(b"set-option", b"-gu", b"@bytes")
        assert run(b"show-options", b"-gv", b"status-left") == b"leftright\n"
        run(b"set-option", b"-g", b"default-shell", b"/bin/sh")
        assert b"not a suitable shell" in reject(
            b"set-option", b"-g", b"default-shell", b"/definitely/not/a/shell"
        )
        assert run(b"show-options", b"-gv", b"default-shell") == b"/bin/sh\n"
        assert b"not a suitable shell" in reject(
            b"set-option", b"-ga", b"default-shell", b"bad"
        )
        assert run(b"show-options", b"-gv", b"default-shell") == b"/bin/sh\n"
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)
        socket.unlink(missing_ok=True)

print("options string CLI checks passed")
