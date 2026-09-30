"""Compiler regressions for pane storage access across owner boundaries."""
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from pane_storage_boundary import private_storage_probe

SOURCES = {
    'src/window_pane/mod.rs': '''
mod model;
pub use model::window_pane;
pub fn allocate() -> std::rc::Rc<std::cell::UnsafeCell<window_pane>> { window_pane::new() }
pub unsafe fn read(owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) -> u32 {
    (*owner.get()).0
}
''',
    'src/window_pane/model.rs': '''
#[allow(non_camel_case_types)]
pub struct window_pane(pub(super) u32);
impl window_pane {
    pub(super) fn new() -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        Self(7).into_shared()
    }
    fn into_shared(self) -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        std::rc::Rc::new(std::cell::UnsafeCell::new(self))
    }
}
''',
    'src/consumer.rs': '''
use crate::src::window_pane::window_pane;
pub fn retain(owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) -> std::rc::Weak<std::cell::UnsafeCell<window_pane>> {
    std::rc::Rc::downgrade(owner)
}
''',
}


class PaneStorageProbeTests(unittest.TestCase):
    def test_preserves_non_pane_cells_comments_and_literals(self):
        source = '''
// UnsafeCell<window_pane>
const TEXT: &str = "UnsafeCell<window_pane>";
type Component = std::cell::UnsafeCell<u32>;
type Pane = std::rc::Rc<std::cell::UnsafeCell<window_pane>>;
'''
        changed = private_storage_probe(dict(SOURCES, **{'src/component.rs': source}))
        self.assertIn('// UnsafeCell<window_pane>', changed['src/component.rs'])
        self.assertIn('"UnsafeCell<window_pane>"', changed['src/component.rs'])
        self.assertIn('UnsafeCell<u32>', changed['src/component.rs'])
        self.assertIn('Rc<crate::src::window_pane::PaneStorage>', changed['src/component.rs'])
        self.assertIn('UnsafeCell<window_pane>', SOURCES['src/consumer.rs'])

    def test_import_aliases_and_integration_holder_types_stay_probed(self):
        sources = dict(SOURCES)
        sources['tests/consumer.rs'] = '''
use hmux2::src::window_pane::window_pane as Record;
type Holder = std::rc::Rc<std::cell::UnsafeCell<Record>>;
type Observer = std::rc::Weak<std::cell::UnsafeCell<window_pane>>;
'''
        result = private_storage_probe(sources)['tests/consumer.rs']
        self.assertEqual(result.count('hmux2::src::window_pane::PaneStorage'), 2)
        self.assertNotIn('UnsafeCell<Record>', result)

    def test_changed_factory_or_export_cannot_silently_skip_probe(self):
        for path, old, new, message in [
            ('src/window_pane/mod.rs', 'pub use model::window_pane;', 'pub use model::*;', 'model export'),
            ('src/window_pane/model.rs', 'UnsafeCell::new(self)', 'UnsafeCell::default()', 'factory'),
        ]:
            with self.subTest(path=path):
                sources = dict(SOURCES)
                sources[path] = sources[path].replace(old, new)
                with self.assertRaisesRegex(AssertionError, message):
                    private_storage_probe(sources)

    def test_compiler_rejects_inferred_strong_weak_and_generic_projections(self):
        rustc = shutil.which('rustc')
        self.assertIsNotNone(rustc)
        with tempfile.TemporaryDirectory(prefix='hmux-pane-probe-test-') as directory:
            root = Path(directory)
            for name, source in private_storage_probe(SOURCES).items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            prelude = '''
#![allow(dead_code)]
mod src { pub mod window_pane; pub mod consumer; }
fn main() {
    let owner = src::window_pane::allocate();
    let weak = src::consumer::retain(&owner);
    assert_eq!(unsafe { src::window_pane::read(&owner) }, 7);
'''
            for body, diagnostic in [
                ('', None),
                ('let _ = owner.get();', 'E0624'),
                ('let _ = weak.upgrade().unwrap().get();', 'E0624'),
                ('fn cell<T>(_: &std::rc::Rc<std::cell::UnsafeCell<T>>) {} cell(&owner);', 'E0308'),
                ('let _ = src::window_pane::window_pane::new();', 'E0624'),
            ]:
                with self.subTest(body=body):
                    (root / 'main.rs').write_text(prelude + body + '\n}\n')
                    result = subprocess.run([
                        rustc, '--edition=2021', '--emit=metadata', str(root / 'main.rs'),
                        '-o', str(root / 'main.rmeta')], capture_output=True, text=True)
                    if diagnostic:
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn(diagnostic, result.stderr)
                    else:
                        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
