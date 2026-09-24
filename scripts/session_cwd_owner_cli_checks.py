#!/usr/bin/env python3
"""Compare session cwd creation, attach replacement, and window inheritance."""

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
    with tempfile.TemporaryDirectory(prefix="session-cwd-owner-") as directory:
        directory = os.fsencode(directory)
        first = directory + b"/first path"
        second = directory + b"/cwd"
        os.mkdir(first)
        os.mkdir(second)
        base = [binary, b"-S", directory + b"/socket", b"-f", b"/dev/null"]
        attached = None

        def run(*args):
            result = subprocess.run(base + list(args), cwd=first, env=env, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def session_path():
            return run(b"display-message", b"-p", b"-t", b"cwd", b"#{session_path}")

        def pane_start_path(index):
            return run(b"display-message", b"-p", b"-t", b"cwd:" + index + b".0", b"#{pane_start_path}")

        try:
            run(b"new-session", b"-d", b"-s", b"default", b"exec sleep 60")
            # Keep the server alive while replacing its only session, so the
            # next new-session does not race the previous server's shutdown.
            run(b"set-option", b"-s", b"exit-empty", b"off")
            default_session = run(b"display-message", b"-p", b"-t", b"default", b"#{session_path}")
            default_pane = run(
                b"display-message", b"-p", b"-t", b"default:0.0", b"#{pane_start_path}"
            )
            assert default_session == first + b"\n", default_session
            assert default_pane == default_session, default_pane
            run(b"kill-session", b"-t", b"default")

            run(b"new-session", b"-d", b"-s", b"cwd", b"-c", first, b"exec sleep 60")
            initial = session_path()
            assert initial == first + b"\n", initial
            assert pane_start_path(b"0") == initial

            run(b"new-window", b"-d", b"-t", b"cwd:", b"exec sleep 60")
            before_attach = pane_start_path(b"1")
            assert before_attach == initial, before_attach

            attached = subprocess.Popen(
                base + [b"-C", b"attach-session", b"-t", b"cwd", b"-c", directory + b"/#{session_name}"],
                cwd=first, env=env, stdin=subprocess.PIPE,
                stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
            )
            deadline = time.monotonic() + 5
            while True:
                after_attach = session_path()
                if after_attach == second + b"\n":
                    break
                assert attached.poll() is None, attached.communicate(timeout=5)
                assert time.monotonic() < deadline, after_attach
                time.sleep(0.05)

            attached.stdin.write(b"new-window -d -t cwd: 'exec sleep 60'\n")
            attached.stdin.flush()
            deadline = time.monotonic() + 5
            while True:
                windows = run(b"list-windows", b"-t", b"cwd", b"-F", b"#{window_index}").splitlines()
                if b"2" in windows:
                    break
                assert attached.poll() is None, attached.communicate(timeout=5)
                assert time.monotonic() < deadline, windows
                time.sleep(0.05)
            inherited = pane_start_path(b"2")
            assert inherited == second + b"\n", inherited
            assert session_path() == after_attach
            return tuple(
                value.replace(directory, b"$ROOT")
                for value in (default_session, default_pane, initial, before_attach, after_attach, inherited)
            )
        finally:
            if attached is not None:
                if attached.poll() is None:
                    attached.terminate()
                attached.communicate(timeout=5)
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=15)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("session cwd owner CLI checks passed")
