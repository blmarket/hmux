#!/usr/bin/env python3
"""Exercise recursive JSON node cleanup through display-message."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="json-node-owner-", dir=root / "target") as tmp:
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args, ok=True):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        if ok:
            assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result

    try:
        run("new-session", "-d", "-s", "json-node", "sleep", "30")
        valid = b'{"z":[{"b":true},{"c":[{"d":"text"}]}],"a":"x"}'
        result = run("display-message", "-p", "-j", "-l", valid.decode())
        assert result.stdout == b'{"a":"x","z":[{"b":true},{"c":[{"d":"text"}]}]}\n'

        invalid = b'{"z":[{"b":true},]}'
        result = run("display-message", "-p", "-j", "-l", invalid.decode(), ok=False)
        assert result.returncode != 0
        assert result.stderr

        # A parse failure must leave the server usable for another parse.
        assert run("display-message", "-p", "-j", "-l", valid.decode()).stdout == (
            b'{"a":"x","z":[{"b":true},{"c":[{"d":"text"}]}]}\n'
        )
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("JSON node owner CLI checks passed")
