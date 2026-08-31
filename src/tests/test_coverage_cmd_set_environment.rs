//! Unit tests for [`crate::cmd::cmd_set_environment`], the exec hook behind
//! the `set-environment` command.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entry's `exec` pointer with an item whose target find state has already
//! been resolved. Around it the tests pin the entry's metadata, every
//! constant the generated module declares, and the parsing contract its
//! `Fhgrt:u` template with bounds of one and two gives the command: the six
//! switches, an optional `-t`, and one or two positional words.
//!
//! Each way the hook can go is driven once. A name that is empty or carries
//! an equals sign is refused before anything moves — neither can arrive
//! through the command lexer, which turns `NAME=value` into an assignment
//! token, so those bytes are slipped in over the parsed positional argument,
//! which copies them byte for byte. A run with no resolved session refuses
//! and says whether a `-t` was named; otherwise the variable lands in the
//! target session's environment — set, hidden with `-h`, removed by `-u` or
//! cleared by `-r` — or in the global environment under `-g`, and `-F`
//! expands the value through the target before storing it.
//!
//! Refusals are observed twice over: through the return value and through
//! the wording filed with the server's message log, which a client carrying
//! `CLIENT_ATTACHED` receives because `file_error` declines to open a stream
//! to the absent peer.

use crate::arguments::{args_count, args_get, args_has, args_string, args_value};
use crate::cmd::cmd_set_environment::{
    CMD_AFTERHOOK, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_NORMAL,
    ENVIRON_HIDDEN, cmd_set_environment_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::environ::{
    environ_entries, environ_entry_flags, environ_entry_name, environ_entry_value, environ_find,
    environ_set, environ_t, environ_unset,
};
use crate::fmt_args;
use crate::server::CLIENT_ATTACHED;
use crate::server::message_log;
use crate::tests::test_fixtures::{Args, Clients, Item, Session, globals, seen};
use crate::tmux::global_environ;
use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_set_environment_entry
}

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe { ((*entry()).exec)(&*item.cmd(), item.ptr()) }
}

/// Points the item's target find state at `s`, as a resolved `-t` would have
/// left it for the hook to pick up through
/// [`cmdq_get_target`](crate::cmd::cmdq_get_target).
unsafe fn aimed(item: &mut Item, s: *mut session) {
    unsafe {
        (*item.ptr()).target.set_session(s);
    }
}

/// An item whose errors reach the server's message log: its client carries
/// `CLIENT_ATTACHED`, so `file_error` stays out of the way.
fn observed(clients: &mut Clients, name: &str) -> Item {
    let c = clients.add(name, 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as uint64_t;
        let mut item = Item::new();
        item.set_client(c);
        item
    }
}

/// The last message the server has logged. Entries accumulate across the
/// whole binary, so assertions look for their own wording rather than count
/// lines; the newest entry always sits at the tail.
unsafe fn latest_message() -> String {
    unsafe {
        let mut out = String::new();
        for m in message_log.queue().iter() {
            out = seen(m.msg.as_ptr());
        }
        out
    }
}

/// Slips `raw` in as the command's positional name, replacing the word the
/// parser holds there. This is the only way the hook's first two checks can
/// be handed what they refuse: the lexer cuts words at whitespace and turns
/// `NAME=value` into an assignment token, so no command line carries an
/// empty name or one holding an equals sign intact.
unsafe fn hand_raw_name(item: &mut Item, raw: &'static CStr) {
    unsafe {
        let v = args_value(item.args_ptr(), 0);
        assert!(!v.is_null(), "the command carries a positional argument");
        (*v).value = ArgsValue::String(raw.to_owned());
    }
}

/// The value of one entry, if it has one.
fn value_seen(envent: &environ_entry) -> Option<String> {
    environ_entry_value(envent).map(|value| value.to_string_lossy().into_owned())
}

/// Every entry of `env` in name order: name, value and flags.
unsafe fn dump(env: *mut environ_t) -> Vec<(String, Option<String>, c_int)> {
    unsafe {
        environ_entries(&*env)
            .map(|envent| {
                (
                    seen(environ_entry_name(envent).as_ptr()),
                    value_seen(envent),
                    environ_entry_flags(envent),
                )
            })
            .collect()
    }
}

/// The value of one name, if the entry is there and has one.
unsafe fn value_of(env: *mut environ_t, name: &CStr) -> Option<String> {
    unsafe { environ_find(&*env, name.as_ptr()).and_then(|envent| value_seen(envent)) }
}

/// The flags of one name, if the entry is there.
unsafe fn flags_of(env: *mut environ_t, name: &CStr) -> Option<c_int> {
    unsafe { environ_find(&*env, name.as_ptr()).map(|envent| environ_entry_flags(envent)) }
}

#[test]
fn the_entry_advertises_set_environment_and_its_afterhook_flag() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_string_lossy(), "set-environment");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "setenv"
        );

        assert_eq!((*e).args.template.to_string_lossy(), "Fhgrt:u");
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, 2);
        assert!((*e).args.cb.is_none());

        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-Fhgru] [-t target-session] variable [value]"
        );

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
    }
}

#[test]
fn parsing_takes_one_or_two_words_every_declared_switch_and_the_setenv_alias() {
    let _guard = globals();
    unsafe {
        for line in [c"set-environment VAR value", c"setenv VAR value"] {
            let parsed = Args::parse(line);
            assert!(
                ::core::ptr::eq((*parsed.cmd()).entry, entry()),
                "{line:?} went to the wrong entry"
            );
            let a = parsed.ptr();
            assert_eq!(args_count(&*a), 2, "{line:?}");
            assert_eq!(seen(args_string(&*a, 0)), "VAR", "{line:?}");
            assert_eq!(seen(args_string(&*a, 1)), "value", "{line:?}");
            for flag in *b"Fhgrut" {
                assert_eq!(args_has(&*a, flag), 0, "{line:?}");
            }
        }

        let all = Args::parse(c"set-environment -Fhgrt 7 -u NAME");
        let a = all.ptr();
        assert_eq!(args_has(&*a, b'F'), 1);
        assert_eq!(args_has(&*a, b'h'), 1);
        assert_eq!(args_has(&*a, b'g'), 1);
        assert_eq!(args_has(&*a, b'r'), 1);
        assert_eq!(args_has(&*a, b'u'), 1);
        assert_eq!(args_has(&*a, b't'), 1);
        assert_eq!(seen(args_get(&*a, b't')), "7");
        assert_eq!(args_count(&*a), 1);

        let mut none = cmd_parse_from_string(c"set-environment".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("too few arguments"), "{err}");
        assert!(err.contains("at least 1"), "{err}");

        let mut extra =
            cmd_parse_from_string(c"set-environment one two three".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
        assert!(err.contains("at most 2"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"set-environment -z x".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err = bad_flag.take_error();
        assert!(err.contains("unknown flag"), "{err}");
    }
}

#[test]
fn values_land_in_the_sessions_environment_and_h_can_hide_them() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(703, "plain");
    unsafe {
        let mut open =
            observed(&mut clients, "opener").with_args(c"set-environment VAR some-value");
        aimed(&mut open, s.ptr());
        assert_eq!(run(&mut open), CMD_RETURN_NORMAL);

        let mut hidden =
            observed(&mut clients, "hider").with_args(c"set-environment -h HIDDEN secret");
        aimed(&mut hidden, s.ptr());
        assert_eq!(run(&mut hidden), CMD_RETURN_NORMAL);

        assert_eq!(
            dump(s.environ()),
            [
                (
                    "HIDDEN".to_owned(),
                    Some("secret".to_owned()),
                    ENVIRON_HIDDEN
                ),
                ("VAR".to_owned(), Some("some-value".to_owned()), 0),
            ]
        );
    }
}

#[test]
fn g_targets_the_global_environment_for_setting_and_unsetting() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        environ_set(
            global_environ,
            c"G_UNSET".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"old".as_ptr()],
        );

        let mut set =
            observed(&mut clients, "global").with_args(c"set-environment -g G_SET gvalue");
        assert_eq!(run(&mut set), CMD_RETURN_NORMAL);
        assert_eq!(
            value_of(global_environ, c"G_SET"),
            Some("gvalue".to_owned())
        );

        let mut unset = observed(&mut clients, "gun").with_args(c"set-environment -gu G_UNSET");
        assert_eq!(run(&mut unset), CMD_RETURN_NORMAL);
        assert!(environ_find(&*global_environ, c"G_UNSET".as_ptr()).is_none());
        assert_eq!(flags_of(global_environ, c"G_SET"), Some(0));

        environ_unset(global_environ, c"G_SET".as_ptr());
        assert!(environ_find(&*global_environ, c"G_SET".as_ptr()).is_none());
    }
}

#[test]
fn a_format_value_is_expanded_through_the_target_before_it_is_stored() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(707, "expandee");
    unsafe {
        let mut item = observed(&mut clients, "expander")
            .with_args(c"set-environment -F EXP \"#{session_name}\"");
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(value_of(s.environ(), c"EXP"), Some("expandee".to_owned()));
    }
}
