#!/usr/bin/env python3
"""Exercise valid and invalid bracketed styles in a live status line."""

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

with tempfile.TemporaryDirectory(prefix="format-draw-style-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    attached = None
    master = None
    try:
        run("new-session", "-d", "-s", "draw", "sleep", "30")
        run("set-option", "-g", "status-left-length", "80")
        run("set-option", "-g", "status-right", "")
        run("set-option", "-g", "mouse", "on")
        run(
            "bind-key", "-n", "MouseDown1Status", "set-option", "-gF",
            "@clicked", "#{mouse_status_range}",
        )
        run(
            "set-option",
            "-g",
            "status-left",
            "#[fg=red,range=user|red]REDMARK#[unknown-style]BADMARK"
            "#[fg=green,range=user|green]GREENMARK#[norange]",
        )

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
        attached = subprocess.Popen(
            base + ["attach-session", "-t", "draw"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            start_new_session=True,
        )
        os.close(slave)

        output = b""
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                output += os.read(master, 16384)
                if b"REDMARKBADMARK" in output and b"GREENMARK" in output:
                    break
        assert b"REDMARKBADMARK" in output, output
        assert b"GREENMARK" in output, output
        red = output.index(b"\x1b[31m")
        first_text = output.index(b"REDMARKBADMARK")
        green = output.index(b"\x1b[32m", first_text)
        last_text = output.index(b"GREENMARK", green)
        assert red < first_text < green < last_text
        assert attached.poll() is None

        for column, expected in ((2, b"red\n"), (20, b"green\n")):
            os.write(master, f"\x1b[<0;{column};24M".encode())
            deadline = time.monotonic() + 2
            while time.monotonic() < deadline:
                value = run("show-option", "-gv", "@clicked")
                if value == expected:
                    break
                time.sleep(0.05)
            assert value == expected, (column, value)
            os.write(master, f"\x1b[<0;{column};24m".encode())
            # Let the release settle before the next click is classified.
            time.sleep(0.6)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if attached is not None:
            attached.wait(timeout=5)
        if master is not None:
            os.close(master)

print("format draw style CLI checks passed")
