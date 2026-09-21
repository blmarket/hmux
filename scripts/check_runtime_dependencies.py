#!/usr/bin/env python3
"""Check a built hmux binary/staticlib for any remaining libevent dependency."""
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
profile = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else root / "target/debug"
legacy = re.compile(
    r"^(?:evbuffer_.*|bufferevent_.*|event_(?:active|add|del|get_method|get_version|"
    r"init|initialized|loop|once|pending|reinit|set|set_log_callback))$"
)
for artifact in (profile / "hmux2", profile / "libhmux2.a"):
    result = subprocess.run(["nm", "-u", str(artifact)], capture_output=True, text=True, check=True)
    unresolved = {line.split()[-1].split("@")[0] for line in result.stdout.splitlines() if line.split()}
    remaining = sorted(name for name in unresolved if legacy.fullmatch(name))
    if remaining:
        sys.exit(f"{artifact}: unresolved libevent symbols: {remaining}")
linked = subprocess.check_output(["ldd", str(profile / "hmux2")], text=True)
if "libevent" in linked:
    sys.exit("Binary still links libevent")
print("Binary and staticlib have no libevent dependency.")
