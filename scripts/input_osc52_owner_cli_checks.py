#!/usr/bin/env python3
"""Compare application OSC 52 clipboard writes with the pinned tmux baseline."""

import base64
import fcntl
import os
import pathlib
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANDIDATE = pathlib.Path(os.environ.get("HMUX_BINARY", ROOT / "target/debug/hmux2")).resolve()
BASELINE = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()
ENV = dict(os.environ, TERM="xterm-256color", LC_ALL="C", TMUX="", SHELL="/bin/sh")
PAYLOAD = b"A\0B\xff"
CASES = (
    ("binary", base64.b64encode(PAYLOAD), PAYLOAD, 2),
    ("invalid", b"!!!", b"prior", 1),
    ("empty", b"", b"prior", 1),
)


def setup(binary, tmp):
    base = [str(binary), "-S", str(pathlib.Path(tmp) / "socket"), "-f", "/dev/null"]

    def command(*args):
        return subprocess.run(base + list(args), env=ENV, capture_output=True, timeout=20)

    def run(*args):
        result = command(*args)
        assert result.returncode == 0, (args, result.returncode, result.stderr)
        return result.stdout

    run("new-session", "-d", "-s", "clip", "sleep 30")
    run("set-option", "-s", "set-clipboard", "on")
    run("set-buffer", "--", "prior")
    return base, run


def buffer_state(run):
    names = run("list-buffers", "-F", "#{buffer_name}").splitlines()
    return run("show-buffer"), len(names)


def pane_trace(binary, case, encoded, expected, expected_count):
    with tempfile.TemporaryDirectory(prefix=f"input-osc52-pane-{case}-", dir=ROOT / "target") as tmp:
        base, run = setup(binary, tmp)
        try:
            # Pane output, rather than client input, reaches input_osc_52.
            # The visible marker proves that bytes after the OSC were parsed.
            shell = "printf '\\033]52;c;" + encoded.decode("ascii") + "\\007OSC52_DONE\\n'; sleep 10"
            run("new-window", "-d", "-t", "clip:", "-n", "emit", shell)
            deadline = time.monotonic() + 5
            while b"OSC52_DONE" not in run("capture-pane", "-p", "-t", "clip:emit"):
                assert time.monotonic() < deadline, (case, "pane output marker missing")
                time.sleep(0.02)
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                state = buffer_state(run)
                if state == (expected, expected_count):
                    break
                time.sleep(0.02)
            else:
                raise AssertionError((case, state, (expected, expected_count)))
            time.sleep(0.2)
            state = buffer_state(run)
            assert state == (expected, expected_count), (case, state)
            return state
        finally:
            subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=5)


def popup_trace(binary):
    with tempfile.TemporaryDirectory(prefix="input-osc52-popup-", dir=ROOT / "target") as tmp:
        base, run = setup(binary, tmp)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        attached = None
        popup = None
        try:
            attached = subprocess.Popen(
                base + ["attach-session", "-t", "clip"], env=ENV,
                stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 5
            while not (tty := run("list-clients", "-F", "#{client_tty}").strip()):
                assert attached.poll() is None, "attached client exited"
                assert time.monotonic() < deadline, "attached client missing"
                time.sleep(0.02)

            shell = "printf '\\033]52;c;" + base64.b64encode(PAYLOAD).decode() + "\\007POPUP_DONE\\n'; sleep 1"
            popup = subprocess.Popen(
                base + ["display-popup", "-c", os.fsdecode(tty), "-E", shell],
                env=ENV, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            output = bytearray()
            deadline = time.monotonic() + 5
            while b"POPUP_DONE" not in output and time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    output.extend(os.read(master, 65536))
            assert b"POPUP_DONE" in output, ("popup output marker missing", output[-300:])

            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                state = buffer_state(run)
                if state == (PAYLOAD, 2):
                    break
                time.sleep(0.02)
            else:
                raise AssertionError(("popup", state, (PAYLOAD, 2)))
            popup.communicate(timeout=5)
            assert popup.returncode == 0, popup.returncode
            return state
        finally:
            if popup is not None and popup.poll() is None:
                popup.terminate()
                popup.communicate(timeout=5)
            if attached is not None:
                if attached.poll() is None:
                    attached.terminate()
                attached.wait(timeout=5)
            if slave is not None:
                os.close(slave)
            os.close(master)
            subprocess.run(base + ["kill-server"], env=ENV, capture_output=True, timeout=5)


for case_args in CASES:
    reference = pane_trace(BASELINE, *case_args)
    observed = pane_trace(CANDIDATE, *case_args)
    assert observed == reference, (case_args[0], observed, reference)

reference = popup_trace(BASELINE)
observed = popup_trace(CANDIDATE)
assert observed == reference, ("popup", observed, reference)
print("input OSC 52 owner CLI checks passed")
