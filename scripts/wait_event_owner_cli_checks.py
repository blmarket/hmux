#!/usr/bin/env python3
"""Compare event waiter completion, filtering, wake, and flush with the baseline."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="wait-event-owner-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="")
        waiters = []

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def start_waiter(*args):
            waiter = subprocess.Popen(
                base + ["wait-for", "-E", *args, "window-renamed"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            waiters.append(waiter)
            return waiter

        def clients(count):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                listed = run("wait-for", "-E", "-l", "window-renamed").splitlines()
                if len(listed) == count:
                    return listed
                time.sleep(0.02)
            raise AssertionError(("waiter count", count, listed))

        try:
            run("new-session", "-d", "-s", "event-owner", "sleep 30")
            filtered = start_waiter("-F", "0", "-v")
            filtered_name = clients(1)[0]
            accepted = start_waiter("-F", "1", "-v")
            assert len(clients(2)) == 2

            run("rename-window", "-t", "event-owner:0", "first")
            accepted_out, accepted_err = accepted.communicate(timeout=5)
            assert accepted.returncode == 0 and not accepted_err, (accepted_out, accepted_err)
            assert b"event=window-renamed\n" in accepted_out, accepted_out
            assert b"new_name=first\n" in accepted_out, accepted_out
            assert clients(1) == [filtered_name]
            assert filtered.poll() is None

            run("wait-for", "-E", "-w", filtered_name.decode(), "window-renamed")
            filtered_out, filtered_err = filtered.communicate(timeout=5)
            assert filtered.returncode == 0 and not filtered_err, (filtered_out, filtered_err)
            assert filtered_out == accepted_out, (filtered_out, accepted_out)
            assert clients(0) == []

            flushed = start_waiter()
            assert len(clients(1)) == 1
            run("kill-server")
            flushed_out, flushed_err = flushed.communicate(timeout=5)
            assert flushed.returncode == 0 and not flushed_err, (flushed_out, flushed_err)
            assert flushed_out == b"", flushed_out
            return accepted_out, filtered_out, flushed_out
        finally:
            for waiter in waiters:
                if waiter.poll() is None:
                    waiter.kill()
                    waiter.communicate(timeout=5)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("wait event owner CLI checks passed")
