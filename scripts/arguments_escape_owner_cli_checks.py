#!/usr/bin/env python3
"""Exercise argument escaping through bind-key and list-keys on a private socket."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")

values = (
    b"",
    b"plain",
    b"~",
    b"~ leading",
    b" ",
    b'"',
    b"#",
    b"line\nnext",
    b"\xff",
    "é".encode(),
)
expected = (
    b"display-message ''\n"
    b"display-message plain\n"
    b"display-message \\~\n"
    b'display-message "\\~ leading"\n'
    b'display-message " "\n'
    b'display-message \\"\n'
    b"display-message \\#\n"
    b"display-message line\\nnext\n"
    b"display-message \\377\n"
    b"display-message \xc3\xa9\n"
)


def check(binary, socket):
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C.UTF-8", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [binary, b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"escape", b"sleep 60")
        for index, value in enumerate(values):
            run(b"bind-key", b"-T", b"escape", str(index).encode(), b"display-message", value)
        output = run(b"list-keys", b"-T", b"escape", b"-F", b"#{key_command}")
        assert output == expected, output
        return output
    finally:
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="arguments-escape-owner-") as tmp:
    output = check(candidate, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
        assert check(baseline_binary, pathlib.Path(tmp, "baseline-socket")) == output

print("arguments escape owner CLI checks passed")
