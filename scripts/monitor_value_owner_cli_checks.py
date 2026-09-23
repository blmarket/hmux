#!/usr/bin/env python3
"""Check monitor cached values through a control-client subscription."""

import os
import pathlib
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def check(binary):
    with tempfile.TemporaryDirectory(prefix="monitor-value-owner-", dir=root / "target") as tmp:
        base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        control = None
        try:
            run(b"new-session", b"-d", b"-s", b"mon", b"sleep", b"30")
            run(b"set-option", b"-t", b"mon", b"@watched", b"old\xff")
            control = subprocess.Popen(
                base + [b"-C", b"attach-session", b"-t", b"mon"],
                env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
            )
            deadline = time.monotonic() + 5
            while not (clients := run(b"list-clients", b"-F", b"#{client_name}").splitlines()):
                assert control.poll() is None, "control client exited"
                assert time.monotonic() < deadline, "control client did not attach"
                time.sleep(0.02)
            assert len(clients) == 1, clients

            output = bytearray()

            def values():
                lines = output.split(b"\n")[:-1]
                return [
                    line.split(b" : ", 1)[1]
                    for line in lines
                    if line.startswith(b"%subscription-changed @watch ")
                ]

            def drain(timeout):
                deadline = time.monotonic() + timeout
                while time.monotonic() < deadline:
                    remaining = deadline - time.monotonic()
                    ready, _, _ = select.select([control.stdout], [], [], remaining)
                    if not ready:
                        return
                    chunk = os.read(control.stdout.fileno(), 65536)
                    assert chunk, (output, control.poll())
                    output.extend(chunk)

            def wait_values(expected):
                deadline = time.monotonic() + 6
                while values() != expected:
                    assert time.monotonic() < deadline, (values(), expected, output)
                    drain(min(0.1, deadline - time.monotonic()))
                return values()

            run(b"refresh-client", b"-t", clients[0], b"-B", b"@watch::#{@watched}")
            initial = wait_values([b"old\xff"])

            # A later tick with the same bytes must not emit another update.
            run(b"set-option", b"-t", b"mon", b"@watched", b"old\xff")
            drain(1.5)
            equal = values()
            assert equal == initial, (initial, equal, output)

            run(b"set-option", b"-t", b"mon", b"@watched", b"new\xfe")
            changed = wait_values([b"old\xff", b"new\xfe"])
            run(b"set-option", b"-t", b"mon", b"@watched", b"final")
            changed_again = wait_values([b"old\xff", b"new\xfe", b"final"])
            return initial, equal, changed, changed_again
        finally:
            if control is not None:
                control.terminate()
                control.communicate(timeout=5)
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


actual = check(candidate)
if baseline is not None:
    expected = check(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("monitor value owner CLI checks passed")
