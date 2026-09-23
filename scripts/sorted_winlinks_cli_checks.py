#!/usr/bin/env python3
"""Exercise all-session winlink sorting and the window switch mode."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="sorted-winlinks-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "alpha", "-n", "lemon", "sleep 60")
        run("new-window", "-d", "-t", "alpha:1", "-n", "zebra", "sleep 60")
        run("new-session", "-d", "-s", "beta", "-t", "alpha")
        run("new-session", "-d", "-s", "gamma", "-n", "apple", "sleep 60")

        rows = run(
            "list-windows", "-a", "-O", "name", "-F",
            "#{window_name}|#{session_name}:#{window_index}|#{line}",
        ).splitlines()
        assert [row.split(b"|")[0] for row in rows] == [
            b"apple", b"lemon", b"lemon", b"zebra", b"zebra",
        ], rows
        assert {row.split(b"|")[1] for row in rows} == {
            b"alpha:0", b"alpha:1", b"beta:0", b"beta:1", b"gamma:0",
        }, rows
        assert all(row.endswith(b"|5") for row in rows), rows

        reversed_rows = run(
            "list-windows", "-a", "-O", "name", "-r", "-F", "#{window_name}"
        ).splitlines()
        assert reversed_rows == [b"zebra", b"zebra", b"lemon", b"lemon", b"apple"], reversed_rows

        # The mode builds another all-session snapshot into its owned item list.
        run("switch-mode", "-w", "-t", "alpha:0.0", "-F", "#{session_name}:#{window_index}")
        assert run("display-message", "-p", "-t", "alpha:0.0", "#{pane_mode}") == b"switch-mode\n"
        run("resize-pane", "-t", "alpha:0.0", "-x", "70")
        assert run("display-message", "-p", "-t", "alpha:0.0", "#{pane_mode}") == b"switch-mode\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("sorted winlinks CLI checks passed")
