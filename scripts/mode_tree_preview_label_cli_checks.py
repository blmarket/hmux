#!/usr/bin/env python3
"""Check both preview-label formatting branches through attached clients."""

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
csi = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")
box = "┌".encode()


def check(command, expected):
    with tempfile.TemporaryDirectory(prefix="mode-tree-preview-", dir=root / "target") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def clean(output):
            return csi.sub(b"", output).replace(b"\x1b(B", b"")

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
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)

            run(*command, "-t", "tree:0.0")
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if box + b" " + expected in clean(output):
                    break
            else:
                raise AssertionError(f"missing preview label {expected!r}: {clean(output)[-1000:]!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


def narrow_label(binary_path, label):
    """Render a five-column window preview with a one-column label limit."""
    with tempfile.TemporaryDirectory(prefix="mode-tree-narrow-", dir=root / "target") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        # mode_tree_draw passes w - 4 = 5 columns to window_tree_draw_window.
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 9, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "tree", "-x", "9", "-y", "30", "sleep 30")
            run("set-option", "-w", "-t", "tree:0", "tree-mode-preview-format", label)
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
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)
            # Exclude the initial attached-client redraw from the comparison.
            while select.select([master], [], [], 0.1)[0]:
                os.read(master, 65536)
            run("choose-tree", "-t", "tree:0.0")
            deadline = time.monotonic() + 5
            while run("display-message", "-p", "-t", "tree:0.0", "#{pane_in_mode}").strip() != b"1":
                assert time.monotonic() < deadline, "choose-tree did not enter mode"
                time.sleep(0.05)
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.25)[0]:
                    output.extend(os.read(master, 65536))
                elif output:
                    break
            assert output, "choose-tree did not draw"
            return bytes(output)
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


check(("choose-tree",), b"0 (sort: index) (view: preview)")
check(("choose-tree", "-r"), b"0 (sort: index, reversed) (view: preview)")
check(("customize-mode",), b"Server Options")
wide = narrow_label(binary, "界")
ascii_label = narrow_label(binary, "Z")
outer_box = "┌───────┐".encode()
inner_box = "┌───┐".encode()
assert outer_box in wide and outer_box in ascii_label, (wide, ascii_label)
assert inner_box not in wide and "界".encode() not in wide, wide
assert inner_box in ascii_label and b"Z" in ascii_label, ascii_label
if baseline is not None:
    baseline_binary = pathlib.Path(baseline).resolve()
    wide_baseline = narrow_label(baseline_binary, "界")
    ascii_baseline = narrow_label(baseline_binary, "Z")
    assert outer_box in wide_baseline and outer_box in ascii_baseline
    assert inner_box not in wide_baseline and "界".encode() not in wide_baseline
    assert inner_box in ascii_baseline and b"Z" in ascii_baseline
print("mode-tree preview label CLI checks passed")
