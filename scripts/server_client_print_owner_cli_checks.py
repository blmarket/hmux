#!/usr/bin/env python3
"""Compare control-client display-message output with pinned tmux.

The command's -c target takes the server_client_print(parse=0) path. A raw
non-UTF-8 command argument also checks that escaping happens before control
output; argv cannot contain an interior NUL.
"""

import os
import pathlib
import select
import subprocess
import tempfile
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
ENV = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
MESSAGES = (
    b"owner-plain",
    b"owner\tline\nreturn\rbackspace\x08escape\x1bdelete\x7fhigh\xff",
    b"",
)


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="server-client-print-", dir=ROOT / "target") as tmp:
        base = [os.fsencode(binary), b"-S", os.fsencode(pathlib.Path(tmp) / "socket"), b"-f", b"/dev/null"]

        def run(*args):
            result = subprocess.run(base + list(args), env=ENV, capture_output=True, timeout=20)
            assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result.stdout

        client = None
        try:
            run(b"new-session", b"-d", b"-s", b"print-owner", b"sleep", b"30")
            client = subprocess.Popen(
                base + [b"-C", b"attach-session", b"-t", b"print-owner"],
                env=ENV, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
            )
            deadline = time.monotonic() + 5
            while not (clients := run(b"list-clients", b"-F", b"#{client_name}").splitlines()):
                assert client.poll() is None, "control client exited"
                assert time.monotonic() < deadline, "control client did not attach"
                time.sleep(0.02)
            assert len(clients) == 1, clients

            buffered = bytearray()

            def message_before(marker):
                deadline = time.monotonic() + 5
                while True:
                    end = buffered.find(marker)
                    if end >= 0:
                        start = buffered.find(b"%message ")
                        assert 0 <= start < end, (marker, buffered)
                        message = bytes(buffered[start:end])
                        del buffered[:end + len(marker)]
                        return message
                    remaining = deadline - time.monotonic()
                    assert remaining > 0, ("missing %message", buffered)
                    ready, _, _ = select.select([client.stdout], [], [], remaining)
                    assert ready, ("missing %message", buffered)
                    chunk = os.read(client.stdout.fileno(), 65536)
                    assert chunk, ("control client closed", client.poll(), buffered)
                    buffered.extend(chunk)

            output = []
            for index, message in enumerate(MESSAGES):
                marker = b"%message owner-end-" + str(index).encode() + b"\n"
                run(b"display-message", b"-c", clients[0], b"-l", message)
                run(b"display-message", b"-c", clients[0], b"-l", marker[9:-1])
                output.append(message_before(marker))
            assert output[0] == b"%message owner-plain\n", output
            assert output[2] == b"%message \n", output
            assert output[1] == (
                b"%message owner\tline\nreturn\\rbackspace\\b"
                b"escape\\033delete\\177high\\377\n"
            ), output
            return output
        finally:
            if client is not None:
                client.terminate()
                client.communicate(timeout=5)
            subprocess.run(base + [b"kill-server"], env=ENV, capture_output=True, timeout=20)


actual = trace(CANDIDATE)
expected = trace(BASELINE)
assert actual == expected, (actual, expected)
print("server client print owner CLI checks passed")
