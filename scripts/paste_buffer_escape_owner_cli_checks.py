#!/usr/bin/env python3
"""Compare escaped, raw, and bracketed paste input against the pinned binary.

Set HMUX_BASELINE_BINARY to the pinned pre-migration binary. Each pane records
the exact bytes delivered through its PTY, including NUL and invalid UTF-8.
"""

import os
import pathlib
import shlex
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2"))
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"])
environment = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
payload = b"A\0B\xff\nC\tD"
end_marker = b"__HMUX_PASTE_INPUT_COMPLETE__"
reader_code = """\
import os
import pathlib
import sys
import tty

output = pathlib.Path(sys.argv[1])
marker = sys.argv[2].encode("ascii")
tty.setraw(0)
os.write(1, b"\\x1b[?2004h")
output.with_suffix(".ready").touch()
received = bytearray()
while marker not in received:
    chunk = os.read(0, 4096)
    if not chunk:
        raise RuntimeError("pane input closed before end marker")
    received.extend(chunk)
    output.with_suffix(".progress").write_bytes(received)
    if len(received) > 4096:
        raise RuntimeError("pane input exceeded expected size")
output.write_bytes(received[: received.index(marker)])
"""


def check(binary, directory):
    directory.mkdir()
    socket = directory / "socket"
    source = directory / "payload"
    reader = directory / "reader.py"
    source.write_bytes(payload)
    reader.write_text(reader_code)
    base = [str(binary.resolve()), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=environment, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    def wait_for(condition, detail):
        deadline = time.monotonic() + 5
        while not condition():
            assert time.monotonic() < deadline, detail()
            time.sleep(0.05)

    try:
        run("new-session", "-d", "-s", "paste", "sleep 30")
        run("load-buffer", "-b", "binary", str(source))
        run("set-buffer", "-b", "marker", end_marker.decode("ascii"))
        results = []
        for index, flags in enumerate(((), ("-S",), ("-p",))):
            name = f"input{index}"
            output = directory / f"{name}.bin"
            pane_command = " ".join(
                shlex.quote(arg)
                for arg in ("python3", "-u", str(reader), str(output), end_marker.decode("ascii"))
            )
            run("new-session", "-d", "-s", name, pane_command)
            target = f"{name}:0.0"
            wait_for(
                lambda: output.with_suffix(".ready").exists()
                and run("display-message", "-p", "-t", target, "#{bracket_paste_flag}") == b"1\n",
                lambda: (name, "pane did not enable bracketed paste"),
            )
            run("paste-buffer", "-b", "binary", "-t", target, "-r", *flags)
            progress = output.with_suffix(".progress")
            wait_for(
                lambda: progress.exists() and bool(progress.read_bytes()),
                lambda: (name, "pane did not receive paste bytes"),
            )
            # Queue the marker through the same bufferevent as the tested
            # paste. send-keys can reach the PTY before pending paste output.
            run("paste-buffer", "-S", "-b", "marker", "-t", target, "-r")
            wait_for(output.exists, lambda: (name, "pane did not finish reading input"))
            results.append(output.read_bytes())
        escaped, raw, bracketed = results
        assert raw == payload, raw
        assert escaped != raw, (escaped, raw)
        assert bracketed == b"\x1b[200~" + escaped + b"\x1b[201~", bracketed
        return results
    finally:
        subprocess.run(base + ["kill-server"], env=environment, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="paste-buffer-escape-owner-") as temporary:
    parent = pathlib.Path(temporary)
    assert check(candidate, parent / "candidate") == check(baseline, parent / "baseline")

print("paste-buffer escape owner CLI checks passed")
