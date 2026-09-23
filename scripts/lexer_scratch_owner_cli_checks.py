#!/usr/bin/env python3
"""Compare lexer success and error paths with the pinned binary."""

import os
import pathlib
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = os.environ.get("HMUX_BASELINE_BINARY")


def check(binary, directory):
    directory.mkdir()
    socket = directory / "socket"
    env = os.environ.copy()
    env.update(TERM="xterm-256color", TMUX="", SHELL="/bin/sh", LC_ALL="C.UTF-8")
    command = [str(binary), "-S", str(socket), "-f", "/dev/null"]

    def run(*args):
        return subprocess.run(
            command + list(args), cwd=directory, env=env, capture_output=True, timeout=20
        )

    def source(name, content):
        path = directory / name
        path.write_bytes(content)
        result = run("source-file", str(path))
        source_path = os.fsencode(path)
        return (
            result.returncode,
            result.stdout.replace(source_path, b"<source>"),
            result.stderr.replace(source_path, b"<source>"),
        )

    try:
        assert run("new-session", "-d", "-s", "lexer", "sleep 60").returncode == 0
        assert run("set-environment", "-g", "LEXER_WORD", "expanded").returncode == 0

        long_word = b"A" * 8192 + b"tail"
        success = source(
            "success.conf",
            b"%if #{==:#{==:1,1},1}\n"
            b"set -g @long " + long_word + b"\n"
            b"set -g @variable \"$LEXER_WORD\"\n"
            b"set -g @zero \"A\\000B\"\n"
            b"set -g @unicode \"\\u03bb\"\n"
            b"set -g @home ~/lexer\n"
            b"%else\nset -g @long wrong\n%endif\n",
        )
        assert success == (0, b"", b""), success

        outputs = []
        for name in ("@long", "@variable", "@zero", "@unicode", "@home"):
            result = run("show-options", "-gv", name)
            assert result.returncode == 0, (name, result)
            outputs.append(result.stdout)
        assert outputs == [
            long_word + b"\n",
            b"expanded\n",
            b"A\n",
            "λ\n".encode(),
            os.fsencode(os.path.expanduser("~/lexer")) + b"\n",
        ], outputs

        errors = [
            source("bad-octal.conf", b"set -g @bad \"\\400\"\n"),
            source("bad-format.conf", b"%if #{==:#{==:1,1},1\n"),
            source("bad-variable.conf", b"set -g @bad \"${LEXER_WORD\"\n"),
        ]
        assert all(code != 0 for code, _, _ in errors), errors
        return outputs, errors
    finally:
        run("kill-server")


with tempfile.TemporaryDirectory(prefix="lexer-scratch-owner-") as tmp:
    directory = pathlib.Path(tmp)
    output = check(candidate, directory / "candidate")
    if baseline is not None:
        reference = check(pathlib.Path(baseline).resolve(), directory / "baseline")
        assert reference == output, (reference[1], output[1])

print("lexer scratch owner CLI checks passed")
