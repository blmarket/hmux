#!/usr/bin/env python3
"""Compare spawn environment log bytes with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
marker = b"spawn_pane: environment OWNER_"


def check(binary, directory):
    directory.mkdir()
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [os.fsencode(binary), b"-vv", b"-S", os.fsencode(directory / "socket"), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), cwd=directory, env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)

    try:
        run(
            b"new-session", b"-d", b"-s", b"envowner",
            b"-e", b"OWNER_MARK=owner-value",
            b"-e", b"OWNER_RAW=owner-\xff",
            b"sleep 15",
        )
    finally:
        run(b"kill-server")

    logs = list(directory.glob("tmux-server-*.log"))
    assert len(logs) == 1, logs
    # The timestamp and process ID vary; compare the event text and escaped bytes.
    lines = sorted(line.split(marker, 1)[1] for line in logs[0].read_bytes().splitlines() if marker in line)
    expected = [b"MARK=owner-value", b"RAW=owner-\\377"]
    assert lines == expected, (lines, expected)
    return lines


with tempfile.TemporaryDirectory(prefix="environ-log-owner-") as tmp:
    directory = pathlib.Path(tmp)
    assert check(candidate, directory / "candidate") == check(baseline, directory / "baseline")

print("environment log owner CLI checks passed")
