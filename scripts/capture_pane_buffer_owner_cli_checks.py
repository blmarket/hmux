#!/usr/bin/env python3
"""Compare capture-pane buffer producers and ownership exits on private servers.

Set HMUX_BINARY for the candidate and HMUX_BASELINE_BINARY for the pinned
pre-migration binary. Pending input contains an embedded NUL byte.
"""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2"))
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"])
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh", LC_ALL="C")
command = (
    "printf 'ROW\\n\\033]8;id=owner;https://example.test/one\\007"
    "LINK\\033]8;;\\007\\n\\033[\\000'; sleep 20"
)


def capture(binary):
    with tempfile.TemporaryDirectory(prefix="capture-buffer-owner-") as directory:
        base = [str(binary.resolve()), "-f", "/dev/null", "-S", str(pathlib.Path(directory) / "socket")]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            return result.returncode, result.stdout, result.stderr

        try:
            assert run("new-session", "-d", "-s", "capture", "-x", "30", "-y", "5", command) == (0, b"", b"")
            deadline = time.monotonic() + 5
            while True:
                history = run("capture-pane", "-p", "-S", "0", "-E", "1")
                pending = run("capture-pane", "-P", "-p")
                if history == (0, b"ROW\nLINK\n", b"") and pending == (0, b"\033[\000\n", b""):
                    break
                assert time.monotonic() < deadline, (history, pending)
                time.sleep(0.05)

            cases = (
                ("capture-pane", "-p", "-S", "0", "-E", "1"),
                ("capture-pane", "-L", "-F", "-p", "-S", "0", "-E", "1"),
                ("capture-pane", "-H", "-p", "-S", "0", "-E", "1"),
                ("capture-pane", "-R", "-p"),
                ("capture-pane", "-P", "-p"),
                ("capture-pane", "-P", "-C", "-p"),
                ("capture-pane", "-a", "-p"),
                ("capture-pane", "-a", "-q", "-p"),
                ("capture-pane", "-P", "-b", ""),
            )
            results = [run(*case) for case in cases]
            assert results[2] == (0, b"https://example.test/one\n", b"")
            assert results[4] == (0, b"\033[\000\n", b"")
            assert results[5] == (0, b"\\033[\\000\n", b"")
            assert results[6] == (1, b"", b"no alternate screen\n")
            assert results[7] == (0, b"\n", b"")
            assert results[8] == (1, b"", b"empty buffer name\n")

            results.append(run("capture-pane", "-P", "-b", "raw"))
            results.append(run("show-buffer", "-b", "raw"))
            assert results[-2] == (0, b"", b"")
            assert results[-1] == (0, b"\033[\000", b"")
            results.append(run("capture-pane", "-P"))
            results.append(run("show-buffer"))
            assert results[-2] == (0, b"", b"")
            assert results[-1] == (0, b"\033[\000", b"")
            # Exercise saved-grid restoration and reflow of populated lines.
            assert run("respawn-pane", "-k", "stty raw -echo; printf READY; cat") == (0, b"", b"")

            def expect_row(expected):
                deadline = time.monotonic() + 5
                while True:
                    actual = run("capture-pane", "-p", "-S", "0", "-E", "0")
                    if actual == (0, expected + b"\n", b""):
                        return
                    assert time.monotonic() < deadline, (expected, actual)
                    time.sleep(0.05)

            expect_row(b"READY")
            for width in (12, 40, 9, 30):
                assert run("send-keys", "-l", "\x1b[?1049h\x1b[HALTERNATE") == (0, b"", b"")
                expect_row(b"ALTERNATE")
                results.append(run("capture-pane", "-a", "-p"))
                assert run("resize-window", "-x", str(width), "-y", "5") == (0, b"", b"")
                assert run("send-keys", "-l", "\x1b[?1049l") == (0, b"", b"")
                expect_row(b"READY")
                results.append(run("capture-pane", "-p"))
            assert run("send-keys", "-l", "\x1b[H" + "abcdefghij" * 8) == (0, b"", b"")
            expect_row(b"abcdefghij" * 3)
            for width in (8, 43, 17, 30):
                assert run("resize-window", "-x", str(width), "-y", "5") == (0, b"", b"")
                results.append(run("capture-pane", "-p", "-S", "-", "-J"))
            # Destruction with an alternate grid still active.
            assert run("send-keys", "-l", "\x1b[?1049h\x1b[HLAST") == (0, b"", b"")
            expect_row(b"LAST")
            return results
        finally:
            run("kill-server")


assert capture(candidate) == capture(baseline), "capture-pane output differs from baseline"
print("capture-pane buffer owner CLI checks passed")
