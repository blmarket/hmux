#!/usr/bin/env python3
"""Reproduce and check cleanup of a disconnected `source-file -` client."""

import argparse
import os
import pathlib
import re
import signal
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "TERM": "xterm-256color", "LC_ALL": "C", "TMUX": "", "SHELL": "/bin/sh"}


def poll_until(condition, timeout, description):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = condition()
        if result:
            return result
        time.sleep(0.025)
    raise AssertionError(f"timed out waiting for {description}")


def check(binary, directory):
    directory.mkdir()
    socket = directory / "socket"
    base = [str(binary), "-vv", "-S", str(socket), "-f", "/dev/null"]
    pending = None
    server_pid = None

    def run(*args):
        result = subprocess.run(base + list(args), cwd=directory, env=env, capture_output=True, timeout=5)
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    def server_log():
        logs = list(directory.glob("tmux-server-*.log"))
        assert len(logs) == 1, logs
        return logs[0].read_bytes()

    try:
        run("new-session", "-d", "-s", "file-wait", "sleep 30")
        server_pid = int(run("display-message", "-p", "#{pid}").strip())
        pending = subprocess.Popen(
            base + ["source-file", "-", ";", "set-option", "-g", "@should_not_run", "1"],
            cwd=directory,
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        assert pending.stdin is not None
        pending.stdin.write(b"set-option -g @source_probe loaded\n")
        pending.stdin.flush()
        pid_marker = b"IDENTIFY_CLIENTPID " + str(pending.pid).encode()

        def waiting_pointer():
            log = server_log()
            match = re.search(rb"client (0x[0-9a-f]+) " + pid_marker + rb"(?:\n|\r\n)", log)
            if match is None or b"cmd_source_file_exec" not in log:
                return None
            assert b"@should_not_run" in log, "queued suffix was not parsed"
            assert pending.poll() is None, (pending.returncode, pending.stderr.read())
            return match.group(1), match.end()

        pointer, start = poll_until(waiting_pointer, 5, "waiting source-file client")
        assert run("show-options", "-gqv", "@should_not_run") == b""
        # The parent still owns the write end when this client is killed.
        assert not pending.stdin.closed
        pending.kill()
        pending.wait(timeout=5)
        assert pending.returncode == -signal.SIGKILL, pending.returncode

        lost_marker = b"lost client " + pointer + b"\n"
        free_marker = b"free client " + pointer + b" ("
        poll_until(lambda: lost_marker in server_log()[start:], 5, "lost client log")

        # Event-loop cleanup is immediate. Give it a bounded window so a
        # missing free is distinguishable from a delayed log write.
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline and free_marker not in server_log()[start:]:
            time.sleep(0.025)
        log = server_log()[start:]
        freed = free_marker in log
        related = [
            line for line in log.splitlines()
            if pointer in line and any(marker in line for marker in (b"lost client", b"free client", b"unref client"))
        ]
        assert run("list-sessions", "-F", "#{session_name}") == b"file-wait\n"
        assert run("show-options", "-gqv", "@should_not_run") == b""
        return freed, related
    finally:
        if pending is not None:
            if pending.poll() is None:
                pending.kill()
                pending.wait(timeout=5)
            if pending.stdin is not None:
                pending.stdin.close()
            if pending.stderr is not None:
                pending.stderr.close()
        try:
            subprocess.run(base + ["kill-server"], cwd=directory, env=env, capture_output=True, timeout=5)
        except subprocess.TimeoutExpired:
            # Only signal the private server when its command line still names
            # this test's socket; PID reuse cannot target an unrelated process.
            if server_pid is not None:
                command_line = pathlib.Path(f"/proc/{server_pid}/cmdline")
                if command_line.exists() and os.fsencode(socket) in command_line.read_bytes():
                    os.kill(server_pid, signal.SIGTERM)


parser = argparse.ArgumentParser()
parser.add_argument("--observe", action="store_true", help="report the current leak without requiring its fix")
args = parser.parse_args()
with tempfile.TemporaryDirectory(prefix="disconnected-file-client-") as temporary:
    directory = pathlib.Path(temporary)
    baseline_freed, baseline_log = check(baseline, directory / "baseline")
    candidate_freed, candidate_log = check(candidate, directory / "candidate")

if args.observe:
    print("baseline log:", *baseline_log, sep="\n")
    print("candidate log:", *candidate_log, sep="\n")
assert not baseline_freed, "the pinned tmux unexpectedly freed the disconnected client"
if not args.observe:
    assert candidate_freed, "candidate still retains the disconnected file-reading client"
print(f"disconnected file client: baseline freed={baseline_freed}, candidate freed={candidate_freed}")
