#!/usr/bin/env python3
"""Check sorted session snapshots, including a nested session format loop."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="sorted-sessions-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        for name in ("zeta", "alpha", "beta"):
            run("new-session", "-d", "-s", name, "sleep", "30")

        assert run("list-sessions", "-O", "name", "-F", "#{session_name}") == (
            b"alpha\nbeta\nzeta\n"
        )
        assert run("list-sessions", "-O", "name", "-r", "-F", "#{session_name}") == (
            b"zeta\nbeta\nalpha\n"
        )
        assert run(
            "list-sessions", "-O", "name", "-f", "#{==:#{session_name},beta}",
            "-F", "#{session_name}",
        ) == b"beta\n"

        # The inner S: sort must not overwrite the outer list-sessions snapshot.
        loop = run("display-message", "-p", "-t", "alpha", "#{S:#{session_name};}").strip()
        assert run(
            "list-sessions", "-O", "name", "-F", "#{S:#{session_name};}|#{session_name}",
        ) == b"".join(loop + b"|" + name + b"\n" for name in (b"alpha", b"beta", b"zeta"))

        run("kill-session", "-t", "beta")
        assert run("list-sessions", "-O", "name", "-F", "#{session_name}") == (
            b"alpha\nzeta\n"
        )
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("sorted session CLI checks passed")
