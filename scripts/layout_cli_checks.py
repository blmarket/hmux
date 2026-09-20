#!/usr/bin/env python3
"""Exercise custom-layout application and resize behavior on a private socket."""
import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="layout-cli-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args, ok=True):
        process = subprocess.run(
            base + list(args),
            env=env,
            capture_output=True,
            text=True,
            timeout=20,
        )
        if ok and process.returncode:
            raise AssertionError((args, process.returncode, process.stdout, process.stderr))
        return process

    def pane_geometry():
        return run(
            "list-panes",
            "-F",
            "#{pane_index}:#{pane_width}x#{pane_height}:#{pane_left}:#{pane_top}",
        ).stdout

    try:
        run("new-session", "-d", "-s", "layout", "-x", "80", "-y", "24", "sleep", "60")
        run("split-window", "-h", "-d", "sleep", "60")

        initial = pane_geometry()
        run("select-layout", "even-horizontal")
        run("resize-pane", "-t", "0", "-x", "30")
        resized = pane_geometry()
        assert resized == "0:30x24:0:0\n1:49x24:31:0\n", resized

        valid = run("list-windows", "-F", "#{window_layout}").stdout.strip()
        run("select-layout", valid)
        assert pane_geometry() == resized

        mismatch = (
            '{"V":2,"L":{"t":"p","w":79,"h":23,"x":0,"y":0,"i":0}}'
        )
        failed = run("select-layout", mismatch, ok=False)
        assert failed.returncode != 0
        assert "have 2 panes but need 1" in failed.stderr, failed.stderr
        assert pane_geometry() == resized

        invalid_geometry = (
            '{"V":2,"L":{"t":"h","w":80,"h":24,"x":0,"y":0,"c":['
            '{"t":"p","w":30,"h":24,"x":0,"y":0,"i":0},'
            '{"t":"p","w":30,"h":23,"x":31,"y":0,"i":1}]}}'
        )
        failed = run("select-layout", invalid_geometry, ok=False)
        assert failed.returncode != 0
        assert "size mismatch after applying layout" in failed.stderr, failed.stderr
        assert pane_geometry() == resized
        assert initial != resized
    finally:
        subprocess.run(
            base + ["kill-server"],
            env=env,
            capture_output=True,
            timeout=20,
        )
        socket.unlink(missing_ok=True)

assert not socket.exists()
print("layout CLI checks passed")
