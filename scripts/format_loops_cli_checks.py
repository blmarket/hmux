#!/usr/bin/env python3
"""Exercise session, window, and pane format loops through display-message."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-loops-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def check(fmt, expected):
        actual = run("display-message", "-p", "-t", "first:0.0", fmt)
        assert actual == expected, (fmt, expected, actual)

    try:
        run("new-session", "-d", "-s", "first", "sleep", "30")
        run("new-session", "-d", "-s", "second", "sleep", "30")
        run("new-window", "-d", "-t", "first:1", "sleep", "30")
        run("split-window", "-d", "-t", "first:0", "sleep", "30")

        # No comma uses the full body for every item.
        check("#{S:#{session_name};}", b"first;second;\n")
        check("#{W:#{window_index};}", b"0;1;\n")
        check("#{P:#{pane_index};}", b"0;1;\n")

        # The split preserves nested commas and an escaped comma in both operands.
        check("#{S:#{?#{==:#{session_name},first},F,S};,active;}", b"F;S;\n")
        check("#{W:all#,mark;,active#,mark;}", b"active,mark;all,mark;\n")
        check("#{P:all#,mark;,active#,mark;}", b"active,mark;all,mark;\n")
        check("#{W:#{?#{==:#{window_index},0},Y,N};}", b"Y;N;\n")
        check("#{P:#{?#{==:#{pane_index},0},Y,N};}", b"Y;N;\n")

        # An attached control client gives S: an active session to select.
        control = subprocess.run(
            base + ["-C", "attach-session", "-t", "first"],
            input=b'display-message -p "#{S:#{session_name};,ACTIVE;}"\ndetach-client\n',
            env=env,
            capture_output=True,
            timeout=20,
        )
        assert control.returncode == 0, (control.returncode, control.stderr)
        assert b"\nACTIVE;second;\n" in control.stdout, control.stdout
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("format loop CLI checks passed")
