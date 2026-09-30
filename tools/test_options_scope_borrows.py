"""Exercise the actual OptionsScope/read helpers with one RefCell per model.

Only model adapters, unrelated option payloads and fixture initialization are
stubbed. The scope implementation and selected reader bodies are extracted from
the current source on every run. This is a borrow/lifetime regression test, not
an application lifecycle test or a production storage conversion.
"""

from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

from model_boundary_inventory import mask

ROOT = Path(__file__).resolve().parents[1]
READERS = (
    "options_create_owned", "options_create", "options_get_only", "options_get_only_mut",
    "options_read_entry", "options_get_string", "options_get_string_optional",
    "options_get_number_ref", "options_get_number",
)


def extract_function(source, name):
    """Copy one top-level function, ignoring braces in comments and literals."""
    masked = mask(source)
    matches = list(re.finditer(
        r"^(?:pub(?:\([^)]*\))? )?(?:unsafe )?fn " + re.escape(name) + r"\b",
        masked, re.MULTILINE,
    ))
    if len(matches) != 1:
        raise ValueError(f"src/options.rs: expected exactly one {name}, found {len(matches)}")
    match = matches[0]
    opening = masked.find("{", match.end())
    if opening < 0:
        raise ValueError(f"src/options.rs: {name} has no function body")
    depth = 1
    end = opening + 1
    while depth and end < len(masked):
        depth += (masked[end] == "{") - (masked[end] == "}")
        end += 1
    if depth:
        raise ValueError(f"src/options.rs: {name} has an unclosed function body")
    return source[match.start():end]


def current_source_fixture():
    source = (ROOT / "src/options.rs").read_text()
    readers = "\n".join(extract_function(source, name) for name in READERS)
    scope = (ROOT / "src/options/scope.rs").read_text()
    compact = re.sub(r"\s+", "", mask(scope))
    for variant, model in (("Session", "session"), ("Window", "window"), ("Pane", "window_pane")):
        expected = f"{variant}(Weak<UnsafeCell<{model}>>)"
        alias = f"{variant}({variant}Weak)" if variant != "Pane" else "Pane(PaneWeak)"
        if expected not in compact and alias not in compact:
            raise ValueError(
                f"src/options/scope.rs: {variant} storage declaration changed; "
                "update the whole-model RefCell adapter explicitly"
            )
    # Test-only replacement of model storage; all scope control flow stays exact.
    scope = re.sub(r"\bUnsafeCell\b", "RefCell", scope)
    return FIXTURE.replace("// SOURCE_SCOPE", scope).replace("// SOURCE_READERS", readers)


class OptionsScopeBorrowTests(unittest.TestCase):
    def test_source_extraction_rejects_missing_or_ambiguous_functions(self):
        for source, count in (("", 0), ("fn read() {}\nfn read() {}", 2)):
            with self.assertRaisesRegex(ValueError, f"found {count}"):
                extract_function(source, "read")
        with self.assertRaisesRegex(ValueError, "unclosed function body"):
            extract_function("fn read() {", "read")
        source = 'fn read() { let text = "}"; /* { */ body(); }\nfn next() {}'
        self.assertEqual(extract_function(source, "read"), source.split("\n")[0])

    def test_scope_and_readers_with_whole_model_refcells(self):
        rustc = shutil.which("rustc")
        self.assertIsNotNone(rustc, "rustc is required for the model borrow regression")
        with tempfile.TemporaryDirectory(prefix="hmux2-options-scope-") as directory:
            directory = Path(directory)
            source = directory / "scope.rs"
            executable = directory / "scope-tests"
            source.write_text(current_source_fixture())
            subprocess.run([rustc, "--edition=2021", "--test", str(source), "-o", str(executable)], check=True)
            subprocess.run([str(executable)], check=True)


# Minimal payloads keep this probe independent of unfinished model consumers.
# The Session release stub deliberately queues an Rc: the scope must not use it
# for a temporary view, while existing logical Session owners retain that API.
FIXTURE = r'''
#![allow(
    dead_code,
    unused,
    non_camel_case_types,
    non_upper_case_globals,
    private_interfaces,
    private_bounds,
    static_mut_refs
)]
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    ffi::{CStr, CString},
    rc::{Rc, Weak},
};
const OPTIONS_TABLE_STRING: u32 = 0;
const OPTIONS_TABLE_NUMBER: u32 = 1;
const OPTIONS_TABLE_KEY: u32 = 2;
const OPTIONS_TABLE_COLOUR: u32 = 3;
const OPTIONS_TABLE_FLAG: u32 = 4;
const OPTIONS_TABLE_CHOICE: u32 = 5;
struct options_table_entry {
    type_0: u32,
}
static NUMBER: options_table_entry = options_table_entry {
    type_0: OPTIONS_TABLE_NUMBER,
};
struct options {
    tree: BTreeMap<Vec<u8>, Box<options_entry>>,
    parent: Option<OptionsScope>,
}
struct options_entry {
    tableentry: Option<&'static options_table_entry>,
    value: options_value,
}
enum options_value {
    Empty,
    Number(i64),
    String(CString),
}
impl options_value {
    fn number(&self) -> i64 {
        if let Self::Number(value) = self {
            *value
        } else {
            panic!("not a number")
        }
    }
    fn string_ptr(&self) -> Option<&CStr> {
        match self {
            Self::String(value) => Some(value),
            Self::Empty => None,
            _ => panic!("not a string"),
        }
    }
}
macro_rules! model {
    ($kind:ident) => {
        pub struct $kind {
            table: Box<options>,
            unrelated: u32,
            releases: usize,
        }
        impl $kind {
            fn new(parent: Option<OptionsScope>) -> Rc<RefCell<Self>> {
                Rc::new(RefCell::new(Self {
                    table: options_create(parent),
                    unrelated: 0,
                    releases: 0,
                }))
            }
        }
    };
}
model!(session);
model!(window);
model!(window_pane);
thread_local! {static DEFERRED:RefCell<Vec<Rc<RefCell<session>>>>=RefCell::new(Vec::new());}
mod src {
    pub mod shared {
        pub mod options {
            pub(crate) use crate::{options, options_entry};
        }
        pub mod pane {
            pub use crate::window_pane;
        }
        pub mod window {
            pub use crate::window;
            pub type WindowWeak = std::rc::Weak<std::cell::RefCell<window>>;
            pub type WindowRef = std::rc::Rc<std::cell::RefCell<window>>;
        }
        pub mod session {
            pub use crate::session;
            pub type SessionWeak = std::rc::Weak<std::cell::RefCell<session>>;
        }
    }
    pub mod tmux {
        use crate::options;
        pub(crate) static mut global_options: *mut options = std::ptr::null_mut();
        pub(crate) static mut global_s_options: *mut options = std::ptr::null_mut();
        pub(crate) static mut global_w_options: *mut options = std::ptr::null_mut();
    }
    pub mod session {
        use super::super::*;
        pub trait Session {
            unsafe fn with_options_mut<R>(&self, visit: impl FnOnce(&mut options) -> R) -> R;
        }
        impl Session for Rc<RefCell<session>> {
            unsafe fn with_options_mut<R>(&self, visit: impl FnOnce(&mut options) -> R) -> R {
                visit(&mut *self.borrow_mut().table)
            }
        }
        pub unsafe fn session_remove_ref(owner: Rc<RefCell<session>>, _: &CStr) {
            owner.borrow_mut().releases += 1;
            DEFERRED.with(|pending| pending.borrow_mut().push(owner));
        }
    }
    pub mod window {
        use super::super::*;
        pub trait Window {
            unsafe fn with_options_mut<R>(&self, visit: impl FnOnce(&mut options) -> R) -> R;
            unsafe fn release(self, from: &CStr);
        }
        impl Window for Rc<RefCell<window>> {
            unsafe fn with_options_mut<R>(&self, visit: impl FnOnce(&mut options) -> R) -> R {
                visit(&mut *self.borrow_mut().table)
            }
            unsafe fn release(self, _: &CStr) {
                self.borrow_mut().releases += 1;
                drop(self);
            }
        }
    }
    pub mod window_pane {
        use super::super::*;
        pub trait WindowPane {
            unsafe fn with_options_mut<R>(&self, visit: impl FnOnce(&mut options) -> R) -> R;
            unsafe fn release(self, from: &CStr);
        }
        impl WindowPane for Rc<RefCell<window_pane>> {
            unsafe fn with_options_mut<R>(&self, visit: impl FnOnce(&mut options) -> R) -> R {
                visit(&mut *self.borrow_mut().table)
            }
            unsafe fn release(self, from: &CStr) {
                window_pane_remove_ref(self, from.as_ptr());
            }
        }
        pub unsafe fn window_pane_remove_ref(
            owner: Rc<RefCell<window_pane>>,
            _: *const std::ffi::c_char,
        ) {
            owner.borrow_mut().releases += 1;
            drop(owner);
        }
    }
}
mod scope {
    // SOURCE_SCOPE
}
use scope::OptionsScope;
unsafe fn options_map_name(name: *const std::ffi::c_char) -> *const std::ffi::c_char {
    name
}
fn fatalx(f: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>) -> ! {
    let mut message = Vec::new();
    f(&mut message).unwrap();
    panic!("{}", String::from_utf8_lossy(&message))
}
fn write_cstr(out: &mut dyn std::io::Write, text: *const std::ffi::c_char) -> std::io::Result<()> {
    out.write_all(unsafe { CStr::from_ptr(text).to_bytes() })
}
// SOURCE_READERS
fn insert(scope: &OptionsScope, name: &CStr, value: options_value) {
    unsafe {
        scope.with_local(|table| {
            let tableentry = if matches!(value, options_value::Number(_)) {
                Some(&NUMBER)
            } else {
                None
            };
            table.tree.insert(
                name.to_bytes().to_vec(),
                Box::new(options_entry { value, tableentry }),
            );
        });
    }
}
fn drain_sessions() {
    DEFERRED.with(|pending| pending.borrow_mut().clear());
}
#[test]
fn window_scope_releases_the_model_after_returning_its_whole_cell_borrow() {
    let owner = window::new(None);
    let scope = OptionsScope::Window(Rc::downgrade(&owner));
    unsafe {
        let answer = scope.with_local(|_| {
            assert_eq!(Rc::strong_count(&owner), 2);
            assert!(owner.try_borrow_mut().is_err());
            42
        });
        assert_eq!(answer, 42);
        assert_eq!(Rc::strong_count(&owner), 1);
        assert_eq!(owner.borrow().releases, 1);
        owner.borrow_mut().unrelated = 17;
        scope.with_local(|_| {});
        assert_eq!(owner.borrow().unrelated, 17);
    }
}
#[test]
fn pane_scope_releases_its_transient_owner_without_retaining_the_model() {
    let owner = window_pane::new(None);
    let observer = Rc::downgrade(&owner);
    let scope = OptionsScope::Pane(observer.clone());
    unsafe {
        scope.with_local(|_| {
            assert!(owner.try_borrow_mut().is_err());
        });
    }
    assert_eq!(Rc::strong_count(&owner), 1);
    assert_eq!(owner.borrow().releases, 1);
    drop(owner);
    assert!(observer.upgrade().is_none());
    drop(scope);
}
#[test]
fn session_scope_does_not_defer_a_temporary_visit_owner() {
    let owner = session::new(None);
    let observer = Rc::downgrade(&owner);
    let scope = OptionsScope::Session(observer.clone());
    unsafe {
        scope.with_local(|_| {
            assert!(owner.try_borrow_mut().is_err());
        });
    }
    assert_eq!(Rc::strong_count(&owner), 1);
    assert_eq!(owner.borrow().releases, 0);
    owner.borrow_mut().unrelated = 9;
    drop(owner);
    assert!(observer.upgrade().is_none());
    DEFERRED.with(|pending| assert!(pending.borrow().is_empty()));
    drop(scope);
}
#[test]
fn inherited_numeric_and_string_results_outlive_their_parent_entry() {
    let parent = window::new(None);
    let inherited = OptionsScope::Window(Rc::downgrade(&parent));
    let pane = window_pane::new(Some(inherited.clone()));
    let local = OptionsScope::Pane(Rc::downgrade(&pane));
    insert(&inherited, c"number", options_value::Number(73));
    let original = CString::new([b'a', 0xff, b'b']).unwrap();
    insert(
        &inherited,
        c"@text",
        options_value::String(original.clone()),
    );
    unsafe {
        let number = local.with_local(|table| options_get_number(table, c"number".as_ptr()));
        let string = local.with_local(|table| options_get_string(table, c"@text".as_ptr()));
        inherited.with_local(|table| table.tree.clear());
        assert_eq!(number, 73);
        assert_eq!(string, original);
        assert_eq!(Rc::strong_count(&parent), 1);
        assert_eq!(Rc::strong_count(&pane), 1);
        parent.borrow_mut().unrelated = 1;
        pane.borrow_mut().unrelated = 2;
    }
}
#[test]
fn resolution_releases_each_model_and_keeps_the_defining_scope_after_reparenting() {
    let old = window::new(None);
    let old_scope = OptionsScope::Window(Rc::downgrade(&old));
    let new = window::new(None);
    let new_scope = OptionsScope::Window(Rc::downgrade(&new));
    let pane = window_pane::new(Some(old_scope.clone()));
    let local = OptionsScope::Pane(Rc::downgrade(&pane));
    insert(&old_scope, c"number", options_value::Number(7));
    insert(&new_scope, c"number", options_value::Number(9));
    unsafe {
        assert!(local.resolve(c"number", true).is_none());
        assert!(local.with_entry(c"number", |_| ()).is_none());
        let resolved = local.resolve(c"number", false).unwrap();
        assert!(resolved == old_scope);
        local.with_local(|table| table.parent = Some(new_scope.clone()));
        assert!(local.resolve(c"number", false).unwrap() == new_scope);
        assert_eq!(
            resolved.with_entry(c"number", |entry| entry.value.number()),
            Some(7)
        );
        insert(&local, c"number", options_value::Number(11));
        assert!(local.resolve(c"number", false).unwrap() == local);
        assert_eq!(
            resolved.with_entry(c"number", |entry| entry.value.number()),
            Some(7)
        );
    }
    assert_eq!(Rc::strong_count(&old), 1);
    assert_eq!(Rc::strong_count(&new), 1);
    assert_eq!(Rc::strong_count(&pane), 1);
}
#[test]
fn numeric_and_string_inheritance_traverse_distinct_model_cells() {
    let session = session::new(None);
    let root_scope = OptionsScope::Session(Rc::downgrade(&session));
    let window = window::new(Some(root_scope.clone()));
    let middle = OptionsScope::Window(Rc::downgrade(&window));
    let pane = window_pane::new(Some(middle.clone()));
    let leaf = OptionsScope::Pane(Rc::downgrade(&pane));
    insert(&root_scope, c"number", options_value::Number(5));
    insert(
        &root_scope,
        c"@text",
        options_value::String(c"hello".to_owned()),
    );
    unsafe {
        assert_eq!(
            leaf.with_local(|table| options_get_number_ref(table, c"number")),
            5
        );
        assert_eq!(
            leaf.with_local(|table| options_get_string(table, c"@text".as_ptr())),
            c"hello".to_owned()
        );
    }
    assert_eq!(Rc::strong_count(&window), 1);
    assert_eq!(Rc::strong_count(&pane), 1);
    drain_sessions();
    assert_eq!(Rc::strong_count(&session), 1);
}
#[test]
fn empty_string_values_remain_distinct_from_owned_empty_strings() {
    let owner = window::new(None);
    let scope = OptionsScope::Window(Rc::downgrade(&owner));
    insert(&scope, c"@value", options_value::Empty);
    unsafe {
        assert_eq!(
            scope.with_local(|table| options_get_string_optional(table, c"@value".as_ptr())),
            None
        );
    }
    insert(&scope, c"@value", options_value::String(CString::default()));
    unsafe {
        assert_eq!(
            scope.with_local(|table| options_get_string_optional(table, c"@value".as_ptr())),
            Some(CString::default())
        );
    }
    assert_eq!(Rc::strong_count(&owner), 1);
}
'''


if __name__ == "__main__":
    unittest.main()
