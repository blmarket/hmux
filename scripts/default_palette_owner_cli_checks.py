#!/usr/bin/env python3
"""Exercise pane-colours allocation, lookup, and removal through OSC 4."""

import os
import pathlib
import pty
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="default-palette-owner-", dir=root / "target") as tmp:
    directory = pathlib.Path(tmp)
    socket = directory / "socket"
    start = directory / "start"
    first = directory / "first"
    again = directory / "again"
    expected = b"\x1b]4;123;rgb:1111/2222/3333\x07"
    query = b"\x1b]4;123;?\x1b\\"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    pane = (
        "stty raw -echo; "
        f"while [ ! -e '{start}' ]; do sleep 0.02; done; "
        "printf '\\033]4;123;?\\007'; "
        f"dd bs=1 count={len(expected)} of='{first}' status=none; "
        f"while [ ! -e '{again}' ]; do sleep 0.02; done; "
        "printf '\\033]4;123;?\\007'; sleep 2"
    )
    master, slave = pty.openpty()
    attached = None
    try:
        run("new-session", "-d", "-s", "default-palette", pane)
        run("set-option", "-w", "-t", "default-palette:0", "pane-colours[123]", "#112233")
        attached = subprocess.Popen(
            base + ["attach-session", "-t", "default-palette"],
            env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
        )
        os.close(slave)
        slave = None
        deadline = time.monotonic() + 5
        while not run("list-clients", "-F", "#{client_name}").strip():
            assert attached.poll() is None, "client exited before attachment"
            assert time.monotonic() < deadline, "client did not attach"
            time.sleep(0.02)
        start.touch()
        while (not first.exists() or first.stat().st_size < len(expected)) and time.monotonic() < deadline:
            time.sleep(0.02)
        assert first.read_bytes() == expected

        # Removing the last default entry makes the next query reach the client terminal.
        run("set-option", "-w", "-u", "-t", "default-palette:0", "pane-colours[123]")
        while select.select([master], [], [], 0)[0]:
            os.read(master, 65536)
        again.touch()
        output = bytearray()
        deadline = time.monotonic() + 5
        while query not in output and time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                output.extend(os.read(master, 65536))
        assert query in output, output[-300:]
    finally:
        if attached is not None:
            attached.terminate()
            attached.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("default palette owner CLI checks passed")
