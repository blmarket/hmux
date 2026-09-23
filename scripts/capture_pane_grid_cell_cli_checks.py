#!/usr/bin/env python3
"""Check capture-pane -R cell colours and OSC 8 links on a private server.

Set HMUX_BINARY to select the binary under test. If HMUX_BASELINE_BINARY is
set, compare the complete capture output byte for byte with that binary.
"""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")

cases = (
    (
        "colour-and-id",
        "printf '\\033[31mRED\\033[0m\\n\\033]8;id=demo;https://example.test/a\\033\\\\LINK\\033]8;;\\033\\\\\\n'; sleep 30",
        b"link=https://example.test/a linkid=demo",
    ),
    (
        "no-id",
        "printf '\\033]8;;https://example.test/plain\\033\\\\NOID\\033]8;;\\033\\\\\\n'; sleep 30",
        b"link=https://example.test/plain linkid=NONE",
    ),
)


def capture(binary_path, name, command, expected, tempdir):
    base = [str(binary_path), "-S", str(tempdir / f"socket-{name}"), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "capture", "-x", "20", "-y", "5", command)
        deadline = time.monotonic() + 5
        while True:
            output = run("capture-pane", "-R", "-p", "-t", "capture:0.0")
            if expected in output:
                break
            assert time.monotonic() < deadline, (name, output[:1000])
            time.sleep(0.05)

        cells = [line for line in output.splitlines() if line.startswith(b"\t\tC ")]
        assert any(expected in line for line in cells), (name, expected)
        if name == "colour-and-id":
            assert any(b"data=(1,1,R)" in line and b"fg=red[1]" in line for line in cells)
        return output
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="capture-grid-cell-", dir=root / "target") as tmp:
    tempdir = pathlib.Path(tmp)
    for name, command, expected in cases:
        output = capture(binary, f"candidate-{name}", command, expected, tempdir)
        if baseline is not None:
            baseline_output = capture(
                pathlib.Path(baseline).resolve(), f"baseline-{name}", command, expected, tempdir
            )
            assert output == baseline_output, f"{name} capture differs from baseline"

print("capture-pane grid cell CLI checks passed")
