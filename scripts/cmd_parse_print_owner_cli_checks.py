#!/usr/bin/env python3
"""Compare parser verbose output and command-list debug logs with the baseline."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, directory, source):
    directory.mkdir()
    socket = directory / "socket"
    env = os.environ.copy()
    env.update(TERM="xterm-256color", TMUX="", SHELL="/bin/sh", LC_ALL="C.UTF-8")
    command = [str(binary), "-vv", "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(
            command + list(args), cwd=directory, env=env, capture_output=True, timeout=15
        )
        assert result.returncode == 0, (args, result.returncode, result.stdout, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "parse", "sleep 60")
        verbose = run("source-file", "-v", str(source))
        binding = run("list-keys", "-T", "root", "C-g")
    finally:
        subprocess.run(command + ["kill-server"], cwd=directory, env=env, capture_output=True, timeout=15)

    source_bytes = os.fsencode(source)
    assert verbose == b"".join(
        [
            source_bytes + b":1: display-message -p plain-owner-marker\n",
            source_bytes + b":2: display-message -p arg-owner-marker\n",
            source_bytes
            + b":2: bind-key -n C-g if-shell -F 1 { display-message -p arg-owner-marker }\n",
            b"plain-owner-marker\n",
        ]
    ), verbose
    assert binding == (
        b"bind-key  -T root C-g if-shell -F 1 { display-message -p arg-owner-marker }\n"
    ), binding

    logs = list(directory.glob("tmux-server-*.log"))
    assert len(logs) == 1, logs
    entries = [
        line.split(b" ", 1)[1]
        for line in logs[0].read_bytes().splitlines()
        if b"cmd_parse_build_commands" in line and b"owner-marker" in line
    ]
    assert b"cmd_parse_build_commands 0:3: display-message -p arg-owner-marker" in entries
    assert (
        b"cmd_parse_build_commands: display-message -p plain-owner-marker ;;"
        b" bind-key -n C-g if-shell -F 1"
        b" { display-message -p arg-owner-marker }"
    ) in entries, entries
    return verbose, binding, entries


with tempfile.TemporaryDirectory(prefix="cmd-parse-print-owner-") as tmp:
    directory = pathlib.Path(tmp)
    source = directory / "source.conf"
    source.write_bytes(
        b"display-message -p plain-owner-marker\n"
        b"bind-key -n C-g if-shell -F 1 { display-message -p arg-owner-marker }\n"
    )
    output = check(candidate, directory / "candidate", source)
    if baseline is not None:
        assert check(pathlib.Path(baseline).resolve(), directory / "baseline", source) == output

print("command parser print owner CLI checks passed")
