#!/usr/bin/env python3
"""Compare client MSG_COMMAND payload and error paths with the pinned baseline."""

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
        result = subprocess.run(
            command + list(args), env=env, stdin=subprocess.DEVNULL,
            capture_output=True, timeout=15,
        )
        return result.returncode, result.stdout, result.stderr

    try:
        assert run(b"new-session", b"-d", b"-s", b"command", b"sleep 60") == (0, b"", b"")
        results = [
            run(b"display-message", b"-p", b"normal"),
            run(b"display-message", b"-p", b"\xff\xfe"),
            run(),  # argc == 0 still sends a four-byte MSG_COMMAND payload.
            run(b"display-message", b"-p", b"X" * 16340),
            run(b"display-message", b"-p", b"X" * 16345),
            run(b"display-message", b"-p", b"X" * 16384),
            run(b"not-a-command"),
            run(b"display-message", b"-z"),
            run(b"display-message", b"-p", b"first;"),
            run(b"display-message", b"-p", b"semi\\;"),
            run(b"display-message", b"-p", b"left;", b"display-message", b"-p", b"right"),
        ]
        assert run(
            b"set-option", b"-s", b"command-alias[0]",
            b"short=display-message -p aliased",
        ) == (0, b"", b"")
        results.append(run(b"short"))
        assert results == [
            (0, b"normal\n", b""),
            (0, b"\xff\xfe\n", b""),
            (1, b"", b"open terminal failed: not a terminal\n"),
            (0, b"X" * 16340 + b"\n", b""),
            (1, b"", b"failed to send command\n"),
            (1, b"", b"command too long\n"),
            (1, b"", b"unknown command: not-a-command\n"),
            (1, b"", b"command display-message: unknown flag -z\n"),
            (0, b"first\n", b""),
            (0, b"semi;\n", b""),
            (0, b"left\nright\n", b""),
            (0, b"aliased\n", b""),
        ], [(code, len(stdout), stderr) for code, stdout, stderr in results]
        return results
    finally:
        run(b"kill-server")


with tempfile.TemporaryDirectory(prefix="client-command-payload-") as tmp:
    output = check(candidate, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
        assert check(baseline_binary, pathlib.Path(tmp, "baseline-socket")) == output

print("client command payload CLI checks passed")
