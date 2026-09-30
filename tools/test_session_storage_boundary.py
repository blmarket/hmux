"""Mutation tests for the Session private-storage compile probe."""
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from session_storage_boundary import private_storage_probe


SOURCES = {
    "src/shared/session.rs": """
pub use crate::src::session::session;
pub type SessionRef = std::rc::Rc<std::cell::UnsafeCell<session>>;
pub type SessionWeak = std::rc::Weak<std::cell::UnsafeCell<session>>;
""",
    "src/session/mod.rs": """
mod model;
pub use model::{session, sessions};
pub trait Session {
    fn allocate() -> Self;
    fn id(&self) -> u32;
}
impl Session for crate::src::shared::session::SessionRef {
    fn allocate() -> Self { session::new() }
    fn id(&self) -> u32 { unsafe { (*self.get()).id } }
}
pub fn observer_matches(owner: &crate::src::shared::session::SessionRef) -> bool {
    unsafe { (*owner.get()).observer.ptr_eq(&std::rc::Rc::downgrade(owner)) }
}
""",
    "src/session/model.rs": """
#[allow(non_camel_case_types)]
pub struct session {
    pub(super) observer: crate::src::shared::session::SessionWeak,
    pub(super) id: u32,
}
pub struct sessions;
impl session {
    pub(super) fn new() -> crate::src::shared::session::SessionRef {
        std::rc::Rc::new_cyclic(|observer| {
            let mut value = Self { observer: std::rc::Weak::new(), id: 7 };
            value.observer = observer.clone();
            std::cell::UnsafeCell::new(value)
        })
    }
}
""",
    "src/external.rs": "fn consumer(owner: &SessionRef) { owner.get(); }",
}


class SessionStorageProbeTests(unittest.TestCase):
    def test_storage_projection_is_private_only_in_disposable_copy(self):
        result = private_storage_probe(SOURCES)
        self.assertIn("UnsafeCell<session>", SOURCES["src/shared/session.rs"])
        self.assertNotIn("UnsafeCell<session>", result["src/shared/session.rs"])
        self.assertEqual(result["src/shared/session.rs"].count("SessionStorage"), 2)
        self.assertIn("pub(super) fn get", result["src/session/model.rs"])
        self.assertIn("pub(super) fn new()", result["src/session/model.rs"])
        self.assertIn("SessionStorage::new(value)", result["src/session/model.rs"])
        self.assertIn("pub use model::{session, sessions};", result["src/session/mod.rs"])
        self.assertIn("(*self.get()).id", result["src/session/mod.rs"])
        self.assertEqual(result["src/external.rs"], SOURCES["src/external.rs"])

    def test_export_formatting_preserves_the_actual_factory_and_registry(self):
        changed = dict(SOURCES)
        changed["src/session/mod.rs"] = changed["src/session/mod.rs"].replace(
            "pub use model::{session, sessions};", "pub use model::{\n    session,\n    sessions,\n};")
        result = private_storage_probe(changed)
        self.assertIn("pub use model::{\n    session,\n    sessions,\n};", result["src/session/mod.rs"])
        self.assertIn("pub use model::SessionStorage;", result["src/session/mod.rs"])
        self.assertIn("fn allocate() -> Self { session::new() }", result["src/session/mod.rs"])

    def test_changed_source_shape_fails_clearly_instead_of_skipping_probe(self):
        for path, old, new, message in [
            ("src/shared/session.rs", "UnsafeCell<session>", "OtherStorage", "holder definitions"),
            ("src/session/mod.rs", "pub use model::{session, sessions};", "pub use model::*;", "model export"),
            ("src/session/model.rs", "UnsafeCell::new(value)", "UnsafeCell::default()", "factory"),
            ("src/session/mod.rs", "pub use model::{session, sessions};", "pub use model::{session, sessions};\npub use model::{session, sessions};", "model export"),
            ("src/session/model.rs", "std::cell::UnsafeCell::new(value)",
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
        with tempfile.TemporaryDirectory(prefix="hmux-session-probe-test-") as directory:
            root = Path(directory)
            for name, source in private_storage_probe(SOURCES).items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            prelude = """
#![allow(dead_code, non_camel_case_types)]
mod src { pub mod shared { pub mod session; } pub mod session; }
use src::session::Session;
fn main() {
    let owner = src::shared::session::SessionRef::allocate();
    let weak: src::shared::session::SessionWeak = std::rc::Rc::downgrade(&owner);
    assert_eq!(owner.id(), 7);
    assert!(src::session::observer_matches(&owner));
    assert_eq!(weak.upgrade().unwrap().id(), 7);
"""
            for body in ["", "let _ = owner.get();", "let _ = weak.upgrade().unwrap().get();",
                         "let _ = src::session::session::new();"]:
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
