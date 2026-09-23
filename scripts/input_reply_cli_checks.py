#!/usr/bin/env python3
"""Check immediate and timeout-flushed formatted replies from pane queries."""

import os
import pathlib
import pty
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="input-reply-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    reply = pathlib.Path(tmp) / "reply"
    env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
    pane = f"stty raw -echo; printf '\\033[6n'; dd bs=1 count=6 of='{reply}' status=none; sleep 1"

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)

    def wait_for_reply(path):
        deadline = time.monotonic() + 5
        while (not path.exists() or path.stat().st_size < 6) and time.monotonic() < deadline:
            time.sleep(0.05)
        assert path.exists(), "pane did not receive a reply"
        assert path.read_bytes() == b"\x1b[1;1R", path.read_bytes()

    try:
        run("new-session", "-d", "-s", "query", "sleep", "30")
        run("new-window", "-d", "-t", "query:1", pane)
        wait_for_reply(reply)

        # The attached terminal receives an OSC palette request but never
        # answers it. CSI 6n then waits in the pane's reply queue until the
        # palette request times out.
        master, slave = pty.openpty()
        client = subprocess.Popen(
            base + ["attach-session", "-t", "query"],
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

            queued = pathlib.Path(tmp) / "queued-reply"
            query = f"stty raw -echo; printf '\\033]4;123;?\\007\\033[6n'; dd bs=1 count=6 of='{queued}' status=none; sleep 1"
            run("new-window", "-d", "-t", "query:2", query)
            terminal_bytes = bytearray()
            deadline = time.monotonic() + 5
            while b"\x1b]4;123;?\x1b\\" not in terminal_bytes and time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    terminal_bytes.extend(os.read(master, 65536))
            assert b"\x1b]4;123;?\x1b\\" in terminal_bytes, terminal_bytes[-200:]
            assert not queued.exists(), "CSI reply was sent before palette timeout"
            wait_for_reply(queued)
        finally:
            client.terminate()
            client.wait(timeout=5)
            os.close(master)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)

print("input reply CLI checks passed")
