#!/usr/bin/env python3
"""Compare quote modifiers, including chained modifiers, with the pinned binary."""

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
        b"#{q:@quote}",
        b"#{q|s:@quote}",
        b"#{q|e:@quote}",
        b"#{q;q|e:@quote}",
        b"#{q|e;q:@quote}",
        b"#{q;q|s;q|e:@quote}",
        b"#{q|a:@quote}",
    )
    try:
        run(b"new-session", b"-d", b"-s", b"quotes", b"sleep", b"60")
        rows = []
        for value in (b"", b"a #\xff'b", b"line\n\t end"):
            run(b"set-option", b"-g", b"@quote", value)
            rows.append(tuple(run(b"display-message", b"-p", expression) for expression in expressions))
        assert rows[1][0] == b"a\\ \\#\xff\\'b\n", rows[1][0]
        assert rows[1][2] == b"a ##\xff'b\n", rows[1][2]
        assert rows[1][3] == b"a\\ \\##\xff\\'b\n", rows[1][3]
        return rows
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="format-quote-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("format quote owner CLI checks passed")
