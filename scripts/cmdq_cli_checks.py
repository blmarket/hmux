#!/usr/bin/env python3
"""Exercise formatted hook names and command errors on a private socket."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="cmdq-cli-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args, ok=True):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        if ok:
            assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result

    try:
        run(b"new-session", b"-d", b"-s", b"cmdq", b"sleep", b"60")
        run(b"set-option", b"-g", b"@cmdq-hook", b"")
        run(b"set-hook", b"-g", b"after-new-window", b"set-option -ga @cmdq-hook fired")
        run(b"new-window", b"-d", b"-t", b"cmdq:", b"sleep", b"60")
        assert run(b"show-options", b"-gqv", b"@cmdq-hook").stdout == b"fired\n"
        run(b"new-window", b"-d", b"-t", b"cmdq:", b"sleep", b"60")
        assert run(b"show-options", b"-gqv", b"@cmdq-hook").stdout == b"firedfired\n"

        for table in (b"missing", b"\xff"):
            error = run(b"list-keys", b"-T", table, ok=False)
            assert error.returncode == 1, (table, error.returncode)
            assert error.stderr == b"table " + table + b" doesn't exist\n", error.stderr
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)
        socket.unlink(missing_ok=True)

print("command queue CLI checks passed")
