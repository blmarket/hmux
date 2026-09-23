#!/usr/bin/env python3
"""Compare complete format output assembly with the pinned binary."""

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

    try:
        run(b"new-session", b"-d", b"-s", b"owner", b"sleep", b"60")
        run(b"set-option", b"-g", b"@bytes", b"A \xff#B")
        cases = (
            (b"literal", b"literal\n"),
            (b"x" * 300 + b"#", b"x" * 300 + b"\n"),
            (b"L#{@bytes}R", b"LA \xff#BR\n"),
            (b"#{q:@bytes}", b"A\\ \xff\\#B\n"),
            (b"#{?#{==:#{session_name},owner},yes,no}", b"yes\n"),
            (b"#[fg=red]red#[default]", None),
            (b"##[fg=red]red", None),
            (b"#z #, ## #}", None),
            (b"\xff:#{@bytes}", b"\xff:A \xff#B\n"),
        )
        output = []
        for expression, expected in cases:
            actual = run(b"display-message", b"-p", expression)
            if expected is not None:
                assert actual == expected, (expression, actual, expected)
            output.append(actual)
        return tuple(output)
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="format-expand-output-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("format expand output owner CLI checks passed")
