#!/usr/bin/env python3
"""Exercise window_active_sessions_list as linked sessions change windows."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-active-sessions-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def active():
        return run("display-message", "-p", "-t", "alpha:0", "#{window_active_sessions_list}")

    try:
        for name in ("alpha", "beta", "écho"):
            run("new-session", "-d", "-s", name, "sleep 60")

        assert active() == b"alpha\n"
        run("link-window", "-d", "-s", "alpha:0", "-t", "beta:1")
        run("link-window", "-d", "-s", "alpha:0", "-t", "beta:2")
        run("link-window", "-d", "-s", "alpha:0", "-t", "écho:1")
        assert active() == b"alpha\n"

        run("select-window", "-t", "beta:1")
        assert active() == b"alpha,beta\n"
        run("select-window", "-t", "écho:1")
        assert active() == b"alpha,beta,\xc3\xa9cho\n"
        run("select-window", "-t", "beta:2")
        assert active() == b"alpha,beta,\xc3\xa9cho\n"

        run("select-window", "-t", "beta:0")
        assert active() == b"alpha,\xc3\xa9cho\n"
        run("select-window", "-t", "écho:0")
        assert active() == b"alpha\n"
        run("new-window", "-d", "-t", "alpha:1", "sleep 60")
        run("select-window", "-t", "alpha:1")
        assert active() == b"\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("active sessions format CLI checks passed")
