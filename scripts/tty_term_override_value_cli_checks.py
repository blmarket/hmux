#!/usr/bin/env python3
"""Compare attached terminal overrides with the pinned pre-migration binary."""

import os
import pathlib
import pty
import signal
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
names = (b"bold", b"colors", b"cud1", b"rmkx", b"smkx", b"tsl", b"XT")


def check(executable, directory):
    base = [executable, b"-S", os.fsencode(directory) + b"/socket", b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    pid = None
    master = None
    try:
        run(b"new-session", b"-d", b"-s", b"caps", b"sleep", b"30")
        run(
            b"set-option", b"-g", b"terminal-overrides",
            b"xterm-256color:tsl=hello\\040\\377:colors=42:XT:cud1@"
            b":smkx=\\q:rmkx=\\000hidden:bold",
        )
        pid, master = pty.fork()
        if pid == 0:
            os.execve(executable, base + [b"attach-session", b"-t", b"caps"],
                      os.environb | {b"TERM": b"xterm-256color", b"LC_ALL": b"C",
                                     b"TMUX": b"", b"SHELL": b"/bin/sh"})

        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            report = run(b"show-messages", b"-T")
            if b"tsl: (string) hello \\377" in report:
                break
            time.sleep(0.05)
        else:
            raise AssertionError(report[-2000:])

        rows = tuple(
            next(line for line in report.splitlines() if b" " + name + b":" in line)
            for name in names
        )
        assert rows[0].endswith(b"bold: (string) "), rows[0]
        assert b"colors: (number) 42" in rows[1]
        assert b"cud1: [missing]" in rows[2]
        assert rows[3].endswith(b"rmkx: (string) "), rows[3]
        assert rows[4].endswith(b"smkx: (string) \\\\q"), rows[4]
        assert b"tsl: (string) hello \\377" in rows[5]
        assert b"XT: (flag) true" in rows[6]

        client_tty = run(b"list-clients", b"-F", b"#{client_tty}").strip()
        assert client_tty
        run(b"detach-client", b"-t", client_tty)
        os.waitpid(pid, 0)
        pid = None
        return rows
    finally:
        if pid is not None:
            os.kill(pid, signal.SIGTERM)
            os.waitpid(pid, 0)
        if master is not None:
            os.close(master)
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="tty-term-override-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("tty term override value CLI checks passed")
