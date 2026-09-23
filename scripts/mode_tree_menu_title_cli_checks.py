#!/usr/bin/env python3
"""Exercise both mode-tree right-click menu titles through an attached client."""

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
box_top_left = "┌".encode()
csi = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")


def check_menu(binary_path, outside):
    with tempfile.TemporaryDirectory(prefix="mode-tree-menu-", dir=root / "target") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
        base = [str(binary_path), "-S", str(socket), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        client = None

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        def read_until(marker):
            output = bytearray()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    output.extend(os.read(master, 65536))
                if marker in output:
                    return bytes(output)
            raise AssertionError(f"missing {marker!r}; terminal tail={output[-1000:]!r}")

        try:
            run("new-session", "-d", "-s", "tree", "sleep 30")
            run("set-option", "-g", "mouse", "on")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "tree"],
                env=env,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                start_new_session=True,
            )
            os.close(slave)
            slave = None
            read_until(b"\x1b[?1006h")
            run("choose-tree", "-t", "tree:0.0")
            read_until(b"(view: preview)")

            # SGR button 3 down at x=3, inside the first item or below the list.
            os.write(master, b"\x1b[<2;3;25M" if outside else b"\x1b[<2;3;3M")
            marker = b"Scroll Left" if outside else b"Select  (Enter)"
            output = read_until(marker)
            assert (b"Scroll Right" if outside else b"Expand  (Right)") in output

            header_start = output[: output.index(marker)].rfind(box_top_left)
            assert header_start >= 0, output[-1000:]
            header = re.split(rb"[\r\n]", output[header_start:], maxsplit=1)[0]
            header = csi.sub(b"", header).replace(b"\x1b(B", b"")
            for border in ("┌", "─", "┐"):
                header = header.replace(border.encode(), b"")
            title = header.strip()
            if outside:
                assert title == b"", title
            else:
                assert title == b"0", title
            # Both a selection and Cancel return through mode_tree_menu_callback.
            os.write(master, b"q" if outside else b"\r")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                mode = run("display-message", "-p", "-t", "tree:0.0", "#{pane_in_mode}").strip()
                if mode == b"0":
                    break
                time.sleep(0.05)
            else:
                raise AssertionError(f"menu selection did not finish: outside={outside}, mode={mode!r}")
            return title
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                if client.poll() is None:
                    client.terminate()
                client.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)


actual = [check_menu(binary, outside) for outside in (False, True)]
if baseline is not None:
    expected = [check_menu(pathlib.Path(baseline).resolve(), outside) for outside in (False, True)]
    assert actual == expected, (actual, expected)
print("mode-tree menu title CLI checks passed")
