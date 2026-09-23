#!/usr/bin/env python3
"""Check old control-client layout bytes and select-layout's saved snapshot."""

import os
import pathlib
import re
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="layout-old-format-", dir=root / "target") as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def geometry():
        return run(
            "list-panes", "-t", "layout:0", "-F",
            "#{pane_index}:#{pane_width}x#{pane_height}:#{pane_left}:#{pane_top}",
        )

    def read_until(pattern, start=0):
        deadline = time.monotonic() + 10
        while (match := re.search(pattern, output[start:], re.M | re.S)) is None:
            remaining = deadline - time.monotonic()
            assert remaining > 0, output
            ready, _, _ = select.select([control.stdout], [], [], remaining)
            assert ready, output
            chunk = os.read(control.stdout.fileno(), 65536)
            assert chunk, (output, control.poll())
            output.extend(chunk)
        return match

    def control_command(command):
        start = len(output)
        control.stdin.write(command + b"\n")
        control.stdin.flush()
        match = read_until(rb"^%begin [^\n]+\n(.*?)^%end [^\n]+\n", start)
        return match.group(1)

    control = None
    try:
        run("new-session", "-d", "-s", "layout", "-x", "80", "-y", "24", "sleep 60")
        run("split-window", "-h", "-d", "sleep 60")
        run("select-layout", "even-horizontal")
        run("resize-pane", "-t", "layout:0.0", "-x", "30")
        original = b"0:30x24:0:0\n1:49x24:31:0\n"
        assert geometry() == original

        control = subprocess.Popen(
            base + ["-C", "attach-session", "-t", "layout"],
            env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        output = bytearray()
        read_until(rb"%session-changed \$0 layout\n")

        serialized = control_command(b"display-message -p '#{window_layout}'").strip()
        expected = b"7d15,80x24,0,0{30x24,0,0,0,49x24,31,0,1}"
        assert serialized == expected, serialized

        # The first four bytes are a checksum of the exact old-format body.
        checksum = 0
        for byte in serialized[5:]:
            checksum = ((checksum >> 1) | ((checksum & 1) << 15)) & 0xffff
            checksum = (checksum + byte) & 0xffff
        assert serialized[:4] == f"{checksum:04x}".encode(), serialized

        assert control_command(b"select-layout even-vertical") == b""
        assert geometry() != original
        assert control_command(b"select-layout -o") == b""
        assert geometry() == original
    finally:
        if control is not None:
            control.terminate()
            control.communicate(timeout=5)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("old-format layout owner CLI checks passed")
