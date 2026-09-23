#!/usr/bin/env python3
"""Exercise switch-mode string ownership and incremental filtering in an attached client."""

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
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def exercise(binary_path):
    with tempfile.TemporaryDirectory(prefix="window-switch-strings-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
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

        def wait_for_render(needle):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                rendered = re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\([A-Za-z0-9]", b"", output)
                if needle in rendered:
                    return
            raise AssertionError((needle, output[-1200:]))

        def enter_mode():
            run("switch-mode", "-F", "OWNED-ROW:#{session_name}", "-t", "alpha:0.0",
                "set-option -gq @switch-picked '%1'")
            wait_until(
                lambda: run("display-message", "-p", "-t", "alpha:0.0", "#{pane_mode}")
                == b"switch-mode\n",
                "switch-mode",
            )

        def picked(expected):
            wait_until(
                lambda: run("show-option", "-gqv", "@switch-picked") == expected + b"\n",
                f"selected target {expected!r}",
            )
            wait_until(
                lambda: run("display-message", "-p", "-t", "alpha:0.0", "#{pane_mode}")
                == b"\n",
                "mode teardown after selection",
            )
            return expected

        try:
            run("new-session", "-d", "-s", "alpha", "sleep 60")
            run("new-session", "-d", "-s", "beta", "sleep 60")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "alpha"],
                env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            wait_until(lambda: bool(run("list-clients", "-F", "#{client_tty}").strip()),
                       "attached client")

            enter_mode()
            wait_for_render(b"OWNED-ROW:alpha")
            run("send-keys", "-l", "-t", "alpha:0.0", "beta")
            wait_for_render(b"OWNED-ROW:beta")
            run("send-keys", "-t", "alpha:0.0", "Enter")
            first = picked(b"=beta:")

            enter_mode()
            wait_for_render(b"OWNED-ROW:alpha")
            run("send-keys", "-l", "-t", "alpha:0.0", "beta")
            wait_for_render(b"OWNED-ROW:beta")
            run("send-keys", "-t", "alpha:0.0", "BSpace", "BSpace", "BSpace", "BSpace")
            wait_for_render(b"OWNED-ROW:alpha")
            run("send-keys", "-l", "-t", "alpha:0.0", "alpha")
            wait_for_render(b"OWNED-ROW:alpha")
            run("send-keys", "-t", "alpha:0.0", "Enter")
            second = picked(b"=alpha:")

            enter_mode()
            wait_for_render(b"OWNED-ROW:alpha")
            run("send-keys", "-t", "alpha:0.0", "Escape")
            wait_until(
                lambda: run("display-message", "-p", "-t", "alpha:0.0", "#{pane_mode}")
                == b"\n",
                "mode teardown after cancel",
            )
            return first, second
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


result = exercise(binary)
if baseline is not None:
    assert result == exercise(pathlib.Path(baseline).resolve())
print("window-switch mode strings CLI checks passed")
