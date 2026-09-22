#!/usr/bin/env python3
"""Exercise owned winlinks and compare session behavior with a reference tmux."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()


def exercise(executable):
    snapshots = []
    with tempfile.TemporaryDirectory(prefix="session-cli-", dir=root / "target") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
        base = [str(executable), "-S", str(socket), "-f", "/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
            return result.stdout

        def snapshot():
            snapshots.append(run(
                "list-windows", "-a", "-F",
                "#{session_name}:#{window_index}:#{window_id}:#{window_active}:#{window_last_flag}",
            ))

        try:
            run("new-session", "-d", "-s", "a", "-x", "80", "-y", "24", "sleep", "60")
            run("new-window", "-d", "-t", "a:4", "sleep", "60")
            run("new-window", "-d", "-t", "a:9", "sleep", "60")
            run("select-window", "-t", "a:4")
            run("select-window", "-t", "a:9")
            snapshot()

            # Rebuild the owning map, preserving the current and last windows.
            run("move-window", "-r", "-t", "a")
            assert run("list-windows", "-t", "a", "-F", "#{window_index}") == b"0\n1\n2\n"
            assert run("display-message", "-p", "-t", "a", "#{window_id}") == b"@2\n"
            run("last-window", "-t", "a")
            assert run("display-message", "-p", "-t", "a", "#{window_id}") == b"@1\n"
            snapshot()

            # Insert before an occupied index: shuffle existing owners between keys.
            run("new-window", "-b", "-d", "-t", "a:1", "sleep", "60")
            assert run("list-windows", "-t", "a", "-F", "#{window_index}:#{window_id}") == (
                b"0:@0\n1:@3\n2:@1\n3:@2\n"
            )
            run("last-window", "-t", "a")
            assert run("display-message", "-p", "-t", "a", "#{window_id}") == b"@2\n"
            snapshot()

            # Group synchronization replaces each session's links to shared windows.
            run("new-session", "-d", "-s", "b", "-t", "a")
            snapshot()
            run("new-window", "-d", "-t", "b:7", "sleep", "60")
            snapshot()
            run("kill-window", "-t", "@1")
            snapshot()
            run("rename-session", "-t", "a", "z")
            snapshot()
            run("kill-session", "-t", "z")
            snapshot()
            run("select-window", "-t", "b:0")
            run("select-window", "-t", "b:7")
            run("last-window", "-t", "b")
            assert run("display-message", "-p", "-t", "b", "#{window_id}") == b"@0\n"
            snapshot()
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)
            socket.unlink(missing_ok=True)
    return snapshots


actual = exercise(binary)
reference = os.environ.get("TMUX_UPDATE_REFERENCE")
if reference:
    expected = exercise(pathlib.Path(reference).resolve())
    assert actual == expected, (actual, expected)
    print("session CLI checks passed; snapshots match reference tmux")
else:
    print("session CLI checks passed (reference comparison not configured)")
