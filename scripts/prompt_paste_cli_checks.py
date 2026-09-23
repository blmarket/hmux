#!/usr/bin/env python3
"""Paste a UTF-8 buffer into the middle of a live command prompt."""

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

with tempfile.TemporaryDirectory(prefix="prompt-paste-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    client = None
    prompt_command = None

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
                    return
        raise AssertionError(f"prompt not visible: {output[-300:]!r}")

    try:
        run("new-session", "-d", "-s", "paste", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "paste"],
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

        run("set-buffer", "--", "éZ")
        prompt_command = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-I", "ab", "-p", "probe", "set-option -g @probe '%%'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(b"probe")
        os.write(master, b"\x1b[D\x19\r")  # Left, C-y, Enter.
        _, stderr = prompt_command.communicate(timeout=5)
        assert prompt_command.returncode == 0, stderr

        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = subprocess.run(
                base + ["show-options", "-gqv", "@probe"],
                env=env,
                capture_output=True,
                timeout=10,
            )
            if result.returncode == 0 and result.stdout:
                assert result.stdout == "aéZb\n".encode(), result.stdout
                break
            time.sleep(0.05)
        else:
            raise AssertionError(f"prompt did not set @probe: {result.stderr!r}")

        # A copied prompt word takes precedence over the top paste buffer.
        prompt_command = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-I", "abc def", "-p", "copied", "set-option -g @copied '%%'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(b"copied")
        os.write(master, b"\x17\x19\r")  # C-w, C-y, Enter.
        _, stderr = prompt_command.communicate(timeout=5)
        assert prompt_command.returncode == 0, stderr
        assert run("show-options", "-gqv", "@copied") == b"abc def\n"
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if prompt_command is not None and prompt_command.poll() is None:
            prompt_command.terminate()
            prompt_command.wait(timeout=5)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("prompt paste CLI checks passed")
