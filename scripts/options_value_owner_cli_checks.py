#!/usr/bin/env python3
"""Compare scalar, array, and command option rendering with the pinned binary."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary_path, directory):
    socket = os.fsencode(directory) + b"/socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [binary_path, b"-S", socket, b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"opts", b"sleep", b"60")

        # Empty first item and non-UTF8 later item exercise the join boundary.
        run(b"set-option", b"-g", b"update-environment", b"")
        empty_array = run(b"display-message", b"-p", b"#{update-environment}")
        assert empty_array == b"\n", empty_array
        run(b"set-option", b"-g", b"update-environment[0]", b"")
        run(b"set-option", b"-g", b"update-environment[1]", b"ONE")
        run(b"set-option", b"-g", b"update-environment[2]", b"\xff")
        run(b"set-option", b"-g", b"update-environment[\x80]", b"TAIL")
        joined = run(b"display-message", b"-p", b"#{update-environment}")
        assert joined == b" ONE \xff TAIL\n", joined
        array = run(b"show-options", b"-gv", b"update-environment")
        assert array == b"\nONE\n\xff\nTAIL\n", array
        indexed = run(b"show-options", b"-gv", b"update-environment[\x80]")
        assert indexed == b"TAIL\n", indexed
        missing = run(b"show-options", b"-gv", b"update-environment[99]")
        assert missing == b"\n", missing

        for name, value in (
            (b"history-limit", b"1234"),
            (b"mouse", b"on"),
            (b"status-keys", b"vi"),
            (b"status-fg", b"red"),
            (b"status", b"2"),
            (b"prefix", b"C-a"),
            (b"@raw", b"\xff"),
        ):
            run(b"set-option", b"-g", name, value)
        scalar_format = run(
            b"display-message",
            b"-p",
            b"#{history-limit}|#{mouse}|#{status-keys}|#{status-fg}|#{status}|#{prefix}|#{@raw}",
        )
        assert scalar_format == b"1234|1|vi|red|2|C-a|\xff\n", scalar_format
        flag_text = run(b"show-options", b"-gv", b"mouse")
        assert flag_text == b"on\n", flag_text
        scalar_bytes = run(b"show-options", b"-gv", b"@raw")
        assert scalar_bytes == b"\xff\n", scalar_bytes

        run(
            b"set-hook",
            b"-g",
            b"after-set-option[7]",
            b"display-message one ; display-message two",
        )
        command = run(b"display-message", b"-p", b"#{after-set-option}")
        assert command == b"display-message one ; display-message two\n", command
        hook = run(b"show-hooks", b"-g", b"after-set-option[7]")
        assert hook == b"after-set-option[7] display-message one ; display-message two\n", hook

        return empty_array, joined, array, indexed, missing, scalar_format, flag_text, scalar_bytes, command, hook
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="options-value-owner-", dir="/tmp") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("options value owner CLI checks passed")
