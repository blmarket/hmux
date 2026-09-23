#!/usr/bin/env python3
"""Exercise client target lookup and its error text on a private socket."""
import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()

with tempfile.TemporaryDirectory(prefix="find-client-", dir=root / "target") as tmp:
    socket = pathlib.Path(tmp) / "socket"
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [os.fsencode(binary), b"-S", os.fsencode(socket), b"-f", b"/dev/null"]

    def run(*args):
        return subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)

    attached = None
    try:
        started = run(b"new-session", b"-d", b"-s", b"find-client", b"sleep", b"60")
        assert started.returncode == 0, started.stderr

        # A control client gives lookup a live, named client without a terminal.
        attached = subprocess.Popen(
            base + [b"-C", b"attach-session", b"-t", b"find-client"],
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        for _ in range(50):
            listed = run(b"list-clients", b"-F", b"#{client_name}")
            assert listed.returncode == 0, listed.stderr
            if listed.stdout:
                break
            assert attached.poll() is None, attached.communicate(timeout=5)
            time.sleep(0.1)
        else:
            raise AssertionError("control client did not attach")

        client_name = listed.stdout.rstrip(b"\n")
        assert client_name
        for target in (client_name, client_name + b":"):
            result = run(b"refresh-client", b"-t", target)
            assert (result.returncode, result.stdout, result.stderr) == (0, b"", b""), (
                target, result.returncode, result.stdout, result.stderr
            )

        for target, name in (
            (b"missing", b"missing"),
            (b"missing:", b"missing"),
            (b"", b""),
            (b":", b""),
            (b"bad\xff:", b"bad\xff"),
            (b"bad:extra:", b"bad:extra"),
        ):
            result = run(b"refresh-client", b"-t", target)
            assert (result.returncode, result.stdout, result.stderr) == (
                1, b"", b"can't find client: " + name + b"\n"
            ), (target, result.returncode, result.stdout, result.stderr)
    finally:
        if attached is not None:
            if attached.poll() is None:
                attached.terminate()
            attached.communicate(timeout=5)
        run(b"kill-server")
        socket.unlink(missing_ok=True)

print("find client CLI checks passed")
