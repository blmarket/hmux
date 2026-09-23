#!/usr/bin/env python3
"""Compare escaped format modifier arguments with the pinned binary."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def check(executable, directory):
    base = [executable, b"-S", os.fsencode(directory) + b"/socket", b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    expressions = (
        b"#{=4:@quote}",                  # unpunctuated modifier argument
        b"#{q|s:@quote}",                  # punctuation-delimited argument
        b"#{s|a#,b|Z:@quote}",             # escaped comma in the first argument
        b"#{s|a#,b|Z;=6:@quote}",         # chained modifiers
        b"#{l:a#,#b}",                   # literal output still uses a C-owned return
    )
    try:
        run(b"new-session", b"-d", b"-s", b"fmt", b"sleep", b"60")
        run(b"set-option", b"-g", b"@quote", b"a,b#c \xffz")
        rows = tuple(run(b"display-message", b"-p", expression) for expression in expressions)
        assert rows == (
            b"a,b#\n",
            b"'a,b#c \xffz'\n",
            b"Z#c \xffz\n",
            b"Z#c z\n",
            b"a,#b\n",
        ), rows
        return rows
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="format-unescape-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("format unescape owner CLI checks passed")
