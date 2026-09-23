#!/usr/bin/env python3
"""Exercise dynamic codepoint widths, duplicate replacement, and cache reset."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")


def check(binary):
    with tempfile.TemporaryDirectory(prefix="utf8-width-cache-", dir=root / "target") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "width", "sleep", "30")
            run("set-option", "-gq", "@cell", "\ue010")

            def width():
                return run("display-message", "-p", "#{w:#{@cell}}")

            observed = [width()]
            run("set-option", "-g", "codepoint-widths[0]", "U+E010=2")
            observed.append(width())
            run("set-option", "-g", "codepoint-widths[1]", "U+E010=1")
            observed.append(width())
            run("set-option", "-gu", "codepoint-widths[1]")
            observed.append(width())
            run("set-option", "-gu", "codepoint-widths[0]")
            observed.append(width())
            assert observed == [b"1\n", b"2\n", b"1\n", b"2\n", b"1\n"], observed
            return observed
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=15)


actual = check(candidate)
if baseline is not None:
    expected = check(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("UTF-8 width cache CLI checks passed")
