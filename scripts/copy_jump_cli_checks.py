#!/usr/bin/env python3
"""Exercise copy-mode jump text replacement on a real pane."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="copy-jump-cli-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        process = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert process.returncode == 0, (args, process.returncode, process.stderr)
        return process.stdout

    def cursor():
        return run("display-message", "-p", "#{copy_cursor_x}:#{copy_cursor_y}")

    def jump(command, character, expected):
        run("send-keys", "-X", command, character)
        actual = cursor()
        assert actual == expected, (command, character, actual, expected)

    try:
        run("new-session", "-d", "-x", "40", "-y", "8", "-s", "jump",
            "printf 'abcébc界bca\\n'; sleep 30")
        deadline = time.monotonic() + 5
        while not run("capture-pane", "-p", "-S", "0").startswith("abcébc界bca\n".encode()):
            assert time.monotonic() < deadline, "pane output missing"
            time.sleep(0.02)

        run("copy-mode")
        run("send-keys", "-X", "cursor-up")
        run("send-keys", "-X", "start-of-line")
        assert cursor() == b"0:0\n"

        jump("jump-forward", "b", b"1:0\n")
        run("send-keys", "-X", "jump-again")
        assert cursor() == b"4:0\n"
        run("send-keys", "-X", "jump-reverse")
        assert cursor() == b"1:0\n"
        jump("jump-forward", "界", b"6:0\n")
        jump("jump-backward", "é", b"3:0\n")
        jump("jump-to-forward", "a", b"9:0\n")
        jump("jump-to-backward", "é", b"4:0\n")

        # Exercise replacement and copy-mode teardown after several targets.
        jump("jump-backward", "b", b"1:0\n")
        run("send-keys", "-X", "cancel")
        assert run("display-message", "-p", "#{pane_in_mode}") == b"0\n"
        run("copy-mode")
        assert run("display-message", "-p", "#{pane_in_mode}") == b"1\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("copy jump CLI checks passed")
