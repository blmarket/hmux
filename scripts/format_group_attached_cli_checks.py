#!/usr/bin/env python3
"""Check grouped attached-client names across joins and a detach."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-group-attached-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def attached_names(session):
        value = run("display-message", "-p", "-t", session, "#{session_group_attached_list}")
        assert value.endswith(b"\n"), value
        return value[:-1].split(b",") if value != b"\n" else []

    def wait_for_clients(group_count, outsider_count):
        deadline = time.monotonic() + 5
        while True:
            rows = run("list-clients", "-F", "#{client_name}|#{session_name}").splitlines()
            group = []
            outsider = []
            for row in rows:
                name, session = row.rsplit(b"|", 1)
                if session in (b"first", b"second"):
                    group.append(name)
                elif session == b"outsider":
                    outsider.append(name)
                else:
                    assert False, row
            if len(group) == group_count and len(outsider) == outsider_count:
                return group, outsider
            assert time.monotonic() < deadline, rows
            time.sleep(0.05)

    attached = []
    try:
        run("new-session", "-d", "-s", "first", "sleep 60")
        run("new-session", "-d", "-s", "second", "-t", "first")
        run("new-session", "-d", "-s", "outsider", "sleep 60")
        assert attached_names("first") == []
        assert attached_names("second") == []
        assert attached_names("outsider") == []

        for session in ("first", "outsider", "second", "first"):
            attached.append(
                subprocess.Popen(
                    base + ["-C", "attach-session", "-t", session],
                    env=env,
                    stdin=subprocess.PIPE,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            )

        group, outsider = wait_for_clients(3, 1)
        assert attached_names("first") == group
        assert attached_names("second") == group
        assert attached_names("outsider") == []
        assert not set(group) & set(outsider)

        attached[0].terminate()
        attached[0].communicate(timeout=5)
        group, outsider = wait_for_clients(2, 1)
        assert attached_names("first") == group
        assert attached_names("second") == group
        assert attached_names("outsider") == []
    finally:
        for process in attached:
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=5)
        if socket.exists():
            run("kill-server")

print("group attached-list CLI checks passed")
