#!/usr/bin/env python3
"""Compare default names from spawned windows and broken panes with tmux."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.fsencode(pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve())
env = os.environb | {b"TERM": b"xterm-256color", b"LC_ALL": b"C", b"TMUX": b"", b"SHELL": b"/bin/sh"}


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="default-window-name-") as directory:
        directory = os.fsencode(directory)
        base = [binary, b"-S", directory + b"/socket", b"-f", b"/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        try:
            run(b"new-session", b"-d", b"-s", b"names", b"-n", b"anchor", b"sleep 60")
            run(b"set-window-option", b"-g", b"automatic-rename", b"off")
            run(b"set-window-option", b"-g", b"remain-on-exit", b"on")

            # One shell command, with exec stripped before basename selection.
            shell_name = run(b"new-window", b"-d", b"-P", b"-F", b"#{window_name}", b"-t", b"names:", b"exec /bin/sleep 60").rstrip(b"\n")
            assert shell_name == b"sleep", shell_name

            # An argv command beginning with a quoted path. The parser truncates
            # this executable name at its first space.
            spaced = directory + b"/quoted executable"
            os.symlink(b"/bin/sleep", spaced)
            quoted_name = run(b"new-window", b"-d", b"-P", b"-F", b"#{window_name}", b"-t", b"names:", spaced, b"60").rstrip(b"\n")
            assert quoted_name == b"quoted", quoted_name

            # Invalid UTF-8 bytes in argv are escaped by command stringification
            # before the name parser sees them.
            raw = directory + b"/raw-\xff"
            os.symlink(b"/bin/sleep", raw)
            raw_name = run(b"new-window", b"-d", b"-P", b"-F", b"#{window_name}", b"-t", b"names:", raw, b"60").rstrip(b"\n")
            assert raw_name, raw_name

            # The new window receives its name from the moved pane's command.
            run(b"split-window", b"-d", b"-t", b"names:0.0", b"exec /bin/sleep 60")
            run(b"break-pane", b"-d", b"-s", b"names:0.1", b"-t", b"names:")
            broken_shell = run(b"list-windows", b"-t", b"names", b"-F", b"#{window_name}").splitlines()[-1]
            assert broken_shell == b"sleep", broken_shell

            run(b"split-window", b"-d", b"-t", b"names:0.0", raw, b"60")
            run(b"break-pane", b"-d", b"-s", b"names:0.1", b"-t", b"names:")
            broken_raw = run(b"list-windows", b"-t", b"names", b"-F", b"#{window_name}").splitlines()[-1]
            assert broken_raw == raw_name, (broken_raw, raw_name)
            return shell_name, quoted_name, raw_name, broken_shell, broken_raw
        finally:
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=15)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("default window name owner CLI checks passed")
