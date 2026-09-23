#!/usr/bin/env python3
"""Compare pane stdin completion and cancellation with the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary, directory):
    command = [str(binary), "-S", str(directory / "socket"), "-f", "/dev/null"]
    env = os.environ.copy()
    env.update({"TERM": "xterm-256color", "TMUX": ""})

    def run(*args, data=None):
        result = subprocess.run(
            command + list(args), input=data, env=env, capture_output=True, timeout=20
        )
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "pane-input", "sleep", "60")
        run("split-window", "-d", "-I", data=b"FIRST\n" + b"A" * 20000 + b"\nEND\n")
        run("display-message", "-I", "-t", "pane-input:0.1", data=b"SECOND\n")
        captured = run("capture-pane", "-p", "-t", "pane-input:0.1", "-S", "-")
        assert b"END\nSECOND\n" in captured, captured[-100:]

        # A control client cannot supply pane stdin. file_read returns null
        # after scheduling its terminal callback for this EBADF path.
        control = subprocess.run(
            command[:1] + ["-C"] + command[1:] + [
                "display-message", "-I", "-t", "pane-input:0.1"
            ],
            input=b"",
            env=env,
            capture_output=True,
            timeout=20,
        )
        assert control.returncode == 0, (control.returncode, control.stderr)
        assert b"%begin " in control.stdout and b"%end " in control.stdout
        assert control.stdout.endswith(b"%exit\n"), control.stdout

        run("new-session", "-d", "-s", "cancel", "sleep", "60")
        pending = subprocess.Popen(
            command + ["split-window", "-d", "-I", "-t", "cancel:0"],
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                panes = run("list-panes", "-t", "cancel:0", "-F", "#{pane_index}")
                if b"1\n" in panes:
                    break
                time.sleep(0.02)
            else:
                raise AssertionError("input pane was not created")

            pending.stdin.write(b"BEFORE CANCEL\n")
            pending.stdin.flush()
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                pane = run("capture-pane", "-p", "-t", "cancel:0.1")
                if b"BEFORE CANCEL\n" in pane:
                    break
                time.sleep(0.02)
            else:
                raise AssertionError("input was not parsed before cancellation")
            run("kill-pane", "-t", "cancel:0.1")
            pending.stdin.close()
            return_code = pending.wait(timeout=20)
            stdout, stderr = pending.stdout.read(), pending.stderr.read()
            assert b"1\n" not in run(
                "list-panes", "-t", "cancel:0", "-F", "#{pane_index}"
            )
            assert run("display-message", "-p", "-t", "cancel:0", "#{session_name}") == b"cancel\n"
        finally:
            if pending.poll() is None:
                pending.kill()
                pending.wait(timeout=20)
        control_events = tuple(line.split(b" ", 1)[0] for line in control.stdout.splitlines())
        return captured, control_events, return_code, stdout, stderr
    finally:
        subprocess.run(command + ["kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="pane-input-owner-") as temporary:
    candidate_dir = pathlib.Path(temporary, "candidate")
    baseline_dir = pathlib.Path(temporary, "baseline")
    candidate_dir.mkdir()
    baseline_dir.mkdir()
    assert trace(candidate, candidate_dir) == trace(baseline, baseline_dir)

print("pane input owner CLI checks passed")
