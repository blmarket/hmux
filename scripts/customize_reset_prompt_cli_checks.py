#!/usr/bin/env python3
"""Check current and tagged reset prompts in an attached customize tree."""

import fcntl
import os
import pathlib
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
csi = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")


def check(binary_path, tagged):
    with tempfile.TemporaryDirectory(prefix="customize-reset-prompt-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
        client = None
        output = bytearray()

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def wait_for(needle):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                cleaned = csi.sub(b"", output).replace(b"\x1b(B", b"")
                if needle in cleaned:
                    return needle
            raise AssertionError(f"missing {needle!r}: {output[-3000:]!r}")

        try:
            run("new-session", "-d", "-s", "opts", "sleep 30")
            run("set-option", "-t", "opts", "@café", "latte")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "opts"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            run(
                "customize-mode", "-t", "opts:0.0",
                "-f", "#{==:#{option_name},@café}",
                "-F", "#{option_name}:#{option_value}",
            )
            wait_for(b"Session Options")
            run("send-keys", "-t", "opts:0.0", "Down", "Right", "Down")
            output.clear()
            if tagged:
                run("send-keys", "-t", "opts:0.0", "t", "D")
                prompt = wait_for(b"Reset 1 tagged to default? ")
            else:
                run("send-keys", "-t", "opts:0.0", "d")
                prompt = wait_for(b"Reset @caf\xc3\xa9 to default? ")
            run("send-keys", "-t", "opts:0.0", "y")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                result = subprocess.run(
                    base + ["show-options", "-t", "opts", "@café"],
                    env=env, capture_output=True, timeout=10,
                )
                if result.returncode != 0:
                    return prompt
                time.sleep(0.05)
            raise AssertionError(f"reset did not remove the option: {result.stdout!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [check(binary, tagged) for tagged in (False, True)]
if baseline is not None:
    expected = [check(pathlib.Path(baseline).resolve(), tagged) for tagged in (False, True)]
    assert actual == expected, (actual, expected)

print("customize reset prompt CLI checks passed")
