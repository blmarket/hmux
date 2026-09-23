#!/usr/bin/env python3
"""Check window-tree prompts through an attached client and private socket."""

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


with tempfile.TemporaryDirectory(prefix="window-tree-prompt-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)

    def read_until(marker):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                output.extend(os.read(master, 65536))
            if marker in output:
                return
        raise AssertionError(f"missing {marker!r}; terminal tail={output[-1000:]!r}")

    def send_and_expect(key, marker):
        run("send-keys", "-t", "session'owner:0.0", key)
        read_until(marker)

    try:
        run("new-session", "-d", "-s", "session'owner", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "session'owner"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None
        read_until(b"\x1b[?1006h")
        run("choose-tree", "-t", "session'owner:0.0")
        read_until(b"(view: preview)")

        # choose-tree selects the current window by default.
        send_and_expect("x", b"Kill window 0? ")
        run("send-keys", "-t", "session'owner:0.0", "Escape")
        run("send-keys", "-t", "session'owner:0.0", "Up")
        send_and_expect("x", b"Kill session session'owner? ")
        run("send-keys", "-t", "session'owner:0.0", "Escape")
        run("send-keys", "-t", "session'owner:0.0", "Down")
        run("send-keys", "-t", "session'owner:0.0", "Down")
        send_and_expect("x", b"Kill pane 0? ")
        run("send-keys", "-t", "session'owner:0.0", "Escape")

        run("send-keys", "-t", "session'owner:0.0", "t")
        send_and_expect("X", b"Kill 1 tagged? ")
        run("send-keys", "-t", "session'owner:0.0", "Escape")
        send_and_expect(":", b"(1 tagged) ")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("window-tree prompt CLI checks passed")
