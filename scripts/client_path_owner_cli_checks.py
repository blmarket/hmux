#!/usr/bin/env python3
"""Exercise pane OSC 7 updates through the attached client's path output."""

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
prefix = b"\x1b]777;"


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
    pane_fd = None
    try:
        run(b"new-session", b"-d", b"-s", b"path-owner", b"sleep", b"30")
        run(b"set-option", b"-g", b"set-titles", b"on")
        # A distinctive outer code shows the path was emitted by tty_set_path,
        # rather than merely forwarded from the pane's OSC 7 input.
        run(b"set-option", b"-g", b"terminal-overrides",
            b"xterm-256color:Swd=\\E]777;:fsl=\\a")
        child, master = pty.fork()
        if child == 0:
            os.execve(executable, base + [b"attach-session", b"-t", b"path-owner"], env)

        tty = b""
        deadline = time.monotonic() + 10
        while not tty and time.monotonic() < deadline:
            tty = run(b"list-clients", b"-F", b"#{client_tty}").strip()
            if not tty:
                time.sleep(0.05)
        assert tty
        read_until(master, prefix + b"\x07")
        read_quiet(master)

        pane_tty = run(b"display-message", b"-p", b"-t", b"path-owner:0.0",
                       b"#{pane_tty}").strip()
        assert pane_tty.startswith(b"/dev/"), pane_tty
        pane_fd = os.open(pane_tty, os.O_WRONLY | os.O_NOCTTY)

        def send_path(value):
            os.write(pane_fd, b"\x1b]7;" + value + b"\x07")

        first = b"file:///tmp/owner-one"
        send_path(first)
        output = read_until(master, prefix + first + b"\x07")
        assert output.count(prefix) == 1, output
        read_quiet(master)

        send_path(first)
        assert prefix not in read_quiet(master)
        run(b"refresh-client", b"-t", tty)
        assert prefix not in read_quiet(master)

        second = b"file:///tmp/owner-two"
        send_path(second)
        output = read_until(master, prefix + second + b"\x07")
        assert output.count(prefix) == 1, output
        read_quiet(master)

        send_path(b"")
        output = read_until(master, prefix + b"\x07")
        assert output.count(prefix) == 1, output

        run(b"detach-client", b"-t", tty)
        os.waitpid(child, 0)
        child = None
        return first, second, b""
    finally:
        if pane_fd is not None:
            os.close(pane_fd)
        if child is not None:
            os.kill(child, signal.SIGTERM)
            os.waitpid(child, 0)
        if master is not None:
            os.close(master)
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="client-path-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = trace(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        assert candidate == trace(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)

print("client path owner CLI checks passed")
