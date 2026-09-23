#!/usr/bin/env python3
"""Check session_attached_list with clients joining and leaving sessions."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="session-attached-list-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def session_names(session):
        value = run("display-message", "-p", "-t", session, "#{session_attached_list}")
        assert value.endswith(b"\n"), value
        return value[:-1].split(b",") if value != b"\n" else []

    def wait_for_names(first_count, second_count):
        deadline = time.monotonic() + 5
        while True:
            rows = run("list-clients", "-F", "#{client_name}|#{session_name}").splitlines()
            names = {b"first": [], b"second": []}
            for row in rows:
                name, session = row.rsplit(b"|", 1)
                names[session].append(name)
            if len(names[b"first"]) == first_count and len(names[b"second"]) == second_count:
                return names
            assert time.monotonic() < deadline, rows
            time.sleep(0.05)

    attached = []
    try:
        for session in ("first", "second"):
            run("new-session", "-d", "-s", session, "sleep 60")
        assert session_names("first") == []

        for session in ("first", "first", "second"):
            attached.append(
                subprocess.Popen(
                    base + ["-C", "attach-session", "-t", session],
                    env=env,
                    stdin=subprocess.PIPE,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            )

        names = wait_for_names(2, 1)
        assert sorted(session_names("first")) == sorted(names[b"first"]), names
        assert session_names("second") == names[b"second"], names

        attached[0].terminate()
        attached[0].communicate(timeout=5)
        names = wait_for_names(1, 1)
        assert session_names("first") == names[b"first"], names
    finally:
        for process in attached:
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=5)
        if socket.exists():
            run("kill-server")

print("session attached list CLI checks passed")
