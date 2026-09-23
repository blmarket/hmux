#!/usr/bin/env python3
"""Compare customize environment editor input with pinned tmux."""

import fcntl
import os
import pathlib
import pty
import shlex
import struct
import subprocess
import tempfile
import termios
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = os.environ.get("HMUX_BASELINE_BINARY")
ENV = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def editor_input(binary, global_scope):
    with tempfile.TemporaryDirectory(prefix="customize-environment-editor-") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
        captured = directory / "editor-input"
        partial = directory / "editor-input.partial"
        editor = directory / "capture-editor.sh"
        editor.write_text(
            "#!/bin/sh\n"
            f"cat \"$1\" > {shlex.quote(str(partial))}\n"
            f"mv {shlex.quote(str(partial))} {shlex.quote(str(captured))}\n"
        )
        editor.chmod(0o700)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=ENV, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "envé", "sleep 30")
            if global_scope:
                name = "OWN_EDITOR_EMPTY"
                run("set-environment", "-g", name, "")
                up = ["Up"] * 6
            else:
                name = "OWN_EDITOR_é"
                run("set-environment", "-t", "envé", name, "café")
                up = ["Up"] * 5
            run("set-option", "-g", "editor", str(editor))

            client = subprocess.Popen(
                base + ["attach-session", "-t", "envé"],
                env=ENV, stdin=slave, stdout=slave, stderr=slave,
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
            # These keys select the value row in the same tree as the
            # environment prompt check; e opens that value in the editor.
            run("send-keys", "-t", "envé:0.0", "G", *up, "Right", "Down", "e")
            deadline = time.monotonic() + 5
            while not captured.exists():
                assert time.monotonic() < deadline, "environment editor did not start"
                time.sleep(0.05)
            assert client.poll() is None, "attached client exited"
            return captured.read_bytes()
        finally:
            subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [editor_input(CANDIDATE, global_scope) for global_scope in (False, True)]
assert actual == ["café".encode(), b"\n"], actual
if BASELINE is not None:
    expected = [
        editor_input(pathlib.Path(BASELINE).resolve(), global_scope)
        for global_scope in (False, True)
    ]
    assert actual == expected, (actual, expected)

print("customize environment editor owner CLI checks passed")
