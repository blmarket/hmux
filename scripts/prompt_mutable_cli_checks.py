#!/usr/bin/env python3
"""Exercise prompt label updates and saved incremental input in an attached client."""

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

with tempfile.TemporaryDirectory(prefix="prompt-mutable-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    client = None
    command = None

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
        run("new-session", "-d", "-s", "prompt-owner", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "prompt-owner"],
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

        run("set-option", "-g", "status-keys", "emacs")

        command = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-p", "first,second",
                    "-I", "alpha,beta", "set-option -g @prompt_rows '%1:%2'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(b"first")
        os.write(master, b"\r")
        wait_for_terminal(b"second")
        os.write(master, b"\r")
        _, stderr = command.communicate(timeout=5)
        assert command.returncode == 0, stderr
        command = None
        assert run("show-options", "-gqv", "@prompt_rows") == b"alpha:beta\n"

        command = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-p", "history-entry",
                    "set-option -g @prompt_history_entry '%%'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(b"history-entry")
        os.write(master, b"history-owner\r")
        _, stderr = command.communicate(timeout=5)
        assert command.returncode == 0, stderr
        command = None
        assert run("show-options", "-gqv", "@prompt_history_entry") == b"history-owner\n"

        command = subprocess.Popen(
            base + ["command-prompt", "-t", client_tty, "-p", "history-recall",
                    "set-option -g @prompt_history_recall '%%'"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        wait_for_terminal(b"history-recall")
        os.write(master, b"\x1b[A\r")  # Recall the previous command prompt input.
        _, stderr = command.communicate(timeout=5)
        assert command.returncode == 0, stderr
        command = None
        assert run("show-options", "-gqv", "@prompt_history_recall") == b"history-owner\n"

        run("command-prompt", "-i", "-t", client_tty, "-p", "incremental",
            "-I", "seed", "set-option -g @prompt_saved '%%'")
        wait_for_terminal(b"incremental")
        os.write(master, b"\x12")  # C-r restores the saved incremental input.
        wait_for_terminal(b"seed")
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = subprocess.run(base + ["show-options", "-gqv", "@prompt_saved"],
                                    env=env, capture_output=True, timeout=10)
            if result.returncode == 0 and b"seed" in result.stdout:
                break
            time.sleep(0.05)
        else:
            raise AssertionError(
                f"incremental restore did not emit saved input: {result.returncode}, "
                f"{result.stdout!r}, {result.stderr!r}"
            )
        # Left calls prompt_check_move; command-prompt's MOVE callback closes it.
        os.write(master, b"\x1b[D")
        time.sleep(0.1)
        os.write(master, b"z")
        time.sleep(0.1)
        assert run("show-options", "-gqv", "@prompt_saved") == b"=seed\n"

        run("command-prompt", "-i", "-t", client_tty, "-p", "typed-incremental",
            "set-option -g @prompt_key_incremental '%%'")
        wait_for_terminal(b"typed-incremental")
        os.write(master, b"z\xc3\xa9")
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = subprocess.run(base + ["show-options", "-gqv", "@prompt_key_incremental"],
                                    env=env, capture_output=True, timeout=10)
            if result.returncode == 0 and result.stdout == b"=z\xc3\xa9\n":
                break
            time.sleep(0.05)
        else:
            raise AssertionError(
                f"incremental typing did not emit prefixed input: {result.returncode}, "
                f"{result.stdout!r}, {result.stderr!r}"
            )
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if command is not None and command.poll() is None:
            command.terminate()
            command.wait(timeout=5)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("prompt mutable CLI checks passed")
