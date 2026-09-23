#!/usr/bin/env python3
"""Exercise load-buffer completion, errors, and source-client cancellation."""

import fcntl
import os
import pathlib
import pty
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="load-buffer-owner-", dir=root / "target") as tmp:
        directory = pathlib.Path(tmp)
        base = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
        source = directory / "source"
        source.write_bytes(b"A\0B\xff")
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

            run("load-buffer", "-w", "-t", target, "-b", "filled", str(source))
            filled = run("show-buffer", "-b", "filled")
            assert filled == b"A\0B\xff", filled

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
            return filled, empty_lookup.returncode, missing.returncode, invalid.returncode, cancelled.returncode
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
