#!/usr/bin/env python3
"""Compare detach-client -E's command and shell payload with the baseline."""

import os
import pathlib
import pty
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="client-exec-owner-") as tmp:
        directory = pathlib.Path(tmp)
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
        base = [os.fsencode(binary), b"-S", os.fsencode(directory / "socket"), b"-f", b"/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        client = None
        master = None
        try:
            run(b"new-session", b"-d", b"-s", b"exec-owner", b"sleep", b"30")
            run(b"set-option", b"-g", b"default-shell", b"/bin/sh")
            client_pid, master = pty.fork()
            if client_pid == 0:
                os.execve(binary, base + [b"attach-session", b"-t", b"exec-owner"], env)
            client = client_pid

            deadline = time.monotonic() + 10
            tty = b""
            while time.monotonic() < deadline:
                tty = run(b"list-clients", b"-F", b"#{client_tty}").strip()
                if tty:
                    break
                time.sleep(0.03)
            assert tty, "client did not attach"

            payload = directory / "payload"
            shell = directory / "shell"
            command = (
                b"printf '\xffpayload' > " + os.fsencode(payload)
                + b"; printf '%s' \"$SHELL\" > " + os.fsencode(shell)
            )
            run(b"detach-client", b"-t", tty, b"-E", command)
            deadline = time.monotonic() + 10
            while True:
                done, status = os.waitpid(client, os.WNOHANG)
                if done:
                    client = None
                    break
                assert time.monotonic() < deadline, "executed command did not exit"
                time.sleep(0.03)
            assert os.waitstatus_to_exitcode(status) == 0, status
            result = (payload.read_bytes(), shell.read_bytes())
            assert result == (b"\xffpayload", b"/bin/sh"), result
            return result
        finally:
            if client is not None:
                os.kill(client, 15)
                os.waitpid(client, 0)
            if master is not None:
                os.close(master)
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)


assert trace(candidate) == trace(baseline)
print("server client exec owner CLI checks passed")
