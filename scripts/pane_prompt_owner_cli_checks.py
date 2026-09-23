#!/usr/bin/env python3
"""Exercise pane prompt submit, cancel, and pane teardown on both binaries."""

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
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="pane-prompt-owner-") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None
        client_tty = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def read_until(needle):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                    if needle in output:
                        return
            raise AssertionError((needle, output[-500:]))

        def start_prompt(label):
            run("command-prompt", "-P", "-b", "-t", client_tty, "-p", label,
                "set-option -g @pane_prompt_owner '%%'")
            read_until(label.encode())

        def option_value():
            result = subprocess.run(
                base + ["show-options", "-gqv", "@pane_prompt_owner"],
                env=env, capture_output=True, timeout=10,
            )
            return result.stdout if result.returncode == 0 else None

        try:
            run("new-session", "-d", "-s", "prompt", "sleep 30")
            run("split-window", "-h", "-t", "prompt:0", "sleep 30")
            run("select-pane", "-t", "prompt:0.1")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "prompt"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not (client_tty := run("list-clients", "-F", "#{client_tty}").strip().decode()):
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.02)

            start_prompt("CANCEL_OWNER:")
            os.write(master, b"\x03")
            time.sleep(0.1)
            assert option_value() in (None, b"")
            start_prompt("SUBMIT_OWNER:")
            os.write(master, b"alpha\r")
            deadline = time.monotonic() + 5
            while option_value() != b"alpha\n":
                assert time.monotonic() < deadline, "submitted prompt did not run"
                time.sleep(0.02)

            start_prompt("TEARDOWN_OWNER:")
            run("kill-pane", "-t", "prompt:0.1")
            assert run("list-panes", "-t", "prompt:0", "-F", "#{pane_index}") == b"0\n"
            assert client.poll() is None
            return b"alpha\n"
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


assert trace(candidate) == trace(baseline)
print("pane prompt owner CLI checks passed")
