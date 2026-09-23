#!/usr/bin/env python3
"""Exercise multi-pair OSC 4 setting and querying through a live pane."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="osc4-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    bel_reply = pathlib.Path(tmp) / "bel-reply"
    st_reply = pathlib.Path(tmp) / "st-reply"
    env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)

    def wait_for_bytes(path, expected):
        deadline = time.monotonic() + 5
        while (not path.exists() or path.stat().st_size < len(expected)) and time.monotonic() < deadline:
            time.sleep(0.05)
        assert path.exists(), f"pane did not write {path.name}"
        assert path.read_bytes() == expected, path.read_bytes()

    bel_expected = (
        b"\x1b]4;123;rgb:1212/3434/5656\x07"
        b"\x1b]4;124;rgb:abab/cdcd/efef\x07"
    )
    st_expected = b"\x1b]4;124;rgb:abab/cdcd/efef\x1b\\"
    pane = (
        "stty raw -echo; "
        "printf '\\033]4;123;#123456;124;#abcdef;123;?;124;?\\007'; "
        f"dd bs=1 count={len(bel_expected)} of='{bel_reply}' status=none; "
        "printf '\\033]4;124;?\\033\\\\'; "
        f"dd bs=1 count={len(st_expected)} of='{st_reply}' status=none; sleep 1"
    )

    try:
        run("new-session", "-d", "-s", "osc4", "sleep", "30")
        run("new-window", "-d", "-t", "osc4:1", pane)
        wait_for_bytes(bel_reply, bel_expected)
        wait_for_bytes(st_reply, st_expected)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("OSC 4 CLI checks passed")
