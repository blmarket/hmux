#!/usr/bin/env python3
"""Exercise popup strings, job output, modification, and overlay teardown."""

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

with tempfile.TemporaryDirectory(prefix="popup-owner-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    client = None
    first_popup = None
    detached_popup = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def collect(*needles):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                chunk = os.read(master, 65536)
                if not chunk:
                    break
                output.extend(chunk)
            if all(needle in output for needle in needles):
                return bytes(output)
        raise AssertionError(f"popup output missing {needles!r}: {output[-1000:]!r}")

    try:
        run("new-session", "-d", "-s", "popup", "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "popup"],
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
            tty = run("list-clients", "-F", "#{client_tty}").strip()
            if tty:
                break
            time.sleep(0.05)
        else:
            raise AssertionError("client did not attach")
        tty = tty.decode()

        first_popup = subprocess.Popen(
            base + ["display-popup", "-c", tty, "-T", "First popup",
            "-s", "fg=green,bg=black", "-S", "fg=red,bg=black",
            "-w", "40", "-h", "10",
            "printf 'POPUP_BODY\\n'; sleep 5"],
            env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        collect(b"First popup", b"POPUP_BODY")

        # A second display-popup updates the existing owner and its strings.
        run(
            "display-popup", "-c", tty, "-T", "Second popup",
            "-s", "fg=blue,bg=black", "-S", "fg=yellow,bg=black",
        )
        collect(b"Second popup")

        run("display-popup", "-c", tty, "-C")
        stdout, stderr = first_popup.communicate(timeout=10)
        assert first_popup.returncode is not None, (stdout, stderr)
        assert run("list-clients", "-F", "#{client_tty}").strip() == tty.encode()

        # Automatic close also runs the job completion and overlay free callbacks.
        run(
            "display-popup", "-c", tty, "-E", "-T", "Exit popup",
            "-s", "fg=cyan,bg=black", "-S", "fg=magenta,bg=black",
            "printf 'EXIT_BODY\\n'; sleep 0.2",
        )
        collect(b"Exit popup", b"EXIT_BODY")
        time.sleep(0.4)
        assert run("list-clients", "-F", "#{client_tty}").strip() == tty.encode()

        # Losing the attached client releases an active popup through its
        # overlay callback while its job is still running.
        detached_popup = subprocess.Popen(
            base + ["display-popup", "-c", tty, "-T", "Detach popup",
            "-s", "fg=green,bg=black", "-S", "fg=red,bg=black",
            "printf 'DETACH_BODY\\n'; sleep 5"],
            env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        collect(b"Detach popup", b"DETACH_BODY")
        run("detach-client", "-t", tty)
        detached_popup.communicate(timeout=10)
        assert detached_popup.returncode is not None
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if detached_popup is not None and detached_popup.poll() is None:
            detached_popup.terminate()
            detached_popup.wait(timeout=5)
        if first_popup is not None and first_popup.poll() is None:
            first_popup.terminate()
            first_popup.wait(timeout=5)
        if client is not None:
            if client.poll() is None:
                client.terminate()
            client.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)

print("popup owner CLI checks passed")
