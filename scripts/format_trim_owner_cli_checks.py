#!/usr/bin/env python3
"""Compare format trim modifiers with the pinned pre-migration binary."""

import os
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CURRENT = Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2"))
BASELINE = Path(
    os.environ.get(
        "HMUX_BASELINE_BINARY", "/tmp/hmux2-migration-baseline/target/debug/hmux2"
    )
)


def sample(binary: Path) -> list[bytes]:
    with tempfile.TemporaryDirectory(prefix="hmux-trim-") as tmp:
        env = dict(os.environ, TMUX_TMPDIR=tmp)
        command = [os.fsencode(binary), b"-L", b"trim-owner", b"-f", b"/dev/null"]

        def run(*args: bytes) -> bytes:
            result = subprocess.run(
                command + list(args), env=env, capture_output=True, timeout=15, check=True
            )
            return result.stdout

        output = []
        try:
            run(b"new-session", b"-d", b"-s", b"trim")
            for value in (b"abcdef", "\u00e9abcdef".encode(), b"abc###def", b"\xffabcdef"):
                run(b"set-option", b"-g", b"@value", value)
                for limit in (b"1", b"3", b"-3", b"-1"):
                    output.append(run(b"display-message", b"-p", b"#{=" + limit + b":#{@value}}"))
        finally:
            subprocess.run(command + [b"kill-server"], env=env, capture_output=True, timeout=15)
        return output


if __name__ == "__main__":
    current = sample(CURRENT)
    baseline = sample(BASELINE)
    assert current == baseline, (current, baseline)
    assert current[:4] == [b"a\n", b"abc\n", b"def\n", b"f\n"]
    print("format trim CLI bytes match pinned baseline")
