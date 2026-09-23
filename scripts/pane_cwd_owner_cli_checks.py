#!/usr/bin/env python3
"""Compare pane cwd creation, respawn, and byte preservation with tmux."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.fsencode(pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve())
env = os.environb | {b"TERM": b"xterm-256color", b"LC_ALL": b"C", b"TMUX": b"", b"SHELL": b"/bin/sh"}


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="pane-cwd-owner-") as directory:
        directory = os.fsencode(directory)
        first = directory + b"/first"
        relative = first + b"/cwd"
        second = directory + b"/second"
        raw = directory + b"/raw-\xff"
        for path in (first, relative, second, raw):
            os.mkdir(path)
        base = [binary, b"-S", directory + b"/socket", b"-f", b"/dev/null"]

        def run(*args, success=True):
            result = subprocess.run(base + list(args), cwd=first, env=env, capture_output=True, timeout=15)
            assert (result.returncode == 0) == success, (args, result.returncode, result.stderr)
            return result.stdout

        def start_path(target):
            return run(b"display-message", b"-p", b"-t", target, b"#{pane_start_path}")

        def wait_current_path(target, expected):
            deadline = time.monotonic() + 5
            while True:
                actual = run(b"display-message", b"-p", b"-t", target, b"#{pane_current_path}")
                if actual == expected + b"\n":
                    return actual
                assert time.monotonic() < deadline, (target, expected, actual)
                time.sleep(0.05)

        try:
            run(b"new-session", b"-d", b"-s", b"cwd", b"-c", first, b"exec sleep 60")
            initial = start_path(b"cwd:0.0")
            assert initial == first + b"\n", initial
            wait_current_path(b"cwd:0.0", first)

            run(b"split-window", b"-d", b"-t", b"cwd:0.0", b"-c", b"#{session_name}", b"exec sleep 60")
            pane = b"cwd:0.1"
            expanded = start_path(pane)
            assert expanded == relative + b"\n", expanded
            wait_current_path(pane, relative)

            run(b"respawn-pane", b"-t", pane, b"-c", second, success=False)
            after_error = start_path(pane)
            assert after_error == expanded, (after_error, expanded)

            run(b"respawn-pane", b"-k", b"-t", pane)
            retained = start_path(pane)
            assert retained == expanded, (retained, expanded)
            wait_current_path(pane, relative)

            run(b"respawn-pane", b"-k", b"-t", pane, b"-c", second)
            replaced = start_path(pane)
            assert replaced == second + b"\n", replaced
            wait_current_path(pane, second)

            run(b"respawn-pane", b"-k", b"-t", pane, b"-c", b"")
            empty = start_path(pane)
            assert empty == first + b"\n", empty
            wait_current_path(pane, first)

            run(b"new-window", b"-d", b"-t", b"cwd:", b"-c", raw, b"exec sleep 60")
            raw_start = start_path(b"cwd:1.0")
            assert raw_start == raw + b"\n", raw_start
            raw_current = wait_current_path(b"cwd:1.0", raw)
            outputs = initial, expanded, after_error, retained, replaced, empty, raw_start, raw_current
            return tuple(output.replace(directory, b"$ROOT") for output in outputs)
        finally:
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=15)


reference = trace(baseline)
assert trace(candidate) == reference
print("pane cwd owner CLI checks passed")
