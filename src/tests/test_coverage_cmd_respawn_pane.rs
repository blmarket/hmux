//! Unit tests for [`crate::cmd::cmd_respawn_pane`], the exec hook behind the
//! `respawn-pane` command.
//!
//! Everything the hook decides is driven here through [`cmd_respawn_pane_entry`]
//! `.exec`, the very hook the command queue calls: the entry's metadata and its
//! template `c:e:kt:`, the parser's resolution of both spellings
//! `respawn-pane` and `respawnp`, the collection of `-e` values into the
//! spawn context's private environment, `-c`, the trailing command words, and
//! the whole failure half of the hook — reached through [`spawn_pane`]'s
//! deterministic refusal of a respawn whose pane is still attached, which
//! answers "pane … still active" without touching a descriptor or forking.
//! The refusal lands back in the hook's cleanup: the cause goes out through
//! the item's client, the argv vector and the private environment are freed,
//! and the command answers error.
//!
//! One limit worth recording. Every route to a *successful* respawn passes
//! through `spawn_pane` past the refusal — where a live pane would be closed
//! with `close(fd)` under `-k`, and every route at all ends in `fdforkpty`,
//! which forks a real pty child and chdirs the process on the way. No fixture
//! may go there, so the success half of the hook (the `PANE_REDRAW` marking,
//! the border redraw and status update, the normal answer) stays out of reach,
//! and no test arms `-k`.

use crate::arguments::{args_has, args_string, args_value_list};
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_respawn_pane::{
    CMD_FIND_PANE, CMD_RETURN_ERROR, PANE_REDRAW, SPAWN_KILL, SPAWN_RESPAWN, cmd_respawn_pane_entry,
};
use crate::file::{file_find_ref, file_free};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{Item, Target, globals, seen, zeroed};
use crate::types::*;
use ::core::ffi::{c_char, c_int};

/// A descriptor number to park in the fixture pane's `fd`, so that
/// `spawn_pane` sees a pane that is still attached. Nothing ever closes it:
/// the refusal this arranges returns before any descriptor work.
const FAKE_FD: c_int = 10;

/// A peer for the fixture client, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session stays null and its flags stay clear, which
/// sends `cmdq_error` down the branch that files the message in the server's
/// message log before writing the client's error stream.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_respawn_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// The lines the server has recorded so far, oldest first.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Reads the text the item's client was handed on its error stream, closing
/// the stream again afterwards.
unsafe fn drain_error_stream(c: *mut client) -> String {
    unsafe {
        let cf = file_find_ref(&raw mut (*c).files, 2).expect("the error stream was opened");
        let cf_ptr = cf.as_ptr();
        let text = String::from_utf8_lossy((*cf_ptr).buffer.as_mut().as_slice()).into_owned();
        file_free(cf);
        assert!(
            file_find_ref(&raw mut (*c).files, 2).is_none(),
            "the error stream was taken away again"
        );
        text
    }
}

#[test]
fn the_entry_advertises_respawn_pane() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_respawn_pane_entry;
        assert_eq!((*e).name.to_string_lossy(), "respawn-pane");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "respawnp"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "c:e:kt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-k] [-c start-directory] [-e environment] [-t target-pane] [shell-command [argument ...]]"
        );
        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, 0);

        assert_eq!(PANE_REDRAW, 0x1);
        assert_eq!(SPAWN_KILL, 0x1);
        assert_eq!(SPAWN_RESPAWN, 0x4);
    }
}

#[test]
fn parsing_resolves_both_spellings_and_the_template_flags_of_the_command() {
    let _guard = globals();
    let mut plain = Item::new().with_args(c"respawn-pane");
    unsafe {
        assert!(
            ::core::ptr::eq((*plain.cmd()).entry, &cmd_respawn_pane_entry),
            "the plain spelling resolves to this entry"
        );
        for flag in *b"cekt" {
            assert_eq!(args_has(cmd_get_args(&*plain.cmd()), flag), 0);
        }
    }

    let mut alias = Item::new().with_args(c"respawnp");
    unsafe {
        assert!(
            ::core::ptr::eq((*alias.cmd()).entry, &cmd_respawn_pane_entry),
            "the alias spelling resolves to this entry too"
        );
    }

    let mut killed = Item::new().with_args(c"respawn-pane -kt 0.1");
    unsafe {
        assert_eq!(args_has(cmd_get_args(&*killed.cmd()), b'k'), 1);
        assert_eq!(args_has(cmd_get_args(&*killed.cmd()), b't'), 1);
    }

    let mut full = Item::new().with_args(c"respawn-pane -c /tmp -e A=B pos extra");
    unsafe {
        let args = cmd_get_args(&*full.cmd());
        assert_eq!(args_has(args, b'c'), 1);
        assert_eq!(args_has(args, b'e'), 1);
        assert_eq!(seen(args_string(args, 0)), "pos");
        assert_eq!(seen(args_string(args, 1)), "extra");

        let values = args_value_list(args, b'e');
        assert_eq!(values.len(), 1, "a single -e value loops exactly once");
        assert_eq!((*values[0]).value.string(), c"A=B");
    }
}

#[test]
fn respawn_pane_exec_refuses_when_pane_is_still_active() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(80, 24);
        let wp = target.pane(0);
        (*wp).fd = FAKE_FD;

        let mut item = Item::with_client()
            .with_args(c"respawn-pane -c /tmp -e FOO=BAR -e BAZ=QUX echo hi")
            .targeting(&mut target);
        wire(item.client());

        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let err_text = drain_error_stream(item.client());
        assert!(err_text.contains("respawn pane failed"));

        (*wp).fd = -1;
        crate::tests::test_fixtures::release_client(item.client());
    }
}
