//! Unit tests for [`crate::cmd::cmd_lock_server`] — the three lock commands
//! `lock-server`, `lock-session` and `lock-client`, the constants this
//! generated file declares, and `cmd_lock_server_exec`, the one hook behind
//! all three `exec` pointers.
//!
//! The hook picks its work by comparing the running command's entry against
//! the three entries: the whole server is walked for `lock-server`, only the
//! clients of the target session for `lock-session`, and one target client
//! for `lock-client`; every run ends in `recalculate_sizes` and answers
//! normal.
//!
//! Locking a client for real would stop its terminal and send it a message,
//! so each walk here is arranged to leave `server_lock_client` by one of its
//! deterministic early exits: a control or suspended flag on the client, or a
//! session whose `lock-command` option has been emptied. Nothing is written
//! to a terminal, no message leaves the process, and the assertions read the
//! flags the walks did *not* set.

use crate::arguments::{args_get, args_has};
use crate::client::CLIENT_CONTROL;
use crate::cmd::cmd_lock_server::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CMD_AFTERHOOK, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT, CMD_TARGET_CLIENT_USAGE,
    CMD_TARGET_SESSION_USAGE, MSG_LOCK, MSG_UNLOCK, MSG_VERSION, cmd_lock_client_entry,
    cmd_lock_server_entry, cmd_lock_session_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::fmt_args;
use crate::options::options_set_string;
use crate::resize::{CLIENT_STATUSOFF, CLIENT_SUSPENDED};
use crate::tests::test_fixtures::{Args, Clients, Item, Session, globals, seen};
use crate::types::*;
use ::core::ffi::{CStr, c_char};

/// Points the item's target find state at `s`, as a resolved `-t` would have
/// left it for the hook to pick up through
/// [`cmdq_get_target`](crate::cmd::cmdq_get_target).
unsafe fn aimed(item: &mut Item, s: *mut session) {
    unsafe {
        (*item.ptr()).target.set_session(s);
    }
}

/// Runs the item's parsed command through `entry`'s exec hook, the way the
/// command queue would.
unsafe fn run(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe { ((*entry).exec)(&*item.cmd(), item.ptr()) }
}

#[test]
fn the_entries_describe_three_lock_commands_sharing_one_exec() {
    unsafe {
        let server = &raw const cmd_lock_server_entry;
        let session = &raw const cmd_lock_session_entry;
        let client = &raw const cmd_lock_client_entry;
        assert_ne!(server, session);
        assert_ne!(session, client);
        assert_ne!(server, client);

        assert_eq!((*server).name.to_string_lossy(), "lock-server");
        assert_eq!(
            (*server)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "lock"
        );
        assert_eq!((*session).name.to_string_lossy(), "lock-session");
        assert_eq!(
            (*session)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "locks"
        );
        assert_eq!((*client).name.to_string_lossy(), "lock-client");
        assert_eq!(
            (*client)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "lockc"
        );

        assert_eq!((*server).args.template.to_string_lossy(), "");
        assert_eq!((*session).args.template.to_string_lossy(), "t:");
        assert_eq!((*client).args.template.to_string_lossy(), "t:");
        for e in [server, session, client] {
            assert_eq!((*e).args.lower, 0);
            assert_eq!((*e).args.upper, 0);
            assert!((*e).args.cb.is_none());
        }

        assert_eq!((*server).usage.to_string_lossy(), "");
        assert_eq!((*session).usage.to_string_lossy(), "[-t target-session]");
        assert_eq!((*client).usage.to_string_lossy(), "[-t target-client]");

        for e in [server, session, client] {
            assert_eq!((*e).source.flag, 0);
            assert_eq!((*e).source.type_0, CMD_FIND_PANE);
            assert_eq!((*e).source.flags, 0);
        }
        assert_eq!((*server).target.flag, 0);
        assert_eq!((*server).target.type_0, CMD_FIND_PANE);
        assert_eq!((*server).target.flags, 0);
        assert_eq!((*session).target.flag, b't' as c_char);
        assert_eq!((*session).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*session).target.flags, 0);
        assert_eq!((*client).target.flag, 0);
        assert_eq!((*client).target.type_0, CMD_FIND_PANE);
        assert_eq!((*client).target.flags, 0);

        assert_eq!((*server).flags, CMD_AFTERHOOK);
        assert_eq!((*session).flags, CMD_AFTERHOOK);
        assert_eq!((*client).flags, CMD_AFTERHOOK | CMD_CLIENT_TFLAG);
        assert!(::core::ptr::fn_addr_eq((*server).exec, (*session).exec));
        assert!(::core::ptr::fn_addr_eq((*session).exec, (*client).exec));
    }
}

#[test]
fn the_declared_constants_pin_the_values_the_hook_and_server_read() {
    assert_eq!(MSG_LOCK, 206);
    assert_eq!(MSG_UNLOCK, 215);
    assert_eq!(MSG_VERSION, 12);

    assert_eq!(ARGS_PARSE_INVALID, 0);
    assert_eq!(ARGS_PARSE_STRING, 1);
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(ARGS_PARSE_COMMANDS, 3);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);

    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);

    assert_eq!(CMD_AFTERHOOK, 0x4);
    assert_eq!(CMD_CLIENT_TFLAG, 0x10);

    unsafe {
        assert_eq!(
            CStr::from_ptr(CMD_TARGET_SESSION_USAGE.as_ptr()).to_bytes(),
            b"[-t target-session]"
        );
        assert_eq!(
            CStr::from_ptr(CMD_TARGET_CLIENT_USAGE.as_ptr()).to_bytes(),
            b"[-t target-client]"
        );
    }
}

#[test]
fn parsing_resolves_both_names_of_every_lock_command_to_its_entry() {
    let _guard = globals();
    unsafe {
        for line in [c"lock-server", c"lock"] {
            let parsed = Args::parse(line);
            assert!(
                ::core::ptr::eq((*parsed.cmd()).entry, &cmd_lock_server_entry),
                "{line:?} did not resolve"
            );
            assert_eq!(args_has(&*parsed.ptr(), b't'), 0);
        }

        let session = Args::parse(c"lock-session -t 0");
        assert!(::core::ptr::eq(
            (*session.cmd()).entry,
            &cmd_lock_session_entry
        ));
        assert_eq!(args_has(&*session.ptr(), b't'), 1);
        assert_eq!(seen(args_get(&*session.ptr(), b't')), "0");

        let alias = Args::parse(c"locks");
        assert!(::core::ptr::eq(
            (*alias.cmd()).entry,
            &cmd_lock_session_entry
        ));

        for line in [c"lock-client", c"lockc"] {
            let parsed = Args::parse(line);
            assert!(
                ::core::ptr::eq((*parsed.cmd()).entry, &cmd_lock_client_entry),
                "{line:?} did not resolve"
            );
        }
    }
}

/// One `lock-server` run walks the whole client list; here every stop along
/// that walk is an early exit — a client with no session is skipped outright,
/// a control-flagged and a suspended client are refused before anything is
/// read, and the last one's session carries an empty `lock-command`. No
/// client comes away suspended and nothing reaches a terminal or a peer.
#[test]
fn exec_of_lock_server_walks_every_client_and_suspends_none_of_them() {
    let _guard = globals();
    let mut clients = Clients::new();
    let bare = clients.add("bare", 80, 24);
    let mut quiet = Session::new(50, "quiet");
    let plain = clients.add("plain", 80, 24);
    let controlled = clients.add("controlled", 80, 24);
    let suspended = clients.add("suspended", 80, 24);
    unsafe {
        options_set_string(
            quiet.options(),
            c"lock-command".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"".as_ptr()],
        );
        (*plain).session = quiet.ptr();
        (*controlled).session = quiet.ptr();
        (*controlled).flags |= CLIENT_CONTROL as uint64_t;
        (*suspended).session = quiet.ptr();
        (*suspended).flags |= CLIENT_SUSPENDED as uint64_t;

        let mut item = Item::new().with_args(c"lock-server");
        assert_eq!(
            run(&raw const cmd_lock_server_entry, &mut item),
            CMD_RETURN_NORMAL
        );

        assert!((*bare).session.is_null());
        assert_eq!((*bare).flags, 0);
        assert_eq!((*plain).flags, 0);
        assert_eq!(
            (*controlled).flags & CLIENT_CONTROL as uint64_t,
            CLIENT_CONTROL as uint64_t
        );
        assert_eq!(
            (*suspended).flags & CLIENT_SUSPENDED as uint64_t,
            CLIENT_SUSPENDED as uint64_t
        );
        for c in [plain, controlled, suspended] {
            assert_eq!((*c).session, quiet.ptr());
            assert_eq!((*c).flags & CLIENT_STATUSOFF as uint64_t, 0);
        }
    }
}

#[test]
fn exec_of_lock_session_answers_normal_and_leaves_other_sessions_clients_alone() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(52, "home");
    let mut away = Session::new(53, "away");
    let resident = clients.add("resident", 80, 24);
    let visitor = clients.add("visitor", 80, 24);
    unsafe {
        (*resident).session = home.ptr();
        (*visitor).session = away.ptr();
        (*resident).flags |= CLIENT_CONTROL as uint64_t;
        (*visitor).flags |= CLIENT_SUSPENDED as uint64_t;

        let mut item = Item::new().with_args(c"lock-session");
        aimed(&mut item, home.ptr());

        assert_eq!(
            run(&raw const cmd_lock_session_entry, &mut item),
            CMD_RETURN_NORMAL
        );

        assert_eq!((*resident).session, home.ptr());
        assert_eq!((*visitor).session, away.ptr());
        assert_eq!((*resident).flags, CLIENT_CONTROL as uint64_t);
        assert_eq!((*visitor).flags, CLIENT_SUSPENDED as uint64_t);
    }
}

#[test]
fn exec_of_lock_client_answers_normal_for_an_already_suspended_target() {
    let _guard = globals();
    let mut clients = Clients::new();
    let held = clients.add("held", 80, 24);
    unsafe {
        (*held).flags |= CLIENT_SUSPENDED as uint64_t;

        let mut item = Item::with_client().with_args(c"lock-client");
        cmdq_set_target_client(item.ptr(), held);

        assert_eq!(
            run(&raw const cmd_lock_client_entry, &mut item),
            CMD_RETURN_NORMAL
        );
        assert_eq!(
            (*held).flags & CLIENT_SUSPENDED as uint64_t,
            CLIENT_SUSPENDED as uint64_t
        );
        assert_eq!((*held).flags & CLIENT_CONTROL as uint64_t, 0);
    }
}
