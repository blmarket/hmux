#!/usr/bin/env python3
"""Check generated modified-key sequences against a pinned tmux binary.

Set HMUX_BASELINE_BINARY to the pinned binary. A pane in raw mode records the
exact bytes produced by send-keys, covering all generated modifier values 2-8.
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

# The final CSI parameter is generated from input_key_modifiers (2 through 8).
# F keys, arrows, and editing keys also exercise different sequence templates.
keys = (
    ("S-F1", b"\x1b[1;2P"),
    ("M-F5", b"\x1b[15;3~"),
    ("M-S-F2", b"\x1b[1;4Q"),
    ("C-Up", b"\x1b[1;5A"),
    ("C-S-Down", b"\x1b[1;6B"),
    ("C-M-Right", b"\x1b[1;7C"),
    ("C-M-S-Insert", b"\x1b[2;8~"),
)
expected = b"".join(sequence for _, sequence in keys)
reader_code = """\
import os
import pathlib
import sys
import tty

output = pathlib.Path(sys.argv[1])
remaining = int(sys.argv[2])
tty.setraw(0)
output.with_suffix('.ready').touch()
received = bytearray()
while remaining:
    chunk = os.read(0, remaining)
    if not chunk:
        raise RuntimeError('pane input closed early')
    received.extend(chunk)
    remaining -= len(chunk)
output.write_bytes(received)
"""


def check(binary, directory):
    directory.mkdir()
    socket = directory / "socket"
    reader = directory / "reader.py"
    output = directory / "input.bin"
    reader.write_text(reader_code)
    command = " ".join(
        shlex.quote(argument)
        for argument in ("python3", "-u", str(reader), str(output), str(len(expected)))
    )
    base = [str(binary.resolve()), "-S", str(socket), "-f", "/dev/null"]

    def run(*arguments):
        result = subprocess.run(base + list(arguments), env=environment, capture_output=True, timeout=20)
        assert result.returncode == 0, (arguments, result.returncode, result.stdout, result.stderr)

    def wait_for(path):
        deadline = time.monotonic() + 5
        while not path.exists() and time.monotonic() < deadline:
            time.sleep(0.05)
        assert path.exists(), f"pane did not write {path}"

    try:
        run("new-session", "-d", "-s", "generated", command)
        wait_for(output.with_suffix(".ready"))
        run("send-keys", "-t", "generated:0.0", *(name for name, _ in keys))
        wait_for(output)
        actual = output.read_bytes()
        assert actual == expected, (binary, actual, expected)
        return actual
    finally:
        subprocess.run(base + ["kill-server"], env=environment, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="input-key-generated-owner-") as temporary:
    parent = pathlib.Path(temporary)
    assert check(candidate, parent / "candidate") == check(baseline, parent / "baseline")

print("input key generated owner CLI checks passed")
