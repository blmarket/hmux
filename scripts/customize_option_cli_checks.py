#!/usr/bin/env python3
"""Exercise adding a server user option through the customize-mode prompt."""

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

with tempfile.TemporaryDirectory(prefix="customize-option-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def wait_for_terminal(needle):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
            if ready:
                try:
                    output.extend(os.read(master, 65536))
                except OSError as exc:
                    raise AssertionError(f"attached client closed: {output[-300:]!r}") from exc
                if needle in output:
                    return
        raise AssertionError(f"did not see {needle!r} on terminal: {output[-300:]!r}")

    try:
        run("new-session", "-d", "-s", "opts", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "opts"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None

        run("customize-mode", "-t", "opts:0.0")
        run("send-keys", "-t", "opts:0.0", "s")
        wait_for_terminal(b"New user option: @")
        run("send-keys", "-t", "opts:0.0", "probe value with spaces", "Enter")

        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = subprocess.run(
                base + ["show-options", "-sv", "@probe"],
                env=env,
                capture_output=True,
                timeout=10,
            )
            if result.returncode == 0:
                assert result.stdout == b"value with spaces\n", result.stdout
                break
            time.sleep(0.05)
        else:
            raise AssertionError(f"customize-mode did not add @probe: {result.stderr!r}")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("customize option CLI checks passed")
