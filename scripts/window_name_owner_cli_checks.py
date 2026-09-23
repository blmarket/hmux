#!/usr/bin/env python3
"""Compare explicit, spawned, broken-pane, and terminal-driven window names."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="window-name-owner-") as directory:
        base = [str(binary), "-S", str(pathlib.Path(directory) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def fail(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode != 0, (args, result.returncode, result.stderr)
            return result.stderr

        def names():
            return run("list-windows", "-t", "names", "-F", "#{window_index}|#{window_name}").splitlines()

        try:
            run("new-session", "-d", "-s", "names", "-n", "first", "sleep 60")
            assert names() == [b"0|first"]
            run("rename-window", "-t", "names:0", "renamed")
            assert names() == [b"0|renamed"]
            run("new-window", "-d", "-t", "names:", "-n", "second", "sleep 60")
            assert names() == [b"0|renamed", b"1|second"]

            run("split-window", "-d", "-t", "names:0.0", "sleep 60")
            run("break-pane", "-d", "-n", "broken", "-s", "names:0.1", "-t", "names:")
            assert names() == [b"0|renamed", b"1|second", b"2|broken"]

            run("split-window", "-d", "-t", "names:0.0", "sleep 60")
            run("break-pane", "-d", "-s", "names:0.1", "-t", "names:")
            before_terminal = names()
            assert len(before_terminal) == 4, before_terminal
            assert before_terminal[-1].startswith(b"3|") and len(before_terminal[-1]) > 2

            # tmux's ESC k title sequence reaches input_exit_rename and window_set_name.
            run("set-window-option", "-g", "allow-rename", "on")
            run("new-window", "-d", "-t", "names:", "-n", "terminal", "printf '\033kFromPane\033\\'; sleep 60")
            deadline = time.monotonic() + 5
            while True:
                after_terminal = names()
                if after_terminal[-1] == b"4|FromPane":
                    break
                assert time.monotonic() < deadline, after_terminal
                time.sleep(0.05)

            run("new-window", "-d", "-S", "-t", "names:", "-n", "second", "sleep 60")
            assert names() == after_terminal
            run("new-window", "-d", "-t", "names:", "-n", "duplicate", "sleep 60")
            run("new-window", "-d", "-t", "names:", "-n", "duplicate", "sleep 60")
            multiple = fail("new-window", "-d", "-S", "-t", "names:", "-n", "duplicate", "sleep 60")
            spawn_error = fail("new-window", "-d", "-t", "names:0", "-n", "failed", "sleep 60")

            before_terminal[-1] = b"3|$DEFAULT"
            after_terminal[-2] = b"3|$DEFAULT"
            return tuple(before_terminal + after_terminal + [multiple, spawn_error])
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("window name owner CLI checks passed")
