#!/usr/bin/env python3
"""Compare JSON string values and recursive node cleanup with the baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, socket):
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [binary, b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        return result.returncode, result.stdout, result.stderr

    try:
        code, out, err = run(b"new-session", b"-d", b"-s", b"json-node", b"sleep", b"30")
        assert code == 0, (code, out, err)

        def display(value):
            return run(b"display-message", b"-p", b"-j", b"-l", value)

        valid = b'{"z":[{"b":"\xff"},{"c":[{"d":"\xc2\xa3"}]}],"a":"x"}'
        expected = b'{"a":"x","z":[{"b":"\xff"},{"c":[{"d":"\xc2\xa3"}]}]}\n'
        result = display(valid)
        assert result[0] == 0 and result[1] == expected, result

        # Empty strings are rejected by the existing parser before a node exists.
        empty = display(b'{"outer":[{"value":""}]}')
        assert empty[0] != 0 and empty[2], empty

        # Fail after creating several nested string nodes, then parse again on
        # the same server to exercise recursive destruction and recovery.
        invalid = display(b'{"outer":[{"left":"one"},{"right":{"deep":"two"}}],"broken":[{"leaf":"three"},]}')
        assert invalid[0] != 0 and invalid[2], invalid
        again = display(valid)
        assert again[0] == 0 and again[1] == expected, again
        return result, empty, invalid, again
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="json-node-owner-", dir=root / "target") as tmp:
    candidate = check(binary, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), pathlib.Path(tmp, "baseline-socket"))
        assert candidate == reference, (candidate, reference)

print("JSON node owner CLI checks passed")
