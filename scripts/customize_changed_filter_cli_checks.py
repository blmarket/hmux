#!/usr/bin/env python3
"""Compare scalar and array default filtering in an attached customize tree."""

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
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
csi = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")
border = "┌".encode()


def check(binary, kind, changed):
    with tempfile.TemporaryDirectory(prefix="customize-changed-filter-") as tmp:
        base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def read_frame():
            output = bytearray()
            deadline = time.monotonic() + 5
            quiet_since = None
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.05)[0]:
                    output.extend(os.read(master, 65536))
                    quiet_since = None
                elif quiet_since is None:
                    quiet_since = time.monotonic()
                cleaned = csi.sub(b"", output).replace(b"\x1b(B", b"")
                if border in cleaned and quiet_since is not None and time.monotonic() - quiet_since >= 0.15:
                    tree = cleaned[cleaned.rfind(b"(0)"):].split(border, 1)[0]
                    return tree
            raise AssertionError(f"missing customize redraw: {output[-1000:]!r}")

        try:
            run("new-session", "-d", "-s", "opts", "sleep 30")
            option = "status-left" if kind == "scalar" else "status-format[0]"
            default = run("show-options", "-gv", option).removesuffix(b"\n").decode()
            value = ("café" if kind == "scalar" else "array-marker") if changed else default
            run("set-option", "-t", "opts", option, value)
            client = subprocess.Popen(
                base + ["attach-session", "-t", "opts"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            name = "status-left" if kind == "scalar" else "status-format"
            run(
                "customize-mode", "-t", "opts:0.0",
                "-f", "#{==:#{option_name}," + name + "}",
                "-F", "#{option_name}:#{option_value}",
            )
            read_frame()
            keys = ("Down", "Right") if kind == "scalar" else ("Down", "Right", "Down", "Right")
            run("send-keys", "-t", "opts:0.0", *keys)
            before = read_frame()
            row = b"status-left:" if kind == "scalar" else b"status-format[0]:"
            assert row in before, (kind, changed, "row missing before C", before)

            run("send-keys", "-t", "opts:0.0", "C")
            after = read_frame()
            assert (row in after) == changed, (kind, changed, "wrong row visibility after C", after)
            if kind == "array":
                # The local one-element array differs from the full default array,
                # so its parent survives even when its entry matches the default.
                assert b"status-format" in after, (kind, changed, after)
            return kind, changed, row in after
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


cases = (("scalar", False), ("scalar", True), ("array", False), ("array", True))
expected = [check(baseline, *case) for case in cases]
actual = [check(candidate, *case) for case in cases]
assert actual == expected, (actual, expected)
print("customize changed-filter CLI checks passed")
