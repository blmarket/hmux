#!/usr/bin/env python3
"""Exercise client terminfo identify messages through attached terminal setup."""

import os
import pathlib
import pty
import signal
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
(root / "target").mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="server-term-caps-", dir=root / "target") as tmp:
    tmp = pathlib.Path(tmp)
    source = tmp / "owner.src"
    source.write_text(
        "hmux-owner-cap|ownership terminfo fixture,\n"
        "\tclear=\\E[?42h\\E[H,\n"
        "\tuse=xterm-256color,\n"
    )
    terminfo = tmp / "terminfo"
    subprocess.run(["tic", "-x", "-o", str(terminfo), str(source)], check=True)

    env = dict(os.environ, TERM="hmux-owner-cap", TERMINFO=str(terminfo), TMUX="", SHELL="/bin/sh")
    socket = tmp / "socket"
    base = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    pid = None
    master = None
    try:
        run("new-session", "-d", "-s", "caps", "sleep", "30")
        pid, master = pty.fork()
        if pid == 0:
            os.execve(binary, base + ["attach-session", "-t", "caps"], env)

        deadline = time.monotonic() + 10
        report = b""
        while time.monotonic() < deadline:
            report = run("show-messages", "-T")
            if b"hmux-owner-cap" in report:
                break
            time.sleep(0.05)
        assert b"hmux-owner-cap" in report, report[-1000:]
        clear_lines = [line for line in report.splitlines() if b"clear:" in line]
        assert b"\\033[?42h\\033[H" in report, clear_lines

        client_tty = run("list-clients", "-F", "#{client_tty}").strip().decode()
        assert client_tty, "attached terminal client is missing"
        run("detach-client", "-t", client_tty)
        os.waitpid(pid, 0)
        pid = None
    finally:
        if pid is not None:
            os.kill(pid, signal.SIGTERM)
            os.waitpid(pid, 0)
        if master is not None:
            os.close(master)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)

print("server term caps CLI checks passed")
