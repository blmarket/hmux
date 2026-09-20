#!/usr/bin/env python3
"""Exercise argument conversion through real commands on a private socket."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", str(root / "target/debug/hmux2"))).resolve()
work = root / "target/consolidation"
work.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


with tempfile.TemporaryDirectory(prefix="arguments-cli-", dir=work) as tmp:
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(
            base + list(args), env=env, capture_output=True, text=True, timeout=20
        )
        if result.returncode:
            raise RuntimeError((args, result.returncode, result.stdout, result.stderr))
        return result.stdout

    def fail(*args, stderr):
        result = subprocess.run(
            base + list(args), env=env, capture_output=True, text=True, timeout=20
        )
        if result.returncode == 0 or result.stderr != stderr:
            raise RuntimeError((args, result.returncode, result.stdout, result.stderr))

    try:
        run("new-session", "-d", "-s", "arguments", "-x", "80", "-y", "24", "sleep", "60")
        assert run("display-message", "-p", "-d", "0", "#{window_width}x#{window_height}") == "80x24\n"
        fail("display-message", "-p", "-d", "1x", "#{window_width}", stderr="delay invalid\n")

        run("resize-window", "-x", "1")
        fail("resize-window", "-x", "0", stderr="width too small\n")
        run("resize-pane", "-t", "0", "-x", "1000%")
        fail("resize-pane", "-t", "0", "-x", "1001%", stderr="width too large\n")
        fail("resize-pane", "-t", "0", "-x", "-1%", stderr="width too small\n")

        run("send-keys", "-N", "1", "a")
        fail("send-keys", "-N", "bad", "a", stderr="repeat count invalid\n")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("arguments CLI checks passed")
