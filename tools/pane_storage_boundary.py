#!/usr/bin/env python3
"""Compile pane consumers with an implementation-private storage projection.

The disposable source copy substitutes a small UnsafeCell wrapper for pane cells.
This resolves inferred and unused get() calls, including generic storage helpers;
it neither changes production holders nor emulates RefCell borrow checking.
Combine it with field privacy, syntax checks and callback behavior tests.
"""
import argparse
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from model_boundary_inventory import mask

ROOT = Path(__file__).resolve().parents[1]


def private_storage_probe(sources):
    sources = dict(sources)
    for path, source in sources.items():
        namespace = 'hmux2' if path.startswith('tests/') else 'crate'
        code = mask(source)
        names = {'window_pane'}
        names.update(re.findall(r'\bwindow_pane\s+as\s+(\w+)', code))
        names.update(re.findall(r'\btype\s+(\w+)\s*=\s*window_pane\s*;', code))
        cell = re.compile(r'(?:(?:::)?(?:std|core)::cell::)?UnsafeCell\s*<\s*'
                          r'(?:(?:crate|hmux2)::src::(?:shared::pane|window_pane)::)?'
                          r'(?:' + '|'.join(sorted(names)) + r')\s*>')
        edits = list(cell.finditer(code))
        for match in reversed(edits):
            source = source[:match.start()] + namespace + '::src::window_pane::PaneStorage' + source[match.end():]
        sources[path] = source
    path = 'src/window_pane/mod.rs'
    export = 'pub use model::window_pane;'
    assert sources[path].count(export) == 1, 'Pane model export changed'
    sources[path] = sources[path].replace(export, 'pub use model::{window_pane, PaneStorage};')
    path = 'src/window_pane/model.rs'
    constructor = 'std::cell::UnsafeCell::new(self)'
    assert sources[path].count(constructor) == 1, 'Pane factory changed'
    assert sources[path].count('std::cell::UnsafeCell<Self>') == 2, 'Pane factory holder definitions changed'
    sources[path] = sources[path].replace(constructor, 'PaneStorage::new(self)')
    sources[path] = sources[path].replace('std::cell::UnsafeCell<Self>', 'PaneStorage')
    sources[path] += '''

// Probe only: storage access belongs exclusively to the pane implementation.
pub struct PaneStorage(std::cell::UnsafeCell<window_pane>);
impl PaneStorage {
    pub(super) fn new(value: window_pane) -> Self { Self(std::cell::UnsafeCell::new(value)) }
    pub(super) fn get(&self) -> *mut window_pane { self.0.get() }
}
'''
    return sources


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lib', action='store_true', help='check production library only')
    parser.add_argument('--diagnostics', type=Path, help='write Cargo JSON diagnostics')
    args = parser.parse_args()
    sources = {str(path.relative_to(ROOT)): path.read_text()
               for directory in ('src', 'tests') for path in (ROOT / directory).rglob('*.rs')}
    sources = private_storage_probe(sources)
    with tempfile.TemporaryDirectory(prefix='hmux-pane-storage-') as directory:
        probe = Path(directory)
        for name in ('src', 'tests', '.cargo', 'hmux-buffer', 'hmux-cmdparse', 'hmux-refbox', 'hmux-rt'):
            shutil.copytree(ROOT / name, probe / name,
                            ignore=shutil.ignore_patterns('target', '.git'))
        for name in ('Cargo.toml', 'Cargo.lock', 'build.rs'):
            shutil.copy2(ROOT / name, probe / name)
        for name, source in sources.items():
            (probe / name).write_text(source)
        command = ['cargo', 'check', '--offline', '--lib' if args.lib else '--all-targets',
                   '--target-dir', str(ROOT / 'target/pane-storage-boundary')]
        if args.diagnostics:
            with args.diagnostics.open('w') as output:
                return subprocess.run(command + ['--message-format=json'], cwd=probe,
                                      stdout=output).returncode
        return subprocess.run(command, cwd=probe).returncode


if __name__ == '__main__':
    raise SystemExit(main())
