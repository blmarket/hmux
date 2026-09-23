#!/usr/bin/env python3
"""Exercise session_group_list as grouped sessions change."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-session-group-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def group(session):
        return run("display-message", "-p", "-t", session, "#{session_group_list}")

    try:
        run("new-session", "-d", "-s", "alpha", "sleep 60")
        assert group("alpha") == b"\n"

        run("new-session", "-d", "-s", "beta", "-t", "alpha")
        assert group("alpha") == b"alpha,beta\n"
        assert group("beta") == b"alpha,beta\n"

        run("new-session", "-d", "-s", "écho", "-t", "alpha")
        assert group("alpha") == b"alpha,beta,\xc3\xa9cho\n"
        run("rename-session", "-t", "beta", "delta")
        assert group("alpha") == b"alpha,delta,\xc3\xa9cho\n"
        run("kill-session", "-t", "écho")
        assert group("alpha") == b"alpha,delta\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("session group list CLI checks passed")
