#!/usr/bin/env python3
"""Exercise named and automatic buffer operations across every name lifetime."""

import os
import pathlib
import select
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, socket):
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C.UTF-8", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [binary, b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=15)
        return (result.returncode, result.stdout, result.stderr)

    def success(*args):
        result = run(*args)
        assert result[0] == 0, (args, result)
        return result[1]

    def read_until(marker, start=0):
        deadline = time.monotonic() + 10
        while marker not in output[start:]:
            remaining = deadline - time.monotonic()
            assert remaining > 0, (marker, output[start:], control.poll())
            ready, _, _ = select.select([control.stdout], [], [], remaining)
            assert ready, (marker, output[start:], control.poll())
            chunk = os.read(control.stdout.fileno(), 65536)
            assert chunk, (marker, output[start:], control.poll())
            output.extend(chunk)

    observed = []
    control = None
    try:
        success(b"new-session", b"-d", b"-s", b"buffers", b"sleep 60")
        control = subprocess.Popen(
            command + [b"-C", b"attach-session", b"-t", b"buffers"],
            env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        output = bytearray()
        read_until(b"%session-changed ")

        observed.append(run(b"delete-buffer"))
        observed.append(run(b"set-buffer", b"-n", b"unused"))
        observed.append(run(b"delete-buffer", b"-b", b"missing"))
        assert all(result[0] != 0 for result in observed), observed

        start = len(output)
        success(b"set-buffer", b"-b", "café".encode(), b"one")
        read_until("%paste-buffer-changed café\n".encode(), start)
        start = len(output)
        success(b"set-buffer", b"-a", b"-b", "café".encode(), b" two")
        read_until("%paste-buffer-changed café\n".encode(), start)
        assert success(b"show-buffer", b"-b", "café".encode()) == b"one two"
        start = len(output)
        success(b"set-buffer", b"-b", "café".encode(), b"-n", b"renamed")
        read_until(b"%paste-buffer-changed renamed\n", start)
        renamed = output[start:]
        assert "%paste-buffer-deleted café\n".encode() in renamed, renamed
        assert renamed.index("%paste-buffer-deleted café\n".encode()) < renamed.index(
            b"%paste-buffer-changed renamed\n"
        ), renamed
        assert success(b"show-buffer", b"-b", b"renamed") == b"one two"
        observed.append(run(b"set-buffer", b"-b", b"missing", b"-n", b"unused"))
        observed.append(run(b"set-buffer", b"-b", b"\xff", b"bad"))
        observed.append(run(b"set-buffer", b"-b", b"renamed", b"-n", b"\xff"))
        assert all(result[0] != 0 for result in observed[3:]), observed

        success(b"set-buffer", b"automatic")
        success(b"set-buffer", b"-n", b"from-top")
        assert success(b"show-buffer", b"-b", b"from-top") == b"automatic"
        success(b"set-buffer", b"second automatic")
        success(b"delete-buffer")
        start = len(output)
        success(b"delete-buffer", b"-b", b"renamed")
        read_until(b"%paste-buffer-deleted renamed\n", start)
        assert success(b"list-buffers", b"-F", b"#{buffer_name}:#{buffer_size}") == b"from-top:9\n"
        observed.append(run(b"delete-buffer", b"-b", b"renamed"))
        assert observed[-1][0] != 0, observed[-1]

        success(b"set-option", b"-g", b"buffer-limit", b"1")
        success(b"set-buffer", b"evict me")
        success(b"set-buffer", b"keep me")
        assert success(b"show-buffer") == b"keep me"
        names = success(b"list-buffers", b"-F", b"#{buffer_name}").splitlines()
        assert names.count(b"from-top") == 1 and len(names) == 2, names
        success(b"delete-buffer")
        assert success(b"list-buffers", b"-F", b"#{buffer_name}") == b"from-top\n"
        return tuple(observed)
    finally:
        if control is not None:
            control.terminate()
            control.communicate(timeout=5)
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="set-buffer-name-owner-") as tmp:
    output = check(candidate, pathlib.Path(tmp, "candidate-socket"))
    if baseline is not None:
        baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
        assert check(baseline_binary, pathlib.Path(tmp, "baseline-socket")) == output

print("set buffer name owner CLI checks passed")
