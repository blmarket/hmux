#!/usr/bin/env python3
"""Check scalar, array-add, and array-entry customize prompts in a client."""

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


def check(binary_path, kind):
    with tempfile.TemporaryDirectory(prefix="customize-set-option-prompt-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
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
                    return
            raise AssertionError(f"missing {needle!r}: {output[-3000:]!r}")

        try:
            run("new-session", "-d", "-s", "optsé", "sleep 30")
            run("set-option", "-t", "optsé", "status-left", "café")
            run("set-option", "-t", "optsé", "status-format[7]", "array-marker")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "optsé"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None

            option = "status-left" if kind == "scalar" else "status-format"
            run(
                "customize-mode", "-t", "optsé:0.0",
                "-f", "#{==:#{option_name}," + option + "}",
                "-F", "#{option_name}:#{option_value}",
            )
            wait_for(b"Session Options")
            run("send-keys", "-t", "optsé:0.0", "Down", "Right")
            if kind == "scalar":
                run("send-keys", "-t", "optsé:0.0", "Down")
                wait_for(b"The default is: [#{session_name}]")
                expected = b"(status-left, for session opts\xc3\xa9) caf\xc3\xa9"
            elif kind == "array-add":
                run("send-keys", "-t", "optsé:0.0", "Down")
                expected = b"(status-format[+], for session opts\xc3\xa9) "
            else:
                run("send-keys", "-t", "optsé:0.0", "Down", "Right", "Down")
                expected = b"(status-format[7], for session opts\xc3\xa9) array-marker"
            output.clear()
            run("send-keys", "-t", "optsé:0.0", "s")
            wait_for(expected)
            return expected
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


kinds = ("scalar", "array-add", "array-entry")
actual = [check(binary, kind) for kind in kinds]
if baseline is not None:
    expected = [check(pathlib.Path(baseline).resolve(), kind) for kind in kinds]
    assert actual == expected, (actual, expected)

print("customize set-option prompt CLI checks passed")
