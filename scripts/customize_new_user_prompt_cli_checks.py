#!/usr/bin/env python3
"""Check new user option and hook prompts in an attached customize tree."""

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


def check(binary_path, hook):
    with tempfile.TemporaryDirectory(prefix="customize-new-user-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "users", "sleep 30")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "users"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)

            run("customize-mode", "-t", "users:0.0", "-f", "0")
            run("send-keys", "-t", "users:0.0", "g")
            if hook:
                run("send-keys", "-t", "users:0.0", "Down", "Down", "Down")
            run("send-keys", "-t", "users:0.0", "s")
            needle = b"New user hook: @" if hook else b"New user option: @"
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if needle in output:
                    return needle
            raise AssertionError(f"missing prompt {needle!r}: {output[-3000:]!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [check(binary, hook) for hook in (False, True)]
if baseline is not None:
    expected = [check(pathlib.Path(baseline).resolve(), hook) for hook in (False, True)]
    assert actual == expected, (actual, expected)

print("customize new-user prompt CLI checks passed")
