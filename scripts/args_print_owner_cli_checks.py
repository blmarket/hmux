#!/usr/bin/env python3
"""Compare printed hook arguments from private servers with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, socket):
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C.UTF-8", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"printer", b"sleep 60")
        run(b"set-hook", b"-g", b"after-display-message", b'set-option -gF @printed "#{hook_arguments}"')
        cases = (
            ((b"-p", b""), b"-p ''\n"),
            ((b"-p", b"\xff arg"), b'-p "\\377 arg"\n'),
            ((b"-p", b"plain"), b"-p plain\n"),
        )
        outputs = []
        for args, expected in cases:
            run(b"display-message", *args)
            printed = run(b"show-options", b"-gv", b"@printed")
            assert printed == expected, (args, printed, expected)
            outputs.append(printed)
        run(b"set-hook", b"-g", b"after-display-message", b'set-option -gF @flags "#{hook_flag_F}|#{hook_flag_p}|#{hook_flag_v}"')
        run(b"display-message", b"-p", b"-v", b"-F", b"tag")
        flags = run(b"show-options", b"-gv", b"@flags")
        assert flags == b"tag|1|1\n", flags
        outputs.append(flags)
        return outputs
    finally:
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="args-print-owner-") as tmp:
    output = check(candidate, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        assert check(pathlib.Path(baseline).resolve(), pathlib.Path(tmp, "baseline-socket")) == output

print("arguments print owner CLI checks passed")
