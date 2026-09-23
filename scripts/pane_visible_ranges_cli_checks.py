#!/usr/bin/env python3
"""Exercise pane-owned visible ranges through an attached floating-pane redraw."""

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
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")


def read_until(fd, needle, timeout=5):
    output = b""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if select.select([fd], [], [], 0.1)[0]:
            output += os.read(fd, 65536)
            if needle in output:
                return output
    raise AssertionError((needle, output[-500:]))


def drain(fd):
    while select.select([fd], [], [], 0)[0]:
        os.read(fd, 65536)


with tempfile.TemporaryDirectory(prefix="pane-visible-ranges-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    attached = None
    master = None
    try:
        run(
            "new-session",
            "-d",
            "-s",
            "ranges",
            "-x",
            "100",
            "-y",
            "24",
            "printf '\\n\\n\\n\\n\\n\\n\\nLEFT_MARK%60sRIGHT_MARK\\n' ''; sleep 30",
        )
        run("set-option", "-g", "status", "off")

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
        attached = subprocess.Popen(
            base + ["attach-session", "-t", "ranges"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            start_new_session=True,
        )
        os.close(slave)
        read_until(master, b"RIGHT_MARK")
        drain(master)
        client_tty = run("list-clients", "-F", "#{client_tty}").strip().decode()

        run("new-pane", "-d", "-x", "20", "-y", "10", "-X", "30", "-Y", "5", "sleep 30")
        panes = run("list-panes", "-F", "#{pane_floating_flag} #{pane_left} #{pane_top}")
        assert b"1 31 6" in panes, panes
        run("refresh-client", "-t", client_tty)
        output = read_until(master, b"RIGHT_MARK")
        assert b"LEFT_MARK" in output, output[-500:]
        assert attached.poll() is None

        run("kill-pane", "-t", "ranges:0.1")
        run("kill-pane", "-t", "ranges:0.0")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if attached is not None:
            attached.wait(timeout=5)
        if master is not None:
            os.close(master)

print("pane visible ranges CLI checks passed")
