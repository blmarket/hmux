#!/usr/bin/env python3
"""Compare argv stringification at pane event, format, and name call sites."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def exercise(binary_path):
    with tempfile.TemporaryDirectory(prefix="cmd-stringify-owner-") as tmp:
        base = [os.fsencode(binary_path), b"-S", os.fsencode(pathlib.Path(tmp) / "sock"), b"-f", b"/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run(b"new-session", b"-d", b"-s", b"owner")
            empty = run(b"display-message", b"-p", b"-t", b"owner:0", b"#{pane_start_command}")
            assert empty == b"\n", empty

            run(b"set-hook", b"-g", b"pane-created", b'set-option -gF @created "#{hook_pane_command}"')
            run(b"new-window", b"-d", b"-t", b"owner:", b"/bin/sh", b"-c", b"exec sleep 60", b"o'hara", b"", b"\xff")
            event = run(b"show-options", b"-gqv", b"@created")
            start = run(b"display-message", b"-p", b"-t", b"owner:1", b"#{pane_start_command}")
            name = run(b"display-message", b"-p", b"-t", b"owner:1", b"#{window_name}")
            expected = b'/bin/sh -c "exec sleep 60" "o\'hara" \'\' \\377\n'
            assert event == start == expected, (event, start)
            assert name.strip(), name

            run(b"set-window-option", b"-g", b"remain-on-exit", b"on")
            run(b"new-window", b"-d", b"-t", b"owner:", b"/bin/sh", b"-c", b"exit 0", b"x y")
            deadline = time.monotonic() + 5
            while True:
                dead = run(b"display-message", b"-p", b"-t", b"owner:2", b"#{pane_dead}")
                if dead == b"1\n":
                    break
                assert time.monotonic() < deadline, dead
                time.sleep(0.05)
            current = run(b"display-message", b"-p", b"-t", b"owner:2", b"#{pane_current_command}")
            assert current == b"sh\n", current
            alternate_shell = os.fsencode(pathlib.Path(tmp) / "alternate-shell")
            os.symlink(b"/bin/sh", alternate_shell)
            run(b"set-option", b"-g", b"default-shell", alternate_shell)
            assert run(b"show-option", b"-gqv", b"default-shell") == alternate_shell + b"\n"
            run(b"respawn-pane", b"-t", b"owner:2")
            deadline = time.monotonic() + 5
            while run(b"display-message", b"-p", b"-t", b"owner:2", b"#{pane_dead}") != b"1\n":
                assert time.monotonic() < deadline, "respawned pane did not exit"
                time.sleep(0.05)
            after_respawn = run(b"display-message", b"-p", b"-t", b"owner:2", b"#{pane_current_command}")
            assert after_respawn == b"sh\n", after_respawn
            # The automatic window name starts as the executable's own name
            # before its async rename; that initial name differs by binary.
            return empty, event, start, current, after_respawn
        finally:
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=15)


actual = exercise(binary)
if baseline is not None:
    expected = exercise(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)

print("cmd stringify owner CLI checks passed")
