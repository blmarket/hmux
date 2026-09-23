#!/usr/bin/env python3
"""Check sorted paste-buffer lists across creation, filters, and deletion."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="sorted-buffers-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "buffers", "sleep", "30")
        assert run("list-buffers", "-F", "#{buffer_name}") == b""

        run("set-buffer", "-b", "beta", "BBB")
        run("set-buffer", "-b", "alpha", "A")
        run("set-buffer", "-b", "gamma", "GG")

        assert run("list-buffers", "-O", "order", "-F", "#{buffer_name}") == (
            b"gamma\nalpha\nbeta\n"
        )
        assert run("list-buffers", "-O", "order", "-r", "-F", "#{buffer_name}") == (
            b"beta\nalpha\ngamma\n"
        )
        assert run("list-buffers", "-O", "name", "-F", "#{buffer_name}") == (
            b"alpha\nbeta\ngamma\n"
        )
        assert run("list-buffers", "-O", "size", "-F", "#{buffer_name}") == (
            b"alpha\ngamma\nbeta\n"
        )
        assert run("list-buffers", "-O", "name", "-r", "-F", "#{buffer_name}") == (
            b"gamma\nbeta\nalpha\n"
        )
        assert run(
            "list-buffers", "-O", "name", "-f", "#{==:#{buffer_name},beta}",
            "-F", "#{buffer_name}:#{buffer_size}",
        ) == b"beta:3\n"

        run("delete-buffer", "-b", "beta")
        assert run("list-buffers", "-O", "name", "-F", "#{buffer_name}") == (
            b"alpha\ngamma\n"
        )
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("sorted buffer CLI checks passed")
