#!/usr/bin/env python3
"""Compare search-match formatting and copy fallback against the pinned binary."""

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
    with tempfile.TemporaryDirectory(prefix="copy-match-owner-") as directory:
        base = [str(binary), "-S", str(pathlib.Path(directory) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        try:
            run(
                "new-session", "-d", "-x", "40", "-y", "8", "-s", "match",
                "printf 'alpha\\nbeta\\t漢字 gamma\\n'; sleep 30",
            )
            deadline = time.monotonic() + 5
            while b"beta" not in run("capture-pane", "-p", "-S", "0"):
                assert time.monotonic() < deadline, "pane output missing"
                time.sleep(0.02)

            run("copy-mode")
            run("send-keys", "-X", "search-forward", "alpha")
            first = run("display-message", "-p", "#{search_match}")
            cursor = run("display-message", "-p", "#{copy_cursor_word}|#{copy_cursor_line}|#{copy_cursor_hyperlink}|#{search_match}|#{search_match}")
            assert cursor == b"alpha|alpha||alpha|alpha\n", cursor
            # Enumeration invokes the same lazy callbacks and caches their results.
            listed = dict(line.split(b"=", 1) for line in run("display-message", "-a").splitlines() if b"=" in line)
            for key, expected in [(b"copy_cursor_word", b"alpha"), (b"copy_cursor_line", b"alpha"), (b"copy_cursor_hyperlink", b""), (b"search_match", b"alpha")]:
                assert listed[key] == expected, (key, listed.get(key))
            first_cached = run("display-message", "-p", "#{pane_search_string}")
            run("send-keys", "-X", "search-forward", "-F", "#{pane_search_string}")
            formatted_cached = run("display-message", "-p", "#{pane_search_string}")
            run("send-keys", "-X", "search-forward", "[")  # Invalid regex clears old marks.
            invalid = run("display-message", "-p", "#{search_match}")
            run("send-keys", "-X", "search-forward", "beta\t漢字")
            formatted = run("display-message", "-p", "#{search_match}")
            final_cached = run("display-message", "-p", "#{pane_search_string}")
            run("send-keys", "-X", "copy-selection")
            copied = run("show-buffer")
            return first, first_cached, formatted_cached, invalid, formatted, final_cached, copied
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = (b"alpha\n", b"alpha\n", "beta\t漢字\n".encode(), "beta\t漢字".encode())
reference = trace(baseline)
assert (reference[1], reference[2], reference[5], reference[6]) == expected, reference
assert trace(candidate) == reference
print("copy match owner CLI checks passed")
