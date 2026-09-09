//! Coverage for [`crate::options`] – options_get, options_set helpers with [`Options`] fixture.

use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::fmt_args;
use crate::options::OPTIONS_TABLE_SESSION;
use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};
use crate::tests::test_fixtures::{Options, globals};

// ---------------------------------------------------------------------------
// options_get / options_get_only with parent fallback
// ---------------------------------------------------------------------------

#[test]
fn options_get_falls_back_to_parent_while_get_only_does_not() {
    let _g = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(Some(&parent));
    unsafe {
        // child has nothing of its own
        assert!(child.with_entry(c"status", true, |entry| entry.is_none()));
        assert!(child.with_entry(c"status-left", true, |entry| entry.is_none()));
        // but get walks to parent
        child.with_entry(c"status", false, |entry| {
            assert_eq!(RustOptionsEngine.name(entry.unwrap()), c"status");
        });
        // get_only on unknown returns null, get also null when no parent has it
        assert!(child.with_entry(c"nonsuch", true, |entry| entry.is_none()));
        assert!(child.with_entry(c"nonsuch", false, |entry| entry.is_none()));
        // parent itself answers via get_only
        assert!(!parent.with_entry(c"status", true, |entry| entry.is_none()));
    }
}

#[test]
fn options_get_and_set_parent_linkage() {
    let parent = Options::empty(None);
    let child = Options::empty(None);
    {
        assert!(child.parent().is_none());
        child.set_parent(Some(&*parent));
        assert_eq!(child.parent().unwrap(), *parent);
        // re-parent to null
        child.set_parent(None);
        assert!(child.parent().is_none());
        // parent with defaults is visible through child after linking again
        let session_parent = Options::defaults(OPTIONS_TABLE_SESSION);
        child.set_parent(Some(&*session_parent));
        assert!(!child.with_entry(c"status", false, |entry| entry.is_none()));
        child.set_parent(None);
    }
}

// ---------------------------------------------------------------------------
// options_get_string / options_set_string
// ---------------------------------------------------------------------------

#[test]
fn options_set_string_and_get_string_roundtrip() {
    let _g = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let before =
            std::ffi::CString::new(oo.string_ref(c"status-left").to_string_lossy().into_owned())
                .unwrap();
        assert!(!before.as_bytes().is_empty());
        oo.set_string(c"status-left", 0, c"%s", fmt_args![c"hello"]);
        assert_eq!(
            oo.string_ref(c"status-left").to_string_lossy().into_owned(),
            "hello"
        );
        // via options_to_string the value reads back identically
        oo.with_entry(c"status-left", false, |entry| {
            assert_eq!(
                RustOptionsEngine
                    .display(entry.unwrap(), -1, 0)
                    .to_string_lossy(),
                "hello"
            );
        });
        // restore
        oo.set_string(c"status-left", 0, c"%s", fmt_args![before.as_c_str()]);
    }
}

#[test]
fn options_set_string_append_uses_table_separator_or_empty() {
    let _g = globals();
    // status-left has no separator in table, so append concatenates directly
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        oo.set_string(c"status-left", 0, c"%s", fmt_args![c"a"]);
        oo.set_string(c"status-left", 1, c"%s", fmt_args![c"b"]);
        assert_eq!(
            oo.string_ref(c"status-left").to_string_lossy().into_owned(),
            "ab"
        );
        // overwrite (append=0) replaces
        oo.set_string(c"status-left", 0, c"%s", fmt_args![c"x"]);
        assert_eq!(
            oo.string_ref(c"status-left").to_string_lossy().into_owned(),
            "x"
        );
    }
}

#[test]
fn options_user_option_is_string_appended_with_empty_separator() {
    let oo = Options::empty(None);
    unsafe {
        oo.set_string(c"@myopt", 0, c"%s", fmt_args![c"one"]);
        assert_eq!(
            oo.string_ref(c"@myopt").to_string_lossy().into_owned(),
            "one"
        );
        oo.set_string(c"@myopt", 1, c"%s", fmt_args![c"two"]);
        assert_eq!(
            oo.string_ref(c"@myopt").to_string_lossy().into_owned(),
            "onetwo"
        );
        // overwrite discards previous
        oo.set_string(c"@myopt", 0, c"%s", fmt_args![c"three"]);
        assert_eq!(
            oo.string_ref(c"@myopt").to_string_lossy().into_owned(),
            "three"
        );
        // get via generic get
        assert!(oo.with_entry(c"@myopt", false, |entry| entry.is_some()));
        assert_eq!(
            oo.string_ref(c"@myopt").to_string_lossy().into_owned(),
            "three"
        );
        assert_eq!(
            oo.with_entry(c"@myopt", false, |entry| {
                RustOptionsEngine.display(entry.unwrap(), -1, 0)
            })
            .to_string_lossy(),
            "three"
        );
    }
}

#[test]
fn options_child_inherits_and_then_owns_string_option() {
    let _g = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(Some(&parent));
    unsafe {
        // child initially sees parent's value via get
        let parent_val = parent
            .string_ref(c"status-left")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            child
                .string_ref(c"status-left")
                .to_string_lossy()
                .into_owned(),
            parent_val
        );
        // setting on child creates owned entry
        child.set_string(c"status-left", 0, c"%s", fmt_args![c"child-val"]);
        assert!(!child.with_entry(c"status-left", true, |entry| entry.is_none()));
        assert_eq!(
            child
                .string_ref(c"status-left")
                .to_string_lossy()
                .into_owned(),
            "child-val"
        );
        // parent unchanged
        assert_eq!(
            parent
                .string_ref(c"status-left")
                .to_string_lossy()
                .into_owned(),
            parent_val
        );
    }
}

// ---------------------------------------------------------------------------
// options_get_number / options_set_number
// ---------------------------------------------------------------------------

#[test]
fn options_set_number_and_get_number_roundtrip() {
    let _g = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let orig = oo.number(c"history-limit");
        assert_eq!(orig, 2000);
        oo.set_number(c"history-limit", 5000);
        assert_eq!(oo.number(c"history-limit"), 5000);
        oo.with_entry(c"history-limit", false, |entry| {
            assert_eq!(
                RustOptionsEngine
                    .display(entry.unwrap(), -1, 0)
                    .to_string_lossy(),
                "5000"
            );
        });
        // flag option
        assert_eq!(oo.number(c"status"), 1);
        oo.set_number(c"status", 0);
        assert_eq!(oo.number(c"status"), 0);
        assert_eq!(
            oo.with_entry(c"status", false, |entry| {
                RustOptionsEngine.display(entry.unwrap(), -1, 0)
            })
            .to_string_lossy(),
            "off"
        );
        // restore
        oo.set_number(c"history-limit", orig);
        oo.set_number(c"status", 1);
    }
}

#[test]
fn options_set_number_on_child_creates_owned_entry() {
    let _g = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(Some(&parent));
    unsafe {
        assert!(child.with_entry(c"history-limit", true, |entry| entry.is_none()));
        child.set_number(c"history-limit", 1234);
        assert!(!child.with_entry(c"history-limit", true, |entry| entry.is_none()));
        assert_eq!(child.number(c"history-limit"), 1234);
        assert_eq!(parent.number(c"history-limit"), 2000);
        // updating again overwrites same entry, not duplicate
        child.set_number(c"history-limit", 999);
        assert_eq!(child.number(c"history-limit"), 999);
    }
}

// ---------------------------------------------------------------------------
// options_get_command / options_set_command
// ---------------------------------------------------------------------------

#[test]
fn options_set_command_and_get_command_roundtrip() {
    let _g = globals();
    let parent = Options::defaults(crate::options::OPTIONS_TABLE_SERVER);
    let child = Options::empty(Some(&parent));
    unsafe {
        let mut pr = cmd_parse_from_string(c"display-message hi", None);
        assert_eq!(pr.status, CMD_PARSE_SUCCESS);
        let list = pr.cmdlist.take();
        child.set_command(c"default-client-command", list.clone());
        let got = child.command(c"default-client-command");
        assert!(got.is_some());
        assert_eq!(got, list);
        let s = child.with_entry(c"default-client-command", true, |entry| {
            RustOptionsEngine
                .display(entry.unwrap(), -1, 0)
                .to_string_lossy()
                .into_owned()
        });
        assert!(s.contains("display-message"), "got {s:?}");
        assert!(s.contains("hi"), "got {s:?}");
        let mut pr2 = cmd_parse_from_string(c"display-message bye", None);
        assert_eq!(pr2.status, CMD_PARSE_SUCCESS);
        let list2 = pr2.cmdlist.take();
        child.set_command(c"default-client-command", list2.clone());
        let s2 = child.with_entry(c"default-client-command", true, |entry| {
            RustOptionsEngine
                .display(entry.unwrap(), -1, 0)
                .to_string_lossy()
                .into_owned()
        });
        assert!(s2.contains("bye"), "got {s2:?}");
        let got2 = child.command(c"default-client-command");
        assert_eq!(got2, list2);
    }
}
