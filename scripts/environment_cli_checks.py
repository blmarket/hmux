#!/usr/bin/env python3
"""Exercise environment propagation and respawn through an isolated server."""
import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
work = root / "target/environment-cli"
work.mkdir(parents=True, exist_ok=True)
environment = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="env-", dir=work) as directory:
    socket = pathlib.Path(directory) / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args, ok=True):
        result = subprocess.run(
            base + list(args),
            env=environment,
            capture_output=True,
            timeout=20,
        )
        if ok and result.returncode:
            raise RuntimeError((args, result.returncode, result.stdout, result.stderr))
        return result

    def wait_for_output(target, expected):
        for _ in range(100):
            result = run("capture-pane", "-p", "-t", target, ok=False)
            if result.returncode == 0 and expected in result.stdout:
                return
            time.sleep(0.05)
        raise AssertionError((target, expected, result.stdout, result.stderr))

    try:
        run("new-session", "-d", "-s", "environment", "-x", "80", "-y", "24", "sleep 60")
        run("set-environment", "-g", "HMUX_OWNER", "live")
        assert run("show-environment", "-g", "HMUX_OWNER").stdout == b"HMUX_OWNER=live\n"

        # -r retains the entry but removes its value; an unknown name is an
        # error, so missing and valueless entries are distinct at the CLI too.
        run("set-environment", "-g", "-r", "HMUX_EMPTY")
        assert run("show-environment", "-g", "HMUX_EMPTY").stdout == b"-HMUX_EMPTY\n"
        assert run("show-environment", "-g", "HMUX_MISSING", ok=False).returncode != 0

        command = 'printf "spawn:%s\\n" "$HMUX_OWNER"; sleep 60'
        run("new-window", "-d", "-n", "spawn", command)
        wait_for_output("environment:spawn.0", b"spawn:live")

        respawn = 'printf "respawn:%s\\n" "$HMUX_OWNER"; sleep 60'
        run("respawn-pane", "-k", "-t", "environment:spawn.0", respawn)
        wait_for_output("environment:spawn.0", b"respawn:live")
    finally:
        run("kill-server", ok=False)

print("environment CLI checks passed")
