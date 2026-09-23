#!/usr/bin/env python3
"""Exercise indexed four-byte UTF-8 cells through a live pane and capture."""

import os
import pathlib
import subprocess
import tempfile
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = os.environ.get("HMUX_BASELINE_BINARY")
ENV = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
EXPECTED = "A😀B\nA😀B\n".encode()


def check(binary):
    with tempfile.TemporaryDirectory(prefix="utf8-item-cache-", dir=ROOT / "target") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=ENV, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "utf8", "-x", "40", "-y", "8", "sh", "-c",
                "printf 'A😀B\\nA😀B\\n'; sleep 30")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                output = run("capture-pane", "-p", "-t", "utf8:0.0")
                if output.startswith(EXPECTED):
                    assert output.count("😀".encode()) == 2, output
                    return output
                time.sleep(0.05)
            raise AssertionError(("four-byte cells were not captured", output))
        finally:
            subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=15)


actual = check(CANDIDATE)
if BASELINE is not None:
    expected = check(pathlib.Path(BASELINE).resolve())
    assert actual == expected, (actual, expected)

print("UTF-8 item cache CLI checks passed")
