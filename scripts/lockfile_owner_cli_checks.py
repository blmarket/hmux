#!/usr/bin/env python3
"""Exercise daemon lockfile ownership with a private non-UTF-8 socket.

Set HMUX_BINARY for the candidate and HMUX_BASELINE_BINARY to compare with
the pinned pre-migration binary.
"""

import fcntl
import os
import pathlib
import subprocess
import tempfile
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
BINARY = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve())
BASELINE = os.environ.get("HMUX_BASELINE_BINARY")


def check_case(binary, directory, case):
    socket_path = os.fsencode(directory) + b"/socket-\xff"
    lock_path = socket_path + b".lock"
    env = os.environb.copy()
    env.update({b"HOME": os.fsencode(directory), b"TERM": b"xterm-256color", b"SHELL": b"/bin/sh"})
    env.pop(b"TMUX", None)
    base = [binary, b"-S", socket_path, b"-f", b"/dev/null"]

    def run(*args):
        return subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)

    lock_fd = None
    try:
        if case == "unavailable":
            os.mkdir(lock_path)
        elif case == "contended":
            lock_fd = os.open(lock_path, os.O_WRONLY | os.O_CREAT, 0o600)
            fcntl.flock(lock_fd, fcntl.LOCK_EX)

        process = subprocess.Popen(
            base + [b"new-session", b"-d", b"-s", b"locked", b"sleep", b"60"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if case == "contended":
            time.sleep(0.2)
            assert process.poll() is None, "client unexpectedly completed while lock was held"
            fcntl.flock(lock_fd, fcntl.LOCK_UN)
            os.close(lock_fd)
            lock_fd = None
        stdout, stderr = process.communicate(timeout=15)
        assert process.returncode == 0, (stdout, stderr)
        assert os.path.exists(socket_path), socket_path
        if case == "unavailable":
            assert os.path.isdir(lock_path), lock_path
        else:
            assert not os.path.exists(lock_path), lock_path

        listed = run(b"list-sessions", b"-F", b"#{session_name}")
        assert listed.returncode == 0, (listed.stdout, listed.stderr)
        assert listed.stdout == b"locked\n", listed.stdout
        return (process.returncode, stdout, stderr, listed.stdout, listed.stderr)
    finally:
        if lock_fd is not None:
            fcntl.flock(lock_fd, fcntl.LOCK_UN)
            os.close(lock_fd)
        run(b"kill-server")


with tempfile.TemporaryDirectory(prefix="lockfile-owner-", dir="/tmp") as tmp:
    for case in ("normal", "contended", "unavailable"):
        candidate_dir = pathlib.Path(tmp, f"candidate-{case}")
        candidate_dir.mkdir()
        candidate = check_case(BINARY, candidate_dir, case)
        if BASELINE:
            baseline_dir = pathlib.Path(tmp, f"baseline-{case}")
            baseline_dir.mkdir()
            expected = check_case(os.fsencode(pathlib.Path(BASELINE).resolve()), baseline_dir, case)
            assert candidate == expected, (case, candidate, expected)

print("lockfile owner CLI checks passed")
