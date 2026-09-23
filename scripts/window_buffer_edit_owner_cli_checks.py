#!/usr/bin/env python3
"""Exercise choose-buffer edit completion and mode teardown with a real client."""

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
    with tempfile.TemporaryDirectory(prefix="window-buffer-edit-owner-") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary_path), "-S", str(directory / "socket"), "-f", "/dev/null"]
        editor = directory / "editor.sh"
        ready = directory / "ready"
        release = directory / "release"
        complete_path = directory / "complete-path"
        binary_path = directory / "binary-path"
        cancel_path = directory / "cancel-path"
        binary_before = directory / "binary-before"
        binary_after = directory / "binary-after"
        binary_before.write_bytes(b"prefix\x00\xffsuffix")
        binary_after.write_bytes(b"edited\x00\x80\xffpayload")
        editor.write_text(
            "#!/bin/sh\n"
            "if [ \"$1\" = complete ]; then\n"
            f"  printf '%s' \"$2\" > '{complete_path}'\n"
            "  printf '\\ncompleted' >> \"$2\"\n"
            "elif [ \"$1\" = binary ]; then\n"
            f"  cmp -s \"$2\" '{binary_before}' || exit 19\n"
            f"  printf '%s' \"$2\" > '{binary_path}'\n"
            f"  cp '{binary_after}' \"$2\"\n"
            "else\n"
            f"  printf '%s' \"$2\" > '{cancel_path}'\n"
            f"  touch '{ready}'\n"
            f"  while [ ! -e '{release}' ]; do sleep 0.05; done\n"
            "  printf '\\nignored' >> \"$2\"\n"
            "fi\n"
        )
        editor.chmod(0o755)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def wait_for(predicate, description):
            deadline = time.monotonic() + 5
            while not predicate():
                assert time.monotonic() < deadline, description
                time.sleep(0.05)

        try:
            run("new-session", "-d", "-s", "edit", "sleep 30")
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
            wait_for(lambda: bool(run("list-clients", "-F", "#{client_tty}").strip()), "client attach")

            run("set-buffer", "-b", "complete", "before")
            run("set-option", "-g", "editor", f"{editor} complete")
            run("choose-buffer", "-t", "edit:0.0")
            run("send-keys", "-t", "edit:0.0", "e")
            wait_for(
                lambda: run("show-buffer", "-b", "complete") == b"before\ncompleted",
                "editor completion",
            )
            completed = run("show-buffer", "-b", "complete")
            wait_for(
                lambda: complete_path.exists()
                and not pathlib.Path(complete_path.read_text()).exists(),
                "completed editor temporary file cleanup",
            )
            run("delete-buffer", "-b", "complete")

            run("load-buffer", "-b", "binary", str(binary_before))
            assert run("show-buffer", "-b", "binary") == binary_before.read_bytes()
            run("set-option", "-g", "editor", f"{editor} binary")
            run("choose-buffer", "-t", "edit:0.0")
            run("send-keys", "-t", "edit:0.0", "e")
            wait_for(
                lambda: run("show-buffer", "-b", "binary") == binary_after.read_bytes(),
                "binary editor completion",
            )
            edited_binary = run("show-buffer", "-b", "binary")
            wait_for(
                lambda: binary_path.exists()
                and not pathlib.Path(binary_path.read_text()).exists(),
                "binary editor temporary file cleanup",
            )
            run("delete-buffer", "-b", "binary")

            run("set-buffer", "-b", "cancel", "unchanged")
            run("set-option", "-g", "editor", f"{editor} cancel")
            run("choose-buffer", "-t", "edit:0.0")
            run("send-keys", "-t", "edit:0.0", "e")
            wait_for(ready.exists, "editor waiting")
            run("send-keys", "-t", "edit:0.0", "q")
            wait_for(
                lambda: run("display-message", "-p", "-t", "edit:0.0", "#{pane_in_mode}").strip() == b"0",
                "buffer mode teardown",
            )
            release.touch()
            wait_for(
                lambda: len(run("list-panes", "-a", "-F", "#{pane_id}").splitlines()) == 1,
                "editor pane exit",
            )
            canceled = run("show-buffer", "-b", "cancel")
            assert canceled == b"unchanged", canceled
            wait_for(
                lambda: cancel_path.exists()
                and not pathlib.Path(cancel_path.read_text()).exists(),
                "canceled editor temporary file cleanup",
            )
            return completed, edited_binary, canceled
        finally:
            release.touch()
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                try:
                    client.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    client.kill()
                    client.wait(timeout=2)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = exercise(binary)
if baseline is not None:
    expected = exercise(pathlib.Path(baseline).resolve())
    assert actual == expected, (actual, expected)
print("window buffer edit ownership CLI checks passed")
