#!/usr/bin/env python3
"""Check the pane scope description exposed by the user systemd manager."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def check(binary_path, socket):
    base = [str(binary_path), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "scope", "-x", "80", "-y", "24", "sleep 30")
        pane_pid = int(run("list-panes", "-t", "scope:0", "-F", "#{pane_pid}").strip())
        parent = subprocess.run(
            ["ps", "-o", "ppid=", "-p", str(pane_pid)],
            env=env, capture_output=True, check=True, timeout=10,
        )
        server_pid = int(parent.stdout.strip())
        expected = f"tmux child pane {pane_pid} launched by process {server_pid}"
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            units = subprocess.run(
                ["systemctl", "--user", "list-units", "tmux-spawn-*.scope",
                 "--no-legend", "--plain"],
                env=env, capture_output=True, check=True, timeout=10,
            ).stdout.decode()
            if any(line.endswith(expected) for line in units.splitlines()):
                return True
            time.sleep(0.1)
        raise AssertionError(f"pane scope description missing: {expected!r}")
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="systemd-pane-description-", dir=root / "target") as tmp:
    directory = pathlib.Path(tmp)
    assert check(binary, directory / "candidate.socket")
    if baseline is not None:
        assert check(pathlib.Path(baseline).resolve(), directory / "baseline.socket")

print("systemd pane description CLI checks passed")
