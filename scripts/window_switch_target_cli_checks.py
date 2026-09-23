#!/usr/bin/env python3
"""Check switch-mode command targets through an attached client."""

import fcntl
import os
import pathlib
import pty
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="window-switch-target-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    def query(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        return result.stdout if result.returncode == 0 else b""

    def wait_until(check, label):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if check():
                return
            time.sleep(0.05)
        raise AssertionError(f"timed out waiting for {label}")

    try:
        run("new-session", "-d", "-s", "alpha", "sleep 30")
        run("move-window", "-s", "alpha:0", "-t", "alpha:7")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "alpha"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None
        wait_until(lambda: bool(query("list-clients", "-F", "#{client_tty}").strip()), "client")

        for flags, expected in (((), b"=alpha:"), (("-w",), b"=alpha:7.")):
            run("switch-mode", *flags, "-t", "alpha:7.0", "set-option -g @switch_target %1")
            wait_until(
                lambda: query("display-message", "-p", "-t", "alpha:7.0", "#{pane_mode}")
                == b"switch-mode\n",
                "switch mode",
            )
            run("send-keys", "-t", "alpha:7.0", "Enter")
            wait_until(
                lambda: query("show-option", "-gqv", "@switch_target") == expected + b"\n",
                f"target {expected!r}",
            )
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("window-switch target CLI checks passed")
