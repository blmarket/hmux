#!/usr/bin/env python3
"""Compare copy-mode backing refresh and view-mode teardown with a baseline."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="copy-backing-owner-", dir=root / "target") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def wait_mode(expected):
            deadline = time.monotonic() + 5
            while True:
                mode = run("display-message", "-p", "-t", "owner:0.0", "#{pane_in_mode}|#{pane_mode}")
                if mode == expected:
                    return mode
                assert time.monotonic() < deadline, (expected, mode)
                time.sleep(0.02)

        def capture_until(marker):
            deadline = time.monotonic() + 5
            while True:
                output = run("capture-pane", "-p", "-t", "owner:0.0")
                if marker in output:
                    return output
                assert time.monotonic() < deadline, (marker, output)
                time.sleep(0.02)

        try:
            run("new-session", "-d", "-x", "40", "-y", "8", "-s", "owner", "printf 'first\\nsecond\\n'; sleep 30")
            capture_until(b"second")
            run("copy-mode", "-t", "owner:0.0")
            copy_mode = wait_mode(b"1|copy-mode\n")
            before = run("capture-pane", "-p", "-t", "owner:0.0")
            assert b"first" in before and b"second" in before, before
            run("resize-window", "-t", "owner:0", "-x", "31", "-y", "6")
            run("send-keys", "-t", "owner:0.0", "-X", "refresh-now")
            after = run("capture-pane", "-p", "-t", "owner:0.0")
            size = run("list-panes", "-t", "owner:0", "-F", "#{pane_width}x#{pane_height}")
            assert size == b"31x6\n", size
            run("send-keys", "-t", "owner:0.0", "-X", "cancel")
            left_copy = wait_mode(b"0|\n")

            run("run-shell", "-t", "owner:0.0", "printf 'view-one\\nview-two\\n'")
            view_mode = wait_mode(b"1|view-mode\n")
            run("send-keys", "-t", "owner:0.0", "-X", "cancel")
            left_view = wait_mode(b"0|\n")
            return copy_mode, before, after, size, left_copy, view_mode, left_view
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


reference = trace(baseline)
observed = trace(candidate)
assert observed == reference, list(zip(reference, observed))
print("window copy backing owner CLI checks passed")
