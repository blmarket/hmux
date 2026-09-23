#!/usr/bin/env python3
"""Exercise menu row rendering, filtering, selection, and overlay teardown."""

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

with tempfile.TemporaryDirectory(prefix="menu-owner-", dir=root / "target") as tmp:
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

    try:
        run("new-session", "-d", "-s", "menu", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "menu"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None

        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = subprocess.run(
                base + ["list-clients", "-F", "#{client_tty}"],
                env=env,
                capture_output=True,
                timeout=10,
            )
            if result.returncode == 0 and result.stdout.strip():
                client_tty = result.stdout.strip().decode()
                break
            time.sleep(0.05)
        else:
            raise AssertionError("client did not attach")

        run(
            "display-menu",
            "-c",
            client_tty,
            "-T",
            "Menu title",
            "-s",
            "fg=green,bg=black",
            "-H",
            "fg=red,bg=black",
            "-S",
            "fg=blue,bg=black",
            "first row",
            "a",
            "set-option -g @choice first",
            "",  # separator
            "",  # duplicate separator
            "#{?0,hidden,}",  # omitted after format expansion
            "x",
            "set-option -g @choice hidden",
            "second row",
            "b",
            "set-option -g @choice second",
            "L" * 150,  # trimmed label and key suffix
            "c",
            "set-option -g @choice long",
        )

        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                output.extend(os.read(master, 65536))
            if all(label in output for label in (b"Menu title", b"first row", b"second row", b"L" * 30)):
                break
        else:
            raise AssertionError(f"menu did not render: {output[-1000:]!r}")
        assert b"hidden" not in output, output[-1000:]
        assert b"(c)" in output and re.search(rb"L{30,}>", output), output[-1000:]

        # Replacing the first overlay releases its screen and optional styles.
        run(
            "display-menu", "-c", client_tty, "-T", "Replacement menu",
            "replacement row", "b", "set-option -g @choice second",
        )
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                output.extend(os.read(master, 65536))
            if b"Replacement menu" in output and b"replacement row" in output:
                break
        else:
            raise AssertionError(f"replacement menu did not render: {output[-1000:]!r}")

        os.write(master, b"b")
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = subprocess.run(
                base + ["show-option", "-gqv", "@choice"],
                env=env,
                capture_output=True,
                timeout=10,
            )
            if result.returncode == 0 and result.stdout == b"second\n":
                break
            time.sleep(0.05)
        else:
            raise AssertionError(f"menu choice did not run: {result.stderr!r}")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("menu owner CLI checks passed")
