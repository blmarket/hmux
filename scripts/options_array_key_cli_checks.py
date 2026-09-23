#!/usr/bin/env python3
"""Exercise array-key normalization and rejection on a private server."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="options-array-key-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args, status=0):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == status, (args, result.returncode, result.stderr)
        return result.stdout, result.stderr

    try:
        run(b"new-session", b"-d", b"-s", b"opts", b"sleep", b"60")
        run(b"set-option", b"-g", b"update-environment[0007]", b"FOO")
        assert run(b"show-options", b"-gv", b"update-environment[7]")[0] == b"FOO\n"
        assert run(b"show-options", b"-gv", b"update-environment[0007]")[0] == b"FOO\n"

        run(b"set-option", b"-g", b"update-environment[\xff]", b"BAR")
        assert run(b"show-options", b"-gv", b"update-environment[\xff]")[0] == b"BAR\n"

        for invalid in (b"update-environment[]", b"update-environment[4294967296]"):
            _, error = run(b"set-option", b"-g", invalid, b"NOPE", status=1)
            assert error == b"invalid option: " + invalid + b"\n"

        run(b"set-option", b"-gu", b"update-environment[7]")
        assert run(b"show-options", b"-gv", b"update-environment[7]")[0] == b"\n"
        assert run(b"show-options", b"-gv", b"update-environment[\xff]")[0] == b"BAR\n"
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)
        socket.unlink(missing_ok=True)

print("options array-key CLI checks passed")
