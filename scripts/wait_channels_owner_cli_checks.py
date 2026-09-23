#!/usr/bin/env python3
"""Compare channel wait, wake, signal, lock, unlock, and flush with baseline."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="wait-channels-owner-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="")
        pending = []

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def start(*args):
            process = subprocess.Popen(
                base + list(args), env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE
            )
            pending.append(process)
            return process

        def listed(channel, count):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                names = run("wait-for", "-l", channel).splitlines()
                if len(names) == count:
                    return names
                time.sleep(0.02)
            raise AssertionError(("waiting count", channel, count, names))

        def finished(process):
            out, err = process.communicate(timeout=5)
            assert process.returncode == 0 and not err, (process.returncode, out, err)
            return out

        try:
            run("new-session", "-d", "-s", "wait-owner", "sleep 30")

            # A signal issued before a waiter is remembered once.
            run("wait-for", "-S", "early")
            assert run("wait-for", "early") == b""
            assert listed("early", 0) == []

            first = start("wait-for", "wakers")
            first_name = listed("wakers", 1)[0]
            run("wait-for", "-w", first_name.decode(), "wakers")
            assert finished(first) == b""
            assert listed("wakers", 0) == []

            second = start("wait-for", "signalled")
            third = start("wait-for", "signalled")
            assert len(listed("signalled", 2)) == 2
            run("wait-for", "-S", "signalled")
            assert finished(second) == b""
            assert finished(third) == b""
            assert listed("signalled", 0) == []

            run("wait-for", "-L", "mutex")
            locker = start("wait-for", "-L", "mutex")
            assert len(listed("mutex", 1)) == 1
            run("wait-for", "-U", "mutex")
            assert finished(locker) == b""
            run("wait-for", "-U", "mutex")
            assert listed("mutex", 0) == []

            waiter = start("wait-for", "flush-wait")
            assert len(listed("flush-wait", 1)) == 1
            run("wait-for", "-L", "flush-lock")
            queued_locker = start("wait-for", "-L", "flush-lock")
            assert len(listed("flush-lock", 1)) == 1
            run("kill-server")
            assert finished(waiter) == b""
            assert finished(queued_locker) == b""
            return (bool(first_name),)
        finally:
            for process in pending:
                if process.poll() is None:
                    process.kill()
                    process.communicate(timeout=5)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("wait channels owner CLI checks passed")
