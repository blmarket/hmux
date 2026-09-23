#!/usr/bin/env python3
"""Compare complete format output assembly with the pinned binary."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = os.fsencode(pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve())
baseline = os.environ.get("HMUX_BASELINE_BINARY")
env = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh", TZ="UTC")


def check(executable, directory):
    base = [executable, b"-S", os.fsencode(directory) + b"/socket", b"-f", b"/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=20)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    try:
        run(b"new-session", b"-d", b"-s", b"owner", b"sleep", b"60")
        run(b"set-option", b"-g", b"@bytes", b"A \xff#B")
        run(b"set-option", b"-g", b"@nested", b"#{R:xy,3}")
        run(b"set-option", b"-g", b"@path", b"/tmp/\xff item")
        run(b"set-option", b"-g", b"@timestamp", b"1")
        run(b"set-option", b"-g", b"@date_format", b"%Y")
        run(b"set-option", b"-g", b"@zero", b"0")
        cases = (
            (b"literal", b"literal\n"),
            (b"#{E:@nested}", b"xyxyxy\n"),
            (b"#{R:\xff,3}", b"\xff\xff\xff\n"),
            (b"#{p8;E:@nested}", b"xyxyxy  \n"),
            (b"#{p-8;E:@nested}", b"  xyxyxy\n"),
            (b"#{=|3|..;E:@nested}", b"xyx..\n"),
            (b"#{=|-3|..;E:@nested}", b"..yxy\n"),
            (b"#{s|x|Z|;E:@nested}", b"ZyZyZy\n"),
            (b"#{s|[|Z|;E:@nested}", b"xyxyxy\n"),
            (b"#{n;E:@nested}:#{w;E:@nested}", b"6:6\n"),
            (b"#{b:@path}", b"\xff item\n"),
            (b"#{d:@path}", b"/tmp\n"),
            (b"#{q;b:@path}", b"\xff\\ item\n"),
            (b"#{t/f/#{@date_format}/:@timestamp}", b"1970\n"),
            (b"#{t:@zero}", b"\n"),
            (b"#{?@zero,yes,no}:#{?@absent,yes,no}", b"no:no\n"),
            (b"#{m/p:ac,abc}", b"0,2\n"),
            (b"#{m/z:ac,abc}:#{m/r:[,abc}", b"1:0\n"),
            (b"#{e|/|f|3:1,8}", b"0.125\n"),
            (b"#{e|/|f|-1:1,8}", b"0.125000\n"),
            (b"#{e|/|f:1,0}", None),
            (b"#{e|/|f:0,0}", None),
            (b"x" * 300 + b"#", b"x" * 300 + b"\n"),
            (b"L#{@bytes}R", b"LA \xff#BR\n"),
            (b"#{q:@bytes}", b"A\\ \xff\\#B\n"),
            (b"#{?#{==:#{session_name},owner},yes,no}", b"yes\n"),
            (b"#[fg=red]red#[default]", None),
            (b"##[fg=red]red", None),
            (b"#z #, ## #}", None),
            (b"\xff:#{@bytes}", b"\xff:A \xff#B\n"),
        )
        output = []
        for expression, expected in cases:
            actual = run(b"display-message", b"-p", expression)
            if expected is not None:
                assert actual == expected, (expression, actual, expected)
            output.append(actual)
        # Enumeration and direct lookup consume the same built-in result contract.
        run(b"set-buffer", b"A\xff\tB")
        listed = dict(line.split(b"=", 1) for line in run(b"display-message", b"-a").splitlines() if b"=" in line)
        stable_keys = (
            b"buffer_name", b"buffer_size", b"buffer_sample", b"buffer_full",
            b"session_name", b"session_windows", b"session_id", b"window_id",
            b"pane_id", b"pane_width", b"pane_height", b"pane_left", b"pane_top",
            b"window_width", b"window_height", b"alternate_on", b"pane_dead",
            b"pane_in_mode", b"cursor_x", b"cursor_y", b"pane_tabs",
        )
        for key in stable_keys + (b"pid", b"session_created", b"start_time", b"buffer_created"):
            expanded = run(b"display-message", b"-p", b"#{" + key + b"}").rstrip(b"\n")
            assert listed[key] == expanded, (key, listed.get(key), expanded)
            if key in stable_keys:
                output.append(expanded)
        # Exercise buffer escaping and the 200-byte preview cap on both paths.
        run(b"set-buffer", b"x" * 220)
        assert run(b"display-message", b"-p", b"#{buffer_sample}") == b"x" * 200 + b"...\n"
        return tuple(output)
    finally:
        subprocess.run(base + [b"kill-server"], env=env, capture_output=True, timeout=20)


with tempfile.TemporaryDirectory(prefix="format-expand-output-owner-") as tmp:
    candidate_dir = pathlib.Path(tmp, "candidate")
    candidate_dir.mkdir()
    candidate = check(binary, candidate_dir)
    if baseline is not None:
        baseline_dir = pathlib.Path(tmp, "baseline")
        baseline_dir.mkdir()
        reference = check(os.fsencode(pathlib.Path(baseline).resolve()), baseline_dir)
        assert candidate == reference, (candidate, reference)

print("format expand output owner CLI checks passed")
