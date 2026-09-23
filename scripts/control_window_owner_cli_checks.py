#!/usr/bin/env python3
"""Exercise per-window control-client size ownership on a private server."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="control-window-owner-", dir=root / "target") as tmp:
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def size():
        return run("display-message", "-p", "-t", "cw:0", "#{window_width}x#{window_height}")

    control = None
    try:
        run("new-session", "-d", "-x", "80", "-y", "24", "-s", "cw", "sleep", "30")
        window = run("display-message", "-p", "-t", "cw:0", "#{window_id}").strip().decode()
        control = subprocess.Popen(
            base + ["-C", "attach-session", "-t", "cw"],
            env=env, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        deadline = time.monotonic() + 5
        while not (name := run("list-clients", "-F", "#{client_name}").strip()):
            assert control.poll() is None, "control client exited"
            assert time.monotonic() < deadline, "control client did not attach"
            time.sleep(0.02)
        target = name.decode()

        # Subscription parsing lends its owned name and format only while
        # control_add_sub copies them. Invalid input removes the named entry.
        run("refresh-client", "-t", target, "-B", "@watch::#{session_name}")
        run("refresh-client", "-t", target, "-B", "@watch")
        run("refresh-client", "-t", target, "-B", "@pane:%*:#{pane_id}")

        run("refresh-client", "-t", target, "-C", "100x30")
        assert size() == b"100x30\n"
        run("refresh-client", "-t", target, "-C", f"{window}:40x12")
        assert size() == b"40x12\n"
        run("refresh-client", "-t", target, "-C", f"{window}:50x14")
        assert size() == b"50x14\n"
        run("refresh-client", "-t", target, "-C", f"{window}:")
        assert size() == b"100x30\n"

        # Leave a live record for control_stop to release on disconnect.
        run("refresh-client", "-t", target, "-C", f"{window}:45x13")
        assert size() == b"45x13\n"
        control.terminate()
        control.communicate(timeout=5)
        control = None
        assert run("list-clients", "-F", "#{client_name}") == b""
    finally:
        if control is not None:
            control.terminate()
            control.communicate(timeout=5)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("control window owner CLI checks passed")
