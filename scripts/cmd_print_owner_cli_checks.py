#!/usr/bin/env python3
"""Compare printed command logs from private servers with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, socket):
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C.UTF-8", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [binary, b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"printer", b"sleep 60")
        run(b"list-sessions")
        assert run(b"display-message", b"-p", b"") == b"\n"
        assert run(b"display-message", b"-p", b"\xff arg") == b"\xff arg\n"
        assert run(b"if-shell", b"-F", b"1", b"display-message -p nested") == b"nested\n"
        messages = run(b"show-messages")
        printed = [line.split(b" command: ", 1)[1] for line in messages.splitlines() if b" command: " in line]
        assert printed == [
            b"show-messages",
            b"display-message -p nested",
            b'if-shell -F 1 "display-message -p nested"',
            b'display-message -p "\\377 arg"',
            b"display-message -p ''",
            b"list-sessions",
            b'new-session -d -s printer "sleep 60"',
        ], printed
        return printed
    finally:
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="cmd-print-owner-") as tmp:
    output = check(candidate, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
        assert check(baseline_binary, pathlib.Path(tmp, "baseline-socket")) == output

print("command print owner CLI checks passed")
