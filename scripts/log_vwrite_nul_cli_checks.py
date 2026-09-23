#!/usr/bin/env python3
"""Check the first-NUL view of a formatted server log message."""

import os
import pathlib
import re
import subprocess
import tempfile
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = os.environ.get("HMUX_BASELINE_BINARY")
ENV = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
TIMESTAMP = re.compile(rb"^[0-9]+\.[0-9]+ (.*)$")


def check(binary):
    with tempfile.TemporaryDirectory(prefix="log-vwrite-nul-") as tmp:
        directory = pathlib.Path(tmp)
        pane_command = directory / "emit-nul.sh"
        pane_command.write_bytes(b"#!/bin/sh\nprintf '\\000'\nsleep 2\n")
        pane_command.chmod(0o755)
        base = [str(binary), "-vv", "-S", str(directory / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(
                base + list(args), cwd=directory, env=ENV, capture_output=True, timeout=10,
            )
            assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)

        try:
            run("new-session", "-d", "-s", "nul", str(pane_command))
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                logs = list(directory.glob("tmux-server-*.log"))
                if logs:
                    lines = [line for line in logs[0].read_bytes().splitlines() if b"input_c0_dispatch:" in line]
                    if lines:
                        assert len(lines) == 1, lines
                        match = TIMESTAMP.fullmatch(lines[0])
                        assert match is not None, lines[0]
                        # printf emitted NUL. The source format has a closing quote
                        # after %c, but stravis receives the first-NUL C-string view.
                        assert match.group(1) == b"input_c0_dispatch: '", match.group(1)
                        return match.group(1)
                time.sleep(0.05)
            raise AssertionError("server did not log the pane's NUL byte")
        finally:
            subprocess.run(base + ["kill-server"], cwd=directory, env=ENV, capture_output=True, timeout=10)


actual = check(CANDIDATE)
if BASELINE is not None:
    expected = check(pathlib.Path(BASELINE).resolve())
    assert actual == expected, (actual, expected)

print("log_vwrite NUL CLI checks passed")
