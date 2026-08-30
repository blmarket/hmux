//! Coverage for [`crate::environ`] — edge cases for `environ_set`,
//! `environ_find`, `environ_copy` and related helpers.
//!
//! `environ.rs` is at 100% line coverage; these tests add deterministic edge
//! cases around ordering, flag preservation, missing entries, and the
//! session-environment builder. All tests use the [`Environ`] fixture and hold
//! [`globals`] where global state is touched.

use crate::environ::{
    ENVIRON_HIDDEN, environ_clear, environ_copy, environ_entry_flags, environ_entry_name,
    environ_entry_value, environ_find, environ_put, environ_set, environ_unset,
};
use crate::fmt_args;
use crate::tests::test_fixtures::{Environ, Session, globals, seen};
use crate::tmux::{global_environ, socket_path};
use ::core::ptr::null_mut;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

unsafe fn dump_names(env: *mut crate::environ::environ_t) -> Vec<String> {
    unsafe {
        crate::environ::environ_entries(&*env)
            .map(|envent| seen(environ_entry_name(envent)))
            .collect()
    }
}

unsafe fn value_of(
    env: *mut crate::environ::environ_t,
    name: &::core::ffi::CStr,
) -> Option<String> {
    unsafe {
        environ_find(&*env, name.as_ptr())
            .map(|e| environ_entry_value(e))
            .filter(|value| !value.is_null())
            .map(|value| seen(value))
    }
}

unsafe fn set(
    env: *mut crate::environ::environ_t,
    name: &::core::ffi::CStr,
    flags: ::core::ffi::c_int,
    val: &::core::ffi::CStr,
) {
    unsafe {
        environ_set(
            env,
            name.as_ptr(),
            flags,
            c"%s".as_ptr(),
            fmt_args![val.as_ptr()],
        );
    }
}

// ---------------------------------------------------------------------------
// environ_find / environ_set / environ_clear
// ---------------------------------------------------------------------------

#[test]
fn environ_find_missing_returns_null() {
    let env = Environ::new();
    unsafe {
        assert!(environ_find(&*env.ptr(), c"MISSING".as_ptr()).is_none());
        assert!(environ_find(&*env.ptr(), c"".as_ptr()).is_none());
        assert!(value_of(env.ptr(), c"MISSING").is_none());
    }
}

#[test]
fn environ_set_replaces_value_and_flags() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"FLAGVAR", ENVIRON_HIDDEN, c"first");
        let e = environ_find(&*env.ptr(), c"FLAGVAR".as_ptr());
        assert_eq!(environ_entry_flags(e.unwrap()), ENVIRON_HIDDEN);
        assert_eq!(value_of(env.ptr(), c"FLAGVAR"), Some("first".to_owned()));

        set(env.ptr(), c"FLAGVAR", 0, c"second");
        let e2 = environ_find(&*env.ptr(), c"FLAGVAR".as_ptr());
        assert_eq!(environ_entry_flags(e2.unwrap()), 0);
        assert_eq!(value_of(env.ptr(), c"FLAGVAR"), Some("second".to_owned()));
        // still one entry, not duplicated
        assert_eq!(dump_names(env.ptr()).len(), 1);
    }
}

#[test]
fn environ_clear_then_set_restores_value() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"CLEAR_ME", 0, c"orig");
        environ_clear(env.ptr(), c"CLEAR_ME".as_ptr());
        assert_eq!(value_of(env.ptr(), c"CLEAR_ME"), None);
        // entry still present as cleared
        assert!(environ_find(&*env.ptr(), c"CLEAR_ME".as_ptr()).is_some());
        assert_eq!(dump_names(env.ptr()), ["CLEAR_ME"]);

        set(env.ptr(), c"CLEAR_ME", ENVIRON_HIDDEN, c"restored");
        assert_eq!(
            value_of(env.ptr(), c"CLEAR_ME"),
            Some("restored".to_owned())
        );
        assert_eq!(
            environ_entry_flags(environ_find(&*env.ptr(), c"CLEAR_ME".as_ptr()).unwrap()),
            ENVIRON_HIDDEN
        );
    }
}

#[test]
fn environ_put_without_equals_does_nothing() {
    let env = Environ::new();
    unsafe {
        environ_put(env.ptr(), c"NOEQUALS".as_ptr(), 0);
        assert!(environ_find(&*env.ptr(), c"NOEQUALS".as_ptr()).is_none());
        assert!(dump_names(env.ptr()).is_empty());

        // empty value is allowed
        environ_put(env.ptr(), c"EMPTY=".as_ptr(), 0);
        assert_eq!(value_of(env.ptr(), c"EMPTY"), Some(String::new()));

        // value with extra equals keeps them
        environ_put(env.ptr(), c"EQ=a=b=c".as_ptr(), 0);
        assert_eq!(value_of(env.ptr(), c"EQ"), Some("a=b=c".to_owned()));
    }
}

// ---------------------------------------------------------------------------
// environ_copy
// ---------------------------------------------------------------------------

#[test]
fn environ_copy_from_empty_leaves_dst_intact() {
    let src = Environ::new();
    let dst = Environ::new();
    unsafe {
        set(dst.ptr(), c"KEEP", 0, c"yes");
        environ_copy(src.ptr(), dst.ptr());
        assert_eq!(dump_names(dst.ptr()), ["KEEP"]);
        assert_eq!(value_of(dst.ptr(), c"KEEP"), Some("yes".to_owned()));
    }
}

#[test]
fn environ_copy_propagates_values_flags_and_cleared() {
    let src = Environ::new();
    let dst = Environ::new();
    unsafe {
        set(src.ptr(), c"A", ENVIRON_HIDDEN, c"alpha");
        environ_clear(src.ptr(), c"B".as_ptr());
        set(src.ptr(), c"C", 0, c"gamma");

        set(dst.ptr(), c"B", 0, c"stale");
        set(dst.ptr(), c"Z", 0, c"zebra");

        environ_copy(src.ptr(), dst.ptr());

        // A copied with flag, B cleared, C added, Z untouched
        assert_eq!(value_of(dst.ptr(), c"A"), Some("alpha".to_owned()));
        assert_eq!(
            environ_entry_flags(environ_find(&*dst.ptr(), c"A".as_ptr()).unwrap()),
            ENVIRON_HIDDEN
        );
        assert_eq!(value_of(dst.ptr(), c"B"), None);
        assert!(environ_find(&*dst.ptr(), c"B".as_ptr()).is_some());
        assert_eq!(value_of(dst.ptr(), c"C"), Some("gamma".to_owned()));
        assert_eq!(value_of(dst.ptr(), c"Z"), Some("zebra".to_owned()));
        assert_eq!(dump_names(dst.ptr()), ["A", "B", "C", "Z"]);
    }
}

// ---------------------------------------------------------------------------
// environ_next / environ_first traversal
// ---------------------------------------------------------------------------

#[test]
fn the_entry_walk_runs_in_sorted_order() {
    let env = Environ::new();
    unsafe {
        assert!(dump_names(env.ptr()).is_empty());
        for name in [c"ZZZ", c"AAA", c"MMM", c"BBB"] {
            set(env.ptr(), name, 0, c"v");
        }
        let names = dump_names(env.ptr());
        for pair in names.windows(2) {
            assert!(pair[1] > pair[0]);
        }
        assert_eq!(names, ["AAA", "BBB", "MMM", "ZZZ"]);
    }
}

#[test]
fn environ_unset_idempotent_and_removes_entry() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"ONE", 0, c"1");
        set(env.ptr(), c"TWO", 0, c"2");
        set(env.ptr(), c"THREE", 0, c"3");

        environ_unset(env.ptr(), c"TWO".as_ptr());
        assert_eq!(dump_names(env.ptr()), ["ONE", "THREE"]);
        assert!(environ_find(&*env.ptr(), c"TWO".as_ptr()).is_none());

        // unsetting missing is a no-op
        environ_unset(env.ptr(), c"TWO".as_ptr());
        environ_unset(env.ptr(), c"ABSENT".as_ptr());
        assert_eq!(dump_names(env.ptr()), ["ONE", "THREE"]);

        // unset first and last
        environ_unset(env.ptr(), c"ONE".as_ptr());
        assert_eq!(dump_names(env.ptr()), ["THREE"]);
        environ_unset(env.ptr(), c"THREE".as_ptr());
        assert!(dump_names(env.ptr()).is_empty());
        assert!(dump_names(env.ptr()).is_empty());
    }
}

// ---------------------------------------------------------------------------
// environ_for_session / global_environ interaction (needs globals)
// ---------------------------------------------------------------------------

#[test]
fn environ_for_session_sets_tmux_and_defaults() {
    let _guard = globals();
    unsafe {
        let saved = socket_path;
        socket_path = c"/tmp/c2rs-auto12.sock".as_ptr();
        // ensure global has a marker so we can see it copied
        set(global_environ, c"C2RS_AUTO12_GLOBAL", 0, c"globalval");

        let env = Environ::from_box(crate::environ::environ_for_session(
            null_mut::<crate::types::session>(),
            0,
        ));
        assert_eq!(
            value_of(env.ptr(), c"C2RS_AUTO12_GLOBAL"),
            Some("globalval".to_owned())
        );
        assert_eq!(
            value_of(env.ptr(), c"TERM_PROGRAM"),
            Some("tmux".to_owned())
        );
        assert_eq!(
            value_of(env.ptr(), c"COLORTERM"),
            Some("truecolor".to_owned())
        );
        assert!(value_of(env.ptr(), c"TERM").is_some());
        // LISTEN vars are always cleared
        assert_eq!(value_of(env.ptr(), c"LISTEN_PID"), None);
        // TMUX contains socket path and pid with -1 when no session
        let tmux = value_of(env.ptr(), c"TMUX").unwrap();
        assert!(tmux.starts_with("/tmp/c2rs-auto12.sock,"), "{tmux}");
        assert!(tmux.ends_with(",-1"), "{tmux}");

        socket_path = saved;
        crate::environ::environ_unset(global_environ, c"C2RS_AUTO12_GLOBAL".as_ptr());
    }
}

#[test]
fn environ_for_session_with_session_overrides_global() {
    let _guard = globals();
    let mut s = Session::new(42, "auto12");
    unsafe {
        let saved = socket_path;
        socket_path = c"/tmp/c2rs-auto12b.sock".as_ptr();
        set(s.environ(), c"C2RS_S_VAR", 0, c"sessval");
        set(global_environ, c"C2RS_S_VAR", 0, c"globalval");

        let env = Environ::from_box(crate::environ::environ_for_session(s.ptr(), 1));
        // session overrides global
        assert_eq!(
            value_of(env.ptr(), c"C2RS_S_VAR"),
            Some("sessval".to_owned())
        );
        // no_TERM skips TERM_PROGRAM
        assert_eq!(value_of(env.ptr(), c"TERM_PROGRAM"), None);
        let tmux = value_of(env.ptr(), c"TMUX").unwrap();
        assert!(tmux.ends_with(",42"), "{tmux}");

        socket_path = saved;
        crate::environ::environ_unset(global_environ, c"C2RS_S_VAR".as_ptr());
    }
}
