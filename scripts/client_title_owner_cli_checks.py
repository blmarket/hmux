#!/usr/bin/env python3
"""Compare attached-client terminal title updates with a matching tmux binary."""

import os
import pathlib
import pty
import select
import signal
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

    def read_until(master, marker):
        data = b""
        deadline = time.monotonic() + 10
        while marker not in data and time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                data += os.read(master, 65536)
        assert marker in data, (marker, data[-1000:])
        return data

    def read_quiet(master):
        data = b""
        deadline = time.monotonic() + 0.3
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.05)
            if ready:
                data += os.read(master, 65536)
        return data

    child = None
    master = None
    try:
        run(b"new-session", b"-d", b"-s", b"title-owner", b"sleep", b"30")
        run(b"set-option", b"-g", b"set-titles", b"on")
        run(b"set-option", b"-g", b"set-titles-string", b"OWNER-ONE")
        child, master = pty.fork()
        if child == 0:
            os.execve(executable, base + [b"attach-session", b"-t", b"title-owner"], env)

        first = b"\x1b]0;OWNER-ONE\x07"
        read_until(master, first)
        tty = run(b"list-clients", b"-F", b"#{client_tty}").strip()
        assert tty
        read_quiet(master)

        run(b"refresh-client", b"-t", tty)
        assert b"\x1b]0;" not in read_quiet(master)

        run(b"set-option", b"-g", b"set-titles-string", b"OWNER-TWO")
        run(b"refresh-client", b"-t", tty)
        second = read_until(master, b"\x1b]0;OWNER-TWO\x07")
        assert second.count(b"\x1b]0;OWNER-TWO\x07") == 1
        read_quiet(master)

        run(b"refresh-client", b"-t", tty)
        assert b"\x1b]0;" not in read_quiet(master)

        run(b"set-option", b"-g", b"set-titles-string", b"")
        run(b"refresh-client", b"-t", tty)
        empty = read_until(master, b"\x1b]0;\x07")
        assert empty.count(b"\x1b]0;\x07") == 1

        run(b"detach-client", b"-t", tty)
        os.waitpid(child, 0)
        child = None
        return first, b"\x1b]0;OWNER-TWO\x07", b"\x1b]0;\x07"
    finally:
        if child is not None:
            os.kill(child, signal.SIGTERM)
            os.waitpid(child, 0)
        if master is not None:
            os.close(master)
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="client-title-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = trace(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        assert candidate == trace(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)

print("client title owner CLI checks passed")
