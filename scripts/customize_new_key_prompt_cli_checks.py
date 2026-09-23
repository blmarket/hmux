#!/usr/bin/env python3
"""Check the new-key prompt and binding in an attached customize tree."""

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
table = "ownership-\u00e9"
prompt = f"New key in {table}: ".encode()


def check(binary_path):
    with tempfile.TemporaryDirectory(prefix="customize-new-key-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None
        output = bytearray()

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def wait_for(needle):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if needle in output:
                    return
            raise AssertionError(f"missing {needle!r}: {output[-3000:]!r}")

        try:
            run("new-session", "-d", "-s", "keys", "sleep 30")
            run("bind-key", "-T", table, "F12", "display-message", "existing")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "keys"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None

            run("customize-mode", "-t", "keys:0.0", "-f", "#{m:F12,#{key}}")
            wait_for(f"Key Table - {table}".encode())
            run("send-keys", "-t", "keys:0.0", "G", "Up", "Up")
            run("send-keys", "-t", "keys:0.0", "s")
            wait_for(prompt)
            run("send-keys", "-t", "keys:0.0", "F11 display-message added", "Enter")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                bindings = run("list-keys", "-T", table)
                if b"F11" in bindings:
                    assert b"display-message added" in bindings, bindings
                    return bindings
                time.sleep(0.05)
            raise AssertionError(f"new key binding missing: {bindings!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
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

print("customize new-key prompt CLI checks passed")
