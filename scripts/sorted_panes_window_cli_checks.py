#!/usr/bin/env python3
"""Check per-window pane snapshots, including a nested pane format loop."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="sorted-panes-window-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "panes", "sleep", "30")
        run("split-window", "-d", "-t", "panes:0", "sleep", "30")
        run("split-window", "-d", "-t", "panes:0", "sleep", "30")

        assert run("list-panes", "-t", "panes:0", "-O", "index", "-F", "#{pane_index}") == b"0\n1\n2\n"
        assert run("list-panes", "-t", "panes:0", "-O", "index", "-r", "-F", "#{pane_index}") == b"2\n1\n0\n"
        assert run(
            "list-panes", "-t", "panes:0", "-O", "index",
            "-f", "#{==:#{pane_index},1}", "-F", "#{pane_index}",
        ) == b"1\n"

        # The inner P: call must leave the outer reverse-sorted list intact.
        loop = run("display-message", "-p", "-t", "panes:0.0", "#{P:#{pane_index};}").strip()
        nested = run(
            "list-panes", "-t", "panes:0", "-O", "index", "-r",
            "-F", "#{P:#{pane_index};}|#{pane_index}",
        )
        assert nested == b"".join(loop + b"|" + str(i).encode() + b"\n" for i in (2, 1, 0)), nested
        loop_metadata = run(
            "display-message", "-p", "-t", "panes:0.0",
            "#{P:#{pane_index}:#{loop_index}:#{loop_last_flag};}",
        ).strip().rstrip(b";").split(b";")
        assert {part.split(b":", 1)[0] for part in loop_metadata} == {b"0", b"1", b"2"}
        assert [part.rsplit(b":", 2)[1:] for part in loop_metadata] == [
            [b"0", b"0"], [b"1", b"0"], [b"2", b"1"]
        ], loop_metadata

        run("kill-pane", "-t", "panes:0.2")
        assert run("list-panes", "-t", "panes:0", "-O", "index", "-F", "#{pane_index}") == b"0\n1\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("sorted panes per window CLI checks passed")
