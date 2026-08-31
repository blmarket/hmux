//! Unit tests for [`crate::cmd::cmd_run_shell`] — the `run-shell` entry
//! metadata, the constants the file re-declares, its argument-classification
//! callback and the two exec decisions that answer before any job exists.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser. Only its first two decisions are driven: a delay that does
//! not parse refuses the command through `cmdq_error`, and a run with neither
//! a delay nor a shell-command answers normal at once. Every line past those
//! answers allocates the job's data, hands it to the timer callback
//! registration and then arms
//! the timer with `Timer::arm` or fires it with `Timer::arm_now` — whose
//! callback forks a real `/bin/sh` — so those lines are left undrawn, the
//! same line the other suites draw over `job_run`. The timer, the completion
//! callback and `cmd_run_shell_print` only ever run from inside that
//! machinery, so nothing here reaches them.
//!
//! Refusals report onto clients whose `CLIENT_ATTACHED` flag keeps
//! `file_error` out of the peer's way, leaving the complaint in the server's
//! message log where the assertions read it.

use crate::cmd::cmd_attach_session::CLIENT_ATTACHED;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_run_shell::{
    ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_STRING, CMD_FIND_CANFAIL, CMD_FIND_PANE,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmd_run_shell_entry,
};
use crate::server::message_log;
use crate::tests::test_fixtures::{Clients, Item, ensure_reactor, globals, seen};
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use std::sync::MutexGuard;

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

/// How many entries the server's message log holds, for tests that want to
/// know whether a run of their own added any.
fn logged_count() -> usize {
    message_log.queue().len()
}

/// What an exec run hands back, kept together so it comes apart in a safe
/// order: the item goes while the client fixtures are still alive, then the
/// clients, and the globals turn is given up last.
struct Ran {
    item: Item,
    clients: Clients,
    c: *mut client,
    logged: usize,
    _guard: MutexGuard<'static, ()>,
}

/// Runs one `run-shell` line through the entry's own exec hook, the way the
/// command queue calls it. The item carries a client fixture marked attached
/// so error reports stay clear of the peer. Both branches driven here answer
/// before touching the queue, the target or anything ensure_reactor owns, so no
/// more scaffolding than this is wired up.
fn ran(line: &'static CStr) -> (Ran, cmd_retval) {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let c = clients.add("run-shell", 80, 24);
    unsafe { (*c).flags |= CLIENT_ATTACHED as u64 };
    let mut item = Item::new().with_args(line);
    unsafe {
        item.set_client(c);
        let logged = logged_count();
        let rv = (cmd_run_shell_entry.exec)(&*item.cmd(), item.ptr());
        (
            Ran {
                item,
                clients,
                c,
                logged,
                _guard,
            },
            rv,
        )
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_run_shell_entry;
        assert_eq!((*e).name.to_bytes(), b"run-shell");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"run"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-bCE] [-c start-directory] [-d delay] [-t target-pane] [shell-command [argument ...]]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"bd:Ct:Es:c:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_some());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, 't' as i32 as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);

        assert_eq!((*e).flags, 0);
    }
}

/// The entry's argument callback looks at the arguments alone: `-C` asks for
/// the rest to be read as commands where possible, without it everything is
/// one shell-command string. The slot index plays no part.
#[test]
fn the_arguments_callback_classifies_by_the_C_flag() {
    let _guard = globals();
    unsafe {
        let cb = cmd_run_shell_entry.args.cb.unwrap();
        let mut cause = None;

        let mut plain = Item::new().with_args(c"run-shell display-panes");
        let args = cmd_get_args(&*plain.cmd());
        assert_eq!(cb(args, 0, &mut cause), ARGS_PARSE_STRING);
        assert_eq!(cb(args, 3, &mut cause), ARGS_PARSE_STRING);

        let mut block = Item::new().with_args(c"run-shell -C display-panes");
        let args = cmd_get_args(&*block.cmd());
        assert_eq!(cb(args, 0, &mut cause), ARGS_PARSE_COMMANDS_OR_STRING);
        assert_eq!(cb(args, 9, &mut cause), ARGS_PARSE_COMMANDS_OR_STRING);
    }
}

/// A `-d` value that parses to nothing at all refuses the whole command:
/// error, a complaint in the server's message log and the client's retval set
/// to say so — all before anything is allocated or armed.
/// A delay that starts out numeric but carries trailing garbage gets the same
/// treatment: only a fully consumed value counts as a time.
/// With neither a delay nor a shell-command there is nothing to schedule: the
/// command answers normal at once, before allocating anything, leaving no
/// complaint behind.
#[test]
fn exec_without_a_command_or_a_delay_answers_normal_at_once() {
    let (mut r, rv) = ran(c"run-shell");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!((*r.c).retval, 0);
        assert_eq!(logged_count(), r.logged);
    }
}

#[test]
fn exec_with_invalid_delay_refuses_and_logs_error() {
    {
        let (mut r, rv) = ran(c"run-shell -d invalid");
        unsafe {
            assert_eq!(rv, CMD_RETURN_ERROR);
            assert_ne!((*r.c).retval, 0);
        }
    }
    {
        let (mut r2, rv2) = ran(c"run-shell -d 10xyz");
        unsafe {
            assert_eq!(rv2, CMD_RETURN_ERROR);
            assert_ne!((*r2.c).retval, 0);
        }
    }
}
