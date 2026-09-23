#!/usr/bin/env python3
"""Exercise clock-mode's screen, timer, resize, and teardown on an attached PTY."""

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
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="clock-owner-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 16, 60, 0, 0))
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    def wait_until(check, label):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if check():
                return
            time.sleep(0.05)
        raise AssertionError(f"timed out waiting for {label}")

    def drain():
        output = bytearray()
        while select.select([master], [], [], 0)[0]:
            output.extend(os.read(master, 65536))
        return bytes(output)

    def pane_mode():
        return run("display-message", "-p", "-t", "clock:0.0", "#{pane_mode}")

    try:
        run("new-session", "-d", "-x", "60", "-y", "16", "-s", "clock", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "clock"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None
        wait_until(lambda: bool(run("list-clients", "-F", "#{client_tty}").strip()), "client")
        drain()

        for _ in range(2):
            run("clock-mode", "-t", "clock:0.0")
            wait_until(lambda: pane_mode() == b"clock-mode\n", "clock mode")
            time.sleep(0.1)
            assert drain(), "clock mode did not draw on the attached terminal"

            run("resize-window", "-x", "50", "-y", "14", "-t", "clock:0")
            time.sleep(0.1)
            drain()
            time.sleep(1.2)
            assert drain(), "clock timer did not redraw after a second"

            os.write(master, b"x")
            wait_until(lambda: pane_mode() == b"\n", "clock mode exit")
            drain()
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("clock mode owner CLI checks passed")
