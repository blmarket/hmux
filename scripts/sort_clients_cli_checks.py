#!/usr/bin/env python3
"""Exercise client sorting, nested client formats, and choose-client on a live server."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="sort-clients-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    attached = []
    try:
        for session in ("sort-a", "sort-b"):
            run("new-session", "-d", "-s", session, "sleep 60")
            attached.append(
                subprocess.Popen(
                    base + ["-C", "attach-session", "-t", session],
                    env=env,
                    stdin=subprocess.PIPE,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            )

        deadline = time.monotonic() + 5
        while True:
            names = run("list-clients", "-F", "#{client_name}").splitlines()
            if len(names) == 2:
                break
            assert all(p.poll() is None for p in attached), "control client exited"
            assert time.monotonic() < deadline, "control clients did not attach"
            time.sleep(0.05)

        names = sorted(names)
        listed = run(
            "list-clients", "-O", "name", "-F", "#{client_name}|#{session_name}|#{line}"
        ).splitlines()
        assert [line.split(b"|")[0] for line in listed] == names, listed
        assert [line.split(b"|")[2] for line in listed] == [b"0", b"1"], listed
        assert {line.split(b"|")[1] for line in listed} == {b"sort-a", b"sort-b"}, listed

        reversed_rows = run(
            "list-clients", "-O", "name", "-r", "-F", "#{client_name}|#{line}"
        ).splitlines()
        assert reversed_rows == [names[1] + b"|0", names[0] + b"|1"], reversed_rows

        # The nested L modifier calls sort_get_clients again while list-clients
        # still traverses its reverse-sorted result.
        nested = run(
            "list-clients", "-O", "name", "-r", "-F", "#{client_name}|#{L:#{client_name},}"
        ).splitlines()
        assert [line.split(b"|")[0] for line in nested] == names[::-1], nested
        for line in nested:
            inner = line.split(b"|", 1)[1]
            assert sorted(inner.rstrip(b",").split(b",")) == names, line

        assert run("display-message", "-p", "-t", "sort-a:0.0", "#{pane_in_mode}") == b"0\n"
        run("choose-client", "-t", "sort-a:0.0", "-O", "name", "-F", "#{client_name}")
        assert (
            run("display-message", "-p", "-t", "sort-a:0.0", "#{pane_in_mode}|#{pane_mode}")
            == b"1|client-mode\n"
        )
    finally:
        for process in attached:
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=5)
        if socket.exists():
            run("kill-server")

print("sorted clients CLI checks passed")
