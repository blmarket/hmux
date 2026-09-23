#!/usr/bin/env python3
"""Exercise mode-tree filter and search ownership through an attached client."""

import fcntl
import os
import pathlib
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
csi = re.compile(rb"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\([A-Za-z0-9]")


def exercise(binary_path):
    with tempfile.TemporaryDirectory(prefix="mode-tree-filter-search-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout.strip()

        def read_until(markers, excluded=()):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                rendered = csi.sub(b"", output)
                if all(marker in rendered or marker in output for marker in markers):
                    # Collect the rest of this redraw before checking excluded rows.
                    while select.select([master], [], [], 0.05)[0]:
                        output.extend(os.read(master, 65536))
                    rendered = csi.sub(b"", output)
                    frame = rendered[rendered.rfind(b"(0) - "):] if b"(0) - " in rendered else rendered
                    if all(marker in frame or marker in output for marker in markers):
                        assert all(marker not in frame for marker in excluded), frame[-1200:]
                        return frame
            raise AssertionError((markers, output[-1200:]))

        def keys(*values):
            run("send-keys", "-t", "alpha:0.0", *values)

        def literal(value):
            run("send-keys", "-l", "-t", "alpha:0.0", value)

        def in_mode(expected):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if run("display-message", "-p", "-t", "alpha:0.0", "#{pane_in_mode}") == expected:
                    return
                time.sleep(0.05)
            raise AssertionError(f"pane_in_mode did not become {expected!r}")

        try:
            run("new-session", "-d", "-s", "alpha", "sleep 30")
            run("new-session", "-d", "-s", "beta", "sleep 30")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "alpha"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            read_until((b"\x1b[?1006h",))

            alpha_filter = "#{==:#{session_name},alpha}"
            beta_filter = "#{==:#{session_name},beta}"
            options = ("-N", "-F", "ROW:#{session_name}")
            run("choose-tree", "-t", "alpha:0.0", *options, "-f", beta_filter,
                "set-option -gq @tree-picked '%%'")
            read_until((b"ROW:beta",), (b"ROW:alpha",))

            keys("f")
            read_until((b"(filter) " + beta_filter.encode(),))
            keys("C-u")
            literal(alpha_filter)
            keys("Enter")
            read_until((b"ROW:alpha",), (b"ROW:beta",))

            keys("f")
            read_until((b"(filter) " + alpha_filter.encode(),))
            keys("C-u", "Enter")
            read_until((b"ROW:alpha", b"ROW:beta"))

            keys("/")
            read_until((b"(search) ",))
            literal("beta")
            keys("Enter")
            read_until((b"ROW:alpha", b"ROW:beta"))
            keys("/")
            read_until((b"(search) ",))
            literal("alpha")
            keys("Enter")
            read_until((b"ROW:alpha", b"ROW:beta"))
            keys("Enter")
            in_mode(b"0")
            picked = run("show-option", "-gqv", "@tree-picked")
            assert picked == b"=alpha:", picked

            # Closing a pane with an active prompt releases the mode-tree record.
            run("new-window", "-d", "-t", "alpha", "-n", "survivor", "sleep 30")
            run("choose-tree", "-t", "alpha:0.0", *options, "-f", beta_filter)
            read_until((b"ROW:beta",), (b"ROW:alpha",))
            keys("f")
            read_until((b"(filter) " + beta_filter.encode(),))
            run("kill-window", "-t", "alpha:0")
            assert b"survivor" in run("list-windows", "-t", "alpha", "-F", "#{window_name}")
            return picked
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = exercise(binary)
if baseline is not None:
    expected = exercise(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("mode-tree filter/search ownership CLI checks passed")
