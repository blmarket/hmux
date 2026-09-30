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
pub trait Window {
    fn empty() -> Self;
    fn id(&self) -> u32;
}
impl Window for crate::src::shared::window::WindowRef {
    fn empty() -> Self { window::new() }
    fn id(&self) -> u32 { unsafe { (*self.get()).id } }
}
pub fn observer_matches(owner: &crate::src::shared::window::WindowRef) -> bool {
    unsafe { (*owner.get()).observer.ptr_eq(&std::rc::Rc::downgrade(owner)) }
}
""",
    "src/window/model.rs": """
#[allow(non_camel_case_types)]
pub struct window {
    pub(super) observer: crate::src::shared::window::WindowWeak,
    pub(super) id: u32,
}
impl window {
    pub(super) fn new() -> crate::src::shared::window::WindowRef {
        std::rc::Rc::new_cyclic(|observer| {
            let mut value = Self { observer: std::rc::Weak::new(), id: 7 };
            value.observer = observer.clone();
            std::cell::UnsafeCell::new(value)
        })
    }
}
""",
    "src/external.rs": "fn consumer(owner: &WindowRef) { owner.get(); }",
}


class WindowStorageProbeTests(unittest.TestCase):
    def test_storage_projection_is_private_only_in_disposable_copy(self):
        result = private_storage_probe(SOURCES)
        self.assertIn("UnsafeCell<window>", SOURCES["src/shared/window.rs"])
        self.assertNotIn("UnsafeCell<window>", result["src/shared/window.rs"])
        self.assertEqual(result["src/shared/window.rs"].count("WindowStorage"), 2)
        self.assertIn("pub(super) fn get", result["src/window/model.rs"])
        self.assertIn("pub(super) fn new()", result["src/window/model.rs"])
        self.assertIn("WindowStorage::new(value)", result["src/window/model.rs"])
        self.assertIn("pub use model::window;", result["src/window/mod.rs"])
        self.assertIn("(*self.get()).id", result["src/window/mod.rs"])
        self.assertEqual(result["src/external.rs"], SOURCES["src/external.rs"])

    def test_export_formatting_preserves_the_actual_factory_and_registry(self):
        changed = dict(SOURCES)
        changed["src/window/mod.rs"] = changed["src/window/mod.rs"].replace(
            "pub use model::window;", "pub use model::{\n    window,\n};")
        result = private_storage_probe(changed)
        self.assertIn("pub use model::{\n    window,\n};", result["src/window/mod.rs"])
        self.assertIn("pub use model::WindowStorage;", result["src/window/mod.rs"])
        self.assertIn("fn empty() -> Self { window::new() }", result["src/window/mod.rs"])

    def test_changed_source_shape_fails_clearly_instead_of_skipping_probe(self):
        for path, old, new, message in [
            ("src/shared/window.rs", "UnsafeCell<window>", "OtherStorage", "holder definitions"),
            ("src/window/mod.rs", "pub use model::window;", "pub use model::*;", "model export"),
            ("src/window/model.rs", "UnsafeCell::new(value)", "UnsafeCell::default()", "factory"),
            ("src/window/mod.rs", "pub use model::window;", "pub use model::window;\npub use model::window;", "model export"),
            ("src/window/model.rs", "std::cell::UnsafeCell::new(value)",
             "std::cell::UnsafeCell::new(value); std::cell::UnsafeCell::new(value)", "factory"),
        ]:
            with self.subTest(path=path, new=new):
                changed = dict(SOURCES)
                changed[path] = changed[path].replace(old, new)
                with self.assertRaisesRegex(AssertionError, message):
                    private_storage_probe(changed)

    def test_compiler_accepts_trait_factory_and_rejects_external_storage_and_constructor_access(self):
        rustc = shutil.which("rustc")
        self.assertIsNotNone(rustc, "rustc is required for the storage probe regression")
        with tempfile.TemporaryDirectory(prefix="hmux-window-probe-test-") as directory:
            root = Path(directory)
            for name, source in private_storage_probe(SOURCES).items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            prelude = """
#![allow(dead_code, non_camel_case_types)]
mod src { pub mod shared { pub mod window; } pub mod window; }
use src::window::Window;
fn main() {
    let owner = src::shared::window::WindowRef::empty();
    let weak: src::shared::window::WindowWeak = std::rc::Rc::downgrade(&owner);
    assert_eq!(owner.id(), 7);
    assert!(src::window::observer_matches(&owner));
    assert_eq!(weak.upgrade().unwrap().id(), 7);
"""
            for body in ["", "let _ = owner.get();", "let _ = weak.upgrade().unwrap().get();",
                         "let _ = src::window::window::new();"]:
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
