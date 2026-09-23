#!/usr/bin/env python3
"""Compare parsed command-list argument cache behavior with the baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def check(binary, directory):
    directory.mkdir()
    socket = directory / "socket"
    config = directory / "commands.conf"
    config.write_bytes(
        b"if-shell -F 1 { display-message -p 'cache yes' } "
        b"{ display-message -p 'cache no' }\n"
        b"if-shell -F 0 { display-message -p 'cache yes' } "
        b"{ display-message -p 'cache no' }\n"
    )
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"cache", b"sleep 30")
        output = run(b"source-file", os.fsencode(config))
        assert output == b"cache yes\ncache no\n", output
        return output
    finally:
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="args-cache-owner-") as temporary:
    directory = pathlib.Path(temporary)
    result = check(candidate, directory / "candidate")
    assert check(baseline, directory / "baseline") == result

print("arguments cached commands CLI checks passed")
