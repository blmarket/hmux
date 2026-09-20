#!/usr/bin/env python3
"""Style text checks using an isolated server, without linking a Rust test binary."""
import os
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
work = root / "target/style-cli"
work.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(dir=work) as tmp:
    env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def run(*args):
        return subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)

    try:
        p = run("new-session", "-d", "-s", "style", "sleep 60")
        assert p.returncode == 0, p.stderr
        for value in [
            "bold,dim", "fg=#112233,bg=colour123,us=#aabbcc,bold,dim",
            "fg=COLOR255,bg=grey100", "fg=colour+1,bg=colour-0",
            "fg=themeblack,bg=terminal",
        ]:
            p = run("set-option", "-g", "status-style", value)
            assert p.returncode == 0, (value, p.stderr)
            p = run("show-options", "-gv", "status-style")
            assert p.returncode == 0 and p.stdout.rstrip(b"\n") == value.encode(), (value, p.stdout, p.stderr)
        for value in ["fg=colour256", "fg=colour-1", "fg=8", "fg=#fffffg",
                      "fg=grey101", "fg=colour9223372036854775808", b"fg=\xff"]:
            p = run("set-option", "-g", "status-style", value)
            assert p.returncode != 0, value
        p = run("set-option", "-g", "window-style", "fg=#112233,bg=colour123")
        assert p.returncode == 0, p.stderr
        p = run("display-message", "-p", "#{pane_fg}:#{pane_bg}")
        assert p.stdout == b"#112233:colour123\n", p.stdout
        p = run("set-window-option", "-g", "clock-mode-colour", "COLOR255")
        assert p.returncode == 0, p.stderr
        p = run("show-window-options", "-gv", "clock-mode-colour")
        assert p.stdout == b"COLOR255\n", p.stdout
        for expression, expected in [
            ("#{c:color255}", b"eeeeee\n"),
            ("#{c:colour256}", b"\n"),
        ]:
            p = run("display-message", "-p", expression)
            assert p.returncode == 0 and p.stdout == expected, (expression, p.stdout, p.stderr)
    finally:
        run("kill-server")
print("style CLI checks passed")
