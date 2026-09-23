#!/usr/bin/env python3
"""Compare copy-mode selection, append, and pipe bytes against the pinned binary."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="copy-selection-owner-", dir=root / "target") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-x", "40", "-y", "8", "-s", "selection", "sleep 30")
            run("run-shell", "-t", "selection:0.0", "printf 'alpha\\nbeta\\t漢字 gamma\\nlast\\n'")
            deadline = time.monotonic() + 5
            while run("display-message", "-p", "#{pane_in_mode}|#{pane_mode}") != b"1|view-mode\n":
                assert time.monotonic() < deadline, "run-shell did not open view mode"
                time.sleep(0.02)

            run("send-keys", "-X", "select-line")
            run("send-keys", "-X", "copy-selection")
            first = run("show-buffer", "-b", "buffer0")

            run("send-keys", "-X", "cursor-down")
            run("send-keys", "-X", "select-line")
            run("send-keys", "-X", "append-selection")
            appended = run("show-buffer", "-b", "buffer0")

            run("send-keys", "-X", "cursor-down")
            run("send-keys", "-X", "select-line")
            pipe_file = directory / "piped"
            run("send-keys", "-X", "copy-pipe", f"cat > {pipe_file}")
            deadline = time.monotonic() + 5
            while not pipe_file.exists() or pipe_file.read_bytes() != b"last\n":
                assert time.monotonic() < deadline, "copy-pipe did not write"
                time.sleep(0.02)
            piped = pipe_file.read_bytes()
            copied_pipe = run("show-buffer")

            run("send-keys", "-X", "select-line")
            pipe_only_file = directory / "pipe-only"
            run("send-keys", "-X", "pipe", f"cat > {pipe_only_file}")
            deadline = time.monotonic() + 5
            while not pipe_only_file.exists() or pipe_only_file.read_bytes() != b"last\n":
                assert time.monotonic() < deadline, "pipe did not write"
                time.sleep(0.02)
            pipe_only = pipe_only_file.read_bytes()
            return first, appended, piped, copied_pipe, pipe_only
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


def binary_append_trace(binary):
    with tempfile.TemporaryDirectory(prefix="copy-selection-binary-", dir=root / "target") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
        source = directory / "source"
        source.write_bytes(b"P\0Q")

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-x", "40", "-y", "8", "-s", "selection", "sleep 30")
            run("run-shell", "-t", "selection:0.0", "printf 'alpha\\n'")
            deadline = time.monotonic() + 5
            while run("display-message", "-p", "#{pane_in_mode}|#{pane_mode}") != b"1|view-mode\n":
                assert time.monotonic() < deadline, "run-shell did not open view mode"
                time.sleep(0.02)
            run("load-buffer", str(source))
            before = run("show-buffer", "-b", "buffer0")
            run("send-keys", "-X", "select-line")
            run("send-keys", "-X", "append-selection")
            after = run("show-buffer", "-b", "buffer0")
            return before, after
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


expected = (b"alpha\n", "alpha\nbeta\t漢字 gamma\n".encode(), b"last\n", b"last\n", b"last\n")
reference = trace(baseline)
assert reference == expected, (reference, expected)
assert trace(candidate) == reference
binary_reference = binary_append_trace(baseline)
assert binary_reference == (b"P\0Q", b"P\0Qalpha\n"), binary_reference
assert binary_append_trace(candidate) == binary_reference
print("window copy selection owner CLI checks passed")
