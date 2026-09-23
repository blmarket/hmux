#!/usr/bin/env python3
"""Check command and note prompts for a key in an attached customize tree."""

import fcntl
import os
import pathlib
import pty
import re
import select
import shlex
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
    with tempfile.TemporaryDirectory(prefix="customize-set-key-prompt-") as tmp:
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
                    return
            raise AssertionError(f"missing {needle!r}: {output[-3000:]!r}")

        try:
            run("new-session", "-d", "-s", "keys", "sleep 30")
            run(
                "bind-key", "-N", "owner-note-é", "-T", "ownertable", "F12",
                "display-message", "owner-command-é",
            )
            client = subprocess.Popen(
                base + ["attach-session", "-t", "keys"],
                env=env, stdin=slave, stdout=slave, stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            run("customize-mode", "-t", "keys:0.0", "-f", "#{m:F12,#{key}}")
            wait_for(b"Key Table - ownertable")
            run("send-keys", "-t", "keys:0.0", "G", "Up", "Up", "Right", "Down", "Right")
            if kind == "note":
                run("send-keys", "-t", "keys:0.0", "Down", "Down")
                expected = "(F12) owner-note-é".encode()
            else:
                run("send-keys", "-t", "keys:0.0", "Down")
                expected = "(F12) display-message owner-command-é".encode()
            output.clear()
            run("send-keys", "-t", "keys:0.0", "s")
            wait_for(expected)
            if kind == "note":
                edited_note = b"edited-owner-note-42"
                run("send-keys", "-t", "keys:0.0", "C-u")
                run("send-keys", "-t", "keys:0.0", "-l", edited_note.decode())
                run("send-keys", "-t", "keys:0.0", "Enter")
                assert run(
                    "list-keys", "-F", "#{key_note}", "-T", "ownertable", "F12"
                ).strip() == edited_note
                output.clear()
                run("send-keys", "-t", "keys:0.0", "s")
                wait_for(b"(F12) " + edited_note)
                run("send-keys", "-t", "keys:0.0", "Enter")

                # The prompt rejects an empty value, so clear the note using
                # the customize tree's editor action and a deterministic editor.
                editor = pathlib.Path(tmp) / "clear-note-editor"
                editor_input = pathlib.Path(tmp) / "note-editor-input"
                editor.write_text(
                    "#!/bin/sh\n"
                    f"cat \"$1\" > {shlex.quote(str(editor_input))}\n"
                    "printf '\\n' > \"$1\"\n"
                )
                editor.chmod(0o700)
                run("set-option", "-g", "editor", str(editor))
                run("send-keys", "-t", "keys:0.0", "e")
                deadline = time.monotonic() + 5
                while run(
                    "list-keys", "-F", "#{key_note}", "-T", "ownertable", "F12"
                ).strip():
                    assert time.monotonic() < deadline, "note editor did not clear note"
                    time.sleep(0.05)
                assert editor_input.read_bytes() == edited_note, editor_input.read_bytes()
                assert run(
                    "list-keys", "-F", "#{key_command}", "-T", "ownertable", "F12"
                ).strip() == "display-message owner-command-é".encode()
            return expected
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                try:
                    client.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    client.kill()
                    client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [check(binary, kind) for kind in ("command", "note")]
if baseline is not None:
    expected = [check(pathlib.Path(baseline).resolve(), kind) for kind in ("command", "note")]
    assert actual == expected, (actual, expected)

print("customize set-key prompt CLI checks passed")
