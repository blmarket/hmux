#!/usr/bin/env python3
"""Reject foreign event-loop declarations/linkage in application and artifacts."""
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
for path in [root / 'build.rs', root / 'Cargo.toml', root / 'Cargo.lock', *root.glob('src/**/*.rs')]:
    source = path.read_text()
    assert not re.search(r'libevent|event_core|rustc-link-lib[^\n]*event', source), path
for name in sys.argv[1:]:
    path = pathlib.Path(name)
    symbols = subprocess.run(['nm', '-u', str(path)], capture_output=True, text=True, check=True).stdout
    assert not re.search(r'\bU (?:evbuffer_|bufferevent_|event_(?:add|del|set|loop|init|once|active|pending|reinit|get_|base_|new|free))', symbols), path
    if path.suffix != '.a':
        dynamic = subprocess.run(['readelf', '-d', str(path)], capture_output=True, text=True, check=True).stdout
        linked = subprocess.run(['ldd', str(path)], capture_output=True, text=True, check=True).stdout
        assert 'libevent' not in dynamic + linked, path
print('Runtime source and supplied artifacts have no libevent dependency.')
