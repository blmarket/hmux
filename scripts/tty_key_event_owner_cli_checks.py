#!/usr/bin/env python3
"""Exercise queued terminal and synthetic key events on a private socket."""
import os
import pathlib
import pty
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="tty-key-event-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    attached = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        if result.returncode:
            raise AssertionError((args, result.returncode, result.stdout, result.stderr))
        return result.stdout

    def wait_for_lines(expected):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            lines = run("capture-pane", "-p", "-t", "events:0.0").splitlines()
            if lines[:len(expected)] == expected:
                return
            time.sleep(0.05)
        raise AssertionError((expected, lines[:len(expected)]))

    try:
        run("new-session", "-d", "-s", "events", "cat")
        master, slave = pty.openpty()
        tty = os.ttyname(slave)
        attached = subprocess.Popen(
            base + ["attach-session", "-t", "events"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        deadline = time.monotonic() + 5
        while tty.encode() not in run("list-clients", "-F", "#{client_tty}"):
            assert time.monotonic() < deadline, "client did not attach"
            time.sleep(0.05)

        # Terminal bytes supply the borrowed evbuffer source for the queued event.
        os.write(master, b"A\x00B\n")
        wait_for_lines([b"A^@B", b"AB"])

        # -K creates an event with no byte buffer and inserts it into the same queue.
        run("send-keys", "-K", "-c", tty, "C")
        run("send-keys", "-K", "-c", tty, "Enter")
        wait_for_lines([b"A^@B", b"AB", b"C", b"C"])
        print(repr(run("capture-pane", "-p", "-t", "events:0.0").splitlines()[:4]))
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)
        if attached is not None:
            if attached.poll() is None:
                attached.terminate()
            attached.wait(timeout=5)
        if "master" in locals():
            os.close(master)

print("tty key event CLI checks passed")
