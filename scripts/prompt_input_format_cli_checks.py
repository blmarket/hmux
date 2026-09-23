#!/usr/bin/env python3
"""Exercise live #{prompt_input} expansion in an attached command prompt."""

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

with tempfile.TemporaryDirectory(prefix="prompt-input-format-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    client = None
    prompt = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

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
        run("new-session", "-d", "-s", "prompt-format", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "prompt-format"],
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
            client_tty = run("list-clients", "-F", "#{client_tty}").strip().decode()
            if client_tty:
                break
            time.sleep(0.05)
        else:
            raise AssertionError("attached client did not appear")

        prompt = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-I", "é", "-p",
                    "input=#{prompt_input}:", "set-option -g @prompt_input_format '%%'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(b"input=\xc3\xa9:")
        os.write(master, b"x")
        wait_for_terminal(b"input=\xc3\xa9x:")
        os.write(master, b"\r")
        _, stderr = prompt.communicate(timeout=5)
        assert prompt.returncode == 0, stderr
        prompt = None
        assert run("show-options", "-gqv", "@prompt_input_format") == b"\xc3\xa9x\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if prompt is not None and prompt.poll() is None:
            prompt.terminate()
            prompt.wait(timeout=5)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("prompt input format CLI checks passed")
