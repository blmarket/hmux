#!/usr/bin/env python3
"""Exercise client terminfo identify messages through attached terminal setup."""

import os
import pathlib
import pty
import pwd
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
        "hmux-owner-no-clear|ownership invalid terminfo fixture,\n"
        "\tcup=\\E[%i%p1%d;%p2%dH,\n"
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
        assert run("list-clients", "-F", "#{client_termname}") == b"hmux-owner-cap\n"
        user = pwd.getpwuid(os.getuid()).pw_name.encode()
        expected = user + b"|" + user + b"\n"
        assert run("list-clients", "-F", "#{client_user}|#{client_user}") == expected
        assert run("list-clients", "-F", "#{client_user}|#{client_user}") == expected
        assert run("list-clients", "-F", "#{client_termtype}") == b"\n"
        os.write(master, b"\x1bP>|XTerm(370)\x1b\\")
        deadline = time.monotonic() + 10
        term_type = b""
        while time.monotonic() < deadline:
            term_type = run("list-clients", "-F", "#{client_termtype}")
            if term_type == b"XTerm(370)\n":
                break
            time.sleep(0.05)
        assert term_type == b"XTerm(370)\n", term_type
        run("detach-client", "-t", client_tty)
        os.waitpid(pid, 0)
        pid = None

        # Creation fails after the terminal record enters the global list.
        # The error path must remove and destroy it without disturbing the server.
        invalid_env = dict(env, TERM="hmux-owner-no-clear")
        pid, master2 = pty.fork()
        if pid == 0:
            os.execve(binary, base + ["attach-session", "-t", "caps"], invalid_env)
        _, status = os.waitpid(pid, 0)
        pid = None
        os.close(master2)
        assert os.WIFEXITED(status) and os.WEXITSTATUS(status) != 0, status
        assert b"hmux-owner-no-clear" not in run("show-messages", "-T")
    finally:
        if pid is not None:
            os.kill(pid, signal.SIGTERM)
            os.waitpid(pid, 0)
        if master is not None:
            os.close(master)
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)

print("server term caps CLI checks passed")
