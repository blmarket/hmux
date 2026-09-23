#!/usr/bin/env python3
"""Check session and global environment prompts in an attached customize tree."""

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


def prompt_line(binary_path, global_scope):
    with tempfile.TemporaryDirectory(prefix="customize-environment-prompt-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "envé", "sleep 30")
            if global_scope:
                name = "OWN_PROMPT_é"
                run("set-environment", "-g", name, "")
                expected = b"(OWN_PROMPT_\xc3\xa9, global) "
                up = ["Up"] * 6
            else:
                name = "OWN_PROMPT_X"
                run("set-environment", "-t", "envé", name, "café")
                expected = b"(OWN_PROMPT_X, for session env\xc3\xa9) caf\xc3\xa9"
                up = ["Up"] * 5

            client = subprocess.Popen(
                base + ["attach-session", "-t", "envé"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not run("list-clients", "-F", "#{client_tty}").strip():
                assert time.monotonic() < deadline, "client did not attach"
                time.sleep(0.05)

            run(
                "customize-mode", "-t", "envé:0.0",
                "-f", "#{==:#{environment_name}," + name + "}",
                "-F", "#{environment_name}:#{environment_value}",
            )
            run("send-keys", "-t", "envé:0.0", "G", *up, "Right", "Down", "Enter")

            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                cleaned = csi.sub(b"", output).replace(b"\x1b(B", b"")
                start = cleaned.find(b"(OWN_PROMPT_")
                if start < 0:
                    continue
                end = cleaned.find("─".encode(), start)
                if end < 0:
                    continue
                prompt = bytes(cleaned[start:end])
                assert prompt == expected, (prompt, expected)
                return prompt
            raise AssertionError(f"missing environment prompt: {output[-3000:]!r}")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.kill()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [prompt_line(binary, global_scope) for global_scope in (False, True)]
if baseline is not None:
    expected = [
        prompt_line(pathlib.Path(baseline).resolve(), global_scope)
        for global_scope in (False, True)
    ]
    assert actual == expected, (actual, expected)

print("customize environment prompt CLI checks passed")
