#!/usr/bin/env python3
"""Check startup socket selection and its raw-byte path ownership.

Set HMUX_BINARY for the candidate and HMUX_BASELINE_BINARY for pinned tmux.
"""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check_binary(executable, directory):
    directory = os.fsencode(directory)
    first = directory + b"/first-\xfe"
    selected = directory + b"/selected-\xff"
    comma_path = directory + b"/literal,comma-\xff"
    env = os.environb.copy()
    env.update({
        b"HOME": directory,
        b"TERM": b"xterm-256color",
        b"LC_ALL": b"C",
        b"SHELL": b"/bin/sh",
    })
    env.pop(b"TMUX", None)

    def run(options, *args, tmux=None):
        command_env = env.copy()
        if tmux is not None:
            command_env[b"TMUX"] = tmux
        result = subprocess.run(
            [executable, *options, b"-f", b"/dev/null", *args],
            env=command_env, capture_output=True, timeout=20,
        )
        assert result.returncode == 0, (options, args, result.returncode, result.stderr)
        return result.stdout

    repeated = [b"-S", first, b"-S", selected]
    explicit_comma = [b"-S", comma_path]
    results = []
    try:
        run(repeated, b"new-session", b"-d", b"-s", b"startup", b"sleep 60")
        assert os.path.exists(selected), selected
        assert not os.path.exists(first), first
        expected_raw = selected + b"\n"
        expected_display = selected.replace(b"\xff", b"_") + b"\n"

        plain_display = run(repeated, b"display-message", b"-p", b"#{socket_path}")
        assert plain_display == expected_display, plain_display
        results.append(plain_display.replace(directory, b"<private-dir>"))

        # The last -S wins, including when TMUX names an unrelated socket.
        displayed = run(repeated, b"display-message", b"-p", b"#{socket_path}",
                        tmux=first + b",11,0")
        assert displayed == expected_raw, displayed
        results.append(displayed.replace(directory, b"<private-dir>"))

        # Without -S or -L, TMUX supplies only its bytes before the first comma.
        derived = run([], b"display-message", b"-p", b"#{socket_path}",
                      tmux=selected + b",27,0,ignored")
        assert derived == expected_raw, derived
        results.append(derived.replace(directory, b"<private-dir>"))

        # A comma in an explicit -S path is literal, unlike TMUX's delimiter.
        run(explicit_comma, b"new-session", b"-d", b"-s", b"comma", b"sleep 60")
        assert os.path.exists(comma_path), comma_path
        comma_display = run(explicit_comma, b"display-message", b"-p", b"#{socket_path}")
        assert comma_display == comma_path.replace(b"\xff", b"_") + b"\n", comma_display
        results.append(comma_display.replace(directory, b"<private-dir>"))
    finally:
        for options in (explicit_comma, repeated):
            subprocess.run(
                [executable, *options, b"-f", b"/dev/null", b"kill-server"],
                env=env, capture_output=True, timeout=20,
            )
    return tuple(results)


with tempfile.TemporaryDirectory(prefix="startup-socket-owner-", dir="/tmp") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check_binary(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        expected = check_binary(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == expected, (candidate, expected)

print("startup socket path owner CLI checks passed")
