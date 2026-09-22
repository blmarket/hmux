#!/usr/bin/env python3
"""Exercise control replies, deferred notifications, and pane output."""
import os
import pathlib
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="control-lines-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)

    def read_until(marker):
        deadline = time.monotonic() + 10
        while marker not in output:
            remaining = deadline - time.monotonic()
            assert remaining > 0, (marker, output)
            ready, _, _ = select.select([control.stdout], [], [], remaining)
            assert ready, (marker, output)
            chunk = os.read(control.stdout.fileno(), 65536)
            assert chunk, (marker, output, control.poll())
            output.extend(chunk)

    try:
        run(b"new-session", b"-d", b"-s", b"ctrl", b"sleep", b"30")
        control = subprocess.Popen(
            base + [b"-C", b"attach-session", b"-t", b"ctrl"],
            env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        output = bytearray()
        try:
            read_until(b"%session-changed $0 ctrl\n")

            control.stdin.write(b"display-message -p bytes-marker\n")
            control.stdin.flush()
            read_until(b"bytes-marker\n")
            assert b"%begin " in output and b"%end " in output, output

            start = len(output)
            control.stdin.write(
                b"new-window -d -t ctrl: 'printf pane-marker; sleep 5'\n"
            )
            control.stdin.flush()
            read_until(b"%output %1 pane-marker\n")
            created = output[start:]
            assert b"%window-add @1\n" in created, created
            assert created.index(b"%end ") < created.index(b"%window-add @1\n"), created
        finally:
            control.terminate()
            control.communicate(timeout=5)
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)

print("control lines CLI checks passed")
