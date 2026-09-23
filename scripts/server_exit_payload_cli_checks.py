#!/usr/bin/env python3
"""Compare MSG_EXIT without and with an error message against the baseline."""

import os
import pathlib
import pty
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
    with tempfile.TemporaryDirectory(prefix="server-exit-payload-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
        try:
            normal = subprocess.run(
                base + ["new-session", "-d", "-s", "exit-payload"],
                env=env, capture_output=True, timeout=10,
            )
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)

        missing_socket = pathlib.Path(tmp) / "missing-parent" / "socket"
        error = subprocess.run(
            [str(binary), "-S", str(missing_socket), "-f", "/dev/null",
             "new-session", "-d", "-s", "exit-payload"],
            env=env, capture_output=True, timeout=10,
        )
        assert normal.returncode == 0, (normal.returncode, normal.stderr)
        assert normal.stdout == normal.stderr == b"", (normal.stdout, normal.stderr)

        attached_session = subprocess.run(
            base + ["new-session", "-d", "-s", "attached-exit", "sleep 30"],
            env=env, capture_output=True, timeout=10,
        )
        assert attached_session.returncode == 0, attached_session.stderr
        master, slave = pty.openpty()
        client = subprocess.Popen(
            base + ["attach-session", "-t", "attached-exit"],
            env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
        )
        os.close(slave)
        try:
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                clients = subprocess.run(
                    base + ["list-clients", "-F", "#{client_tty}"],
                    env=env, capture_output=True, timeout=10,
                )
                if clients.returncode == 0 and clients.stdout.strip():
                    break
                time.sleep(0.05)
            else:
                raise AssertionError("attached client did not appear")
            killed = subprocess.run(
                base + ["kill-session", "-t", "attached-exit"],
                env=env, capture_output=True, timeout=10,
            )
            assert killed.returncode == 0, killed.stderr
            attached_returncode = client.wait(timeout=10)
            output = bytearray()
            while True:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                output.extend(chunk)
            assert attached_returncode == 0, attached_returncode
            assert output.endswith(b"[exited]\r\n"), output[-200:]
        finally:
            if client.poll() is None:
                client.kill()
                client.wait(timeout=5)
            os.close(master)

        expected_error = f"error creating {missing_socket} (No such file or directory)\n".encode()
        assert error.returncode == 1, (error.returncode, error.stderr)
        assert error.stdout == b"" and error.stderr == expected_error, (error.stdout, error.stderr)
        return (
            normal.returncode, normal.stdout, normal.stderr,
            attached_returncode, bytes(output[-10:]),
            error.returncode, error.stdout,
            error.stderr.replace(os.fsencode(missing_socket), b"<socket>"),
        )


assert trace(candidate) == trace(baseline)
print("server exit payload CLI checks passed")
