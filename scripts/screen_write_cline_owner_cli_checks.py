#!/usr/bin/env python3
"""Compare pane output across a screen write-list resize and teardown."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def trace(binary, directory):
    socket = directory / "socket"
    ready = directory / "ready"
    proceed = directory / "proceed"
    pane = (
        "stty raw -echo; "
        "i=0; while [ $i -lt 12 ]; do "
        "printf 'pre-%02d-abcdefghijklmnopqrstuvwxyz\\n' \"$i\"; "
        "i=$((i+1)); done; "
        f"touch '{ready}'; while [ ! -e '{proceed}' ]; do sleep 0.02; done; "
        "i=0; while [ $i -lt 10 ]; do "
        "printf 'post-%02d-abcdefghijklmnopqrstuvwxyz\\n' \"$i\"; "
        "i=$((i+1)); done; sleep 30"
    )
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def capture_until(marker):
        deadline = time.monotonic() + 5
        while True:
            output = run("capture-pane", "-p", "-t", "writer:0.0")
            if marker in output:
                return output
            assert time.monotonic() < deadline, output
            time.sleep(0.02)

    try:
        run("new-session", "-d", "-x", "40", "-y", "8", "-s", "writer", pane)
        before = capture_until(b"pre-11")
        assert ready.exists()
        run("resize-window", "-t", "writer:0", "-x", "30", "-y", "6")
        proceed.touch()
        after = capture_until(b"post-09")
        assert b"pre-11" in before and b"post-09" in after
        size = run("list-panes", "-t", "writer:0", "-F", "#{pane_width}x#{pane_height}")
        assert size == b"30x6\n", size
        return before, after, size
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="screen-write-cline-owner-", dir=root / "target") as tmp:
    root_tmp = pathlib.Path(tmp)
    candidate_dir = root_tmp / "candidate"
    baseline_dir = root_tmp / "baseline"
    candidate_dir.mkdir()
    baseline_dir.mkdir()
    assert trace(candidate, candidate_dir) == trace(baseline, baseline_dir)

print("screen write cline owner CLI checks passed")
