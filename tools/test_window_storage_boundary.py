"""Mutation tests for the Window private-storage compile probe."""
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from window_storage_boundary import private_storage_probe


SOURCES = {
    "src/shared/window.rs": """
pub use crate::src::window::window;
pub type WindowRef = std::rc::Rc<std::cell::UnsafeCell<window>>;
pub type WindowWeak = std::rc::Weak<std::cell::UnsafeCell<window>>;
""",
    "src/window/mod.rs": """
mod model;
pub use model::window;
pub unsafe fn read(owner: &crate::src::shared::window::WindowRef) -> u32 {
    (*owner.get()).0
}
""",
    "src/window/model.rs": """
#[allow(non_camel_case_types)]
pub struct window(pub(super) u32);
impl window {
    pub fn new() -> crate::src::shared::window::WindowRef {
        let value = Self(7);
        std::rc::Rc::new(std::cell::UnsafeCell::new(value))
    }
}
""",
}


class WindowStorageProbeTests(unittest.TestCase):
    def test_storage_projection_is_private_only_in_disposable_copy(self):
        result = private_storage_probe(SOURCES)
        self.assertIn("UnsafeCell<window>", SOURCES["src/shared/window.rs"])
        self.assertNotIn("UnsafeCell<window>", result["src/shared/window.rs"])
        self.assertEqual(result["src/shared/window.rs"].count("WindowStorage"), 2)
        self.assertIn("pub(super) fn get", result["src/window/model.rs"])
        self.assertIn("(*owner.get()).0", result["src/window/mod.rs"])

    def test_changed_source_shape_fails_clearly_instead_of_skipping_probe(self):
        for path, old, new, message in [
            ("src/shared/window.rs", "UnsafeCell<window>", "OtherStorage", "holder definitions"),
            ("src/window/mod.rs", "pub use model::window;", "pub use model::*;", "model export"),
            ("src/window/model.rs", "UnsafeCell::new(value)", "UnsafeCell::default()", "factory"),
        ]:
            with self.subTest(path=path):
                changed = dict(SOURCES)
                changed[path] = changed[path].replace(old, new)
                with self.assertRaisesRegex(AssertionError, message):
                    private_storage_probe(changed)

    def test_compiler_accepts_internal_access_and_rejects_external_alias_projections(self):
        rustc = shutil.which("rustc")
        self.assertIsNotNone(rustc, "rustc is required for the storage probe regression")
        with tempfile.TemporaryDirectory(prefix="hmux-window-probe-test-") as directory:
            root = Path(directory)
            for name, source in private_storage_probe(SOURCES).items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            prelude = """
#![allow(dead_code)]
mod src { pub mod shared { pub mod window; } pub mod window; }
fn main() {
    let owner = src::window::window::new();
    let weak: src::shared::window::WindowWeak = std::rc::Rc::downgrade(&owner);
    assert_eq!(unsafe { src::window::read(&owner) }, 7);
"""
            for body in ["", "let _ = owner.get();", "let _ = weak.upgrade().unwrap().get();"]:
                with self.subTest(body=body):
                    (root / "main.rs").write_text(prelude + body + "\n}\n")
                    result = subprocess.run(
                        [rustc, "--edition=2021", "--emit=metadata", str(root / "main.rs"),
                         "-o", str(root / "main.rmeta")],
                        capture_output=True, text=True,
                    )
                    if body:
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn("E0624", result.stderr)
                        self.assertIn("private", result.stderr)
                    else:
                        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
