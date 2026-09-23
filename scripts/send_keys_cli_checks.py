#!/usr/bin/env python3
"""Check literal send-keys decoding through a real pane on a private socket."""
import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="send-keys-cli-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        process = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        if process.returncode:
            raise AssertionError((args, process.returncode, process.stdout, process.stderr))
        return process.stdout

    def wait_for_lines(expected):
        deadline = time.monotonic() + 3
        while True:
            lines = run("capture-pane", "-p", "-t", "keys:0.0").splitlines()
            if lines[:len(expected)] == expected:
                return
            if time.monotonic() >= deadline:
                raise AssertionError((expected, lines[:len(expected)]))
            time.sleep(0.05)

    try:
        run("new-session", "-d", "-s", "keys", "cat")
        run("send-keys", "-t", "keys:0.0", "-l", "Aé界B")
        run("send-keys", "-t", "keys:0.0", "Enter")
        first = b"A\xc3\xa9\xe7\x95\x8cB"
        wait_for_lines([first, first])
        # Invalid UTF-8 bytes still pass through the terminal as replacement
        # cells, and the valid cells on either side remain in order.
        run("send-keys", "-t", "keys:0.0", "-l", b"C\xffD\xc3\xa9E")
        run("send-keys", "-t", "keys:0.0", "Enter")
        second = b"C\xef\xbf\xbdD\xc3\xa9E"
        wait_for_lines([first, first, second, second])

        # An unrecognized key name uses the same literal decoder without -l.
        run("send-keys", "-t", "keys:0.0", "FéG")
        run("send-keys", "-t", "keys:0.0", "Enter")
        third = b"F\xc3\xa9G"
        wait_for_lines([first, first, second, second, third, third])
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)
        socket.unlink(missing_ok=True)

assert not socket.exists()
print("send-keys CLI checks passed")
