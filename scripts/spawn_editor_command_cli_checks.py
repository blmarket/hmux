#!/usr/bin/env python3
"""Exercise the editor command through an attached choose-buffer client."""

import fcntl
import os
import pathlib
import pty
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def exercise(binary_path):
    with tempfile.TemporaryDirectory(prefix="spawn-editor-command-") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary_path), "-S", str(directory / "socket"), "-f", "/dev/null"]
        marker_before = directory / "before"
        marker_path = directory / "path"
        script = directory / "editor-é.sh"
        script.write_text(
            "#!/bin/sh\n"
            '[ "$1" = "--owner-flag" ] || exit 51\n'
            f'cat "$2" > "{marker_before}"\n'
            "printf '\\nowner-edited-é' >> \"$2\"\n"
            f'printf "%s" "$2" > "{marker_path}"\n'
        )
        script.chmod(0o755)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        try:
            run("new-session", "-d", "-s", "edit", "sleep 30")
            run("set-buffer", "-b", "owner-buffer", "before-editor")
            run("set-option", "-g", "editor", f"{script} --owner-flag")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "edit"],
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

            run("choose-buffer", "-t", "edit:0.0")
            run("send-keys", "-t", "edit:0.0", "e")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if marker_path.exists() and b"owner-edited-\xc3\xa9" in run(
                    "show-buffer", "-b", "owner-buffer"
                ):
                    break
                time.sleep(0.05)
            else:
                raise AssertionError("editor command did not update the buffer")

            path = marker_path.read_bytes()
            assert path.startswith(b"/tmp/tmux."), path
            assert marker_before.read_bytes() == b"before-editor", marker_before.read_bytes()
            return run("show-buffer", "-b", "owner-buffer")
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = exercise(binary)
assert b"before-editor\nowner-edited-\xc3\xa9" in actual, actual
if baseline is not None:
    expected = exercise(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)

print("spawn editor-command CLI checks passed")
