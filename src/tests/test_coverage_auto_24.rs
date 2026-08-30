//! Coverage for [`crate::options`] – options_get, options_set helpers with [`Options`] fixture.

use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::fmt_args;
use crate::options::OPTIONS_TABLE_SESSION;
use crate::options::{
    options_get_command, options_get_number, options_get_parent, options_get_string,
    options_set_command, options_set_number, options_set_parent, options_set_string,
    options_to_string,
};
use crate::options::{options_get_only_ptr, options_get_ptr};
use crate::tests::test_fixtures::{Options, globals, seen};
use crate::types::cstr_ptr;
use ::core::ptr::null_mut;

// ---------------------------------------------------------------------------
// options_get / options_get_only with parent fallback
// ---------------------------------------------------------------------------

#[test]
fn options_get_falls_back_to_parent_while_get_only_does_not() {
    let _g = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(parent.ptr());
    unsafe {
        // child has nothing of its own
        assert!(options_get_only_ptr(child.ptr(), c"status".as_ptr()).is_null());
        assert!(options_get_only_ptr(child.ptr(), c"status-left".as_ptr()).is_null());
        // but get walks to parent
        let o = options_get_ptr(child.ptr(), c"status".as_ptr());
        assert!(!o.is_null());
        assert_eq!(seen(cstr_ptr(&(*o).name)), "status");
        // get_only on unknown returns null, get also null when no parent has it
        assert!(options_get_only_ptr(child.ptr(), c"nonsuch".as_ptr()).is_null());
        assert!(options_get_ptr(child.ptr(), c"nonsuch".as_ptr()).is_null());
        // parent itself answers via get_only
        assert!(!options_get_only_ptr(parent.ptr(), c"status".as_ptr()).is_null());
    }
}

#[test]
fn options_get_and_set_parent_linkage() {
    let parent = Options::empty(null_mut());
    let child = Options::empty(null_mut());
    unsafe {
        assert!(options_get_parent(child.ptr()).is_null());
        options_set_parent(child.ptr(), parent.ptr());
        assert_eq!(options_get_parent(child.ptr()), parent.ptr());
        // re-parent to null
        options_set_parent(child.ptr(), null_mut());
        assert!(options_get_parent(child.ptr()).is_null());
        // parent with defaults is visible through child after linking again
        let session_parent = Options::defaults(OPTIONS_TABLE_SESSION);
        options_set_parent(child.ptr(), session_parent.ptr());
        assert!(!options_get_ptr(child.ptr(), c"status".as_ptr()).is_null());
        // unlink to avoid dangling pointer on drop (child dropped before session_parent)
        options_set_parent(child.ptr(), null_mut());
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
            ::std::ffi::CString::new(seen(options_get_string(oo.ptr(), c"status-left".as_ptr())))
                .unwrap();
        assert!(!before.as_bytes().is_empty());
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"hello".as_ptr()],
        );
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"status-left".as_ptr())),
            "hello"
        );
        // via options_to_string the value reads back identically
        let o = options_get_ptr(oo.ptr(), c"status-left".as_ptr());
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "hello");
        // restore
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![before.as_ptr()],
        );
    }
}

#[test]
fn options_set_string_append_uses_table_separator_or_empty() {
    let _g = globals();
    // status-left has no separator in table, so append concatenates directly
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"a".as_ptr()],
        );
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            1,
            c"%s".as_ptr(),
            fmt_args![c"b".as_ptr()],
        );
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"status-left".as_ptr())),
            "ab"
        );
        // overwrite (append=0) replaces
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"x".as_ptr()],
        );
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"status-left".as_ptr())),
            "x"
        );
    }
}

#[test]
fn options_user_option_is_string_appended_with_empty_separator() {
    let oo = Options::empty(null_mut());
    unsafe {
        options_set_string(
            oo.ptr(),
            c"@myopt".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"one".as_ptr()],
        );
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"@myopt".as_ptr())),
            "one"
        );
        options_set_string(
            oo.ptr(),
            c"@myopt".as_ptr(),
            1,
            c"%s".as_ptr(),
            fmt_args![c"two".as_ptr()],
        );
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"@myopt".as_ptr())),
            "onetwo"
        );
        // overwrite discards previous
        options_set_string(
            oo.ptr(),
            c"@myopt".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"three".as_ptr()],
        );
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"@myopt".as_ptr())),
            "three"
        );
        // get via generic get
        let o = options_get_ptr(oo.ptr(), c"@myopt".as_ptr());
        assert!(!o.is_null());
        assert_eq!(
            seen(options_get_string(oo.ptr(), c"@myopt".as_ptr())),
            "three"
        );
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "three");
    }
}

#[test]
fn options_child_inherits_and_then_owns_string_option() {
    let _g = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(parent.ptr());
    unsafe {
        // child initially sees parent's value via get
        let parent_val = seen(options_get_string(parent.ptr(), c"status-left".as_ptr()));
        assert_eq!(
            seen(options_get_string(child.ptr(), c"status-left".as_ptr())),
            parent_val
        );
        // setting on child creates owned entry
        options_set_string(
            child.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"child-val".as_ptr()],
        );
        assert!(!options_get_only_ptr(child.ptr(), c"status-left".as_ptr()).is_null());
        assert_eq!(
            seen(options_get_string(child.ptr(), c"status-left".as_ptr())),
            "child-val"
        );
        // parent unchanged
        assert_eq!(
            seen(options_get_string(parent.ptr(), c"status-left".as_ptr())),
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
        let orig = options_get_number(oo.ptr(), c"history-limit".as_ptr());
        assert_eq!(orig, 2000);
        options_set_number(oo.ptr(), c"history-limit".as_ptr(), 5000);
        assert_eq!(
            options_get_number(oo.ptr(), c"history-limit".as_ptr()),
            5000
        );
        let o = options_get_ptr(oo.ptr(), c"history-limit".as_ptr());
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "5000");
        // flag option
        assert_eq!(options_get_number(oo.ptr(), c"status".as_ptr()), 1);
        options_set_number(oo.ptr(), c"status".as_ptr(), 0);
        assert_eq!(options_get_number(oo.ptr(), c"status".as_ptr()), 0);
        assert_eq!(
            options_to_string(options_get_ptr(oo.ptr(), c"status".as_ptr()), -1, 0)
                .to_string_lossy(),
            "off"
        );
        // restore
        options_set_number(oo.ptr(), c"history-limit".as_ptr(), orig);
        options_set_number(oo.ptr(), c"status".as_ptr(), 1);
    }
}

#[test]
fn options_set_number_on_child_creates_owned_entry() {
    let _g = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(parent.ptr());
    unsafe {
        assert!(options_get_only_ptr(child.ptr(), c"history-limit".as_ptr()).is_null());
        options_set_number(child.ptr(), c"history-limit".as_ptr(), 1234);
        assert!(!options_get_only_ptr(child.ptr(), c"history-limit".as_ptr()).is_null());
        assert_eq!(
            options_get_number(child.ptr(), c"history-limit".as_ptr()),
            1234
        );
        assert_eq!(
            options_get_number(parent.ptr(), c"history-limit".as_ptr()),
            2000
        );
        // updating again overwrites same entry, not duplicate
        options_set_number(child.ptr(), c"history-limit".as_ptr(), 999);
        assert_eq!(
            options_get_number(child.ptr(), c"history-limit".as_ptr()),
            999
        );
    }
}

// ---------------------------------------------------------------------------
// options_get_command / options_set_command
// ---------------------------------------------------------------------------

#[test]
fn options_set_command_and_get_command_roundtrip() {
    let _g = globals();
    let parent = Options::defaults(crate::options::OPTIONS_TABLE_SERVER);
    let child = Options::empty(parent.ptr());
    unsafe {
        let mut pr = cmd_parse_from_string(c"display-message hi".as_ptr(), null_mut());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS);
        let list = pr.cmdlist.take();
        let o = options_set_command(
            child.ptr(),
            c"default-client-command".as_ptr(),
            list.clone(),
        );
        assert!(!o.is_null());
        let got = options_get_command(child.ptr(), c"default-client-command".as_ptr());
        assert!(got.is_some());
        assert_eq!(got, list);
        let s = options_to_string(o, -1, 0).to_string_lossy().into_owned();
        assert!(s.contains("display-message"), "got {s:?}");
        assert!(s.contains("hi"), "got {s:?}");
        let mut pr2 = cmd_parse_from_string(c"display-message bye".as_ptr(), null_mut());
        assert_eq!(pr2.status, CMD_PARSE_SUCCESS);
        let list2 = pr2.cmdlist.take();
        options_set_command(
            child.ptr(),
            c"default-client-command".as_ptr(),
            list2.clone(),
        );
        let s2 = options_to_string(o, -1, 0).to_string_lossy().into_owned();
        assert!(s2.contains("bye"), "got {s2:?}");
        let got2 = options_get_command(child.ptr(), c"default-client-command".as_ptr());
        assert_eq!(got2, list2);
    }
}
