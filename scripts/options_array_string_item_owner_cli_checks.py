#!/usr/bin/env python3
"""Exercise string array item ownership through the public option commands."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, directory):
    socket = os.fsencode(directory) + b"/socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [binary, b"-S", socket, b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"opts", b"sleep", b"60")
        option = b"update-environment"
        item = option + b"[7]"
        other = option + b"[8]"
        observed = []

        # Keep a second item alive while replacing and removing the first.
        run(b"set-option", b"-g", item, b"FIRST")
        run(b"set-option", b"-g", other, b"KEEP")
        observed.append((run(b"show-options", b"-gv", item), run(b"show-options", b"-gv", other)))

        run(b"set-option", b"-ga", item, b"-" + bytes([255]))
        observed.append((run(b"show-options", b"-gv", item), run(b"show-options", b"-gv", other)))

        run(b"set-option", b"-g", item, b"REPLACED")
        observed.append((run(b"show-options", b"-gv", item), run(b"show-options", b"-gv", other)))

        run(b"set-option", b"-gu", item)
        observed.append((run(b"show-options", b"-gv", item), run(b"show-options", b"-gv", other)))

        assert observed == [
            (b"FIRST\n", b"KEEP\n"),
            (b"FIRST-\xff\n", b"KEEP\n"),
            (b"REPLACED\n", b"KEEP\n"),
            (b"\n", b"KEEP\n"),
        ], observed
        return observed
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="options-array-string-item-owner-", dir="/tmp") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    actual = check(candidate, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        expected = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert actual == expected, (actual, expected)

print("options array string item owner CLI checks passed")
