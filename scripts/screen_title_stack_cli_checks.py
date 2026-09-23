#!/usr/bin/env python3
"""Exercise pane title push, bounded eviction, pop, and teardown."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")


def check(binary):
    with tempfile.TemporaryDirectory(prefix="screen-title-stack-", dir=root / "target") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def wait_title(expected):
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline:
                actual = run("display-message", "-p", "#{pane_title}").strip()
                if actual == expected:
                    return actual
                time.sleep(0.02)
            raise AssertionError((actual, expected))

        fd = None
        try:
            run("new-session", "-d", "-s", "titles", "sleep", "30")
            tty = run("display-message", "-p", "#{pane_tty}").strip().decode("ascii")
            fd = os.open(tty, os.O_WRONLY | os.O_NOCTTY)

            os.write(fd, b"\x1b]2;root\x07")
            wait_title(b"root")
            for index in range(12):
                title = f"T{index}".encode("ascii")
                os.write(fd, b"\x1b[22;2t\x1b]2;" + title + b"\x07")
                wait_title(title)

            observed = []
            for index in range(10, 0, -1):
                os.write(fd, b"\x1b[23;2t")
                observed.append(wait_title(f"T{index}".encode("ascii")))
            os.write(fd, b"\x1b[23;2t")
            observed.append(wait_title(b"T1"))  # Empty stack leaves the title alone.

            os.write(fd, b"\x1b[22;2t\x1b]2;tail\x07")
            wait_title(b"tail")  # Server teardown frees the remaining entry.
            return observed
        finally:
            if fd is not None:
                os.close(fd)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=15)


actual = check(candidate)
if baseline is not None:
    expected = check(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("screen title stack CLI checks passed")
