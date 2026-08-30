//! Unit tests for [`crate::cmd::cmd_detach_client`], the exec hook shared by
//! the `detach-client` and `suspend-client` commands.
//!
//! What is exercised is everything the command decides before any message
//! leaves the process: which entry ran, who may detach whom, how `-P`, `-E`,
//! `-s` and `-a` steer the walk over the server's client list, and what the
//! chosen clients are left carrying — the pending-exit record
//! [`server_client_detach`] writes or the suspension flag
//! [`server_client_suspend`] sets. Every fixture client reports to a peer
//! marked bad, which turns `proc_send` away at the door, so no descriptor,
//! event or other process is ever reached; a detach only writes fields on the
//! client, and an exec is proven by its absence together with the return
//! code. The one error the command raises travels through the unattached
//! branch of `cmdq_error` into the server's message log, where it can be read
//! back; assertions look only at lines appended after the count was taken, so
//! lines from earlier tests stay out of the way.

use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_detach_client::{
    CMD_CLIENT_TFLAG, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_READONLY, MSG_DETACH,
    MSG_DETACHKILL, cmd_detach_client_entry, cmd_suspend_client_entry,
};
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::server::{
    CLIENT_ATTACHED, CLIENT_EXIT, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_READONLY,
    CLIENT_SUSPENDED,
};
use crate::tests::test_fixtures::{Clients, Item, Session, globals, seen, zeroed};
use crate::types::*;
use ::core::ffi::c_char;
use ::core::ptr::null_mut;

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches the buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer, its session — which may be null — and exactly `flags`.
unsafe fn wire(c: *mut client, session: *mut session, flags: uint64_t) {
    unsafe {
        (*c).peer = Some(bad_peer());
        (*c).session = session;
        (*c).flags = flags;
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = cmd_detach_client_entry.exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// Points the item's caller and target client where the test wants them.
unsafe fn aim(item: &mut Item, caller: *mut client, target: *mut client) {
    unsafe {
        item.set_client(caller);
        cmdq_set_target_client(item.ptr(), target);
    }
}

/// Makes the item's source find state carry `s`, as a resolved `-s` would.
unsafe fn sourced(item: &mut Item, s: *mut session) {
    unsafe {
        (*item.ptr()).source.set_session(s);
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

/// Fails unless `c` carries no trace of a detach or a suspension.
unsafe fn assert_untouched(c: *mut client) {
    unsafe {
        assert_eq!(
            (*c).flags & ((CLIENT_EXIT | CLIENT_SUSPENDED) as uint64_t),
            0,
            "{} was touched",
            seen(cstr_ptr(&(*c).name))
        );
        assert_eq!((*c).exit_type, CLIENT_EXIT_RETURN);
        assert_eq!((*c).exit_msgtype, 0);
        assert!((*c).exit_session.is_none());
    }
}

/// Fails unless `c` carries a pending detach whose exit message would be
/// `want` and whose old session was named `session`.
unsafe fn assert_detached(c: *mut client, want: msgtype, session: &str) {
    unsafe {
        assert_ne!(
            (*c).flags & (CLIENT_EXIT as uint64_t),
            0,
            "{} was not detached",
            seen(cstr_ptr(&(*c).name))
        );
        assert_eq!((*c).exit_type, CLIENT_EXIT_DETACH);
        assert_eq!((*c).exit_msgtype, want);
        assert_eq!(seen(cstr_ptr(&(*c).exit_session)), session);
    }
}

/// Fails unless `c` carries the suspension mark.
unsafe fn assert_suspended(c: *mut client) {
    unsafe {
        assert_eq!(
            (*c).flags & (CLIENT_SUSPENDED as uint64_t),
            CLIENT_SUSPENDED as uint64_t,
            "{} was not suspended",
            seen(cstr_ptr(&(*c).name))
        );
    }
}

/// Takes the recorded session name of `c`'s pending exit back again, once the
/// asserts are done with it.
unsafe fn drop_exit_session(c: *mut client) {
    unsafe {
        (*c).exit_session = None;
    }
}

#[test]
fn the_two_entries_advertise_their_names_aliases_arguments_and_flags() {
    let _guard = globals();
    unsafe {
        let d = &raw const cmd_detach_client_entry;
        assert_eq!((*d).name.to_string_lossy(), "detach-client");
        assert_eq!(
            (*d).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "detach"
        );
        assert_eq!((*d).args.template.to_string_lossy(), "aE:s:t:P");
        assert_eq!((*d).args.lower, 0);
        assert_eq!((*d).args.upper, 0);
        assert!((*d).args.cb.is_none());
        assert_eq!(
            (*d).usage.to_string_lossy(),
            "[-aP] [-E shell-command] [-s target-session] [-t target-client]"
        );
        assert_eq!((*d).source.flag, b's' as c_char);
        assert_eq!((*d).source.type_0, CMD_FIND_SESSION);
        assert_eq!((*d).source.flags, CMD_FIND_CANFAIL);
        assert_eq!((*d).target.flag, 0);
        assert_eq!((*d).target.type_0, CMD_FIND_PANE);
        assert_eq!((*d).target.flags, 0);
        assert_eq!((*d).flags, CMD_READONLY | CMD_CLIENT_TFLAG);

        let sc = &raw const cmd_suspend_client_entry;
        assert_eq!((*sc).name.to_string_lossy(), "suspend-client");
        assert_eq!(
            (*sc)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "suspendc"
        );
        assert_eq!((*sc).args.template.to_string_lossy(), "t:");
        assert_eq!((*sc).args.lower, 0);
        assert_eq!((*sc).args.upper, 0);
        assert!((*sc).args.cb.is_none());
        assert_eq!((*sc).usage.to_string_lossy(), "[-t target-client]");
        assert_eq!((*sc).source.flag, 0);
        assert_eq!((*sc).source.type_0, CMD_FIND_PANE);
        assert_eq!((*sc).source.flags, 0);
        assert_eq!((*sc).target.flag, 0);
        assert_eq!((*sc).target.type_0, CMD_FIND_PANE);
        assert_eq!((*sc).target.flags, 0);
        assert_eq!((*sc).flags, CMD_CLIENT_TFLAG);
        assert_eq!((*d).exec as usize, (*sc).exec as usize);
        assert_eq!(MSG_DETACH, 201);
        assert_eq!(MSG_DETACHKILL, 202);
    }
}

#[test]
fn suspend_client_marks_only_its_target_and_answers_normal() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(1, "targeted");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), 0);
        wire(target, s.ptr(), 0);

        let mut item = Item::new().with_args(c"suspend-client -t target");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_suspended(target);
        assert_untouched(caller);
        assert_eq!((*target).retval, 0);
    }
}

#[test]
fn suspend_client_leaves_unattached_and_already_leaving_targets_alone() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(2, "leaving");
    let caller = clients.add("caller", 80, 24);
    let unattached = clients.add("unattached", 80, 24);
    let leaving = clients.add("leaving", 80, 24);
    unsafe {
        wire(caller, null_mut(), 0);
        wire(unattached, null_mut(), 0);
        wire(leaving, s.ptr(), CLIENT_EXIT as uint64_t);

        let mut item = Item::new().with_args(c"suspend-client");

        aim(&mut item, caller, unattached);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_untouched(unattached);

        aim(&mut item, caller, leaving);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            (*leaving).flags & (CLIENT_SUSPENDED as uint64_t),
            0,
            "an already leaving client was suspended anyway"
        );
        assert_eq!((*leaving).exit_msgtype, 0);
        assert!((*leaving).exit_session.is_none());
    }
}

#[test]
fn suspend_client_runs_before_the_read_only_check_or_anything_else() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(3, "quiet");
    let caller = clients.add("caller", 80, 24);
    unsafe {
        wire(
            caller,
            s.ptr(),
            (CLIENT_READONLY | CLIENT_ATTACHED) as uint64_t,
        );

        let before = server_messages().len();
        let mut item = Item::new().with_args(c"suspend-client");
        aim(&mut item, caller, caller);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_suspended(caller);
        assert_eq!((*caller).exit_type, CLIENT_EXIT_RETURN);
        assert_eq!(
            server_messages().len(),
            before,
            "the read-only caller was refused"
        );
    }
}

#[test]
fn a_read_only_caller_may_not_name_a_session_to_detach() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(4, "home");
    let mut away = Session::new(5, "away");
    let caller = clients.add("caller", 80, 24);
    let victim = clients.add("victim", 80, 24);
    let bystander = clients.add("bystander", 80, 24);
    unsafe {
        wire(
            caller,
            null_mut(),
            (CLIENT_READONLY | CLIENT_ATTACHED) as uint64_t,
        );
        wire(victim, home.ptr(), 0);
        wire(bystander, away.ptr(), 0);

        let before = server_messages().len();
        let mut item = Item::new().with_args(c"detach-client -s home");
        aim(&mut item, caller, victim);
        sourced(&mut item, home.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(
            msgs.len(),
            before + 1,
            "expected exactly one refusal, got {msgs:?}"
        );
        assert!(
            msgs[before].ends_with("client is read-only"),
            "{:?}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);
        assert_untouched(victim);
        assert_untouched(bystander);
    }
}

#[test]
fn a_read_only_caller_may_not_ask_for_every_other_client() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(6, "home");
    let caller = clients.add("caller", 80, 24);
    let victim = clients.add("victim", 80, 24);
    unsafe {
        wire(
            caller,
            null_mut(),
            (CLIENT_READONLY | CLIENT_ATTACHED) as uint64_t,
        );
        wire(victim, home.ptr(), 0);

        let before = server_messages().len();
        let mut item = Item::new().with_args(c"detach-client -a");
        aim(&mut item, caller, victim);
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        assert_eq!(server_messages().len(), before + 1);
        assert_eq!((*caller).retval, 1);
        assert_untouched(victim);
    }
}

#[test]
fn a_read_only_caller_may_not_detach_a_different_client() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(7, "home");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(
            caller,
            null_mut(),
            (CLIENT_READONLY | CLIENT_ATTACHED) as uint64_t,
        );
        wire(target, home.ptr(), 0);

        let before = server_messages().len();
        let mut item = Item::new().with_args(c"detach-client");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        assert_eq!(server_messages().len(), before + 1);
        assert_eq!((*caller).retval, 1);
        assert_untouched(target);
    }
}

#[test]
fn a_read_only_caller_may_still_detach_itself() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(8, "readonly-home");
    let caller = clients.add("caller", 80, 24);
    unsafe {
        wire(
            caller,
            home.ptr(),
            (CLIENT_READONLY | CLIENT_ATTACHED) as uint64_t,
        );

        let before = server_messages().len();
        let mut item = Item::new().with_args(c"detach-client");
        aim(&mut item, caller, caller);
        assert_eq!(run(&mut item), CMD_RETURN_STOP);

        assert_detached(caller, MSG_DETACH, "readonly-home");
        drop_exit_session(caller);
        assert_eq!(
            server_messages().len(),
            before,
            "detaching itself needed no permission"
        );
    }
}

#[test]
fn a_plain_detach_sends_the_target_away_with_msg_detach_or_msg_detachkill() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(9, "home");
    let mut away = Session::new(10, "away");
    let caller = clients.add("caller", 80, 24);
    let plain = clients.add("plain", 80, 24);
    let killed = clients.add("killed", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(plain, away.ptr(), 0);
        wire(killed, away.ptr(), 0);

        let before = server_messages().len();
        let mut item = Item::new().with_args(c"detach-client");
        aim(&mut item, caller, plain);
        assert_eq!(run(&mut item), CMD_RETURN_STOP);
        assert_detached(plain, MSG_DETACH, "away");
        drop_exit_session(plain);
        assert_untouched(caller);
        assert_eq!(server_messages().len(), before);

        let mut item = Item::new().with_args(c"detach-client -P");
        aim(&mut item, caller, killed);
        assert_eq!(run(&mut item), CMD_RETURN_STOP);
        assert_detached(killed, MSG_DETACHKILL, "away");
        drop_exit_session(killed);
        assert_untouched(caller);
        assert_eq!(server_messages().len(), before);
    }
}

#[test]
fn detaching_the_target_executes_instead_when_a_shell_command_is_given() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(11, "home");
    let mut away = Session::new(12, "away");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(target, away.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -E exec-target");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_STOP);

        assert_untouched(target);
        assert_untouched(caller);
    }
}

#[test]
fn an_empty_shell_command_leaves_the_target_alone_but_still_answers_stop() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(13, "home");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(target, home.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -E ''");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_STOP);

        assert_untouched(target);
        assert_untouched(caller);
    }
}

#[test]
fn detach_session_takes_every_client_of_that_session_and_no_other() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(14, "home");
    let mut away = Session::new(15, "away");
    let first = clients.add("first", 80, 24);
    let second = clients.add("second", 80, 24);
    let stranger = clients.add("stranger", 80, 24);
    let floating = clients.add("floating", 80, 24);
    unsafe {
        wire(first, home.ptr(), 0);
        wire(second, home.ptr(), 0);
        wire(stranger, away.ptr(), 0);
        wire(floating, null_mut(), 0);

        let mut item = Item::new().with_args(c"detach-client -s home");
        aim(&mut item, first, first);
        sourced(&mut item, home.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_STOP);

        assert_detached(first, MSG_DETACH, "home");
        assert_detached(second, MSG_DETACH, "home");
        drop_exit_session(first);
        drop_exit_session(second);
        assert_untouched(stranger);
        assert_untouched(floating);
    }
}

#[test]
fn detach_session_executes_instead_when_a_shell_command_is_given() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(16, "home");
    let first = clients.add("first", 80, 24);
    let second = clients.add("second", 80, 24);
    unsafe {
        wire(first, home.ptr(), 0);
        wire(second, home.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -s home -E exec-everyone");
        aim(&mut item, first, first);
        sourced(&mut item, home.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_STOP);

        assert_untouched(first);
        assert_untouched(second);
    }
}

#[test]
fn an_unresolved_or_empty_session_source_detaches_nothing() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(17, "home");
    let mut lonely = Session::new(18, "lonely");
    let caller = clients.add("caller", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -s ghost");
        aim(&mut item, caller, caller);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_untouched(caller);

        sourced(&mut item, lonely.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_STOP);
        assert_untouched(caller);
    }
}

#[test]
fn detach_all_spares_only_the_target_and_the_unattached() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(19, "home");
    let mut away = Session::new(20, "away");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    let elsewhere = clients.add("elsewhere", 80, 24);
    let floating = clients.add("floating", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(target, home.ptr(), 0);
        wire(elsewhere, away.ptr(), 0);
        wire(floating, null_mut(), 0);

        let mut item = Item::new().with_args(c"detach-client -a");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_detached(caller, MSG_DETACH, "home");
        assert_detached(elsewhere, MSG_DETACH, "away");
        drop_exit_session(caller);
        drop_exit_session(elsewhere);
        assert_untouched(target);
        assert_untouched(floating);
    }
}

#[test]
fn detach_all_with_p_uses_msg_detachkill_on_everyone_it_takes() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(21, "home");
    let mut away = Session::new(22, "away");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    let elsewhere = clients.add("elsewhere", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(target, home.ptr(), 0);
        wire(elsewhere, away.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -aP");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_detached(caller, MSG_DETACHKILL, "home");
        assert_detached(elsewhere, MSG_DETACHKILL, "away");
        drop_exit_session(caller);
        drop_exit_session(elsewhere);
        assert_untouched(target);
    }
}

#[test]
fn detach_all_executes_instead_when_a_shell_command_is_given() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(23, "home");
    let mut away = Session::new(24, "away");
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    let elsewhere = clients.add("elsewhere", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(target, home.ptr(), 0);
        wire(elsewhere, away.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -aE exec-others");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_untouched(caller);
        assert_untouched(target);
        assert_untouched(elsewhere);
    }
}

#[test]
fn a_client_that_is_already_leaving_cannot_be_detached_again() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(25, "home");
    let caller = clients.add("caller", 80, 24);
    let gone = clients.add("gone", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);
        wire(gone, home.ptr(), CLIENT_EXIT as uint64_t);

        let mut item = Item::new().with_args(c"detach-client -P");
        aim(&mut item, caller, gone);
        assert_eq!(run(&mut item), CMD_RETURN_STOP);

        assert_eq!((*gone).exit_type, CLIENT_EXIT_RETURN);
        assert_eq!((*gone).exit_msgtype, 0);
        assert!((*gone).exit_session.is_none());
        assert_untouched(caller);
    }
}

#[test]
fn the_parsed_arguments_reach_the_hook_and_the_entry_decides_with_them() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut home = Session::new(26, "home");
    let caller = clients.add("caller", 80, 24);
    unsafe {
        wire(caller, home.ptr(), 0);

        let mut item = Item::new().with_args(c"detach-client -P -E run-it -s somewhere");
        aim(&mut item, caller, caller);
        sourced(&mut item, home.ptr());
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(args_has(args, b'P'), 1);
        assert_eq!(args_has(args, b'a'), 0);
        assert_eq!(seen(args_get(args, b'E')), "run-it");
        assert_eq!(seen(args_get(args, b's')), "somewhere");

        assert_eq!(run(&mut item), CMD_RETURN_STOP);
        assert_untouched(caller);

        let mut item = Item::new().with_args(c"detach-client -P -s somewhere");
        aim(&mut item, caller, caller);
        sourced(&mut item, home.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_STOP);
        assert_detached(caller, MSG_DETACHKILL, "home");
        drop_exit_session(caller);
    }
}
