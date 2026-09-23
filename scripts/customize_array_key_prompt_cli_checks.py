#!/usr/bin/env python3
"""Check editing an array key through an attached customize-mode prompt."""

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


def check(binary_path):
    with tempfile.TemporaryDirectory(prefix="customize-array-key-prompt-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
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
                    return cleaned
            raise AssertionError(f"missing {needle!r}: {output[-3000:]!r}")

        try:
            run("new-session", "-d", "-s", "opts", "sleep 30")
            run("set-option", "-g", "status-format[7]", "array-marker")
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
                "-f", "#{m:*status-format*,#{option_name}}",
                "-F", "#{option_name}:#{option_value}",
            )
            run("send-keys", "-t", "opts:0.0", "Down", "Right", "Down", "Right")
            wait_for(b"status-format[7]:array-marker")
            run("send-keys", "-t", "opts:0.0", "Down", "Down", "Down", "Down")
            run("send-keys", "-t", "opts:0.0", "a")
            prompt = b"(status-format[7]) 7"
            wait_for(prompt)
            run("send-keys", "-t", "opts:0.0", "C-u", "8", "Enter")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                value = run("show-options", "-gv", "status-format[8]")
                if value == b"array-marker\n":
                    break
                time.sleep(0.05)
            else:
                raise AssertionError(f"array key was not renamed: {value!r}")
            return prompt, value
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = check(binary)
if baseline is not None:
    expected = check(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)

print("customize array-key prompt CLI checks passed")
