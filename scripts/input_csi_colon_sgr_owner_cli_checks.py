#!/usr/bin/env python3
"""Compare repeated CSI colon SGR parsing, replacement, and pane teardown."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def trace(binary, directory):
    base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
    target = "sgr:1.0"

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (binary, args, result.returncode, result.stderr)
        return result.stdout

    fd = None
    try:
        run("new-session", "-d", "-x", "80", "-y", "20", "-s", "sgr", "sleep", "60")
        run("new-window", "-d", "-t", "sgr:1", "sleep", "60")
        tty = run("display-message", "-p", "-t", target, "#{pane_tty}").strip()
        fd = os.open(os.fsdecode(tty), os.O_WRONLY | os.O_NOCTTY)

        # Each CSI replaces the previous parameter list. The valid and malformed
        # colon strings exercise both the colour parser and parameter cleanup.
        lines = (
            b"\x1b[38:2::17:34:51;48:5:196;4:3mR00\r\n",
            b"\x1b[38:5:82;48:2::4:5:6;4:2mR01\r\n",
            b"\x1b[38:2:abc:2:3;48:5:25;4:1mR02\r\n",
            b"\x1b[38:2:7:8:9;48:2::10:11:12;4:3mR03\r\n",
            b"\x1b[38:5:123;48:5:45;4:2mR04\r\n",
            b"\x1b[38:2::90:80:70;48:5:201;4:1mR05\r\n",
        )
        for line in lines:
            os.write(fd, line)

        deadline = time.monotonic() + 5
        while True:
            plain = run("capture-pane", "-p", "-t", target, "-S", "0")
            if b"R05\n" in plain:
                break
            assert time.monotonic() < deadline, plain
            time.sleep(0.02)
        for marker in (b"R00", b"R01", b"R02", b"R03", b"R04", b"R05"):
            assert marker in plain, (marker, plain)
        escaped = run("capture-pane", "-p", "-e", "-t", target, "-S", "0")
        assert escaped.count(b"\x1b[") >= 6, escaped

        # Keep two INPUT_STRING entries in the final parameter list. Closing
        # this pane destroys its parser before another CSI can replace them.
        os.write(fd, b"\x1b[38:5:123;48:5:45mTAIL")
        deadline = time.monotonic() + 5
        while True:
            final = run("capture-pane", "-p", "-e", "-t", target, "-S", "0")
            if b"TAIL" in final:
                break
            assert time.monotonic() < deadline, final
            time.sleep(0.02)
        os.close(fd)
        fd = None
        run("kill-window", "-t", "sgr:1")
        remaining = run("list-windows", "-t", "sgr", "-F", "#{window_index}")
        assert remaining == b"0\n", remaining
        return plain, escaped, final, remaining
    finally:
        if fd is not None:
            os.close(fd)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="input-csi-colon-sgr-", dir=root / "target") as tmp:
    candidate_dir = pathlib.Path(tmp) / "candidate"
    baseline_dir = pathlib.Path(tmp) / "baseline"
    candidate_dir.mkdir()
    baseline_dir.mkdir()
    assert trace(candidate, candidate_dir) == trace(baseline, baseline_dir)

print("input CSI colon SGR owner CLI checks passed")
