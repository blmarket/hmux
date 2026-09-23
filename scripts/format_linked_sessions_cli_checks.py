#!/usr/bin/env python3
"""Exercise the window_linked_sessions_list format callback through the CLI."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-linked-sessions-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def linked():
        return run("display-message", "-p", "-t", "alpha:0", "#{window_linked_sessions_list}")

    try:
        for name in ("alpha", "beta", "écho"):
            run("new-session", "-d", "-s", name, "sleep", "60")

        assert linked() == b"alpha\n"
        run("link-window", "-s", "alpha:0", "-t", "beta:1")
        run("link-window", "-s", "alpha:0", "-t", "beta:2")
        run("link-window", "-s", "alpha:0", "-t", "écho:1")
        assert linked() == b"alpha,beta,beta,\xc3\xa9cho\n"

        run("unlink-window", "-t", "beta:1")
        assert linked() == b"alpha,beta,\xc3\xa9cho\n"
        run("kill-session", "-t", "écho")
        assert linked() == b"alpha,beta\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("linked sessions format CLI checks passed")
