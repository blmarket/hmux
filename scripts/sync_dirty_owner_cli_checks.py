#!/usr/bin/env python3
"""Check synchronized pane redraw and dirty bitmap cleanup with an attached client."""

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

with tempfile.TemporaryDirectory(prefix="sync-dirty-owner-", dir=root / "target") as tmp:
    directory = pathlib.Path(tmp)
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
    start_one = directory / "start-one"
    finish_one = directory / "finish-one"
    start_two = directory / "start-two"
    finish_two = directory / "finish-two"
    marker_one = b"SYNC_BITMAP_ONE"
    marker_two = b"SYNC_BITMAP_TWO"
    pane = (
        "stty raw -echo; "
        f"while [ ! -e '{start_one}' ]; do sleep 0.02; done; "
        "printf '\\033[?2026hSYNC_BITMAP_ONE\\n'; "
        f"while [ ! -e '{finish_one}' ]; do sleep 0.02; done; "
        "printf '\\033[?2026l'; "
        f"while [ ! -e '{start_two}' ]; do sleep 0.02; done; "
        "printf '\\033[?2026hSYNC_BITMAP_TWO\\n'; "
        f"while [ ! -e '{finish_two}' ]; do sleep 0.02; done; "
        "printf '\\033[?2026l'; sleep 30"
    )

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def capture_until(marker):
        deadline = time.monotonic() + 5
        while True:
            captured = run("capture-pane", "-p", "-t", "sync:0.0")
            if marker in captured:
                return
            assert time.monotonic() < deadline, captured
            time.sleep(0.02)

    def read_terminal(duration):
        output = bytearray()
        deadline = time.monotonic() + duration
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], min(0.05, deadline - time.monotonic()))
            if ready:
                output.extend(os.read(master, 65536))
        return bytes(output)

    def wait_terminal(marker):
        output = bytearray()
        deadline = time.monotonic() + 5
        while marker not in output:
            assert time.monotonic() < deadline, output[-300:]
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                output.extend(os.read(master, 65536))
        return bytes(output)

    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    client = None
    try:
        run("new-session", "-d", "-s", "sync", pane)
        client = subprocess.Popen(
            base + ["attach-session", "-t", "sync"],
            env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
        )
        os.close(slave)
        slave = None
        deadline = time.monotonic() + 5
        while not run("list-clients", "-F", "#{client_name}").strip():
            assert client.poll() is None, "client exited before attach"
            assert time.monotonic() < deadline, "client did not attach"
            time.sleep(0.02)
        read_terminal(0.1)

        start_one.touch()
        capture_until(marker_one)
        assert marker_one not in read_terminal(0.2)
        finish_one.touch()
        assert marker_one in wait_terminal(marker_one)

        run("resize-window", "-t", "sync:0", "-x", "50", "-y", "10")
        read_terminal(0.1)
        start_two.touch()
        capture_until(marker_two)
        assert marker_two not in read_terminal(0.2)
        finish_two.touch()
        assert marker_two in wait_terminal(marker_two)
    finally:
        if client is not None:
            client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("sync dirty owner CLI checks passed")
