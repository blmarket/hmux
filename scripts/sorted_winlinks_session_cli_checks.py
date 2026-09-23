#!/usr/bin/env python3
"""Exercise caller-owned session winlink snapshots with nested format loops."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="sorted-winlinks-session-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "sorted", "-n", "delta", "sleep", "30")
        run("new-window", "-d", "-t", "sorted:3", "-n", "alpha", "sleep", "30")
        run("new-window", "-d", "-t", "sorted:7", "-n", "beta", "sleep", "30")
        run("select-window", "-t", "sorted:3")

        assert run("list-windows", "-t", "sorted", "-O", "index", "-F", "#{window_index}") == (
            b"0\n3\n7\n"
        )
        assert run("list-windows", "-t", "sorted", "-O", "index", "-r", "-F", "#{window_index}") == (
            b"7\n3\n0\n"
        )
        assert run("list-windows", "-t", "sorted", "-O", "name", "-F", "#{window_name}") == (
            b"alpha\nbeta\ndelta\n"
        )

        # W: sorts by index inside the name-sorted list-windows snapshot. The
        # nested W: recursion repeats inside that inner loop.
        nested = "#{W:#{W:#{window_index};}|#{window_index}/#{loop_index}/#{loop_last_flag};}"
        inner = b"0;3;7;|0/0/0;0;3;7;|3/1/0;0;3;7;|7/2/1;"
        assert run("list-windows", "-t", "sorted", "-O", "name", "-F", nested + "|#{window_name}") == (
            b"".join(inner + b"|" + name + b"\n" for name in (b"alpha", b"beta", b"delta"))
        )

        run("kill-window", "-t", "sorted:3")
        assert run("list-windows", "-t", "sorted", "-O", "index", "-F", "#{window_index}") == (
            b"0\n7\n"
        )
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("sorted session winlink CLI checks passed")
