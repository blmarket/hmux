#!/usr/bin/env python3
"""Check -L socket directory creation and errors with raw path bytes.

Set HMUX_BINARY to choose the candidate binary. Set HMUX_BASELINE_BINARY to
compare its CLI output byte for byte after replacing each private temp path.
"""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
label = b"socket-base-e2e"
uid = os.fsencode(str(os.getuid()))


def check_case(binary_path, parent, case):
    parent = os.fsencode(parent)
    socket_base = parent + b"/socket-\xff"
    uid_dir = socket_base + b"/tmux-" + uid
    socket_path = uid_dir + b"/" + label
    os.mkdir(socket_base)
    env = os.environb.copy()
    env.update(
        {
            b"TMUX_TMPDIR": socket_base,
            b"HOME": parent,
            b"TERM": b"xterm-256color",
            b"SHELL": b"/bin/sh",
            b"LC_ALL": b"C",
        }
    )
    env.pop(b"TMUX", None)
    options = [binary_path, b"-L", label, b"-f", b"/dev/null"]

    def run(*args):
        return subprocess.run(options + list(args), env=env, capture_output=True, timeout=15)

    if case == "not-directory":
        with open(uid_dir, "wb") as file:
            file.write(b"occupied")
    elif case == "unsafe-permissions":
        os.mkdir(uid_dir)
        os.chmod(uid_dir, 0o755)

    try:
        create = run(b"new-session", b"-d", b"-s", b"socketbase", b"sleep", b"60")
        if case == "success":
            assert create.returncode == 0, (case, create.stdout, create.stderr)
            assert os.path.exists(socket_path), socket_path
            display = run(b"list-sessions", b"-F", b"#{session_name}")
            assert display.returncode == 0, (case, display.stdout, display.stderr)
            assert display.stdout == b"socketbase\n", display.stdout
        else:
            assert create.returncode != 0, (case, create.stdout, create.stderr)
            expected = {
                "not-directory": b"is not a directory",
                "unsafe-permissions": b"has unsafe permissions",
            }[case]
            assert expected in create.stderr, (case, create.stderr)
            assert socket_base in create.stderr, (case, create.stderr)
            assert not os.path.exists(socket_path), socket_path
            display = None

        def normalized(data):
            return data.replace(parent, b"<private-dir>")

        return (
            create.returncode,
            normalized(create.stdout),
            normalized(create.stderr),
            None if display is None else normalized(display.stdout),
        )
    finally:
        if case == "success":
            run(b"kill-server")


with tempfile.TemporaryDirectory(prefix="socket-label-base-", dir="/tmp") as tmp:
    for case in ("success", "not-directory", "unsafe-permissions"):
        candidate_dir = pathlib.Path(tmp, "candidate-" + case)
        candidate_dir.mkdir()
        candidate_output = check_case(binary, candidate_dir, case)
        if baseline is not None:
            baseline_dir = pathlib.Path(tmp, "baseline-" + case)
            baseline_dir.mkdir()
            baseline_output = check_case(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir, case)
            assert candidate_output == baseline_output, (case, candidate_output, baseline_output)

print("socket label base CLI checks passed")
