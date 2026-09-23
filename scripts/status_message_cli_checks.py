#!/usr/bin/env python3
"""Exercise attached-client status messages and client teardown on a private socket."""

import os
import pathlib
import pty
import signal
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="status-message-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    pid = None
    master = None
    try:
        run(b"new-session", b"-d", b"-s", b"status", b"sleep", b"30")
        pid, master = pty.fork()
        if pid == 0:
            os.execve(binary, base + [b"attach-session", b"-t", b"status"], env)

        deadline = time.monotonic() + 10
        tty = None
        while time.monotonic() < deadline:
            clients = run(b"list-clients", b"-F", b"#{client_tty}").splitlines()
            if clients:
                tty = clients[0]
                break
            time.sleep(0.05)
        assert tty, "terminal client did not attach"

        run(b"display-message", b"-c", tty, b"-d", b"5000", b"first-marker")
        run(b"display-message", b"-c", tty, b"-d", b"5000", b"second-marker")
        run(b"display-message", b"-c", tty, b"-d", b"5000", b"")
        run(b"display-message", b"-c", tty, b"-d", b"100", b"\xff-marker")
        messages = run(b"show-messages")
        for value in (b"first-marker", b"second-marker", b"\xff-marker"):
            assert tty + b" message: " + value in messages, messages
        assert tty + b" message: \n" in messages, messages

        # Let the timer clear one message, then disconnect with another active.
        time.sleep(0.2)
        run(b"display-message", b"-c", tty, b"-d", b"5000", b"teardown-marker")
        run(b"detach-client", b"-t", tty)
        os.waitpid(pid, 0)
        pid = None
    finally:
        if pid is not None:
            os.kill(pid, signal.SIGTERM)
            os.waitpid(pid, 0)
        if master is not None:
            os.close(master)
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)

print("status message CLI checks passed")
