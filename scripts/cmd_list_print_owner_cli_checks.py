#!/usr/bin/env python3
"""Check command-list rendering on a private socket against the pinned baseline."""

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
        run(
            b"bind-key", b"-T", b"printer", b"F11",
            b"display-message one ; display-message two ;; display-message three",
        )
        run(b"bind-key", b"-T", b"printer", b"F12", b"display-message", b"\xff")
        run(
            b"bind-key", b"-T", b"printer", b"F10",
            b"if-shell true { display-message first ; display-message second }",
        )
        keys = run(b"list-keys", b"-T", b"printer", b"-F", b"#{key_command}")
        assert keys == (
            b"if-shell true { display-message first ; display-message second }\n"
            b"display-message one \\; display-message two \\; display-message three\n"
            b"display-message \\377\n"
        ), keys

        run(
            b"set-hook", b"-g", b"client-attached",
            b"display-message one ; display-message two ;; display-message three",
        )
        hooks = run(b"show-hooks", b"-g", b"client-attached")
        assert hooks == (
            b"client-attached[0] display-message one ; display-message two ; "
            b"display-message three\n"
        ), hooks
        return keys, hooks
    finally:
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="cmd-list-print-owner-") as tmp:
    output = check(candidate, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
        assert check(baseline_binary, pathlib.Path(tmp, "baseline-socket")) == output

print("command-list print owner CLI checks passed")
