#!/usr/bin/env python3
"""Exercise message queue retention and pruning on a private socket."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="server-messages-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args, ok=True):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        if ok:
            assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result

    try:
        run(b"new-session", b"-d", b"-s", b"messages", b"sleep", b"60")
        run(b"set-option", b"-s", b"message-limit", b"3")
        for number in range(4):
            marker = b"message-marker-" + str(number).encode()
            assert run(b"display-message", b"-p", marker).stdout == marker + b"\n"

        # show-messages logs its own command, leaving the two newest markers.
        lines = run(b"show-messages").stdout.splitlines()
        assert len(lines) == 3, lines
        assert b"command: show-messages" in lines[0], lines
        assert b"message-marker-3" in lines[1], lines
        assert b"message-marker-2" in lines[2], lines
        assert all(b"message-marker-0" not in line for line in lines), lines
        assert all(b"message-marker-1" not in line for line in lines), lines

        run(b"set-option", b"-s", b"message-limit", b"0")
        assert run(b"show-messages").stdout == b""

        run(b"set-option", b"-s", b"message-limit", b"2")
        failed = run(b"list-keys", b"-T", b"\xff", ok=False)
        assert failed.returncode == 1, failed.returncode
        assert failed.stderr == b"table \xff doesn't exist\n", failed.stderr
        lines = run(b"show-messages").stdout.splitlines()
        assert len(lines) == 2, lines
        assert b"command: show-messages" in lines[0], lines
        assert b"message: table \xff doesn't exist" in lines[1], lines
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)
        socket.unlink(missing_ok=True)

print("server message CLI checks passed")
