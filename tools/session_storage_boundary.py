#!/usr/bin/env python3
"""Compile Session consumers with an implementation-private storage projection.

Only the disposable source copy changes. The wrapper still contains UnsafeCell;
this checks ownership of storage access, not RefCell borrow correctness. Combine
it with model_trait_boundary and component/callback behavior tests.
"""
import argparse
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def private_storage_probe(sources):
    sources = dict(sources)
    path = 'src/shared/session.rs'
    holder = 'std::cell::UnsafeCell<session>'
    assert sources[path].count(holder) == 2, 'Session holder definitions changed'
    sources[path] = sources[path].replace(holder, 'crate::src::session::SessionStorage')
    path = 'src/session/mod.rs'
    export = re.compile(
        r'(?m)^pub\s+use\s+model::(?:session|\{\s*session\s*,\s*sessions\s*,?\s*\})\s*;')
    exports = list(export.finditer(sources[path]))
    assert len(exports) == 1, 'Session model export changed'
    # Preserve the registry-head export and add only the probe wrapper export.
    offset = exports[0].end()
    sources[path] = (sources[path][:offset] + '\npub use model::SessionStorage;'
                     + sources[path][offset:])
    path = 'src/session/model.rs'
    constructor = 'std::cell::UnsafeCell::new(value)'
    assert sources[path].count(constructor) == 1, 'Session factory changed'
    sources[path] = sources[path].replace(constructor, 'SessionStorage::new(value)')
    sources[path] += '''

// Probe only: storage access belongs exclusively to the Session implementation.
pub struct SessionStorage(std::cell::UnsafeCell<session>);
impl SessionStorage {
    pub(super) fn new(value: session) -> Self { Self(std::cell::UnsafeCell::new(value)) }
    pub(super) fn get(&self) -> *mut session { self.0.get() }
}
'''
    return sources


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lib', action='store_true', help='check production library only')
    parser.add_argument('--diagnostics', type=Path, help='write Cargo JSON diagnostics')
    args = parser.parse_args()
    names = ['src/shared/session.rs', 'src/session/mod.rs', 'src/session/model.rs']
    sources = private_storage_probe({name: (ROOT / name).read_text() for name in names})
    with tempfile.TemporaryDirectory(prefix='hmux-session-storage-') as directory:
        probe = Path(directory)
        for name in ('src', 'tests', '.cargo', 'hmux-buffer', 'hmux-cmdparse', 'hmux-refbox', 'hmux-rt'):
            shutil.copytree(ROOT / name, probe / name,
                            ignore=shutil.ignore_patterns('target', '.git'))
        for name in ('Cargo.toml', 'Cargo.lock', 'build.rs'):
            shutil.copy2(ROOT / name, probe / name)
        for name, source in sources.items():
            (probe / name).write_text(source)
        command = ['cargo', 'check', '--offline', '--lib' if args.lib else '--all-targets',
                   '--target-dir', str(ROOT / 'target/session-storage-boundary')]
        if args.diagnostics:
            with args.diagnostics.open('w') as output:
                return subprocess.run(command + ['--message-format=json'], cwd=probe,
                                      stdout=output).returncode
        return subprocess.run(command, cwd=probe).returncode


if __name__ == '__main__':
    raise SystemExit(main())
