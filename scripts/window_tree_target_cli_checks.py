#!/usr/bin/env python3
"""Compare session, window, and pane targets selected from an attached tree."""

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
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
session = "café"


def selected_target(binary_path, kind, path):
    with tempfile.TemporaryDirectory(prefix="window-tree-target-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def wait_for_output(needle):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if needle in output:
                    return
            raise AssertionError((needle, output[-1000:]))

        try:
            run("new-session", "-d", "-s", session, "-n", "first", "sleep 30")
            run("move-window", "-s", f"{session}:0", "-t", f"{session}:7")
            pane_id = run("display-message", "-p", "-t", f"{session}:7.0", "#{pane_id}").strip()
            client = subprocess.Popen(
                base + ["attach-session", "-t", session],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            wait_for_output(b"\x1b[?1006h")
            run("choose-tree", "-t", f"{session}:7.0", "set-option -gq @picked %%")
            wait_for_output(b"(view: preview)")

            if kind == "session":
                run("send-keys", "-t", f"{session}:7.0", "Up")
                expected = f"={session}:".encode()
            elif kind == "window":
                expected = f"={session}:7.".encode()
            else:
                run("send-keys", "-t", f"{session}:7.0", "Down")
                expected = f"={session}:7.".encode() + pane_id
            if path == "enter":
                run("send-keys", "-t", f"{session}:7.0", "Enter")
            else:
                run("send-keys", "-t", f"{session}:7.0", ":")
                wait_for_output(b"(current) ")
                run("send-keys", "-l", "-t", f"{session}:7.0", "set-option -gq @picked %%")
                run("send-keys", "-t", f"{session}:7.0", "Enter")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                value = run("show-option", "-gqv", "@picked").strip()
                if value:
                    assert value == expected, (kind, path, value, expected)
                    return value
                time.sleep(0.05)
            raise AssertionError((kind, path, "selection did not set @picked"))
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


for path in ("enter", "command"):
    for kind in ("session", "window", "pane"):
        actual = selected_target(binary, kind, path)
        if baseline is not None:
            expected = selected_target(pathlib.Path(baseline).resolve(), kind, path)
            assert actual == expected, (kind, path, actual, expected)

print("window-tree target CLI checks passed")
