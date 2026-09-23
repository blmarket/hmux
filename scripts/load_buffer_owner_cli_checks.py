#!/usr/bin/env python3
"""Compare load-buffer ownership paths with the pinned tmux baseline."""

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
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
payload = b"A\0B\xff"


def drain_pty(master):
    while select.select([master], [], [], 0)[0]:
        os.read(master, 65536)


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
    raise AssertionError(("OSC 52 output missing", output[-300:]))


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="load-buffer-owner-", dir=root / "target") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
        source = directory / "source"
        source.write_bytes(payload)
        empty = directory / "empty"
        empty.touch()
        fifo = directory / "pending"
        os.mkfifo(fifo)

        def command(*args):
            return subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)

        def run(*args):
            result = command(*args)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        attached = None
        pending = None
        try:
            run("new-session", "-d", "-s", "owner", "sleep 30")
            run("set-option", "-g", "terminal-features", "xterm-256color:clipboard")
            attached = subprocess.Popen(
                base + ["attach-session", "-t", "owner"], env=env,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not (target := run("list-clients", "-F", "#{client_name}").strip()):
                assert attached.poll() is None, "target client exited"
                assert time.monotonic() < deadline, "target client did not attach"
                time.sleep(0.02)
            target = os.fsdecode(target)

            run("set-buffer", "-w", "-t", target, "-b", "set-clipboard", "prefix")
            run("set-buffer", "-a", "-w", "-t", target, "-b", "set-clipboard", " suffix")
            set_clipboard = run("show-buffer", "-b", "set-clipboard")
            assert set_clipboard == b"prefix suffix", set_clipboard

            run("load-buffer", "-w", "-t", target, "-b", "filled", str(source))
            filled = run("show-buffer", "-b", "filled")
            assert filled == payload, filled

            drain_pty(master)
            run("load-buffer", "-w", "-t", target, "-b", "clipboard-output", str(source))
            osc52 = read_osc52(master)
            assert osc52 == b"\x1b]52;;" + base64.b64encode(payload), osc52

            before_auto = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            run("load-buffer", str(source))
            automatic = run("show-buffer")
            after_auto = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            assert automatic == payload, automatic
            assert len(after_auto) == len(before_auto) + 1, (before_auto, after_auto)

            run("set-buffer", "-b", "replacement", "old")
            before_replace = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            run("load-buffer", "-b", "replacement", str(source))
            replacement = run("show-buffer", "-b", "replacement")
            after_replace = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            assert replacement == payload, replacement
            assert len(after_replace) == len(before_replace), (before_replace, after_replace)

            raw_name = b"\xff-name"
            before_raw = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            invalid_utf8 = command("load-buffer", "-b", raw_name, str(source))
            assert invalid_utf8.returncode != 0, invalid_utf8
            assert b"invalid buffer name" in invalid_utf8.stderr, invalid_utf8
            after_raw = run("list-buffers", "-F", "#{buffer_name}").splitlines()
            assert after_raw == before_raw, (before_raw, after_raw)

            run("load-buffer", "-w", "-t", target, "-b", "empty", str(empty))
            empty_lookup = command("show-buffer", "-b", "empty")
            assert empty_lookup.returncode != 0, empty_lookup

            missing = command("load-buffer", "-w", "-t", target, "-b", "missing", str(directory / "absent"))
            assert missing.returncode != 0, missing
            invalid = command("load-buffer", "-w", "-t", target, "-b", "", str(source))
            assert invalid.returncode != 0 and b"empty buffer name" in invalid.stderr, invalid

            pending = subprocess.Popen(
                base + ["load-buffer", "-w", "-t", target, "-b", "cancelled", str(fifo)],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            time.sleep(0.2)
            assert pending.poll() is None, "FIFO read did not remain pending"
            pending.terminate()
            pending.communicate(timeout=5)
            pending = None
            assert run("has-session", "-t", "owner") == b""
            cancelled = command("show-buffer", "-b", "cancelled")
            assert cancelled.returncode != 0, cancelled

            attached.terminate()
            attached.wait(timeout=5)
            attached = None
            deadline = time.monotonic() + 5
            while run("list-clients", "-F", "#{client_name}") != b"":
                assert time.monotonic() < deadline, "target client remained attached"
                time.sleep(0.02)
            return (
                set_clipboard, filled, osc52, automatic, len(after_auto),
                replacement, len(after_replace), invalid_utf8.returncode,
                invalid_utf8.stderr,
                empty_lookup.returncode, missing.returncode, invalid.returncode,
                cancelled.returncode,
            )
        finally:
            if pending is not None:
                pending.terminate()
                pending.communicate(timeout=5)
            if attached is not None:
                attached.terminate()
                attached.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=20)


reference = trace(baseline)
observed = trace(candidate)
assert observed == reference, (observed, reference)
print("load buffer owner CLI checks passed")
