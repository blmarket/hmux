#!/usr/bin/env python3
"""Exercise directional pane selection and candidate priority on a private socket."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="pane-navigation-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, text=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout.strip()

    def pane_ids():
        return run("list-panes", "-F", "#{pane_id}").splitlines()

    def active_pane():
        return run("display-message", "-p", "#{pane_id}")

    try:
        run("new-session", "-d", "-s", "nav", "-x", "80", "-y", "24", "sleep", "60")
        original = pane_ids()[0]
        run("select-pane", "-U")  # No candidates in a one-pane window.
        assert active_pane() == original

        run("split-window", "-h", "-d", "sleep", "60")
        left, right = pane_ids()
        run("split-window", "-v", "-d", "-t", left, "sleep", "60")
        run("split-window", "-v", "-d", "-t", right, "sleep", "60")
        top_left, bottom_left, top_right, bottom_right = pane_ids()
        run("select-pane", "-t", top_left)
        for direction, expected in (
            ("R", top_right),
            ("D", bottom_right),
            ("L", bottom_left),
            ("U", top_left),
        ):
            run("select-pane", "-" + direction)
            assert active_pane() == expected, (direction, active_pane(), expected)

        # One full-height pane borders two candidates. The most recently active wins.
        run("new-window", "-n", "priority", "sleep", "60")
        run("split-window", "-h", "-d", "sleep", "60")
        left, right = pane_ids()
        run("split-window", "-v", "-d", "-t", right, "sleep", "60")
        _, top_right, bottom_right = pane_ids()
        run("select-pane", "-R")
        assert active_pane() == top_right  # Equal activity keeps list order.
        for wanted in (bottom_right, top_right):
            run("select-pane", "-t", wanted)
            run("select-pane", "-t", left)
            run("select-pane", "-R")
            assert active_pane() == wanted, (wanted, active_pane())
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)
        socket.unlink(missing_ok=True)

print("pane navigation CLI checks passed")
