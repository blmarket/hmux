#!/usr/bin/env python3
"""Compare escaped OSC 8 URI and internal ID output against the pinned baseline."""

import os
import pathlib
import subprocess
import tempfile
import time


root = pathlib.Path(__file__).resolve().parents[1]
current = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2"))
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"])
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh", LC_ALL="C")


def capture(binary):
    with tempfile.TemporaryDirectory(prefix="hyperlink-fields-") as tmp:
        base = [str(binary), "-f", "/dev/null", "-S", str(pathlib.Path(tmp) / "socket")]

        def run(*args):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            assert result.returncode == 0, (binary, args, result.stderr)
            return result.stdout

        # Both retained fields contain invalid UTF-8, forcing the VIS_OCTAL path.
        script = (
            "printf '\\033]8;id=alpha\\377;https://a.example/\\377\\007"
            "A\\033]8;;\\007\\n'; sleep 20"
        )
        try:
            run("new-session", "-d", "-s", "hyperlink-fields", script)
            deadline = time.monotonic() + 5
            while True:
                plain = run("capture-pane", "-p", "-S", "0")
                if plain.startswith(b"A\n"):
                    break
                assert time.monotonic() < deadline, plain
                time.sleep(0.05)
            escaped = run("capture-pane", "-p", "-e", "-S", "0").split(b"\n", 1)[0]
            uris = run("capture-pane", "-p", "-H", "-S", "0")
            return escaped, uris
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


expected = (
    b"\x1b]8;id=alpha\\377;https://a.example/\\377\x1b\\A\x1b]8;;\x1b\\",
    b"https://a.example/\\377\n",
)
assert capture(baseline) == expected
assert capture(current) == expected
print("hyperlink fields owner CLI checks passed")
