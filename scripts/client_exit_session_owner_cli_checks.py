#!/usr/bin/env python3
"""Check that detach delivers the current session name to the client."""

import os
import pathlib
import pty
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = os.environb | {b"TERM": b"xterm-256color", b"TMUX": b"", b"SHELL": b"/bin/sh"}


def trace(executable, directory):
    base = [executable, b"-S", os.fsencode(directory / "socket"), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    client = None
    master = None
    try:
        run(b"new-session", b"-d", b"-s", b"exit-initial", b"sleep", b"30")
        master, slave = pty.openpty()
        client = subprocess.Popen(
            base + [b"attach-session", b"-t", b"exit-initial"],
            env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
        )
        os.close(slave)

        tty = b""
        deadline = time.monotonic() + 10
        while not tty and time.monotonic() < deadline:
            tty = run(b"list-clients", b"-F", b"#{client_tty}").strip()
            if not tty:
                time.sleep(0.05)
        assert tty

        run(b"rename-session", b"-t", b"exit-initial", b"exit-final")
        run(b"detach-client", b"-t", tty)
        assert client.wait(timeout=10) == 0
        client = None

        output = bytearray()
        while True:
            try:
                chunk = os.read(master, 65536)
            except OSError:
                break
            if not chunk:
                break
            output.extend(chunk)
        message = b"[detached (from session exit-final)]\r\n"
        assert output.endswith(message), bytes(output[-200:])
        return bytes(output[-len(message):])
    finally:
        if client is not None and client.poll() is None:
            client.kill()
            client.wait(timeout=5)
        if master is not None:
            os.close(master)
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="client-exit-session-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = trace(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        assert candidate == trace(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)

print("client exit-session owner CLI checks passed")
