#!/usr/bin/env python3
"""Exercise choose-client owned strings through an attached client."""

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


def exercise(binary_path):
    with tempfile.TemporaryDirectory(prefix="window-client-mode-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout.strip()

        def wait_for_output(needle):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if needle in output:
                    return
            raise AssertionError((needle, output[-1200:]))

        def wait_for_option(name):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                value = run("show-option", "-gqv", name)
                if value:
                    return value
                time.sleep(0.05)
            raise AssertionError(f"{name} was not set")

        try:
            run("new-session", "-d", "-s", "mode", "sleep 30")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "mode"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}"):
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)
            tty = run("list-clients", "-F", "#{client_tty}")

            # The first key is supplied by -K and selects the first client.
            run("choose-client", "-F", "OWNED-ROW:#{client_tty}",
                "-f", "#{==:#{client_tty}," + tty.decode() + "}",
                "-K", "#{e|+|:#{line},1}",
                "-t", "mode:0.0", "set-option -gq @client-picked '%%'")
            wait_for_output(b"OWNED-ROW:" + tty)
            run("send-keys", "-t", "mode:0.0", "i")
            wait_for_output(b"Client Name")
            run("send-keys", "-t", "mode:0.0", "1")
            selected = wait_for_option("@client-picked")
            assert selected == tty, (selected, tty)

            # Recreate and cancel the mode, then detach while its data is live.
            # An empty filter result rejects the client before the tree's
            # no-match fallback rebuilds it without the filter.
            run("choose-client", "-F", "CANCEL-ROW", "-f", "#{l:}",
                "-K", "2", "-t", "mode:0.0")
            wait_for_output(b"no matches")
            run("send-keys", "-t", "mode:0.0", "q")
            run("choose-client", "-F", "DETACH-ROW", "-K", "3", "-t", "mode:0.0")
            wait_for_output(b"DETACH-ROW")
            run("detach-client", "-t", tty.decode())
            assert client.wait(timeout=5) == 0
            run("list-sessions")
            return True
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
print("window-client mode strings CLI checks passed")
