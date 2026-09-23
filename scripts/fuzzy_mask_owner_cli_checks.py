#!/usr/bin/env python3
"""Compare fuzzy format masks and positions with a pinned binary."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
cases = [
    (b"#{m/p:bc,abcde}", b"1,2\n"),
    (b"#{m/z:bc,abcde}", b"1\n"),
    (b"#{m/p:q,abcde}", b"\n"),
    (b"#{m/z:q,abcde}", b"0\n"),
    (b"#{m/p:,abcde}", b"\n"),
    (b"#{m/z:,abcde}", b"1\n"),
    (b"#{m/p:\xc3\xa9,ab\xc3\xa9c}", b"2\n"),
    (b"#{m/p:\xff,ab\xffc}", b"\n"),
]


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="fuzzy-mask-owner-") as tmp:
        env = os.environb.copy()
        env.update({b"TERM": b"xterm-256color", b"TMUX": b"", b"SHELL": b"/bin/sh"})
        base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp, "socket")), b"-f", b"/dev/null"]

        def run(*args):
            return subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)

        try:
            started = run(b"new-session", b"-d", b"-s", b"fuzzy", b"sleep 30")
            assert started.returncode == 0, started.stderr
            output = []
            for expression, expected in cases:
                result = run(b"display-message", b"-p", expression)
                assert (result.returncode, result.stdout, result.stderr) == (0, expected, b""), (
                    expression, result.returncode, result.stdout, result.stderr,
                )
                output.append(result.stdout)
            return output
        finally:
            run(b"kill-server")


assert trace(candidate) == trace(baseline)
print("fuzzy mask owner CLI checks passed")
