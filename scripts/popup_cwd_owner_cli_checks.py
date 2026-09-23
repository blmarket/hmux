#!/usr/bin/env python3
"""Check popup job cwd with the inherited and explicit paths on a private server."""

import os
import pathlib
import pty
import shlex
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="popup-cwd-owner-", dir=root / "target") as tmp:
    directory = pathlib.Path(tmp)
    session_cwd = directory / "session cwd"
    explicit_cwd = directory / "explicit cwd"
    session_cwd.mkdir()
    explicit_cwd.mkdir()
    socket = directory / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    master, slave = pty.openpty()
    client = None

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "popup-cwd", "-c", str(session_cwd), "sleep 30")
        client = subprocess.Popen(
            base + ["attach-session", "-t", "popup-cwd"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        slave = None

        deadline = time.monotonic() + 5
        while True:
            tty = run("list-clients", "-F", "#{client_tty}").strip()
            if tty:
                break
            assert client.poll() is None and time.monotonic() < deadline, "client did not attach"

        for marker, expected, override in (
            ("default", session_cwd, None),
            ("explicit", explicit_cwd, explicit_cwd),
        ):
            output = directory / marker
            command = f'printf "%s" "$PWD" > {shlex.quote(str(output))}'
            args = ["display-popup", "-c", os.fsdecode(tty), "-E", "-w", "60", "-h", "10"]
            if override is not None:
                args.extend(["-d", str(override)])
            popup = subprocess.Popen(base + args + [command], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            deadline = time.monotonic() + 5
            while not output.exists():
                assert popup.poll() is None and time.monotonic() < deadline, (marker, popup.poll())
                time.sleep(0.02)
            stdout, stderr = popup.communicate(timeout=10)
            assert popup.returncode == 0, (marker, popup.returncode, stdout, stderr)
            assert output.read_bytes() == os.fsencode(expected), (marker, output.read_bytes(), expected)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
        if client is not None:
            client.communicate(timeout=10)
        os.close(master)
        if slave is not None:
            os.close(slave)

print("popup cwd owner CLI checks passed")
