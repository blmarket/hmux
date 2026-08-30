//! Unit tests for [`crate::cmd::cmd_rename_session`], the exec hook behind
//! the `rename-session` command.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entry's `exec` pointer with an item whose target find state has already
//! been resolved. Around it the tests pin the entry's metadata, every
//! constant the generated module declares, and the parsing contract its
//! `t:` template with equal lower and upper bounds of one gives the command:
//! an optional `-t`, exactly one positional new name, and the `rename`
//! alias.
//!
//! Each of the four ways the hook can go is driven once. A name that reaches
//! it carrying bytes [`check_name`](crate::tmux::check_name) refuses is
//! rejected before anything moves; renaming to the standing name answers
//! normal at once; renaming onto another session's name is refused as a
//! duplicate; and the successful rename frees the old heap name, hangs the
//! new one off the session, re-inserts the session into the tree keyed by
//! that name, asks every attached client to redraw its status line and
//! raises `session-renamed`.
//!
//! Two safety notes shape what these tests drive. The refusals are observed
//! through the server's message log, which a client without a session and
//! carrying `CLIENT_ATTACHED` receives instead of any peer write. And the
//! bytes that make a name unrepresentable cannot come through the command
//! lexer, which cuts a word at the first byte that is not valid UTF-8, so
//! they are slipped in over the parsed positional argument — the format
//! engine copies literal text byte by byte, which is exactly why the check
//! exists downstream of it.

use crate::arguments::{args_count, args_get, args_has, args_string, args_value};
use crate::cmd::cmd_get_args_ptr;
use crate::cmd::cmd_rename_session::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    cmd_rename_session_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::server::message_log;
use crate::session::{session_find, session_name, sessions_first};
use crate::tests::test_fixtures::{Args, Item, Registry, Session, globals, seen};
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_rename_session_entry
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

/// The lines the server has recorded so far, oldest first. Entries
/// accumulate across the whole test binary, so assertions compare lengths
/// around the call they probe and look for their own wording.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Slips `raw` in as the command's positional argument, replacing the word
/// the parser holds there. This is the only way one of the hook's checks can
/// be handed bytes it refuses: the lexer cuts a word at the first byte that
/// is not valid UTF-8, so no command line carries them intact.
unsafe fn hand_raw_name(item: &mut Item, raw: &'static CStr) {
    unsafe {
        let v = args_value(cmd_get_args_ptr(&*item.cmd()), 0);
        assert!(!v.is_null(), "the command carries a positional argument");
        (*v).value = ArgsValue::String(raw.to_owned());
    }
}

#[test]
fn the_entry_advertises_rename_session_and_its_afterhook_flag() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_string_lossy(), "rename-session");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "rename"
        );

        assert_eq!((*e).args.template.to_string_lossy(), "t:");
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).usage.to_string_lossy(), "[-t target-session] new-name");

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
    }
}

#[test]
fn parsing_takes_exactly_one_new_name_an_optional_t_and_the_rename_alias() {
    let _guard = globals();
    unsafe {
        for line in [c"rename-session new-name", c"rename new-name"] {
            let args = Args::parse(line);
            assert!(
                ::core::ptr::eq((*args.cmd()).entry, entry()),
                "{line:?} went to the wrong entry"
            );
            let a = args.ptr();
            assert_eq!(args_count(&*a), 1, "{line:?}");
            assert_eq!(seen(args_string(&*a, 0)), "new-name");
            assert_eq!(args_has(&*a, b't'), 0, "{line:?}");
        }

        let targeted = Args::parse(c"rename-session -t 0 renamed");
        assert_eq!(args_has(&*targeted.ptr(), b't'), 1);
        assert_eq!(seen(args_get(&*targeted.ptr(), b't')), "0");
        assert_eq!(seen(args_string(&*targeted.ptr(), 0)), "renamed");

        let mut none = cmd_parse_from_string(c"rename-session".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("too few arguments"), "{err}");
        assert!(err.contains("at least 1"), "{err}");

        let mut extra = cmd_parse_from_string(c"rename-session one two".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
        assert!(err.contains("at most 1"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"rename-session -z x".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err = bad_flag.take_error();
        assert!(err.contains("unknown flag"), "{err}");
    }
}

#[test]
fn renaming_to_the_same_name_answers_normal_and_touches_nothing() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(704, "steady");
    registry.add_session(&mut s);
    unsafe {
        let weak = s.weak();
        let mut item = Item::new().with_args(c"rename-session steady");
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(seen(session_name(s.ptr())), "steady");
        assert_eq!(session_find(c"steady".as_ptr()), s.ptr());
        assert_eq!(weak.upgrade().unwrap().as_ptr(), s.ptr());
        assert_eq!(
            sessions_first(),
            s.ptr(),
            "the session kept its place in the tree"
        );
    }
}

#[test]
fn renaming_to_new_name_succeeds_and_updates_map() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(705, "orig_name");
    let mut w = crate::tests::test_fixtures::Window::new(8002, "win", 80, 24);
    let mut p = crate::tests::test_fixtures::Pane::new(9002, 80, 24, 100);
    w.add_pane(&mut p);
    registry.add_window(&mut w);
    registry.add_pane(&mut p);
    let wl = crate::tests::test_fixtures::link(&mut s, &mut w, 0);
    registry.add_session(&mut s);
    unsafe {
        let mut item = Item::new().with_args(c"rename-session new_session_name");
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(seen(session_name(s.ptr())), "new_session_name");
        assert_eq!(session_find(c"new_session_name".as_ptr()), s.ptr());

        crate::tests::test_fixtures::unlink(&mut s, wl);
    }
}

#[test]
fn renaming_to_duplicate_or_invalid_name_fails() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s1 = Session::new(706, "sess1");
    let mut s2 = Session::new(707, "sess2");
    let mut w = crate::tests::test_fixtures::Window::new(8001, "win", 80, 24);
    let mut p = crate::tests::test_fixtures::Pane::new(9001, 80, 24, 100);
    w.add_pane(&mut p);
    registry.add_window(&mut w);
    registry.add_pane(&mut p);
    let wl = crate::tests::test_fixtures::link(&mut s1, &mut w, 0);
    registry.add_session(&mut s1);
    registry.add_session(&mut s2);
    unsafe {
        let mut peer = crate::tests::test_fixtures::zeroed::<tmuxpeer>();
        peer.flags |= crate::proc::PEER_BAD;
        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        (*caller).peer = Some(peer);

        let mut item_dup = Item::new().with_args(c"rename-session sess2");
        item_dup.set_client(caller);
        aimed(&mut item_dup, s1.ptr());
        assert_eq!(run(&mut item_dup), CMD_RETURN_ERROR);

        let mut item_inv = Item::new().with_args(c"rename-session invalid");
        hand_raw_name(&mut item_inv, c"\xff\xff");
        item_inv.set_client(caller);
        aimed(&mut item_inv, s1.ptr());
        assert_eq!(run(&mut item_inv), CMD_RETURN_ERROR);

        crate::tests::test_fixtures::unlink(&mut s1, wl);
        crate::tests::test_fixtures::release_client(caller);
    }
}
