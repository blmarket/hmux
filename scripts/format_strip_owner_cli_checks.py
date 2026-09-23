#!/usr/bin/env python3
"""Compare time format stripping with the pinned binary."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh", TZ="UTC")


def check(executable, directory):
    base = [executable, b"-S", os.fsencode(directory) + b"/socket", b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    rows = []
    try:
        run(b"new-session", b"-d", b"-s", b"fmt", b"sleep", b"60")
        run(b"set-option", b"-g", b"@ts", b"1609459200")
        cases = (
            (b"A#,B", b"A,B\n"),  # outside brackets: discard the escape
            (b"A#", b"A\n"),  # terminal NUL is in the old strchr set
            (b"A#xB", b"A#xB\n"),  # ordinary byte after #
            (b"A#:#B", b"A:#B\n"),
            (b"A#{}B", b"A#{}B\n"),  # retain # inside a nested format
            (b"\xff#,B", b"\xff,B\n"),  # non-UTF-8 input stays byte-for-byte
            (b"YEAR=%Y", b"YEAR=2021\n"),  # strftime sees the owned format
        )
        for value, expected in cases:
            run(b"set-option", b"-g", b"@fmt", value)
            got = run(b"display-message", b"-p", b"#{t|f|#{@fmt}|:@ts}")
            assert got == expected, (value, got, expected)
            rows.append(got)

        # A later t/f modifier replaces the first owned format.
        got = run(b"display-message", b"-p", b"#{t|f|#{@fmt}|;t|f|Z|:@ts}")
        assert got == b"Z\n", got
        rows.append(got)

        # The conditional path also borrows the last time format in format_find.
        for expression, expected in (
            (b"#{t|f|#{@fmt}|:?@ts,yes,no}", b"yes\n"),
            (b"#{t|f|#{@fmt}|:?@missing,yes,no}", b"no\n"),
        ):
            got = run(b"display-message", b"-p", expression)
            assert got == expected, (expression, got, expected)
            rows.append(got)
        return tuple(rows)
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="format-strip-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("format strip owner CLI checks passed")
