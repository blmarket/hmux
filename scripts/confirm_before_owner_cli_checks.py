#!/usr/bin/env python3
"""Compare confirmation prompt acceptance, rejection, replacement, and errors."""

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


root = pathlib.Path(__file__).resolve().parents[1]
candidate = pathlib.Path(os.environ.get("HMUX_BINARY", root / "target/debug/hmux2")).resolve()
baseline = pathlib.Path(os.environ["HMUX_BASELINE_BINARY"]).resolve()


def trace(binary):
    with tempfile.TemporaryDirectory(prefix="confirm-before-owner-") as tmp:
        socket = pathlib.Path(tmp) / "socket"
        env = dict(os.environ, TERM="xterm-256color", LC_ALL="C.UTF-8", TMUX="", SHELL="/bin/sh")
        base = [str(binary), "-S", str(socket), "-f", "/dev/null"]
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        client = None

        def run(*args, ok=True):
            result = subprocess.run(base + list(args), env=env, capture_output=True, timeout=10)
            if ok:
                assert result.returncode == 0, (args, result.returncode, result.stderr)
            return result

        def prompt(label):
            output = bytearray()
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
                if ready:
                    output.extend(os.read(master, 65536))
                    if label in output:
                        return
            raise AssertionError(f"prompt {label!r} not displayed: {output[-400:]!r}")

        def option(name):
            result = run("show-options", "-gqv", name, ok=False)
            return result.returncode, result.stdout

        try:
            run("new-session", "-d", "-s", "confirm-owner", "sleep 30")
            client = subprocess.Popen(
                base + ["attach-session", "-t", "confirm-owner"],
                env=env, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            )
            os.close(slave)
            slave = None
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline:
                tty = run("list-clients", "-F", "#{client_tty}").stdout.strip().decode()
                if tty:
                    break
                time.sleep(0.05)
            else:
                raise AssertionError("attached client did not appear")

            accepted = subprocess.Popen(
                base + ["confirm-before", "-t", tty, "-p", "Accept owner?",
                        "set-option -g @accepted yes"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            prompt(b"Accept owner?")
            os.write(master, b"y")
            _, accepted_error = accepted.communicate(timeout=6)
            assert accepted.returncode == 0, accepted_error
            accepted_value = option("@accepted")

            rejected = subprocess.Popen(
                base + ["confirm-before", "-t", tty, "-p", "Reject owner?",
                        "set-option -g @rejected yes"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            prompt(b"Reject owner?")
            os.write(master, b"n")
            _, rejected_error = rejected.communicate(timeout=6)
            rejected_value = option("@rejected")

            default_yes = subprocess.Popen(
                base + ["confirm-before", "-t", tty, "-c", "k", "-y",
                        "set-option -g @default_yes yes"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            prompt(b"Confirm 'set-option'? (k/n)")
            os.write(master, b"\r")
            _, default_yes_error = default_yes.communicate(timeout=6)
            assert default_yes.returncode == 0, default_yes_error
            default_yes_value = option("@default_yes")

            run("confirm-before", "-b", "-t", tty, "-p", "Replace owner?",
                "set-option -g @replaced yes")
            prompt(b"Replace owner?")
            run("confirm-before", "-b", "-t", tty, "-p", "Replacement owner?",
                "set-option -g @replacement yes")
            prompt(b"Replacement owner?")
            os.write(master, b"y")
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline and option("@replacement")[1] != b"yes\n":
                time.sleep(0.05)
            replaced_value = option("@replaced")
            replacement_value = option("@replacement")

            invalid = run("confirm-before", "-t", tty, "-c", "xy",
                          "set-option -g @invalid yes", ok=False)
            result = (
                accepted_value, rejected.returncode, rejected_error, rejected_value,
                default_yes_value,
                replaced_value, replacement_value, invalid.returncode, invalid.stderr,
            )
            assert accepted_value == (0, b"yes\n"), result
            assert rejected_value[1] == b"", result
            assert default_yes_value == (0, b"yes\n"), result
            assert replaced_value[1] == b"", result
            assert replacement_value == (0, b"yes\n"), result
            assert invalid.returncode != 0 and b"invalid confirm key" in invalid.stderr, result
            return result
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
            if client is not None:
                client.wait(timeout=6)
            if slave is not None:
                os.close(slave)
            os.close(master)


expected = trace(baseline)
actual = trace(candidate)
assert actual == expected, (actual, expected)
print("confirm-before owner CLI checks passed")
