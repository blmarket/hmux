#!/usr/bin/env python3
"""Exercise key binding parsing and canonical listing on a private socket."""
import os
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
work = root / "target/key-cli"
work.mkdir(parents=True, exist_ok=True)

with tempfile.TemporaryDirectory(prefix="key-cli-", dir=work) as tmp:
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args, ok=True):
        process = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        if ok and process.returncode:
            raise AssertionError((args, process.returncode, process.stdout, process.stderr))
        return process

    try:
        run("new-session", "-d", "-s", "keys", "sleep", "60")
        run("bind-key", "-T", "keycheck", "C-a", "display-message", "control")
        run("bind-key", "-T", "keycheck", "Insert", "display-message", "insert")
        listing = run("list-keys", "-T", "keycheck")
        assert listing.stdout == (
            b"bind-key  -T keycheck IC  display-message insert\n"
            b"bind-key  -T keycheck C-a display-message control\n"
        ), listing.stdout

        run("unbind-key", "-T", "keycheck", "C-a")
        listing = run("list-keys", "-T", "keycheck")
        assert listing.stdout == b"bind-key  -T keycheck IC display-message insert\n", listing.stdout

        run("unbind-key", "-T", "keycheck", "IC")
        listing = run("list-keys", "-T", "keycheck", ok=False)
        assert listing.returncode != 0 and b"doesn't exist" in listing.stderr, listing.stderr
    finally:
        run("kill-server", ok=False)

print("key CLI checks passed")
