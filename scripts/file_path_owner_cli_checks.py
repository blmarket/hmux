#!/usr/bin/env python3
"""Compare file-record path lifetimes across read, write, and error callbacks."""

import os
import pathlib
import fcntl
import pty
import struct
import subprocess
import tempfile
import termios
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.fsencode(pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve())


def trace(binary, directory):
    directory = os.fsencode(directory)
    home = directory + b"/home-\xfe"
    os.mkdir(home)
    work = directory + b"/work-\xfd"
    os.mkdir(work)
    source = work + b"/source-\xff"
    payload = b"file-path-owner-A\x00B\xff\n"
    with open(source, "wb") as output:
        output.write(payload)
    socket = directory + b"/socket"
    env = os.environb.copy()
    env.update({b"HOME": home, b"TERM": b"xterm-256color", b"SHELL": b"/bin/sh", b"LC_ALL": b"C", b"TMUX": b""})
    base = [binary, b"-S", socket, b"-f", b"/dev/null"]

    def run(*args, ok=True):
        process = subprocess.run(base + list(args), env=env, cwd=work, capture_output=True, timeout=20)
        if ok:
            assert process.returncode == 0, (args, process.returncode, process.stderr)
        return process

    try:
        run(b"new-session", b"-d", b"-s", b"file-path-owner", b"sleep", b"60")
        run(b"load-buffer", b"-b", b"owned", source)
        assert run(b"show-buffer", b"-b", b"owned").stdout == payload

        paths = [
            (b"relative-\xfc", work + b"/relative-\xfc"),
            (b"~/home-relative-\xfb", home + b"/home-relative-\xfb"),
            (directory + b"/absolute-\xfa", directory + b"/absolute-\xfa"),
        ]
        for requested, resolved in paths:
            run(b"save-buffer", b"-b", b"owned", requested)
            with open(resolved, "rb") as output:
                assert output.read() == payload
            run(b"load-buffer", b"-b", b"again", requested)
            assert run(b"show-buffer", b"-b", b"again").stdout == payload

        missing = run(b"load-buffer", b"-b", b"missing", b"missing-\xf9", ok=False)
        assert missing.returncode != 0
        assert b"missing-" in missing.stderr
        # The dash path stays a real C string through the deferred callback.
        assert run(b"save-buffer", b"-b", b"owned", b"-").stdout == payload

        # Key bindings execute from an attached client. Its file paths are
        # handled locally and the completion callback runs on a later event.
        attached_target = work + b"/attached-\xf7"
        run(b"bind-key", b"-n", b"C-g", b"load-buffer", b"-b", b"attached", source)
        run(b"bind-key", b"-n", b"C-h", b"save-buffer", b"-b", b"attached", attached_target)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = subprocess.Popen(
            base + [b"attach-session", b"-t", b"file-path-owner"],
            env=env, cwd=work, stdin=slave, stdout=slave, stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        try:
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline and not run(b"list-clients", b"-F", b"#{client_tty}").stdout.strip():
                time.sleep(0.05)
            assert run(b"list-clients", b"-F", b"#{client_tty}").stdout.strip()
            os.write(master, b"\x07")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                read = run(b"show-buffer", b"-b", b"attached", ok=False)
                if read.returncode == 0 and read.stdout == payload:
                    break
                time.sleep(0.05)
            else:
                raise AssertionError("attached load-buffer did not complete")
            os.write(master, b"\x08")
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline and not os.path.exists(attached_target):
                time.sleep(0.05)
            with open(attached_target, "rb") as output:
                assert output.read() == payload
        finally:
            os.close(master)
            client.wait(timeout=5)
        return payload, missing.returncode, missing.stderr.replace(work, b"<cwd>")
    finally:
        subprocess.run(base + [b"kill-server"], env=env, cwd=work, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="file-path-owner-") as temporary:
    candidate_dir = pathlib.Path(temporary, "candidate")
    baseline_dir = pathlib.Path(temporary, "baseline")
    candidate_dir.mkdir()
    baseline_dir.mkdir()
    assert trace(candidate, candidate_dir) == trace(baseline, baseline_dir)

print("file path owner CLI checks passed")
