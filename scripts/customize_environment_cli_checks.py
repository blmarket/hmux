#!/usr/bin/env python3
"""Compare set and removed environment rows in an attached customize tree."""

import fcntl
import os
import pathlib
import pty
import re
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
csi = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")


def environment_rows(binary_path):
    with tempfile.TemporaryDirectory(prefix="customize-environment-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "envs", "sleep 30")
            run("set-environment", "-t", "envs", "OWN_A_SET", "caf\u00e9")
            run("set-environment", "-r", "-t", "envs", "OWN_B_REM")
            run("set-environment", "-t", "envs", "OWN_C_SKIP", "filtered")
            run("set-environment", "-t", "envs", "OWN_D_EMPTY", "")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "envs"],
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

            run(
                "customize-mode", "-t", "envs:0.0",
                "-f", "#{m/r:^OWN_(A_SET|B_REM|D_EMPTY)$,#{environment_name}}",
                "-F", "#{environment_name}:#{environment_value}:#{environment_removed}",
            )
            run(
                "send-keys", "-t", "envs:0.0", "G", "Up", "Up", "Up",
                "Up", "Up", "Right",
            )

            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                cleaned = csi.sub(b"", output).replace(b"\x1b(B", b"")
                found = {}
                for line in cleaned.splitlines():
                    for name in (b"OWN_A_SET", b"OWN_B_REM", b"OWN_D_EMPTY"):
                        if name in line:
                            found[name] = line
                if len(found) == 3:
                    assert b"OWN_C_SKIP" not in cleaned, cleaned[-3000:]
                    return [found[name] for name in (b"OWN_A_SET", b"OWN_B_REM", b"OWN_D_EMPTY")]
            raise AssertionError(f"missing environment rows: {output[-3000:]!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                try:
                    client.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    client.kill()
                    client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = environment_rows(binary)
assert actual[0].endswith(b"OWN_A_SET:caf\xc3\xa9:0"), actual
assert actual[1].endswith(b"-OWN_B_REM"), actual
assert actual[2].endswith(b"OWN_D_EMPTY::0"), actual
if baseline is not None:
    expected = environment_rows(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)

print("customize environment CLI checks passed")
