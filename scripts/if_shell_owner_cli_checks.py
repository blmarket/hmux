#!/usr/bin/env python3
"""Compare if-shell job callbacks with the pinned pre-migration binary."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="if-shell-owner-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        source = pathlib.Path(tmp) / "nested.conf"
        source.write_text(
            "set-option -g @source_before yes\n"
            "if-shell true 'if-owner-unknown-command'\n"
            "set-option -g @source_after yes\n"
        )
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def value(name):
            return run("show-options", "-gqv", name)

        try:
            run("new-session", "-d", "-s", "if-owner", "sleep 30")
            run("if-shell", "true", "set-option -g @if_true yes", "set-option -g @if_false no")
            true_values = value("@if_true"), value("@if_false")
            run("if-shell", "false", "set-option -g @else_true no", "set-option -g @else_false yes")
            false_values = value("@else_true"), value("@else_false")
            invalid = subprocess.run(
                base + ["if-shell", "true", "if-owner-unknown-command"],
                env=env, capture_output=True, timeout=10,
            )
            assert invalid.returncode != 0 and b"unknown command" in invalid.stderr, invalid
            sourced = subprocess.run(
                base + ["source-file", str(source)],
                env=env, capture_output=True, timeout=10,
            )
            sourced_error = sourced.stderr.replace(os.fsencode(source), b"<source>")
            assert sourced.returncode != 0 and sourced_error == (
                b"<source>:2: unknown command: if-owner-unknown-command\n"
            ), sourced
            source_values = value("@source_before"), value("@source_after")
            assert source_values == (b"yes\n", b"yes\n"), source_values
            run("if-shell", "-b", "true", "set-option -g @background yes")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline and value("@background") != b"yes\n":
                time.sleep(0.05)
            background = value("@background")
            run("if-shell", "-F", "1", "set-option -g @format yes")
            formatted = value("@format")
            result = (
                true_values, false_values, (invalid.returncode, invalid.stderr),
                (sourced.returncode, sourced_error), source_values, background, formatted,
            )
            assert true_values == (b"yes\n", b""), result
            assert false_values == (b"", b"yes\n"), result
            assert background == formatted == b"yes\n", result
            return result
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("if-shell owner CLI checks passed")
