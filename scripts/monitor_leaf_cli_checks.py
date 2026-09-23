#!/usr/bin/env python3
"""Exercise session monitor pane/window leaves and generation sweeps."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")


def check(binary):
    with tempfile.TemporaryDirectory(prefix="monitor-leaf-", dir=root / "target") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def counts(session):
            rows = run("show-hooks", "-t", session, "-B", "-F", "#{option_name}=#{hook_fire_count}")
            return dict(row.split(b"=", 1) for row in rows.splitlines())

        def wait_counts(session, expected):
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline:
                actual = counts(session)
                if actual == expected:
                    return actual
                time.sleep(0.05)
            raise AssertionError((actual, expected))

        try:
            run("new-session", "-d", "-s", "mon", "sleep", "30")
            run("set-hook", "-t", "mon", "-B", "@watchpane:%*:#{session_name}", "true")
            run("set-hook", "-t", "mon", "-B", "@watchwindow:@*:#{session_name}", "true")
            zero = {b"@watchpane": b"0", b"@watchwindow": b"0"}
            assert counts("mon") == zero
            time.sleep(1.5)  # Initial timer check installs one leaf for each kind.

            run("rename-session", "-t", "mon", "mon2")
            one = {b"@watchpane": b"1", b"@watchwindow": b"1"}
            wait_counts("mon2", one)

            pane = run("split-window", "-d", "-P", "-F", "#{pane_id}", "-t", "mon2:0", "sleep", "30").strip()
            window = run("new-window", "-d", "-P", "-F", "#{window_id}", "-t", "mon2", "sleep", "30").strip()
            assert pane.startswith(b"%") and window.startswith(b"@"), (pane, window)
            time.sleep(1.5)  # Allow both new leaves to enter the monitor indexes.
            run("kill-pane", "-t", pane.decode("ascii"))
            run("kill-window", "-t", window.decode("ascii"))
            time.sleep(1.5)  # The next generation removes the stale leaves.

            run("rename-session", "-t", "mon2", "mon3")
            two = {b"@watchpane": b"2", b"@watchwindow": b"2"}
            return wait_counts("mon3", two)
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


actual = check(candidate)
if baseline is not None:
    expected = check(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("monitor leaf CLI checks passed")
