#!/usr/bin/env python3
"""Exercise customize-mode editor completion through an attached client."""

import fcntl
import os
import pathlib
import pty
import select
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


def exercise(binary, kind):
    with tempfile.TemporaryDirectory(prefix=f"customize-editor-{kind}-") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
        editor = directory / "editor.sh"
        entered = directory / "entered"
        edited_path = directory / "edited-path"
        release = directory / "release"
        editor.write_text(
            "#!/bin/sh\n"
            f"printf '%s' \"$2\" > {shlex.quote(str(edited_path))}\n"
            f"touch {shlex.quote(str(entered))}\n"
            "case \"$1\" in\n"
            "  valid) printf 'after-\\303\\251\\n' > \"$2\" ;;\n"
            "  nul) printf 'A\\000B\\n' > \"$2\" ;;\n"
            "  empty) : > \"$2\" ;;\n"
            "  cancel)\n"
            f"    while [ ! -e {shlex.quote(str(release))} ]; do sleep 0.05; done\n"
            "    printf 'ignored\\n' > \"$2\" ;;\n"
            "esac\n"
        )
        editor.chmod(0o755)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
        client = None
        output = bytearray()

        def run(*args):
            result = subprocess.run(base + list(args), env=ENV, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
            return result.stdout

        def option():
            return run("show-options", "-t", "edit", "-v", "status-left")

        def wait_for(predicate, description):
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if predicate():
                    return
                time.sleep(0.05)
            raise AssertionError(f"timed out waiting for {description}: {output[-1000:]!r}")

        try:
            run("new-session", "-d", "-s", "edit", "sleep 30")
            run("set-option", "-t", "edit", "status-left", "before")
            run("set-option", "-g", "editor", f"{editor} {kind}")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "edit"],
                env=ENV, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            wait_for(lambda: bool(run("list-clients", "-F", "#{client_tty}").strip()), "client attach")

            run(
                "customize-mode", "-t", "edit:0.0",
                "-f", "#{==:#{option_name},status-left}",
                "-F", "#{option_name}:#{option_value}",
            )
            run("send-keys", "-t", "edit:0.0", "Down", "Right", "Down")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if b"status-left:before" in output:
                    break
            else:
                raise AssertionError(f"status-left row was not rendered: {output[-1000:]!r}")

            run("send-keys", "-t", "edit:0.0", "e")
            wait_for(entered.exists, "editor startup")
            if kind == "cancel":
                run("send-keys", "-t", "edit:0.0", "q")
                wait_for(
                    lambda: run("display-message", "-p", "-t", "edit:0.0", "#{pane_in_mode}").strip() == b"0",
                    "customize-mode teardown",
                )
                release.touch()
                expected = b"before\n"
            elif kind == "valid":
                expected = b"after-\xc3\xa9\n"
            elif kind == "nul":
                # The editor file is counted data; the option setter sees its first-NUL C string.
                expected = b"A\n"
            else:
                expected = b"before\n"
            wait_for(lambda: option() == expected, f"{kind} option value")
            wait_for(
                lambda: edited_path.exists() and not pathlib.Path(edited_path.read_text()).exists(),
                f"{kind} editor temporary file cleanup",
            )
            assert client.poll() is None, "attached client exited"
            return option()
        finally:
            release.touch()
            subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [exercise(CANDIDATE, kind) for kind in ("valid", "nul", "empty", "cancel")]
if BASELINE is not None:
    expected = [exercise(pathlib.Path(BASELINE).resolve(), kind) for kind in ("valid", "nul", "empty", "cancel")]
    assert actual == expected, (actual, expected)

print("customize editor value owner CLI checks passed")
