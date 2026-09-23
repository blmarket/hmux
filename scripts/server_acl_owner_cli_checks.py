#!/usr/bin/env python3
"""Compare ACL entry insertion, flag updates, and removal with the baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="acl-owner-", dir=root / "target") as tmp:
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

        def run(*args, ok=True):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
            if ok:
                assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result

        try:
            run("new-session", "-d", "-s", "acl-owner", "sleep 30")
            initial = run("server-access", "-l").stdout
            run("server-access", "-g", "-a", "root")
            added = run("server-access", "-l").stdout
            duplicate = run("server-access", "-g", "-a", "root", ok=False)
            run("server-access", "-g", "-r", "root")
            readonly = run("server-access", "-l").stdout
            run("server-access", "-g", "-w", "root")
            writable = run("server-access", "-l").stdout
            run("server-access", "-g", "-d", "root")
            removed = run("server-access", "-l").stdout

            assert b"root (G,W)" in added, added
            assert b"root (G,R)" in readonly, readonly
            assert writable == added, (writable, added)
            assert removed == initial, (removed, initial)
            assert duplicate.returncode != 0 and b"already added" in duplicate.stderr
            return initial, added, readonly, writable, removed, duplicate.returncode
        finally:
            run("kill-server")


reference = trace(baseline)
actual = trace(candidate)
assert actual == reference, (actual, reference)
print("server ACL owner CLI checks passed")
