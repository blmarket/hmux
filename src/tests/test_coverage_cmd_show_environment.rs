//! Unit tests for [`crate::cmd::cmd_show_environment`] — the `show-environment`
//! entry metadata, its parsing contract, the constants the module declares, and
//! every branch of [`cmd_show_environment_exec`] reachable through the entry's
//! exec hook: listing a session's or the global environment, showing one named
//! variable, the `-h` hidden-variable filter, the `-s` shell-export quoting
//! with its metacharacter escaping, and each refusal.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue reaches it, with an item whose target find state has already
//! been resolved — the hook reads only the session out of it. Lines and error
//! wordings are observed by giving the item's client the `CLIENT_CONTROL`
//! flag, so every `cmdq_print` and `cmdq_error` lands in a local buffer event
//! instead of any peer; while an error runs, the server's message log is kept
//! aside and put back exactly as found.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::cmd_show_environment::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::control::control_state;
use crate::environ::{environ_clear, environ_find, environ_set, environ_t, environ_unset};
use crate::fmt_args;
use crate::server::CLIENT_CONTROL;
use crate::server::message_log;
use crate::tests::test_fixtures::{Args, Item, Session, StreamBuffer, globals, seen};
use crate::tmux::global_environ;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
use ::std::collections::VecDeque;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_show_environment_entry
}

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it.
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

/// Puts `name=value` in `env`, flagged hidden when asked.
unsafe fn put(env: *mut environ_t, name: &CStr, value: &CStr, hidden: bool) {
    unsafe {
        environ_set(
            env,
            name.as_ptr(),
            if hidden { ENVIRON_HIDDEN } else { 0 },
            c"%s".as_ptr(),
            fmt_args![value.as_ptr()],
        );
    }
}

/// A control-mode client's write side: the state `control_write` reaches and
/// the buffer event it lands in, over a local socket pair. Nothing runs the
/// event loop, so what was written stays readable through [`Self::written`].
struct Control {
    bev: StreamBuffer,
}

impl Control {
    fn new() -> Control {
        Control {
            bev: StreamBuffer::new(),
        }
    }

    /// Turns the item's client into a control client writing through here.
    fn attach_to(&mut self, item: &mut Item) {
        unsafe {
            let c = item.client();
            let cs = (*c)
                .control_state
                .insert(Box::new(control_state::default()));
            cs.write_event = self.bev.ptr();
            (*c).flags |= CLIENT_CONTROL as uint64_t;
        }
    }

    /// What has been written since the last time this was asked.
    fn written(&self) -> Vec<u8> {
        self.bev.written()
    }
}

/// A turn at the server's message log, taken away for the length of a test so
/// that what the test records is all there is. Put back exactly as found.
struct Aside {
    saved: VecDeque<message_entry>,
}

impl Aside {
    fn take() -> Aside {
        Aside {
            saved: ::core::mem::take(message_log.queue()),
        }
    }
}

impl Drop for Aside {
    fn drop(&mut self) {
        *message_log.queue() = ::core::mem::take(&mut self.saved);
    }
}

#[test]
fn the_entry_advertises_show_environment_and_its_afterhook_flag() {
    let _guard = globals();
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_string_lossy(), "show-environment");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "showenv"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-hgs] [-t target-session] [variable]"
        );

        assert_eq!((*e).args.template.to_string_lossy(), "hgst:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as i32 as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
    }
}

#[test]
fn parsing_takes_hgs_a_target_and_at_most_one_variable() {
    let _guard = globals();
    unsafe {
        for line in [c"show-environment", c"showenv"] {
            let args = Args::parse(line);
            assert!(
                ::core::ptr::eq((*args.cmd()).entry, entry()),
                "{line:?} went to the wrong entry"
            );
            assert_eq!(args_count(&*args.ptr()), 0, "{line:?}");
            assert_eq!(args_has(&*args.ptr(), b'h'), 0, "{line:?}");
            assert_eq!(args_has(&*args.ptr(), b'g'), 0, "{line:?}");
            assert_eq!(args_has(&*args.ptr(), b's'), 0, "{line:?}");
            assert_eq!(args_has(&*args.ptr(), b't'), 0, "{line:?}");
        }

        let mut bare = cmd_parse_from_string(c"show-environment".as_ptr(), null_mut());
        assert_eq!(bare.status, CMD_PARSE_SUCCESS);
        let _ = bare.cmdlist.take();
        let mut alias = cmd_parse_from_string(c"showenv".as_ptr(), null_mut());
        assert_eq!(alias.status, CMD_PARSE_SUCCESS);
        let _ = alias.cmdlist.take();

        let full = Args::parse(c"show-environment -h -g -s -t work VAR");
        let a = full.ptr();
        assert_eq!(args_has(&*a, b'h'), 1);
        assert_eq!(args_has(&*a, b'g'), 1);
        assert_eq!(args_has(&*a, b's'), 1);
        assert_eq!(seen(args_get(&*a, b't')), "work");
        assert_eq!(seen(args_string(&*a, 0)), "VAR");

        let mut extra = cmd_parse_from_string(c"show-environment ONE TWO".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
        assert!(err.contains("at most 1"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"showenv -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err = bad_flag.take_error();
        assert!(err.contains("unknown flag"), "{err}");
    }
}

/// A fixture session whose environment holds two visible variables and one
/// hidden one, inserted out of name order on purpose: the tree walk must sort.
fn session_env() -> Session {
    let mut s = Session::new(9, "work");
    unsafe {
        put(s.environ(), c"B", c"2", false);
        put(s.environ(), c"A", c"1", false);
        put(s.environ(), c"SECRET", c"hush", true);
    }
    s
}

#[test]
fn exec_lists_a_session_environment_sorted_and_skips_hidden_entries() {
    let _guard = globals();
    let mut control = Control::new();
    let mut s = session_env();
    unsafe {
        let mut item = Item::with_client().with_args(c"show-environment");
        control.attach_to(&mut item);
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            control.written(),
            b"A=1\nB=2\n",
            "the hidden entry is left out and the rest come in name order"
        );
    }
}

#[test]
fn exec_shows_only_the_hidden_entries_with_h() {
    let _guard = globals();
    let mut control = Control::new();
    let mut s = session_env();
    unsafe {
        let mut item = Item::with_client().with_args(c"show-environment -h");
        control.attach_to(&mut item);
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"SECRET=hush\n");
    }
}

#[test]
fn exec_applies_the_h_filter_rules_to_a_named_variable() {
    let _guard = globals();
    let mut control = Control::new();
    let mut s = session_env();
    unsafe {
        let cases: [(&CStr, &[u8]); 4] = [
            (c"show-environment B", b"B=2\n"),
            (c"show-environment SECRET", b""),
            (c"show-environment -h SECRET", b"SECRET=hush\n"),
            (c"show-environment -h B", b""),
        ];
        for (line, want) in cases {
            let mut item = Item::with_client().with_args(line);
            control.attach_to(&mut item);
            aimed(&mut item, s.ptr());
            assert_eq!(run(&mut item), CMD_RETURN_NORMAL, "{line:?}");
            assert_eq!(control.written(), want, "{line:?}");
        }
    }
}

#[test]
fn exec_prints_the_dash_and_unset_forms_for_a_variable_with_no_value() {
    let _guard = globals();
    let mut control = Control::new();
    let mut s = Session::new(10, "bare");
    unsafe {
        environ_clear(s.environ(), c"EMPTY".as_ptr());

        let mut plain = Item::with_client().with_args(c"show-environment EMPTY");
        control.attach_to(&mut plain);
        aimed(&mut plain, s.ptr());
        assert_eq!(run(&mut plain), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"-EMPTY\n");

        let mut shell = Item::with_client().with_args(c"show-environment -s EMPTY");
        control.attach_to(&mut shell);
        aimed(&mut shell, s.ptr());
        assert_eq!(run(&mut shell), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"unset EMPTY;\n");
    }
}

#[test]
fn exec_exports_shell_quoted_values_and_escapes_metacharacters_with_s() {
    let _guard = globals();
    let mut control = Control::new();
    let mut s = Session::new(11, "quoted");
    unsafe {
        environ_set(
            s.environ(),
            c"TRICKY".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"a$b\"c\\d`e".as_ptr()],
        );

        let mut item = Item::with_client().with_args(c"show-environment -s TRICKY");
        control.attach_to(&mut item);
        aimed(&mut item, s.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            control.written(),
            b"TRICKY=\"a\\$b\\\"c\\\\d\\`e\"; export TRICKY;\n",
            "each of $ ` \" and backslash gains a backslash inside the quotes"
        );

        let mut plain_value = Item::with_client().with_args(c"show-environment TRICKY");
        control.attach_to(&mut plain_value);
        aimed(&mut plain_value, s.ptr());
        assert_eq!(run(&mut plain_value), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"TRICKY=a$b\"c\\d`e\n");
    }
}

#[test]
fn exec_reports_an_unknown_variable_as_an_error() {
    let _guard = globals();
    let _aside = Aside::take();
    let mut control = Control::new();
    let mut s = session_env();
    unsafe {
        let mut item = Item::with_client()
            .from_file(c"fixture.conf", 4)
            .with_args(c"show-environment NOSUCH");
        control.attach_to(&mut item);
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_ERROR);
        assert_eq!(control.written(), b"unknown variable: NOSUCH\n");
    }
}

#[test]
fn exec_refuses_to_run_without_a_session_unless_g_is_given() {
    let _guard = globals();
    let _aside = Aside::take();
    let mut control = Control::new();
    unsafe {
        let mut targeted = Item::with_client()
            .from_file(c"fixture.conf", 5)
            .with_args(c"show-environment -t nowhere");
        control.attach_to(&mut targeted);
        assert_eq!(run(&mut targeted), CMD_RETURN_ERROR);
        assert_eq!(control.written(), b"no such session: nowhere\n");

        let mut current = Item::with_client()
            .from_file(c"fixture.conf", 6)
            .with_args(c"show-environment");
        control.attach_to(&mut current);
        assert_eq!(run(&mut current), CMD_RETURN_ERROR);
        assert_eq!(control.written(), b"no current session\n");
    }
}

#[test]
fn exec_lists_the_global_environment_with_g_even_without_a_session() {
    let _guard = globals();
    let mut control = Control::new();
    unsafe {
        put(global_environ, c"G_B", c"twenty", false);
        put(global_environ, c"G_A", c"ten", false);
        put(global_environ, c"G_SECRET", c"quiet", true);

        let mut item = Item::with_client().with_args(c"show-environment -g");
        control.attach_to(&mut item);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"G_A=ten\nG_B=twenty\n");

        let mut named = Item::with_client().with_args(c"show-environment -g G_B");
        control.attach_to(&mut named);
        assert_eq!(run(&mut named), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"G_B=twenty\n");

        let mut hidden = Item::with_client().with_args(c"show-environment -g -h");
        control.attach_to(&mut hidden);
        assert_eq!(run(&mut hidden), CMD_RETURN_NORMAL);
        assert_eq!(control.written(), b"G_SECRET=quiet\n");

        environ_unset(global_environ, c"G_A".as_ptr());
        environ_unset(global_environ, c"G_B".as_ptr());
        environ_unset(global_environ, c"G_SECRET".as_ptr());
        assert!(
            environ_find(&*global_environ, c"G_A".as_ptr()).is_none(),
            "the global environment was given back empty"
        );
    }
}

#[test]
fn exec_reads_the_aimed_sessions_environment_when_t_resolves() {
    let _guard = globals();
    let mut control = Control::new();
    let mut s = session_env();
    unsafe {
        put(global_environ, c"A", c"global", false);
        put(global_environ, c"B", c"global", false);

        let mut item = Item::with_client().with_args(c"show-environment -t work");
        control.attach_to(&mut item);
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            control.written(),
            b"A=1\nB=2\n",
            "the session's values win over the global ones of the same names"
        );

        environ_unset(global_environ, c"A".as_ptr());
        environ_unset(global_environ, c"B".as_ptr());
    }
}
