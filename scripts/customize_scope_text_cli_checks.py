#!/usr/bin/env python3
"""Check customize scope labels for server, session, window, and pane items."""

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
    with tempfile.TemporaryDirectory(prefix="customize-scope-text-") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 35, 120, 0, 0))
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
            run("new-session", "-d", "-s", "scopeé", "sleep 30")
            run("set-option", "-s", "@scope-server", "value")
            run("set-option", "-t", "scopeé", "status-left", "value")
            run("set-option", "-w", "-t", "scopeé:0", "window-status-format", "value")
            run("set-option", "-p", "-t", "scopeé:0.0", "pane-border-format", "value")
            run("set-environment", "-t", "scopeé", "OWN_SCOPE", "value")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "scopeé"],
                env=env, stdin=slave, stdout=slave, stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None

            cases = {
                "server": ("@scope-server", "#{option_scope}:#{option_name}", 0,
                           b":@scope-server"),
                "session": ("status-left", "#{option_scope}:#{option_name}", 1,
                            "session scopeé:status-left".encode()),
                "window": ("window-status-format", "#{option_scope}:#{option_name}", 2,
                           b"window 0:window-status-format"),
                "pane": ("pane-border-format", "#{option_scope}:#{option_name}", 2,
                         b"pane 0:pane-border-format"),
                "environment": ("OWN_SCOPE", "#{environment_scope}:#{environment_name}", 6,
                                "session scopeé:OWN_SCOPE".encode()),
            }
            name, display, section, expected = cases[kind]
            filter_name = "environment_name" if kind == "environment" else "option_name"
            run(
                "customize-mode", "-t", "scopeé:0.0",
                "-f", "#{==:#{" + filter_name + "}," + name + "}",
                "-F", display,
            )
            run("send-keys", "-t", "scopeé:0.0", *(["Down"] * section), "Right")
            rendered = wait_for(expected)
            run("send-keys", "-t", "scopeé:0.0", "q")
            deadline = time.monotonic() + 5
            while run("display-message", "-p", "-t", "scopeé:0.0", "#{pane_in_mode}").strip() != b"0":
                assert time.monotonic() < deadline, "customize mode did not close"
                time.sleep(0.05)
            return rendered
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


kinds = ("server", "session", "window", "pane", "environment")
actual = [check(binary, kind) for kind in kinds]
if baseline is not None:
    expected = [check(pathlib.Path(baseline).resolve(), kind) for kind in kinds]
    assert actual == expected, (actual, expected)

print("customize scope-text CLI checks passed")
