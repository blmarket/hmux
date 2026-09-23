#!/usr/bin/env python3
"""Exercise startup configuration path ownership on private servers."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())

with tempfile.TemporaryDirectory(prefix="config-paths-", dir=root / "target") as tmp:
    base_dir = os.fsencode(tmp)
    home = base_dir + b"/home"
    config_dir = home + b"/.config/tmux"
    socket_dir = base_dir + b"/sockets"
    os.makedirs(config_dir)
    os.mkdir(socket_dir)
    env = os.environb.copy()
    env.update(
        {
            b"HOME": home,
            b"XDG_CONFIG_HOME": home + b"/.config",
            b"TMUX_TMPDIR": socket_dir,
            b"TMUX": b"",
            b"TERM": b"xterm-256color",
            b"SHELL": b"/bin/sh",
            b"LC_ALL": b"C",
        }
    )

    def run(options, *args):
        result = subprocess.run([binary, *options, *args], env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (options, args, result.returncode, result.stderr)
        return result.stdout

    def check_server(options, expected_paths, expected_order):
        try:
            run(options, b"new-session", b"-d", b"-s", b"cfg", b"sleep", b"60")
            assert run(options, b"display-message", b"-p", b"#{config_files}") == expected_paths + b"\n"
            assert run(options, b"show-option", b"-gv", b"@path-order") == expected_order + b"\n"
        finally:
            subprocess.run([binary, *options, b"kill-server"], env=env, capture_output=True, timeout=20)

    default1 = home + b"/.tmux.conf"
    default2 = config_dir + b"/tmux.conf"
    with open(default1, "wb") as config:
        config.write(b"set -g @path-order first\n")
    with open(default2, "wb") as config:
        config.write(b"set -g @path-order second\n")
    default_socket = base_dir + b"/default-socket"
    check_server(
        [b"-S", default_socket],
        b"/etc/tmux.conf," + default1 + b"," + default2,
        b"second",
    )

    override1 = base_dir + b"/override-\xff.conf"
    override2 = base_dir + b"/override-two.conf"
    with open(override1, "wb") as config:
        config.write(b"set -g @path-order override-one\n")
    with open(override2, "wb") as config:
        config.write(b"set -g @path-order override-two\n")
    override_socket = base_dir + b"/override-socket"
    check_server(
        [b"-S", override_socket, b"-f", override1, b"-f", override1, b"-f", override2],
        b",".join([override1, override1, override2]),
        b"override-two",
    )

    empty_options = [b"-S", base_dir + b"/empty-socket", b"-f", b""]
    try:
        run(empty_options, b"new-session", b"-d", b"-s", b"empty", b"sleep", b"60")
        assert run(empty_options, b"display-message", b"-p", b"#{config_files}") == b"\n"
    finally:
        subprocess.run([binary, *empty_options, b"kill-server"], env=env, capture_output=True, timeout=20)

    label = b"config-paths-" + os.fsencode(str(os.getpid()))
    label_options = [b"-L", label, b"-f", b"/dev/null"]
    try:
        run(label_options, b"new-session", b"-d", b"-s", b"label", b"sleep", b"60")
        assert os.path.exists(socket_dir + b"/tmux-" + os.fsencode(str(os.getuid())) + b"/" + label)
        assert run(label_options, b"display-message", b"-p", b"#{config_files}") == b"/dev/null\n"
    finally:
        subprocess.run([binary, *label_options, b"kill-server"], env=env, capture_output=True, timeout=20)

print("config paths CLI checks passed")
