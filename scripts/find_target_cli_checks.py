#!/usr/bin/env python3
"""Exercise split target lookup and errors on a private socket."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="find-target-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        return subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)

    try:
        for command in (
            (b"new-session", b"-d", b"-s", b"find-target", b"sleep", b"60"),
            (b"new-window", b"-d", b"-t", b"find-target:3", b"sleep", b"60"),
            (b"split-window", b"-d", b"-t", b"find-target:3", b"sleep", b"60"),
        ):
            result = run(*command)
            assert result.returncode == 0, (command, result.stderr)

        for target in (b"find-target:3.1", b"=find-target:3.1", b"3.1"):
            result = run(
                b"display-message", b"-p", b"-t", target,
                b"#{session_name}:#{window_index}.#{pane_index}",
            )
            assert (result.returncode, result.stdout, result.stderr) == (
                0, b"find-target:3.1\n", b""
            ), (target, result.returncode, result.stdout, result.stderr)

        for target, error in (
            (b"missing:3.1", b"can't find session: missing\n"),
            (b"find-target:99.1", b"can't find window: 99\n"),
            (b"find-target:3.99", b"can't find pane: 99\n"),
            (b"bad\xff:3.1", b"can't find session: bad\xff\n"),
        ):
            result = run(b"select-pane", b"-t", target)
            assert (result.returncode, result.stdout, result.stderr) == (
                1, b"", error
            ), (target, result.returncode, result.stdout, result.stderr)
    finally:
        run(b"kill-server")
        socket.unlink(missing_ok=True)

print("find target CLI checks passed")
