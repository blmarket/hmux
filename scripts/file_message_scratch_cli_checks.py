#!/usr/bin/env python3
"""Compare multi-message file reads and writes with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
payload = bytes(range(256)) * 257 + b"middle\0NUL\xffend"


def trace(binary, directory):
    socket = directory / "socket"
    source = directory / "source"
    saved = directory / "saved"
    source.write_bytes(payload)
    env = os.environ.copy()
    env.update({"HOME": str(directory), "TERM": "xterm-256color", "TMUX": ""})
    command = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "file-message", "sleep", "60")
        run("load-buffer", "-b", "large", str(source))
        shown = run("show-buffer", "-b", "large")
        saved_stdout = run("save-buffer", "-b", "large", "-")
        run("save-buffer", "-b", "large", str(saved))
        saved_file = saved.read_bytes()
        assert shown == saved_stdout == saved_file == payload
        return shown, saved_stdout, saved_file
    finally:
        subprocess.run(command + ["kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="file-message-scratch-") as temporary:
    candidate_dir = pathlib.Path(temporary, "candidate")
    baseline_dir = pathlib.Path(temporary, "baseline")
    candidate_dir.mkdir()
    baseline_dir.mkdir()
    assert trace(candidate, candidate_dir) == trace(baseline, baseline_dir)

print("file message scratch CLI checks passed")
