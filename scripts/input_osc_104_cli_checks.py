#!/usr/bin/env python3
"""Exercise OSC 104 index clearing, malformed lists, and full palette clearing."""

import os
import pathlib
import pty
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")

with tempfile.TemporaryDirectory(prefix="input-osc-104-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    local_reply = pathlib.Path(tmp) / "local-reply"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    terminal_output = bytearray()

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    def wait_for_file(path, size):
        deadline = time.monotonic() + 5
        while (not path.exists() or path.stat().st_size < size) and time.monotonic() < deadline:
            time.sleep(0.05)
        assert path.exists(), f"pane did not write {path}"
        return path.read_bytes()

    def wait_for_terminal_bytes(master, expected):
        deadline = time.monotonic() + 5
        while expected not in terminal_output and time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                terminal_output.extend(os.read(master, 65536))
        assert expected in terminal_output, terminal_output[-300:]

    try:
        run("new-session", "-d", "-s", "osc104", "sleep", "30")
        master, slave = pty.openpty()
        client = subprocess.Popen(
            base + ["attach-session", "-t", "osc104"],
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        try:
            time.sleep(0.2)
            while select.select([master], [], [], 0)[0]:
                os.read(master, 65536)

            expected_local = b"\x1b]4;124;rgb:4444/5555/6666\x07"
            pane = (
                "stty raw -echo; "
                "printf '\\033]4;123;rgb:11/22/33\\007\\033]4;124;rgb:44/55/66\\007'; "
                "printf '\\033]104;123;bad;124\\007\\033]4;124;?\\007'; "
                f"dd bs=1 count={len(expected_local)} of='{local_reply}' status=none; "
                "printf '\\033]4;123;?\\007'; "
                "sleep 1; "
                "printf '\\033]104\\007\\033]4;124;?\\007'; "
                "sleep 1; "
                "printf '\\033]4;125;rgb:77/88/99\\007\\033]4;126;rgb:aa/bb/cc\\007'; "
                "printf '\\033]104;125;126\\007\\033]4;125;?\\007\\033]4;126;?\\007'; "
                "sleep 2"
            )
            run("new-window", "-d", "-t", "osc104:1", pane)
            assert wait_for_file(local_reply, len(expected_local)) == expected_local

            # The first entry is cleared before parsing stops at "bad".
            wait_for_terminal_bytes(master, b"\x1b]4;123;?\x1b\\")
            # Empty OSC 104 clears the still-local second entry.
            wait_for_terminal_bytes(master, b"\x1b]4;124;?\x1b\\")
            # A list containing two valid indices clears both entries.
            wait_for_terminal_bytes(master, b"\x1b]4;125;?\x1b\\")
            wait_for_terminal_bytes(master, b"\x1b]4;126;?\x1b\\")
        finally:
            client.terminate()
            client.wait(timeout=5)
            os.close(master)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("input OSC 104 CLI checks passed")
