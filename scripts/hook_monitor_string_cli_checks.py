#!/usr/bin/env python3
"""Compare monitor strings in show-hooks and customize-mode with the baseline."""

import fcntl
import os
import pathlib
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def check(binary_path):
    with tempfile.TemporaryDirectory(prefix="hook-monitor-string-") as tmp:
        base = [os.fsencode(binary_path), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 110, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run(b"new-session", b"-d", b"-s", b"monitor", b"sleep", b"30")
            specs = (
                b"@allpane:%*:fmt\xff",
                b"@allwindow:@*:fmt\xff",
                b"@session::fmt\xff",
                b"@pane:%1:fmt\xff",
                b"@window:@1:fmt\xff",
            )
            for spec in specs:
                run(b"set-hook", b"-g", b"-B", spec, b"true")
            lines = run(b"show-hooks", b"-g", b"-B", b"-F", b"#{option_value}").splitlines()
            assert lines == sorted(specs), (lines, specs)

            client = subprocess.Popen(
                base + [b"attach-session", b"-t", b"monitor"],
                env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not run(b"list-clients", b"-F", b"#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)

            run(
                b"customize-mode", b"-t", b"monitor:0.0",
                b"-f", b"#{==:#{option_name},@session}", b"-F", b"#{option_monitor}",
            )
            run(b"send-keys", b"-t", b"monitor:0.0", b"Down", b"Down", b"Down", b"Right", b"Down")
            output = bytearray()
            needle = b"@session::fmt\xff"
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if output.count(needle) >= 2 and b"Monitor: " in output:
                    return lines, needle
            raise AssertionError(f"missing customize monitor detail {needle!r}: {output[-3000:]!r}")
        finally:
            subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = check(binary)
if baseline is not None:
    expected = check(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("hook monitor string CLI checks passed")
