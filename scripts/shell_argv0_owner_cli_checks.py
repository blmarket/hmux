#!/usr/bin/env python3
"""Compare job and client shell argv[0] across a private server."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary_path, directory):
    directory = os.fsencode(directory)
    socket = directory + b"/socket"
    shell = directory + b"/shell-\xc3\xa9"
    os.symlink(b"/bin/sh", shell)
    env = os.environb.copy()
    env.update({b"TERM": b"xterm-256color", b"SHELL": b"/bin/sh", b"LC_ALL": b"C.UTF-8"})
    env.pop(b"TMUX", None)
    base = [binary_path, b"-S", socket, b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=15)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    def wait_for(path):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if os.path.exists(path):
                with open(path, "rb") as output:
                    return output.read()
            time.sleep(0.05)
        raise AssertionError(f"shell did not create {path!r}")

    try:
        run(b"new-session", b"-d", b"-s", b"argv0", b"sleep", b"30")
        run(b"set-option", b"-g", b"default-shell", shell)
        shown_shell = run(b"show-option", b"-gqv", b"default-shell")
        assert shown_shell == shell + b"\n", (shown_shell, shell)

        job_output = directory + b"/job-output"
        run(b"run-shell", b"-t", b"argv0:0.0", b"printf %s \"$0\" > " + job_output)
        job_argv0 = wait_for(job_output)
        # run-shell uses the fixed /bin/sh fallback in job_run.
        assert job_argv0 == b"sh", job_argv0

        client_values = []
        for login in (False, True):
            output = directory + (b"/login-output" if login else b"/client-output")
            command = b"printf %s \"$0\" > " + output
            args = [b"-l"] if login else []
            run(*args, b"-c", command)
            value = wait_for(output)
            expected = (b"-" if login else b"") + b"shell-\xc3\xa9"
            assert value == expected, (login, value, expected)
            client_values.append(value)
        return job_argv0, client_values
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="shell-argv0-owner-", dir="/tmp") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    actual = check(binary, candidate_dir)
    if baseline:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        expected = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert actual == expected, (actual, expected)

print("shell argv0 owner CLI checks passed")
