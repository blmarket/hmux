#!/usr/bin/env python3
"""Compare session name creation, rename, lookup, hooks, and teardown with tmux."""

import os
import pathlib
import re
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.fsencode(pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve())
env = os.environb | {b"TERM": b"xterm-256color", b"LC_ALL": b"C.UTF-8", b"TMUX": b"", b"SHELL": b"/bin/sh"}


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="session-name-owner-") as directory:
        base = [binary, b"-S", os.fsencode(pathlib.Path(directory) / "socket"), b"-f", b"/dev/null"]

        def call(*args):
            return subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)

        def run(*args):
            result = call(*args)
            assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
            return result.stdout

        def fail(*args):
            result = call(*args)
            assert result.returncode != 0, (args, result.stdout, result.stderr)
            return result.stderr

        def names():
            return run(b"list-sessions", b"-O", b"name", b"-F", b"#{session_name}").splitlines()

        def lookup(name):
            return run(b"display-message", b"-p", b"-t", name, b"#{session_name}")

        try:
            run(b"new-session", b"-d", b"-s", b"alpha", b"sleep 60")
            run(b"new-session", b"-d", b"-s", "écho".encode(), b"sleep 60")
            run(b"new-session", b"-d", b"-s", b"zeta", b"sleep 60")
            assert names() == [b"alpha", b"zeta", "écho".encode()]

            run(
                b"set-hook", b"-g", b"session-renamed",
                b'set-option -gF @name-event "#{hook_old_name}|#{hook_new_name}|#{hook_session_name}"',
            )
            run(b"rename-session", b"-t", b"alpha", b"alpha")
            assert run(b"show-options", b"-gqv", b"@name-event") == b""
            run(b"rename-session", b"-t", b"alpha", b"beta")
            hook = run(b"show-options", b"-gqv", b"@name-event")
            assert hook == b"alpha|beta|beta\n", hook
            assert lookup(b"beta") == b"beta\n"
            assert b"alpha" not in names()

            duplicate = fail(b"rename-session", b"-t", b"beta", b"zeta")
            invalid = fail(b"rename-session", b"-t", b"beta", b"\xff")
            assert lookup(b"beta") == b"beta\n"
            assert run(b"show-options", b"-gqv", b"@name-event") == hook

            before = set(names())
            run(b"new-session", b"-d", b"sleep 60")
            generated, = set(names()) - before
            assert re.fullmatch(rb"[0-9]+", generated), generated
            assert lookup(generated) == generated + b"\n"

            before = set(names())
            run(b"new-session", b"-d", b"-t", "écho".encode())
            grouped, = set(names()) - before
            assert re.fullmatch("écho-[0-9]+".encode(), grouped), grouped
            assert lookup(grouped) == grouped + b"\n"

            sorted_names = names()
            assert sorted_names == sorted(sorted_names), sorted_names
            assert set(sorted_names) == {b"beta", b"zeta", "écho".encode(), generated, grouped}
            loop = run(b"display-message", b"-p", b"-t", b"beta", b"#{S:#{session_name};}")
            assert loop.endswith(b";\n") and set(loop[:-2].split(b";")) == set(sorted_names), (
                loop, sorted_names
            )

            run(b"kill-session", b"-t", b"beta")
            missing = fail(b"has-session", b"-t", b"beta")
            run(b"new-session", b"-d", b"-s", b"beta", b"sleep 60")
            assert lookup(b"beta") == b"beta\n"
            run(b"kill-session", b"-t", grouped)
            run(b"kill-session", b"-t", generated)
            assert names() == [b"beta", b"zeta", "écho".encode()]

            return (hook, duplicate, invalid, missing, generated, grouped, tuple(sorted_names))
        finally:
            call(b"kill-server")


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("session name owner CLI checks passed")
