#!/usr/bin/env python3
"""Type-check Client consumers with an implementation-private storage projection.

Only a disposable copy changes: ClientRef/ClientWeak use a tiny UnsafeCell wrapper
whose get() is private to server_client. This detects even unused/inferred .get(),
whole-model references forwarded to generic helpers, and storage-specific helper
arguments. It does not convert production storage or emulate RefCell borrowing.
Run the syntax/behavior tests as well to check casts and callback lifetimes.
"""
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def private_storage_probe(sources):
    sources = dict(sources)
    path = 'src/shared/client.rs'
    assert sources[path].count('UnsafeCell<client>') == 2, 'Client holder definitions changed'
    sources[path] = sources[path].replace(
        'UnsafeCell<client>', 'crate::src::server_client::ClientStorage')
    path = 'src/server_client/mod.rs'
    constructor = 'std::cell::UnsafeCell::new(value)'
    assert sources[path].count(constructor) == 1, 'Client factory changed'
    sources[path] = sources[path].replace(constructor, 'ClientStorage::new(value)')
    sources[path] = sources[path].replace('pub use model::client;',
                                        'pub use model::{client, ClientStorage};')
    sources['src/server_client/model.rs'] += '''

// Probe only: storage access belongs exclusively to the Client implementation.
pub struct ClientStorage(std::cell::UnsafeCell<client>);
impl ClientStorage {
    pub(super) fn new(value: client) -> Self { Self(std::cell::UnsafeCell::new(value)) }
    pub(super) fn get(&self) -> *mut client { self.0.get() }
}
'''
    return sources


def main():
    names = ['src/shared/client.rs', 'src/server_client/mod.rs', 'src/server_client/model.rs']
    sources = private_storage_probe({name: (ROOT / name).read_text() for name in names})
    with tempfile.TemporaryDirectory(prefix='hmux-client-storage-') as directory:
        probe = Path(directory)
        for name in ('src', 'tests', '.cargo', 'hmux-buffer', 'hmux-cmdparse', 'hmux-refbox', 'hmux-rt'):
            shutil.copytree(ROOT / name, probe / name,
                            ignore=shutil.ignore_patterns('target', '.git'))
        for name in ('Cargo.toml', 'Cargo.lock', 'build.rs'):
            shutil.copy2(ROOT / name, probe / name)
        for name, source in sources.items():
            (probe / name).write_text(source)
        return subprocess.run(
            ['cargo', 'check', '--offline', '--all-targets',
             '--target-dir', str(ROOT / 'target/client-storage-boundary')],
            cwd=probe).returncode


if __name__ == '__main__':
    raise SystemExit(main())
