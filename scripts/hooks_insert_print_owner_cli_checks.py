#!/usr/bin/env python3
"""Compare the verbose hook command log with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, directory):
    directory.mkdir()
    socket = directory / "socket"
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C.UTF-8", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args, verbose=False):
        prefix = command + ([b"-v"] if verbose else [])
        result = subprocess.run(prefix + list(args), env=env, cwd=directory, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"hooks", b"sleep 60", verbose=True)
        run(b"set-hook", b"-g", b"after-new-window", b"set-option -g @first first")
        run(b"new-window", b"-d", b"-t", b"hooks:", b"sleep 60")
        assert run(b"show-options", b"-gqv", b"@first") == b"first\n"
        logs = list(directory.glob("tmux-server-*.log"))
        assert len(logs) == 1, logs
        marker = b"hooks_insert_one: hook after-new-window is: "
        lines = [line.split(marker, 1)[1] for line in logs[0].read_bytes().splitlines() if marker in line]
        assert len(lines) == 1, lines
        assert b"@first" in lines[0], lines
        return lines[0]
    finally:
        subprocess.run(command + [b"kill-server"], env=env, cwd=directory, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="hooks-insert-print-owner-") as tmp:
    directory = pathlib.Path(tmp)
    output = check(candidate, directory / "candidate")
    if baseline is not None:
        assert check(pathlib.Path(baseline).resolve(), directory / "baseline") == output

print("hooks insert print owner CLI checks passed")
