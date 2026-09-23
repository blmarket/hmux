#!/usr/bin/env python3
"""Compare startup path expansion on private servers, including raw bytes."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
uid = os.fsencode(str(os.getuid()))


def run_case(binary, parent, case):
    parent = os.fsencode(parent)
    home = parent + b"/home-\xfe"
    xdg = parent + b"/xdg-\xfd"
    socket_base = parent + b"/socket-\xff"
    socket_alias = parent + b"/socket-alias"
    os.makedirs(home + b"/.config/tmux")
    os.makedirs(xdg + b"/tmux")
    os.mkdir(socket_base)
    os.symlink(socket_base, socket_alias)
    with open(home + b"/.tmux.conf", "wb") as config:
        config.write(b"set -g @expanded-path home\n")
    with open(xdg + b"/tmux/tmux.conf", "wb") as config:
        config.write(b"set -g @expanded-path xdg\n")

    env = os.environb.copy()
    env.update(
        {
            b"HOME": home,
            b"TMUX_TMPDIR": socket_alias,
            b"TERM": b"xterm-256color",
            b"SHELL": b"/bin/sh",
            b"LC_ALL": b"C",
        }
    )
    env.pop(b"TMUX", None)
    if case != "xdg-absent":
        env[b"XDG_CONFIG_HOME"] = xdg
    else:
        env.pop(b"XDG_CONFIG_HOME", None)
    if case == "missing-socket-base":
        env[b"TMUX_TMPDIR"] = parent + b"/missing"

    label = b"expand-path-owner-" + os.fsencode(str(os.getpid()))
    expected_socket = (
        b"/tmp" if case == "missing-socket-base" else socket_base
    ) + b"/tmux-" + uid + b"/" + label
    options = [binary, b"-L", label]

    def run(*args):
        return subprocess.run(options + list(args), env=env, capture_output=True, timeout=20)

    try:
        create = run(b"new-session", b"-d", b"-s", b"paths", b"sleep", b"60")
        assert create.returncode == 0, (case, create.stderr)
        assert os.path.exists(expected_socket)
        socket_path = run(b"display-message", b"-p", b"#{socket_path}")
        assert socket_path.returncode == 0, (case, socket_path.stderr)
        assert socket_path.stdout == expected_socket.replace(b"\xff", b"_") + b"\n", socket_path.stdout
        config_files = run(b"display-message", b"-p", b"#{config_files}")
        assert config_files.returncode == 0, (case, config_files.stderr)
        expanded_path = run(b"show-option", b"-gv", b"@expanded-path")
        assert expanded_path.returncode == 0, (case, expanded_path.stderr)
        expected = b"home\n" if case == "xdg-absent" else b"xdg\n"
        assert expanded_path.stdout == expected, (case, expanded_path.stdout)
        # The format renderer replaces non-UTF-8 bytes with underscores.
        assert home.replace(b"\xfe", b"_") + b"/.tmux.conf" in config_files.stdout
        if case != "xdg-absent":
            assert xdg.replace(b"\xfd", b"_") + b"/tmux/tmux.conf" in config_files.stdout
        else:
            assert b"(null)/tmux/tmux.conf" not in config_files.stdout
        return (
            create.returncode,
            socket_path.stdout.replace(parent, b"<private-dir>"),
            config_files.stdout.replace(parent, b"<private-dir>"),
            expanded_path.stdout,
        )
    finally:
        run(b"kill-server")


with tempfile.TemporaryDirectory(prefix="expand-path-owner-", dir="/tmp") as tmp:
    for case in ("xdg-present", "xdg-absent", "missing-socket-base"):
        candidate_dir = pathlib.Path(tmp, "candidate-" + case)
        candidate_dir.mkdir()
        candidate_result = run_case(candidate, candidate_dir, case)
        if baseline is not None:
            baseline_dir = pathlib.Path(tmp, "baseline-" + case)
            baseline_dir.mkdir()
            baseline_binary = os.fsencode(pathlib.Path(baseline).resolve())
            baseline_result = run_case(baseline_binary, baseline_dir, case)
            assert candidate_result == baseline_result, (case, candidate_result, baseline_result)

print("expand path owner CLI checks passed")
