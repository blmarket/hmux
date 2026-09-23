#!/usr/bin/env python3
"""Exercise choose-tree string options and deferred teardown in an attached client."""

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
    with tempfile.TemporaryDirectory(prefix="window-tree-strings-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        # The choose-tree help box needs 36 rows, including its border.
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 50, 100, 0, 0))
        client = None
        output = bytearray()

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def wait_for(predicate, description):
            deadline = time.monotonic() + 5
            while not predicate():
                assert time.monotonic() < deadline, (description, output[-1000:])
                time.sleep(0.05)

        def read_until(marker):
            deadline = time.monotonic() + 5
            while marker not in output:
                assert time.monotonic() < deadline, (marker, output[-1000:])
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))

        try:
            run("new-session", "-d", "-s", "tree", "sleep 30")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "tree"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            read_until(b"\x1b[?1006h")

            options = ("-F", "owned-format-#{session_name}", "-K", "z")
            run("choose-tree", "-t", "tree:0.0", *options, "set-option -gq @picked %%")
            read_until(b"owned-format-tree")
            run("send-keys", "-t", "tree:0.0", "z")
            wait_for(lambda: bool(run("show-option", "-gqv", "@picked").strip()), "custom key command")
            picked = run("show-option", "-gqv", "@picked").strip()

            output.clear()
            run("choose-tree", "-t", "tree:0.0", *options)
            read_until(b"owned-format-tree")
            output.clear()
            run("send-keys", "-t", "tree:0.0", "C-h")
            read_until(b"Exit mode")
            help_rendered = csi.sub(b"", output)
            help_lines = (
                b"Move cursor up",
                b"Collapse item",
                b"Choose selected item",
                b"Exit mode",
            )
            assert all(line in help_rendered for line in help_lines), help_rendered[-2000:]
            # The first key dismisses the overlay; the next exits choose-tree.
            run("send-keys", "-t", "tree:0.0", "q", "q")
            wait_for(
                lambda: run("display-message", "-p", "-t", "tree:0.0", "#{pane_in_mode}").strip() == b"0",
                "help overlay teardown",
            )

            output.clear()
            run("choose-tree", "-t", "tree:0.0", *options)
            read_until(b"(view: preview)")
            run("send-keys", "-t", "tree:0.0", ":")
            read_until(b"(current) ")
            run("send-keys", "-t", "tree:0.0", "Enter")
            time.sleep(0.2)
            run("send-keys", "-t", "tree:0.0", "q")
            wait_for(
                lambda: run("display-message", "-p", "-t", "tree:0.0", "#{pane_in_mode}").strip() == b"0",
                "empty prompt teardown",
            )

            output.clear()
            run("choose-tree", "-t", "tree:0.0", *options)
            read_until(b"owned-format-tree")
            run("send-keys", "-t", "tree:0.0", ":")
            run("send-keys", "-l", "-t", "tree:0.0", "run-shell 'sleep 0.2'; set-option -gq @prompt_hit yes")
            run("send-keys", "-t", "tree:0.0", "Enter")
            run("send-keys", "-t", "tree:0.0", "q")
            wait_for(
                lambda: run("display-message", "-p", "-t", "tree:0.0", "#{pane_in_mode}").strip() == b"0",
                "accepted prompt teardown",
            )
            wait_for(lambda: run("show-option", "-gqv", "@prompt_hit").strip() == b"yes", "deferred command")
            prompt_hit = run("show-option", "-gqv", "@prompt_hit").strip()

            # Destroy the pane while its mode-tree prompt is still active.
            run("new-window", "-d", "-t", "tree", "-n", "survivor", "sleep 30")
            output.clear()
            run("choose-tree", "-t", "tree:0.0", *options)
            read_until(b"owned-format-tree")
            run("send-keys", "-t", "tree:0.0", ":")
            read_until(b"(current) ")
            run("kill-window", "-t", "tree:0")
            assert b"survivor" in run("list-windows", "-t", "tree", "-F", "#{window_name}")
            return picked, prompt_hit
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                try:
                    client.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    client.kill()
                    client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = exercise(binary)
if baseline is not None:
    expected = exercise(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("window-tree strings ownership CLI checks passed")
