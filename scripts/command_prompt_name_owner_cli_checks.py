#!/usr/bin/env python3
"""Compare default command-prompt labels for string and command-list templates."""

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


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="command-prompt-name-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        source = pathlib.Path(tmp) / "bindings.conf"
        source.write_text(
            "bind-key -n C-g { command-prompt { set-environment -g PROMPT_BLOCK '%%' } }\n"
        )
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None
        command = None

        def run(*args):
            process = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert process.returncode == 0, (args, process.returncode, process.stderr)
            return process.stdout

        def wait_for_terminal(needle):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
                if ready:
                    output.extend(os.read(master, 65536))
                    if needle in output:
                        return bytes(output)
            raise AssertionError(f"terminal did not show {needle!r}: {output[-400:]!r}")

        try:
            run("new-session", "-d", "-s", "prompt-name-owner", "sleep 30")
            run("source-file", str(source))
            client = subprocess.Popen(
                base + ["attach-session", "-t", "prompt-name-owner"],
                env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None

            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                client_tty = run("list-clients", "-F", "#{client_tty}").strip().decode()
                if client_tty:
                    break
                time.sleep(0.05)
            else:
                raise AssertionError("attached client did not appear")

            command = subprocess.Popen(
                base + ["command-prompt", "-t", client_tty,
                        "set-option -g @prompt_raw '%%'"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            raw_label = b"(set-option)"
            wait_for_terminal(raw_label)
            os.write(master, b"alpha\r")
            _, stderr = command.communicate(timeout=5)
            assert command.returncode == 0, stderr
            command = None
            raw_value = run("show-options", "-gqv", "@prompt_raw")

            os.write(master, b"\x07")  # Trigger the parsed command-list binding.
            block_label = b"(set-environment)"
            wait_for_terminal(block_label)
            os.write(master, b"beta\r")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                result = subprocess.run(
                    base + ["show-environment", "-g", "PROMPT_BLOCK"],
                    env=env, capture_output=True, timeout=10,
                )
                if result.returncode == 0:
                    break
                time.sleep(0.05)
            else:
                raise AssertionError("parsed command-list prompt did not set environment")
            block_value = result.stdout
            assert raw_value == b"alpha\n", raw_value
            assert block_value == b"PROMPT_BLOCK=beta\n", block_value
            return raw_label, raw_value, block_label, block_value
        finally:
            if command is not None:
                command.kill()
                command.communicate(timeout=5)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


assert trace(candidate) == trace(baseline)
print("command prompt name owner CLI checks passed")
