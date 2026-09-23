#!/usr/bin/env python3
"""Compare saved status screen lifetimes on attached terminal clients."""

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
    with tempfile.TemporaryDirectory(prefix="status-saved-screen-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
        clients = []

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def attach():
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
            process = subprocess.Popen(
                base + ["attach-session", "-t", "saved"],
                env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            clients.append((process, master))
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline:
                ttys = run("list-clients", "-F", "#{client_tty}").splitlines()
                if len(ttys) == 1:
                    return ttys[0].decode(), master, process
                time.sleep(0.05)
            raise AssertionError("terminal client did not attach")

        def visible(master, marker):
            output = bytearray()
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
                if ready:
                    output.extend(os.read(master, 65536))
                    if marker in output:
                        return
            raise AssertionError(f"status marker {marker!r} not visible: {output[-400:]!r}")

        try:
            run("new-session", "-d", "-s", "saved", "sleep", "30")
            tty, master, process = attach()
            run("display-message", "-c", tty, "-d", "5000", "first-screen-marker")
            visible(master, b"first-screen-marker")
            run("display-message", "-c", tty, "-d", "5000", "second-screen-marker")
            visible(master, b"second-screen-marker")

            prompt = subprocess.Popen(
                base + ["command-prompt", "-t", tty, "-p", "saved-screen-prompt",
                        "-I", "seed", "set-option -g @saved-screen '%%'"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            visible(master, b"saved-screen-prompt")
            os.write(master, b"\r")
            _, prompt_error = prompt.communicate(timeout=6)
            assert prompt.returncode == 0, prompt_error
            saved = run("show-options", "-gqv", "@saved-screen")
            assert saved == b"seed\n", saved

            run("display-message", "-c", tty, "-d", "100", "timer-screen-marker")
            visible(master, b"timer-screen-marker")
            time.sleep(0.2)
            run("display-message", "-c", tty, "-d", "5000", "teardown-screen-marker")
            visible(master, b"teardown-screen-marker")
            run("detach-client", "-t", tty)
            assert process.wait(timeout=6) == 0

            tty2, master2, process2 = attach()
            run("command-prompt", "-b", "-t", tty2, "-p", "teardown-prompt-marker",
                "set-option -g @unused '%%'")
            visible(master2, b"teardown-prompt-marker")
            run("detach-client", "-t", tty2)
            assert process2.wait(timeout=6) == 0
            return saved, run("list-clients", "-F", "#{client_tty}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            for process, master in clients:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=6)
                os.close(master)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected == (b"seed\n", b""), (actual, expected)
print("status saved screen owner CLI checks passed")
