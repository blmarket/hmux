#!/usr/bin/env python3
"""Compare idle CPU and binary file round trips for supplied hmux binaries.

Example: python3 scripts/runtime_benchmark.py /path/to/baseline target/release/hmux2
This small local benchmark is evidence, not a throughput or latency guarantee.
"""
import argparse
import json
import os
import pathlib
import statistics
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binaries", nargs="+", type=pathlib.Path)
parser.add_argument("--samples", type=int, default=9)
args = parser.parse_args()
env = dict(os.environ, TERM="xterm-256color", TMUX="", SHELL="/bin/sh")
results = []
for binary in args.binaries:
    binary = binary.resolve()
    with tempfile.TemporaryDirectory(prefix="hmux-performance-") as directory:
        work = pathlib.Path(directory)
        base = [str(binary), "-S", str(work / "socket"), "-f", "/dev/null"]

        def run(*command):
            return subprocess.check_output(base + list(command), env=env, stderr=subprocess.PIPE, timeout=15)

        try:
            run("new-session", "-d", "-s", "perf", "sleep 60")
            pid = int(run("display-message", "-p", "#{pid}"))

            def cpu_ticks():
                fields = pathlib.Path(f"/proc/{pid}/stat").read_text().split(") ", 1)[1].split()
                return int(fields[11]) + int(fields[12])

            time.sleep(.2)
            before = cpu_ticks()
            time.sleep(1)
            idle = cpu_ticks() - before
            payload = bytes(range(256)) * 16384
            (work / "input").write_bytes(payload)
            samples = []
            for _ in range(args.samples):
                started = time.monotonic()
                run("load-buffer", "-b", "data", str(work / "input"))
                run("save-buffer", "-b", "data", str(work / "output"))
                samples.append(time.monotonic() - started)
                assert (work / "output").read_bytes() == payload
            results.append(dict(binary=str(binary), idle_cpu_ticks_per_second=idle,
                                transfer_bytes=len(payload), samples=samples,
                                roundtrip_median_seconds=statistics.median(samples)))
        finally:
            subprocess.run(base + ["kill-server"], env=env, capture_output=True, timeout=10)
print(json.dumps(results, indent=2))
