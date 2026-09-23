#!/usr/bin/env python3
"""Exercise command completion with an attached client and a private server."""

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

with tempfile.TemporaryDirectory(prefix="prompt-completion-", dir=root / "target") as tmp:
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

    def wait_for_terminal(*needles):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
            if ready:
                output.extend(os.read(master, 65536))
                if all(needle in output for needle in needles):
                    return bytes(output)
        raise AssertionError(f"terminal did not show {needles!r}: {output[-400:]!r}")

    def prompt(client_tty, label, initial, option):
        global prompt_command
        prompt_command = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-I", initial, "-p", label,
                    f"set-option -g {option} '%%'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(label.encode())

    def finish_prompt():
        global prompt_command
        _, stderr = prompt_command.communicate(timeout=5)
        assert prompt_command.returncode == 0, stderr
        prompt_command = None

    try:
        run("new-session", "-d", "-s", "complete", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "complete"],
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

        # The alias duplicates a built-in command. Completion must retain one name.
        run("set-option", "-ga", "command-alias", "list-clients=display-message duplicate")
        prompt(client_tty, "single", "list-client", "@completion_single")
        os.write(master, b"\t\r")
        finish_prompt()
        assert run("show-options", "-gqv", "@completion_single") == b"list-clients \n"

        run("set-option", "-ga", "command-alias", "zzownerB=display-message B")
        run("set-option", "-ga", "command-alias", "zzownerA=display-message A")
        prompt(client_tty, "multi", "zzowner", "@completion_multi")
        os.write(master, b"\t")
        output = wait_for_terminal(b"zzownerA", b"zzownerB")
        assert output.index(b"zzownerA") < output.index(b"zzownerB"), output[-400:]
        # Typing clears the retained choices; the next Tab builds a single match.
        os.write(master, b"A\t\r")
        finish_prompt()
        assert run("show-options", "-gqv", "@completion_multi") == b"zzownerA \n"

        # The next prompt can retain and then discard its choices on close.
        prompt(client_tty, "close", "zzowner", "@completion_close")
        os.write(master, b"\t")
        wait_for_terminal(b"zzownerA", b"zzownerB")
        os.write(master, b"\x03")  # C-c cancels the command prompt.
        finish_prompt()
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

print("prompt completion CLI checks passed")
