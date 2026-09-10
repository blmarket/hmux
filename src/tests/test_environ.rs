use super::*;
use crate::environ::EnvironmentStore;
use crate::environ::process_environment_value;
use crate::environ::{
    environment_for_session, log_environment, push_environment_to_process, update_environment,
};
use crate::environ::{with_global_environment, with_global_environment_mut};
use crate::ffi::{setenv, unsetenv};
use crate::fmt_args;
use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};
use crate::tests::test_fixtures::{Options, Session, globals};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

fn process_environment() -> Vec<(CString, CString)> {
    unsafe {
        let mut out = Vec::new();
        for entry in crate::environ::process_environment() {
            let entry = entry.to_bytes();
            if let Some(at) = entry.iter().position(|byte| *byte == b'=') {
                out.push((
                    CString::new(&entry[..at]).expect("no NUL"),
                    CString::new(&entry[at + 1..]).expect("no NUL"),
                ));
            }
        }
        out
    }
}

#[test]
fn process_environment_snapshots_own_bytes_before_iteration() {
    let _guard = globals();
    let saved = process_environment();
    unsafe {
        assert_eq!(
            setenv(c"HMUX_ENV_SNAPSHOT".as_ptr(), c"before=\xff".as_ptr(), 1),
            0
        );
        let mut snapshot = crate::environ::process_environment();
        assert_eq!(unsetenv(c"HMUX_ENV_SNAPSHOT".as_ptr()), 0);
        assert_eq!(
            setenv(c"HMUX_ENV_SNAPSHOT".as_ptr(), c"after".as_ptr(), 1),
            0
        );
        restore_process_environment(&saved);
        let entry = snapshot.find(|entry| entry.to_bytes().starts_with(b"HMUX_ENV_SNAPSHOT="));
        assert_eq!(entry.as_deref(), Some(c"HMUX_ENV_SNAPSHOT=before=\xff"));
    }
}

fn restore_process_environment(saved: &[(CString, CString)]) {
    unsafe {
        for (name, _) in &process_environment() {
            unsetenv(name.as_ptr());
        }
        for (name, value) in saved {
            setenv(name.as_ptr(), value.as_ptr(), 1);
        }
    }
}

#[test]
fn process_environment_values_preserve_empty_and_owned_non_utf8_bytes() {
    let _guard = globals();
    let saved = process_environment();
    unsafe {
        let name = c"HMUX_ENV_VALUE";
        assert_eq!(unsetenv(name.as_ptr()), 0);
        let missing = process_environment_value(name);
        assert_eq!(setenv(name.as_ptr(), c"".as_ptr(), 1), 0);
        let empty = process_environment_value(name);
        assert_eq!(setenv(name.as_ptr(), c"before=\xff".as_ptr(), 1), 0);
        let before = process_environment_value(name);
        assert_eq!(setenv(name.as_ptr(), c"after".as_ptr(), 1), 0);
        let after = process_environment_value(name);
        restore_process_environment(&saved);
        assert!(missing.is_none());
        assert_eq!(empty.as_deref(), Some(c""));
        assert_eq!(before.as_deref(), Some(c"before=\xff"));
        assert_eq!(after.as_deref(), Some(c"after"));
    }
}

fn dump(env: &RustEnvironment) -> Vec<(String, Option<String>, c_int)> {
    env.entries()
        .map(|entry| {
            (
                entry.name.to_string_lossy().into_owned(),
                entry
                    .value
                    .map(|value| value.to_string_lossy().into_owned()),
                entry.flags,
            )
        })
        .collect()
}

fn value(env: &RustEnvironment, name: &CStr) -> Option<String> {
    env.find(name)
        .and_then(|entry| entry.value)
        .map(|value| value.to_string_lossy().into_owned())
}

#[test]
fn updating_copies_what_the_option_matches_and_clears_the_rest() {
    let _guard = globals();
    let oo = Options::session();
    let mut src = RustEnvironment::empty();
    let mut dst = RustEnvironment::empty();
    unsafe {
        oo.with_entry_mut(c"update-environment", false, |entry| {
            let entry = entry.unwrap();
            RustOptionsEngine.array_clear(entry);
            let mut cause: Option<CString> = None;
            for (i, pattern) in [c"SSH_*", c"DISPLAY", c"NEVER"].iter().enumerate() {
                assert_eq!(
                    RustOptionsEngine.array_set(entry, i as u_int, Some(pattern), 0, &mut cause),
                    0
                );
            }
        });
        src.set(c"SSH_AUTH_SOCK", ENVIRON_HIDDEN, c"/tmp/sock");
        src.set(c"SSH_CONNECTION", 0, c"conn");
        src.set(c"OTHER", 0, c"other");
        dst.set(c"NEVER", 0, c"stale");
        update_environment(&*oo, &src, &mut dst);
    }
    assert_eq!(
        dump(&dst),
        [
            ("DISPLAY".to_owned(), None, 0),
            ("NEVER".to_owned(), None, 0),
            ("SSH_AUTH_SOCK".to_owned(), Some("/tmp/sock".to_owned()), 0),
            ("SSH_CONNECTION".to_owned(), Some("conn".to_owned()), 0),
        ]
    );
}

#[test]
fn updating_without_the_option_does_nothing() {
    let _guard = globals();
    let oo = Options::empty(None);
    let mut src = RustEnvironment::empty();
    let mut dst = RustEnvironment::empty();
    src.set(c"DISPLAY", 0, c":0");
    unsafe { update_environment(&*oo, &src, &mut dst) };
    assert!(dst.entries().next().is_none());
}

#[test]
fn pushing_sets_what_is_visible_and_named_and_has_a_value() {
    let _guard = globals();
    let mut env = RustEnvironment::empty();
    env.set(c"C2RS_PUSHED", 0, c"yes");
    env.set(c"C2RS_HIDDEN", ENVIRON_HIDDEN, c"no");
    env.set(c"", 0, c"nameless");
    env.clear(c"C2RS_CLEARED");
    let saved = process_environment();
    unsafe { push_environment_to_process(&env) };
    let (pushed, hidden, cleared) = unsafe {
        (
            process_environment_value(c"C2RS_PUSHED"),
            process_environment_value(c"C2RS_HIDDEN"),
            process_environment_value(c"C2RS_CLEARED"),
        )
    };
    restore_process_environment(&saved);
    assert_eq!(pushed.as_deref(), Some(c"yes"));
    assert!(hidden.is_none());
    assert!(cleared.is_none());
    assert!(unsafe { process_environment_value(c"C2RS_PUSHED").is_none() });
}

#[test]
fn logging_walks_every_entry_that_has_a_name_and_a_value() {
    let mut env = RustEnvironment::empty();
    env.set(c"ONE", 0, c"1");
    env.set(c"", 0, c"nameless");
    env.clear(c"CLEARED");
    log_environment(&env, c"%s: ", fmt_args![c"prefix".as_ptr()]);
    assert_eq!(
        env.entries()
            .map(|entry| entry.name.to_bytes())
            .collect::<Vec<_>>(),
        [b"".as_slice(), b"CLEARED".as_slice(), b"ONE".as_slice()]
    );
}

#[test]
fn a_session_environment_is_the_global_one_plus_the_terminal_and_tmux() {
    let _guard = globals();
    unsafe {
        with_global_environment_mut(|env| env.set(c"C2RS_GLOBAL", 0, c"global"));
        let saved = socket_path.take();
        socket_path = Some(c"/tmp/c2rs.sock".to_owned());
        let env = environment_for_session(None, 0);
        assert_eq!(value(&env, c"C2RS_GLOBAL"), Some("global".to_owned()));
        assert_eq!(value(&env, c"TERM_PROGRAM"), Some("tmux".to_owned()));
        assert_eq!(
            value(&env, c"TERM_PROGRAM_VERSION"),
            Some("3.7b".to_owned())
        );
        assert_eq!(value(&env, c"COLORTERM"), Some("truecolor".to_owned()));
        assert!(value(&env, c"TERM").is_some());
        assert_eq!(value(&env, c"LISTEN_PID"), None);
        assert_eq!(value(&env, c"LISTEN_FDS"), None);
        assert_eq!(value(&env, c"LISTEN_FDNAMES"), None);
        assert_eq!(
            value(&env, c"TMUX"),
            Some(format!("/tmp/c2rs.sock,{},-1", getpid()))
        );
        socket_path = saved;
        with_global_environment_mut(|env| env.unset(c"C2RS_GLOBAL"));
    }
}

#[test]
fn a_session_environment_can_leave_the_terminal_out_and_takes_the_session_over() {
    let _guard = globals();
    let mut s = Session::new(9, "envtest");
    unsafe {
        let saved = socket_path.take();
        socket_path = Some(c"/tmp/c2rs.sock".to_owned());
        s.environ_mut().set(c"C2RS_SESSION", 0, c"session");
        let env = environment_for_session(Some(s.handle().as_session()), 1);
        assert_eq!(value(&env, c"C2RS_SESSION"), Some("session".to_owned()));
        assert_eq!(value(&env, c"TERM_PROGRAM"), None);
        assert_eq!(value(&env, c"COLORTERM"), None);
        assert_eq!(
            value(&env, c"TMUX"),
            Some(format!("/tmp/c2rs.sock,{},9", getpid()))
        );
        socket_path = saved;
    }
}

#[test]
fn global_environment_is_private_to_each_thread() {
    use crate::environ::reset_global_environment;

    reset_global_environment();
    with_global_environment_mut(|env| env.set(c"OWNER", ENVIRON_HIDDEN, c"parent"));
    std::thread::spawn(|| {
        with_global_environment(|env| assert!(env.find(c"OWNER").is_none()));
        with_global_environment_mut(|env| env.set(c"OWNER", 0, c"worker"));
        reset_global_environment();
        with_global_environment(|env| assert!(env.find(c"OWNER").is_none()));
        with_global_environment_mut(|env| env.set(c"OWNER", 0, c"exit"));
    })
    .join()
    .unwrap();
    with_global_environment(|env| {
        let entry = env.find(c"OWNER").unwrap();
        assert_eq!(entry.value, Some(c"parent"));
        assert_eq!(entry.flags, ENVIRON_HIDDEN);
    });
    reset_global_environment();
}

#[test]
fn global_environment_checks_reentry_and_recovers_after_rejection() {
    use crate::environ::reset_global_environment;

    reset_global_environment();
    with_global_environment_mut(|env| env.set(c"VALUE", 0, c"original"));
    with_global_environment(|env| {
        with_global_environment(|nested| {
            assert_eq!(value(env, c"VALUE"), value(nested, c"VALUE"));
        });
        assert!(
            std::panic::catch_unwind(|| {
                with_global_environment_mut(|env| env.unset(c"VALUE"));
            })
            .is_err()
        );
        assert_eq!(value(env, c"VALUE").as_deref(), Some("original"));
    });
    with_global_environment_mut(|env| {
        assert!(
            std::panic::catch_unwind(|| {
                with_global_environment(|_| ());
            })
            .is_err()
        );
        env.set(c"VALUE", 0, c"updated");
    });
    with_global_environment(|env| {
        assert_eq!(value(env, c"VALUE").as_deref(), Some("updated"));
    });
    reset_global_environment();
}
