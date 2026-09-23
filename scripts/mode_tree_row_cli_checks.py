#!/usr/bin/env python3
"""Compare rendered mode-tree rows, including byte padding, with a baseline."""

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
row_start = re.compile(rb"^\(([0-9]+)\) ")


def rows(binary_path, tagged):
    with tempfile.TemporaryDirectory(prefix="mode-tree-row-", dir=root / "target") as tmp:
        base = [str(binary_path), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return result.stdout

        def read_rows():
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    output.extend(os.read(master, 65536))
                if b"(view: preview)" in output:
                    # Absolute cursor moves delimit screen rows even when the
                    # redraw does not emit a newline. Keep the latest copy of
                    # each row instead of comparing duplicate redraw fragments.
                    cleaned = csi.sub(
                        lambda match: b"\n" if match[0][-1:] in (b"H", b"f") else b"",
                        output,
                    ).replace(b"\x1b(B", b"")
                    found = {}
                    for line in cleaned.splitlines():
                        match = row_start.match(line)
                        if match:
                            found[int(match[1])] = line
                    if all(index in found for index in range(6)):
                        return [found[index] for index in range(6)]
            raise AssertionError(f"missing tree rows: {output[-1000:]!r}")

        try:
            run("new-session", "-d", "-s", "a", "-n", "fixed", "-x", "100", "-y", "30", "sleep 120")
            run("new-session", "-d", "-s", "longname", "-n", "fixed", "sleep 120")
            run("set-window-option", "-g", "automatic-rename", "off")
            deadline = time.monotonic() + 5
            while run("list-panes", "-a", "-F", "#{pane_current_command}").splitlines() != [b"sleep", b"sleep"]:
                assert time.monotonic() < deadline, "pane commands did not start"
                time.sleep(0.05)
            client = subprocess.Popen(
                base + ["attach-session", "-t", "a"],
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

            run("choose-tree", "-t", "a:0.0")
            initial = read_rows()
            assert any(b"longname" in row for row in initial), initial
            if tagged:
                run("send-keys", "-t", "a:0.0", "t")
                return read_rows()
            return initial
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


for tagged in (False, True):
    actual = rows(binary, tagged)
    if baseline is not None:
        expected = rows(pathlib.Path(baseline).resolve(), tagged)
        assert actual == expected, (tagged, actual, expected)

print("mode-tree row CLI checks passed")
