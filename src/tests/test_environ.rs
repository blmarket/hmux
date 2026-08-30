use super::*;
use crate::environ::{environ_entry_flags, environ_entry_name, environ_entry_value};
use crate::ffi::{getenv, unsetenv};
use crate::fmt_args;
use crate::options::options_get_ptr;
use crate::options::{options_array_clear, options_array_set};
use crate::tests::test_fixtures::{Environ, Options, Session, globals, seen};
use crate::tmux::global_environ;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// What the process environment holds now, name and value apiece. Pushing
/// an environment calls `clearenv`, which gives up the array the process
/// started with, so putting the pointer back afterwards leaves every later
/// test reading freed memory — the array has to be built again entry by
/// entry instead.
fn process_environment() -> Vec<(CString, CString)> {
    unsafe {
        let mut out = Vec::new();
        let mut p = environ;
        while !p.is_null() && !(*p).is_null() {
            let entry = CStr::from_ptr(*p).to_bytes().to_vec();
            if let Some(at) = entry.iter().position(|b| *b == b'=') {
                out.push((
                    CString::new(&entry[..at]).expect("no NUL"),
                    CString::new(&entry[at + 1..]).expect("no NUL"),
                ));
            }
            p = p.offset(1);
        }
        out
    }
}

/// Puts back what [`process_environment`] found, dropping anything set
/// since.
fn restore_process_environment(saved: &[(CString, CString)]) {
    unsafe {
        let now = process_environment();
        for (name, _) in &now {
            unsetenv(name.as_ptr());
        }
        for (name, value) in saved {
            setenv(name.as_ptr(), value.as_ptr(), 1);
        }
    }
}

/// The value of one entry, if it has one.
unsafe fn value_seen(envent: *const environ_entry) -> Option<String> {
    unsafe {
        let value = environ_entry_value(envent);
        (!value.is_null()).then(|| seen(value))
    }
}

/// Every entry of `env` in tree order: name, value and flags.
unsafe fn dump(env: *mut environ_t) -> Vec<(String, Option<String>, c_int)> {
    unsafe {
        environ_entries(&*env)
            .map(|envent| {
                (
                    seen(environ_entry_name(envent)),
                    value_seen(envent),
                    environ_entry_flags(envent),
                )
            })
            .collect()
    }
}

/// The names of `env` in tree order.
unsafe fn names(env: *mut environ_t) -> Vec<String> {
    unsafe { dump(env).into_iter().map(|(name, _, _)| name).collect() }
}

/// The value of one name, if the entry is there and has one.
unsafe fn value(env: *mut environ_t, name: &CStr) -> Option<String> {
    unsafe { environ_find(&*env, name.as_ptr()).and_then(|envent| value_seen(envent)) }
}

/// Sets `name` to `value`, the way every caller of the varargs does.
unsafe fn set(env: *mut environ_t, name: &CStr, flags: c_int, value: &CStr) {
    unsafe {
        environ_set(
            env,
            name.as_ptr(),
            flags,
            c"%s".as_ptr(),
            fmt_args![value.as_ptr()],
        );
    }
}

#[test]
fn a_new_environment_is_empty() {
    let env = Environ::new();
    unsafe {
        assert!((*env.ptr()).is_empty());
        assert!(names(env.ptr()).is_empty());
        assert!(environ_find(&*env.ptr(), c"ANY".as_ptr()).is_none());
    }
}

#[test]
fn freeing_nothing_is_allowed() {
    unsafe { environ_free(null_mut::<environ_t>()) };
}

#[test]
fn entries_come_back_in_name_order() {
    let env = Environ::new();
    unsafe {
        for name in [c"PATH", c"HOME", c"TERM", c"AAA"] {
            set(env.ptr(), name, 0, c"x");
        }
        assert_eq!(names(env.ptr()), ["AAA", "HOME", "PATH", "TERM"]);
    }
}

#[test]
fn setting_a_name_again_replaces_its_value_and_flags() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"NAME", ENVIRON_HIDDEN, c"first");
        assert_eq!(
            dump(env.ptr()),
            [("NAME".to_owned(), Some("first".to_owned()), ENVIRON_HIDDEN)]
        );

        set(env.ptr(), c"NAME", 0, c"second");
        assert_eq!(
            dump(env.ptr()),
            [("NAME".to_owned(), Some("second".to_owned()), 0)]
        );
    }
}

#[test]
fn a_value_is_built_from_the_format_and_its_arguments() {
    let env = Environ::new();
    unsafe {
        environ_set(
            env.ptr(),
            c"NAME".as_ptr(),
            0,
            c"%s-%d".as_ptr(),
            fmt_args![c"tmux".as_ptr(), 7 as c_int],
        );
        assert_eq!(value(env.ptr(), c"NAME"), Some("tmux-7".to_owned()));
    }
}

#[test]
fn clearing_leaves_a_named_entry_with_no_value() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"NAME", ENVIRON_HIDDEN, c"value");
        environ_clear(env.ptr(), c"NAME".as_ptr());
        assert_eq!(dump(env.ptr()), [("NAME".to_owned(), None, ENVIRON_HIDDEN)]);

        environ_clear(env.ptr(), c"OTHER".as_ptr());
        assert_eq!(
            dump(env.ptr()),
            [
                ("NAME".to_owned(), None, ENVIRON_HIDDEN),
                ("OTHER".to_owned(), None, 0),
            ]
        );
    }
}

#[test]
fn putting_splits_at_the_first_equals() {
    let env = Environ::new();
    unsafe {
        environ_put(env.ptr(), c"NAME=a=b".as_ptr(), ENVIRON_HIDDEN);
        assert_eq!(
            dump(env.ptr()),
            [("NAME".to_owned(), Some("a=b".to_owned()), ENVIRON_HIDDEN)]
        );

        environ_put(env.ptr(), c"EMPTY=".as_ptr(), 0);
        assert_eq!(value(env.ptr(), c"EMPTY"), Some(String::new()));

        environ_put(env.ptr(), c"=novalue".as_ptr(), 0);
        assert_eq!(value(env.ptr(), c""), Some("novalue".to_owned()));

        environ_put(env.ptr(), c"NOEQUALS".as_ptr(), 0);
        assert!(environ_find(&*env.ptr(), c"NOEQUALS".as_ptr()).is_none());
    }
}

#[test]
fn unsetting_takes_the_entry_away() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"ONE", 0, c"1");
        set(env.ptr(), c"TWO", 0, c"2");
        environ_unset(env.ptr(), c"ONE".as_ptr());
        assert_eq!(names(env.ptr()), ["TWO"]);

        environ_unset(env.ptr(), c"ONE".as_ptr());
        assert_eq!(names(env.ptr()), ["TWO"]);
    }
}

#[test]
fn copying_carries_values_over_and_clears_what_had_none() {
    let src = Environ::new();
    let dst = Environ::new();
    unsafe {
        set(src.ptr(), c"KEPT", ENVIRON_HIDDEN, c"value");
        environ_clear(src.ptr(), c"GONE".as_ptr());
        set(dst.ptr(), c"GONE", 0, c"was here");
        set(dst.ptr(), c"OWN", 0, c"mine");

        environ_copy(src.ptr(), dst.ptr());

        assert_eq!(
            dump(dst.ptr()),
            [
                ("GONE".to_owned(), None, 0),
                ("KEPT".to_owned(), Some("value".to_owned()), ENVIRON_HIDDEN),
                ("OWN".to_owned(), Some("mine".to_owned()), 0),
            ]
        );
    }
}

#[test]
fn updating_copies_what_the_option_matches_and_clears_the_rest() {
    let _guard = globals();
    let oo = Options::session();
    let src = Environ::new();
    let dst = Environ::new();
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"update-environment".as_ptr());
        options_array_clear(o);
        let mut cause: Option<CString> = None;
        for (i, pattern) in [c"SSH_*", c"DISPLAY", c"NEVER"].iter().enumerate() {
            assert_eq!(
                options_array_set(o, i as u_int, pattern.as_ptr(), 0, &mut cause),
                0
            );
        }

        set(src.ptr(), c"SSH_AUTH_SOCK", ENVIRON_HIDDEN, c"/tmp/sock");
        set(src.ptr(), c"SSH_CONNECTION", 0, c"conn");
        set(src.ptr(), c"OTHER", 0, c"other");
        set(dst.ptr(), c"NEVER", 0, c"stale");

        environ_update(oo.ptr(), src.ptr(), dst.ptr());

        assert_eq!(
            dump(dst.ptr()),
            [
                ("DISPLAY".to_owned(), None, 0),
                ("NEVER".to_owned(), None, 0),
                ("SSH_AUTH_SOCK".to_owned(), Some("/tmp/sock".to_owned()), 0),
                ("SSH_CONNECTION".to_owned(), Some("conn".to_owned()), 0),
            ]
        );
    }
}

#[test]
fn updating_without_the_option_does_nothing() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    let src = Environ::new();
    let dst = Environ::new();
    unsafe {
        set(src.ptr(), c"DISPLAY", 0, c":0");
        environ_update(oo.ptr(), src.ptr(), dst.ptr());
        assert!(names(dst.ptr()).is_empty());
    }
}

#[test]
fn pushing_sets_what_is_visible_and_named_and_has_a_value() {
    let _guard = globals();
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"C2RS_PUSHED", 0, c"yes");
        set(env.ptr(), c"C2RS_HIDDEN", ENVIRON_HIDDEN, c"no");
        set(env.ptr(), c"", 0, c"nameless");
        environ_clear(env.ptr(), c"C2RS_CLEARED".as_ptr());

        let saved = process_environment();
        environ_push(env.ptr());
        let pushed = seen(getenv(c"C2RS_PUSHED".as_ptr()));
        let hidden = getenv(c"C2RS_HIDDEN".as_ptr());
        let cleared = getenv(c"C2RS_CLEARED".as_ptr());
        restore_process_environment(&saved);

        assert_eq!(pushed, "yes");
        assert_eq!(hidden, null_mut());
        assert_eq!(cleared, null_mut());
        assert!(getenv(c"C2RS_PUSHED".as_ptr()).is_null());
    }
}

#[test]
fn logging_walks_every_entry_that_has_a_name_and_a_value() {
    let env = Environ::new();
    unsafe {
        set(env.ptr(), c"ONE", 0, c"1");
        set(env.ptr(), c"", 0, c"nameless");
        environ_clear(env.ptr(), c"CLEARED".as_ptr());
        environ_log(env.ptr(), c"%s: ".as_ptr(), fmt_args![c"prefix".as_ptr()]);
        assert_eq!(names(env.ptr()), ["", "CLEARED", "ONE"]);
    }
}

#[test]
fn a_session_environment_is_the_global_one_plus_the_terminal_and_tmux() {
    let _guard = globals();
    unsafe {
        set(global_environ, c"C2RS_GLOBAL", 0, c"global");
        let saved = socket_path;
        socket_path = c"/tmp/c2rs.sock".as_ptr();

        let env = Environ::from_box(environ_for_session(null_mut::<session>(), 0));
        assert_eq!(value(env.ptr(), c"C2RS_GLOBAL"), Some("global".to_owned()));
        assert_eq!(value(env.ptr(), c"TERM_PROGRAM"), Some("tmux".to_owned()));
        assert_eq!(
            value(env.ptr(), c"TERM_PROGRAM_VERSION"),
            Some("3.7b".to_owned())
        );
        assert_eq!(value(env.ptr(), c"COLORTERM"), Some("truecolor".to_owned()));
        assert!(value(env.ptr(), c"TERM").is_some());
        assert_eq!(value(env.ptr(), c"LISTEN_PID"), None);
        assert_eq!(value(env.ptr(), c"LISTEN_FDS"), None);
        assert_eq!(value(env.ptr(), c"LISTEN_FDNAMES"), None);
        assert_eq!(
            value(env.ptr(), c"TMUX"),
            Some(format!("/tmp/c2rs.sock,{},-1", getpid()))
        );

        socket_path = saved;
        environ_unset(global_environ, c"C2RS_GLOBAL".as_ptr());
    }
}

#[test]
fn a_session_environment_can_leave_the_terminal_out_and_takes_the_session_over() {
    let _guard = globals();
    let mut s = Session::new(9, "envtest");
    unsafe {
        let saved = socket_path;
        socket_path = c"/tmp/c2rs.sock".as_ptr();
        set(s.environ(), c"C2RS_SESSION", 0, c"session");

        let env = Environ::from_box(environ_for_session(s.ptr(), 1));
        assert_eq!(
            value(env.ptr(), c"C2RS_SESSION"),
            Some("session".to_owned())
        );
        assert_eq!(value(env.ptr(), c"TERM_PROGRAM"), None);
        assert_eq!(value(env.ptr(), c"COLORTERM"), None);
        assert_eq!(
            value(env.ptr(), c"TMUX"),
            Some(format!("/tmp/c2rs.sock,{},9", getpid()))
        );

        socket_path = saved;
    }
}
