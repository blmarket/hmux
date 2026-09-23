#!/usr/bin/env python3
"""Exercise named and automatic buffer operations across every name lifetime."""

import base64
import fcntl
import os
import pathlib
import pty
import select
import struct
import subprocess
import tempfile
import termios
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


def check_clipboard(binary, directory):
    directory.mkdir()
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"LC_ALL": b"C", b"SHELL": b"/bin/sh", b"TMUX": b""})
    command = [binary, b"-S", os.fsencode(directory / "clipboard-socket"), b"-f", b"/dev/null"]
    payload = b"A\0B\xff"
    source = directory / "binary-buffer"
    source.write_bytes(payload)

    def run(*args):
        result = subprocess.run(command + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result)
        return result.stdout

    def read_osc52(master):
        output = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                output.extend(os.read(master, 65536))
            start = output.find(b"\x1b]52;")
            if start >= 0:
                bell = output.find(b"\x07", start)
                st = output.find(b"\x1b\\", start)
                end = min((i for i in (bell, st) if i >= 0), default=-1)
                if end >= 0:
                    return bytes(output[start:end])
        raise AssertionError(("set-buffer -w OSC 52 output missing", output[-300:]))

    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    attached = None
    try:
        run(b"new-session", b"-d", b"-s", b"clipboard", b"sleep 30")
        run(b"set-option", b"-g", b"terminal-features", b"xterm-256color:clipboard")
        attached = subprocess.Popen(
            command + [b"attach-session", b"-t", b"clipboard"],
            env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
        )
        os.close(slave)
        slave = None
        deadline = time.monotonic() + 5
        while not (target := run(b"list-clients", b"-F", b"#{client_name}").strip()):
            assert attached.poll() is None, "clipboard client exited"
            assert time.monotonic() < deadline, "clipboard client did not attach"
            time.sleep(0.02)

        run(b"load-buffer", b"-b", b"named", os.fsencode(source))
        before = run(b"list-buffers", b"-F", b"#{buffer_name}:#{buffer_size}")
        while select.select([master], [], [], 0)[0]:
            os.read(master, 65536)
        empty = subprocess.run(
            command + [b"set-buffer", b"-w", b"-t", target, b"-b", b"named", b""],
            env=env, capture_output=True, timeout=15,
        )
        assert empty.returncode == 0, empty
        assert run(b"show-buffer", b"-b", b"named") == payload
        assert run(b"list-buffers", b"-F", b"#{buffer_name}:#{buffer_size}") == before

        invalid = subprocess.run(
            command + [b"set-buffer", b"-w", b"-t", target, b"-b", b"\xff", b"bad"],
            env=env, capture_output=True, timeout=15,
        )
        assert invalid.returncode != 0 and b"invalid buffer name" in invalid.stderr, invalid
        assert run(b"show-buffer", b"-b", b"named") == payload
        assert run(b"list-buffers", b"-F", b"#{buffer_name}:#{buffer_size}") == before

        quiet_output = bytearray()
        deadline = time.monotonic() + 0.2
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
            if ready:
                quiet_output.extend(os.read(master, 65536))
        assert b"\x1b]52;" not in quiet_output, quiet_output[-300:]
        run(b"set-buffer", b"-a", b"-w", b"-t", target, b"-b", b"named", b" tail")
        expected = payload + b" tail"
        assert run(b"show-buffer", b"-b", b"named") == expected
        osc52 = read_osc52(master)
        assert osc52 == b"\x1b]52;;" + base64.b64encode(expected), osc52
        return empty.returncode, invalid.returncode, invalid.stderr, expected, osc52
    finally:
        if attached is not None:
            attached.terminate()
            attached.wait(timeout=5)
        if slave is not None:
            os.close(slave)
        os.close(master)
        subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)


with tempfile.TemporaryDirectory(prefix="set-buffer-name-owner-") as tmp:
    directory = pathlib.Path(tmp)
    output = check(candidate, directory / "candidate-socket")
    clipboard = check_clipboard(candidate, directory / "candidate")
    if baseline is not None:
        baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
        assert check(baseline_binary, directory / "baseline-socket") == output
        assert check_clipboard(baseline_binary, directory / "baseline") == clipboard

print("set buffer name owner CLI checks passed")
