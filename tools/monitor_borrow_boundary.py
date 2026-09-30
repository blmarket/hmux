#!/usr/bin/env python3
"""Run monitor callback regressions with one RefCell for the whole monitor state.

Only a disposable source copy changes. Operation bodies and application tests
are unchanged; the private state accessor acquires a real runtime borrow. This
checks that formatting, callbacks, and explicit destruction can reenter after
the previous state loan ends. Production monitor storage remains UnsafeCell.
"""
import re
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def borrowed_storage_probe(source):
    holder = 'UnsafeCell<MonitorState>'
    assert source.count(holder) == 2, 'Monitor holder definitions changed'
    source = source.replace(holder, 'std::cell::RefCell<MonitorState>')
    constructor = 'Rc::new(UnsafeCell::new(MonitorState'
    assert source.count(constructor) == 1, 'Monitor factory changed'
    source = source.replace(constructor, 'Rc::new(std::cell::RefCell::new(MonitorState')
    projection = r'unsafe\s*\{\s*operation\(&mut\s*\*self\.0\.get\(\)\)\s*\}'
    assert len(re.findall(projection, source)) == 1, 'Monitor state accessor changed'
    return re.sub(projection, 'operation(&mut *self.0.borrow_mut())', source)


def main():
    name = 'src/monitor.rs'
    source = borrowed_storage_probe((ROOT / name).read_text())
    with tempfile.TemporaryDirectory(prefix='hmux-monitor-borrows-') as directory:
        probe = Path(directory)
        for folder in ('src', '.cargo', 'hmux-buffer', 'hmux-cmdparse', 'hmux-refbox', 'hmux-rt'):
            shutil.copytree(ROOT / folder, probe / folder,
                            ignore=shutil.ignore_patterns('target', '.git'))
        for file in ('Cargo.toml', 'Cargo.lock', 'build.rs'):
            shutil.copy2(ROOT / file, probe / file)
        (probe / name).write_text(source)
        return subprocess.run(
            ['cargo', 'test', '--offline', '--lib', 'src::monitor::',
             '--target-dir', str(ROOT / 'target/monitor-borrow-boundary')],
            cwd=probe).returncode


if __name__ == '__main__':
    raise SystemExit(main())
