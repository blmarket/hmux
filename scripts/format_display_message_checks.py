#!/usr/bin/env python3
"""Run deterministic display-message probes against one isolated server.

Each command consumes its own control-mode response. A refresh command gives
queued job-completion work another turn before reading the cached result.  Keeping one control client open is important for
job formats: it exercises the existing per-client cache lifetime and lets the
job-completion event run before the cached result is read again.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import shlex
import subprocess
import tempfile
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = ROOT / "target" / "format-validation" / "display-message-checks.json"
ENVIRONMENT = {
    "TERM": "xterm-256color",
    "LC_ALL": "C",
    "TMUX": "",
    "SHELL": "/bin/sh",
}


class ControlClient:
    def __init__(self, binary: pathlib.Path, socket: pathlib.Path) -> None:
        environment = dict(os.environ, **ENVIRONMENT)
        self.process = subprocess.Popen(
            [str(binary), "-C", "-S", str(socket), "-f", "/dev/null"],
            cwd=ROOT,
            env=environment,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        # Consume startup's frame so later responses correspond to the command
        # just sent, rather than staying one command behind.
        self.response("startup")

    def command(self, command: str) -> list[str]:
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        self.process.stdin.write(command + "\n")
        self.process.stdin.flush()
        return self.response(command)

    def response(self, command: str) -> list[str]:
        assert self.process.stdout is not None
        payload: list[str] = []
        in_response = False
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError(
                    f"control client exited while running {command!r}"
                )
            line = line.rstrip("\n")
            if line.startswith("%begin "):
                in_response = True
                payload = []
            elif line.startswith("%end "):
                if in_response:
                    return payload
            elif line.startswith("%error "):
                raise RuntimeError(f"{command!r}: {line}")
            elif in_response:
                payload.append(line)

    def close(self) -> None:
        if self.process.poll() is not None:
            return
        try:
            self.command("kill-server")
        except (BrokenPipeError, RuntimeError):
            self.process.terminate()
        self.process.wait(timeout=10)


def display_command(format_string: str) -> str:
    return "display-message -p -t format-probe " + shlex.quote(format_string)


def display(client: ControlClient, format_string: str, wait: float = 0.0) -> dict:
    command = display_command(format_string)
    initial_payload = client.command(command)
    if wait:
        time.sleep(wait)
    barrier_payload = client.command("refresh-client")
    return {
        "command": command,
        "initial_response": initial_payload,
        "barrier_command": "refresh-client",
        "barrier_response": barrier_payload,
    }


def check_case(
    name: str,
    format_string: str,
    expected: str,
    client: ControlClient,
) -> dict:
    observation = display(client, format_string)
    actual = "\n".join(observation["initial_response"])
    observation.update(
        {
            "name": name,
            "input": format_string,
            "expected": expected,
            "actual": actual,
            "pass": actual == expected,
        }
    )
    return observation


def check_job(
    name: str,
    format_string: str,
    expected: str,
    client: ControlClient,
    pending: str,
) -> dict:
    first = display(client, format_string, wait=0.35)
    first_actual = "\n".join(first["initial_response"])
    attempts = [first]
    eventual = None
    for _ in range(8):
        attempt = display(client, format_string, wait=0.35)
        attempts.append(attempt)
        actual = "\n".join(attempt["initial_response"])
        if actual == expected:
            eventual = actual
            break
    if eventual is None:
        eventual = "\n".join(attempts[-1]["initial_response"])
    repeat = display(client, format_string)
    repeat_actual = "\n".join(repeat["initial_response"])
    return {
        "name": name,
        "input": format_string,
        "expected_initial": [pending, expected],
        "actual_initial": first_actual,
        "expected_eventual": expected,
        "actual_eventual": eventual,
        "expected_cached_repeat": expected,
        "actual_cached_repeat": repeat_actual,
        "attempts": attempts,
        "cached_repeat": repeat,
        "pass": (
            first_actual in (pending, expected)
            and eventual == expected
            and repeat_actual == expected
        ),
    }


def run(binary: pathlib.Path) -> dict:
    with tempfile.TemporaryDirectory(prefix="format-display-") as directory:
        socket = pathlib.Path(directory) / "socket"
        client = ControlClient(binary, socket)
        try:
            setup = client.command(
                "new-session -d -s format-probe -x 80 -y 24 sleep 60"
            )
            cases = [
                check_case(
                    "nested-expression",
                    "#{?#{==:#{session_name},format-probe},nested-ok,nested-bad}",
                    "nested-ok",
                    client,
                ),
                check_case(
                    "true-conditional",
                    "#{?#{==:1,1},true-branch,false-branch}",
                    "true-branch",
                    client,
                ),
                check_case(
                    "false-conditional",
                    "#{?#{==:1,2},true-branch,false-branch}",
                    "false-branch",
                    client,
                ),
                check_case(
                    "escaping",
                    "escaped:##{session_name}",
                    "escaped:#{session_name}",
                    client,
                ),
                check_job(
                    "job-format",
                    "job:#(sleep 0.2; printf job-value)",
                    "job:job-value",
                    client,
                    "job:",
                ),
                check_job(
                    "recursive-job-format",
                    "recursive:#(printf #{session_name})",
                    "recursive:format-probe",
                    client,
                    "recursive:",
                ),
            ]
        finally:
            client.close()
    result = {
        "schema": 1,
        "binary": str(binary),
        "environment": ENVIRONMENT,
        "setup_command": "new-session -d -s format-probe -x 80 -y 24 sleep 60",
        "setup_response": setup,
        "cases": cases,
    }
    result["pass"] = all(case["pass"] for case in cases)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument(
        "--binary",
        type=pathlib.Path,
        default=pathlib.Path(
            os.environ.get("HMUX_BINARY", str(ROOT / "target" / "debug" / "hmux2"))
        ),
    )
    args = parser.parse_args()
    result = run(args.binary.resolve())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
