#!/usr/bin/env python3
"""Check pane login records through a private utempter shim and server."""

import os
import pathlib
import re
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")
source = r"""
#include <stdio.h>
#include <stdlib.h>

int utempter_add_record(int fd, const char *hostname) {
    (void)fd;
    const char *path = getenv("HMUX_UTEMPTER_RECORD");
    if (path == NULL) return -1;
    FILE *out = fopen(path, "a");
    if (out == NULL) return -1;
    fprintf(out, "%s\n", hostname);
    fclose(out);
    return 0;
}

int utempter_remove_record(int fd) {
    (void)fd;
    return 0;
}
"""


def check(binary_path, tmp, label, shim):
    record = tmp / f"{label}.records"
    socket = tmp / f"{label}.socket"
    env = dict(
        os.environ,
        TERM="xterm-256color",
        LC_ALL="C",
        TMUX="",
        SHELL="/bin/sh",
        LD_PRELOAD=str(shim),
        HMUX_UTEMPTER_RECORD=str(record),
    )
    base = [str(binary_path), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    try:
        run("new-session", "-d", "-s", "record", "-x", "80", "-y", "24", "sleep 30")
        run("split-window", "-d", "-t", "record:0", "sleep 30")
        pane_ids = set(run("list-panes", "-t", "record:0", "-F", "#{pane_id}").splitlines())
        lines = record.read_bytes().splitlines()
        assert len(lines) == 2, lines
        parsed = []
        for line in lines:
            match = re.fullmatch(rb"tmux\(([0-9]+)\)\.(%[0-9]+)", line)
            assert match is not None, line
            assert match.group(2) in pane_ids, (line, pane_ids)
            parsed.append(match.group(2))
        assert set(parsed) == pane_ids, (parsed, pane_ids)
        return sorted(parsed)
    finally:
        subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)


with tempfile.TemporaryDirectory(prefix="spawn-utempter-", dir=root / "target") as directory:
    tmp = pathlib.Path(directory)
    c_file = tmp / "utempter.c"
    shim = tmp / "utempter.so"
    c_file.write_text(source)
    subprocess.run(
        ["cc", "-shared", "-fPIC", "-o", str(shim), str(c_file)],
        check=True,
        capture_output=True,
        timeout=10,
    )
    actual = check(binary, tmp, "candidate", shim)
    if baseline is not None:
        expected = check(pathlib.Path(baseline).resolve(), tmp, "baseline", shim)
        assert actual == expected, (actual, expected)

print("spawn utempter record CLI checks passed")
