#!/usr/bin/env python3
"""Compare run-shell job and command callback lifetimes with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="run-shell-owner-", dir=root / "target") as tmp:
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        output_file = pathlib.Path(tmp) / "delayed"

        def run(*args):
            return subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)

        try:
            assert run("new-session", "-d", "-s", "shell-owner", "sleep 30").returncode == 0

            immediate = run("run-shell", "printf 'first\\nlast'")
            assert immediate.returncode == 0, immediate.stderr
            assert immediate.stdout == b"first\nlast\n", immediate.stdout

            command = run("run-shell", "-C", "set-option -g @run_shell_owner yes")
            assert command.returncode == 0, command.stderr
            value = run("show-options", "-gqv", "@run_shell_owner")
            assert value.returncode == 0 and value.stdout == b"yes\n", value

            delayed = run("run-shell", "-b", "-d", "0.05", f"printf delayed > {output_file}")
            assert delayed.returncode == 0, delayed.stderr
            deadline = time.monotonic() + 5
            while not output_file.exists():
                assert time.monotonic() < deadline, "background delayed job did not finish"
                time.sleep(0.02)
            assert output_file.read_bytes() == b"delayed"

            failure = run("run-shell", "exit 7")
            assert failure.returncode == 7, (failure.returncode, failure.stdout, failure.stderr)
            assert b"returned 7" in failure.stdout, failure.stdout
            return immediate.stdout, value.stdout, output_file.read_bytes(), failure.returncode
        finally:
            run("kill-server")


reference = trace(baseline)
actual = trace(candidate)
assert actual == reference, (actual, reference)
print("run-shell data owner CLI checks passed")
