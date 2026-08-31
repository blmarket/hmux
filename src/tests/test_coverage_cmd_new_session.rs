//! Unit tests for [`crate::cmd::cmd_new_session`] — the `new-session` and
//! `has-session` command entries, the protocol and flag constants the module
//! re-exports, and every branch of the shared exec function the fixtures can
//! reach without a live daemon.
//!
//! A note on how far these tests go. The command's successful tail —
//! `session_create` followed by `spawn_window` — ends in a forked pane
//! process, so the tests stop deliberately short of it: they pin the entry
//! metadata, the `has-session` short circuit, each refusal ahead of any
//! creation (target with a command, invalid window/session/group names,
//! duplicate session, bad `-x`/`-y` sizes, a client still inside tmux), and
//! the two `-A` delegations to `cmd_attach_session`, which its own suite
//! covers in depth. Every refusal is observed twice over: through the return
//! value and through the wording the command files with the server's message
//! log (clients here carry `CLIENT_ATTACHED`, so `file_error` stays out of
//! the way), and each one also checks `next_session_id` to prove that no
//! session slipped into existence on the way to the error.

use crate::arguments::args_set;
use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_new_session::{
    __SHRT_MAX__, CLIENT_ATTACHED, CLIENT_CONTROL, CMD_FIND_CANFAIL, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, CMD_STARTSERVER, CMD_TARGET_SESSION_USAGE, CMDQ_STATE_REPEAT, MSG_READY,
    NEW_SESSION_TEMPLATE, RB_NEGINF, USHRT_MAX, cmd_has_session_entry, cmd_new_session_entry,
};
use crate::environ::environ_set;
use crate::fmt_args;
use crate::server::message_log;
use crate::session::next_session_id;
use crate::tests::test_fixtures::{
    Clients, Environ, Item, Registry, Session, Target, globals, seen,
};
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// Everything the server's message log holds. Entries accumulate across the
/// whole test binary, so assertions look for their own wording rather than
/// count lines.
unsafe fn logged_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Runs `entry`'s exec function against `item`, exactly as the command queue
/// would.
unsafe fn run(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe { ((*entry).exec)(&*item.cmd(), item.ptr()) }
}

/// Hands cfg.rs's cause list to `cfg_print_causes`, which frees every entry.
/// With no client behind the item each cause only reaches `log_debug`.
unsafe fn drain_config_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
    }
}

/// The session id counter, read through a raw pointer: Rust 2024 refuses to
/// form a shared reference to a mutable static.
unsafe fn session_id_counter() -> u_int {
    unsafe { ::core::ptr::read(&raw const next_session_id) }
}

/// A client-backed item reporting errors through the message log: the client
/// carries `CLIENT_ATTACHED`, so `cmdq_error` records the wording there and
/// `file_error` declines to open a stream to the absent peer. Its fresh
/// environment is its own, since the format engine reads the client's
/// environment whenever a command formats one of its arguments.
unsafe fn logged_item(clients: &mut Clients, name: &str) -> Item {
    unsafe {
        let c = clients.add(name, 80, 24);
        (*c).flags |= CLIENT_ATTACHED as uint64_t;
        (*c).environ = Some(Environ::new().owned());
        let mut item = Item::new();
        item.set_client(c);
        item
    }
}

/// Takes an item whose command line was parsed with `-x abc` and slips a
/// copy of `raw` in as `flag`'s value. This is the only way one of the name
/// checks can be handed bytes it rejects: the command lexer cuts a word at
/// the first byte that is not valid UTF-8, so no command line carries them
/// intact.
///
/// The `-x abc` in the parsed line is a safety net: should a rejected-name
/// branch somehow not fire, the width parse refuses what remains of the
/// command before any session could be built, so a wrong turn here costs an
/// assertion and never reaches the spawning tail.
unsafe fn item_with_raw_flag_value(mut item: Item, flag: u_char, raw: &'static CStr) -> Item {
    unsafe {
        let args = item.args_ptr();
        let mut value = Box::new(args_value_t::default());
        value.value = ArgsValue::String(raw.to_owned());
        args_set(args, flag, Some(value), 0);
        item
    }
}

#[test]
fn the_new_session_entry_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_new_session_entry;
        assert_eq!((*e).name.to_bytes(), b"new-session");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"new"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"Ac:dDe:EF:f:n:Ps:t:x:Xy:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);

        assert_eq!((*e).flags & CMD_STARTSERVER, CMD_STARTSERVER);
        assert_eq!((*e).flags & !CMD_STARTSERVER, 0);
    }
}

#[test]
fn the_has_session_entry_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_has_session_entry;
        assert_eq!((*e).name.to_bytes(), b"has-session");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"has"
        );
        assert_eq!((*e).usage, CMD_TARGET_SESSION_USAGE);
        assert_eq!((*e).usage, c"[-t target-session]");

        assert_eq!((*e).args.template.to_bytes(), b"t:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, 0);

        assert!(::core::ptr::fn_addr_eq(
            cmd_has_session_entry.exec,
            cmd_new_session_entry.exec
        ));
    }
}

#[test]
fn constants_used_by_the_new_session_paths_match_upstream() {
    unsafe {
        assert_eq!(
            CStr::from_ptr(NEW_SESSION_TEMPLATE.as_ptr()),
            c"#{session_name}:"
        );
        assert_eq!(
            CStr::from_ptr(CMD_TARGET_SESSION_USAGE.as_ptr()),
            c"[-t target-session]"
        );
    }

    assert_eq!(__SHRT_MAX__, 32767);
    assert_eq!(USHRT_MAX, 65535);
    assert_eq!(RB_NEGINF, -1);
    assert_eq!(CMD_FIND_CANFAIL, 0x40);
    assert_eq!(CMDQ_STATE_REPEAT, 0x1);
    assert_eq!(CMD_STARTSERVER, 0x1);

    assert_eq!(CLIENT_ATTACHED, 0x80);
    assert_eq!(CLIENT_CONTROL, 0x2000);

    assert_eq!(MSG_READY, 207);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);

    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
}

/// The merged `has-session` entry answers normal at once, whatever the
/// arguments name and whether or not any session exists — the translation
/// routes both commands through one exec function whose first act is this
/// short circuit.
#[test]
fn has_session_answers_normal_without_touching_anything() {
    let _guard = globals();
    unsafe {
        let before = session_id_counter();

        let mut plain = Item::new().with_args(c"has-session");
        assert_eq!(
            run(&raw const cmd_has_session_entry, &mut plain),
            CMD_RETURN_NORMAL
        );

        let mut named = Item::new().with_args(c"has-session -t missing");
        assert_eq!(
            run(&raw const cmd_has_session_entry, &mut named),
            CMD_RETURN_NORMAL
        );

        drain_config_causes();
        assert_eq!(session_id_counter(), before, "no session may be created");
    }
}

#[test]
fn a_target_with_a_command_or_window_name_is_refused() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        for line in [
            c"new-session -t missing run",
            c"new-session -t missing -n win",
        ] {
            let before = session_id_counter();
            let c = clients.add("ns-targeted", 80, 24);
            (*c).flags |= CLIENT_ATTACHED as uint64_t;
            let mut item = Item::new().with_args(line);
            item.set_client(c);
            let rv = run(&raw const cmd_new_session_entry, &mut item);
            assert_eq!(rv, CMD_RETURN_ERROR, "{line:?}");
            assert_eq!(session_id_counter(), before, "no session may be created");
            let msgs = logged_messages();
            assert!(
                msgs.iter()
                    .any(|m| m.contains("command or window name given with target")),
                "{msgs:?}"
            );
            assert_eq!((*c).retval, 1);
        }
    }
}

#[test]
fn an_invalid_window_name_is_refused() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        let before = session_id_counter();
        let item = logged_item(&mut clients, "ns-winname");
        let mut item =
            item_with_raw_flag_value(item.with_args(c"new-session -x abc"), b'n', c"bad\xffname");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(session_id_counter(), before, "no session may be created");
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("invalid window name: bad")),
            "{msgs:?}"
        );
    }
}

#[test]
fn an_invalid_session_name_is_refused() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        let before = session_id_counter();
        let item = logged_item(&mut clients, "ns-sessname");
        let mut item =
            item_with_raw_flag_value(item.with_args(c"new-session -x abc"), b's', c"bad\xffsess");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(session_id_counter(), before, "no session may be created");
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("invalid session name: bad")),
            "{msgs:?}"
        );
    }
}

#[test]
fn naming_an_existing_session_without_A_is_a_duplicate() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(7, "dup");
    registry.add_session(&mut s);
    let mut clients = Clients::new();
    unsafe {
        let before = session_id_counter();
        let item = logged_item(&mut clients, "ns-dup");
        let mut item = item.with_args(c"new-session -s dup");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(
            session_id_counter(),
            before,
            "no second session may be created"
        );
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("duplicate session: dup")),
            "{msgs:?}"
        );
    }
}

#[test]
fn an_invalid_session_group_name_is_refused() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        let before = session_id_counter();
        let item = logged_item(&mut clients, "ns-group");
        let mut item =
            item_with_raw_flag_value(item.with_args(c"new-session -x abc"), b't', c"gr\xffp");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(session_id_counter(), before, "no session may be created");
        let msgs = logged_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("invalid session group name: gr")),
            "{msgs:?}"
        );
    }
}

/// The detached form skips the terminal work but still parses `-x`: a width
/// `strtonum` cannot read refuses the command before anything is built.
#[test]
fn a_width_strtonum_cannot_read_is_refused() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        let before = session_id_counter();
        let item = logged_item(&mut clients, "ns-width");
        let mut item = item.with_args(c"new-session -d -x abc");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(session_id_counter(), before, "no session may be created");
        let msgs = logged_messages();
        assert!(msgs.iter().any(|m| m.contains("width invalid")), "{msgs:?}");
    }
}

/// A height beyond `USHRT_MAX` is refused the same way, with strtonum's own
/// complaint carried after "height ".
#[test]
fn a_height_beyond_ushrt_max_is_refused() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        let before = session_id_counter();
        let item = logged_item(&mut clients, "ns-height");
        let mut item = item.with_args(c"new-session -d -y 70000");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(session_id_counter(), before, "no session may be created");
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("height too large")),
            "{msgs:?}"
        );
    }
}

/// A client carrying `$TMUX` whose terminal matches a pane the server knows
/// is refused as nested — and, crucially, before `tcgetattr` is pointed at
/// the fixture's fake descriptor, which would abort the process through
/// `fatal`.
#[test]
fn a_client_already_inside_tmux_is_refused_before_the_terminal_is_touched() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    unsafe {
        let ttyname = CString::new("/dev/pts/new-session").expect("no NUL");
        for (i, &b) in ttyname.as_bytes_with_nul().iter().enumerate() {
            (*t.pane(0)).tty[i] = b as c_char;
        }

        let c = clients.add("ns-nested", 80, 24);
        (*c).flags |= CLIENT_ATTACHED as uint64_t;
        (*c).ttyname = Some(ttyname);
        (*c).environ = Some(Environ::new().owned());
        environ_set(
            (*c).environ_ptr(),
            c"TMUX".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"/tmp/tmux-fixture/0,default,1".as_ptr()],
        );
        (*c).fd = 123;

        let before = session_id_counter();
        let mut item = Item::new().with_args(c"new-session");
        item.set_client(c);
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(session_id_counter(), before, "no session may be created");
        assert_eq!((*c).session, null_mut::<session>());
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("sessions should be nested")),
            "{msgs:?}"
        );
    }
}

/// `-A` naming a session that already exists hands the whole item to
/// `cmd_attach_session`; with no client behind the item the attach answers
/// normal at once and the command frees its formatted names on the way out.
#[test]
fn A_with_an_existing_session_delegates_to_attach_session() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(9, "live");
    registry.add_session(&mut s);
    unsafe {
        let before = session_id_counter();
        let mut item = Item::new().with_args(c"new-session -A -s live");
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(
            session_id_counter(),
            before,
            "no second session may be created"
        );
        drain_config_causes();
    }
}

/// The other `-A` shape: no `-s`, so the existing session comes from the
/// item's target state instead of a name lookup.
#[test]
fn A_against_the_target_state_delegates_too() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let before = session_id_counter();
        let mut item = Item::new()
            .with_args(c"new-session -A -t 0")
            .targeting(&mut t);
        let rv = run(&raw const cmd_new_session_entry, &mut item);
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(
            session_id_counter(),
            before,
            "no second session may be created"
        );
        drain_config_causes();
    }
}
