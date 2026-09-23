#!/usr/bin/env python3
"""Check window_active_clients_list with attached clients and a linked window."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="format-active-clients-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def active_names(target):
        value = run("display-message", "-p", "-t", target, "#{window_active_clients_list}")
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
        assert active_names("first:0") == []

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
        assert sorted(active_names("first:0")) == sorted(names[b"first"]), names
        assert active_names("second:0") == names[b"second"], names

        run("link-window", "-s", "first:0", "-t", "second:1")
        run("select-window", "-t", "second:1")
        assert sorted(active_names("first:0")) == sorted(
            names[b"first"] + names[b"second"]
        ), names
        assert active_names("second:0") == [], names

        attached[0].terminate()
        attached[0].communicate(timeout=5)
        names = wait_for_names(1, 1)
        assert sorted(active_names("first:0")) == sorted(
            names[b"first"] + names[b"second"]
        ), names
    finally:
        for process in attached:
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=5)
        if socket.exists():
            run("kill-server")

print("active clients format CLI checks passed")
