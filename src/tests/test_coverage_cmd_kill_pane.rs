//! Unit tests for [`crate::cmd::cmd_kill_pane`], the exec hook behind the
//! `kill-pane` command.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entry's `.exec` pointer, over an item whose arguments come from the real
//! command parser and whose target state is the resolved find state of a
//! registered [`Target`]. Around it the tests pin the entry's metadata and
//! the constants this file re-declares for its own compilation, the parser's
//! resolution of both spellings `kill-pane` and `killp`, and the two
//! deterministic branches of exec that need nothing destroyed: `-a` over a
//! window whose only pane is the target itself, where the loop walks that one
//! pane, skips it as the target and answers normal after a redraw nobody can
//! observe; and a target carrying no active pane at all, which reports "no
//! active pane to kill" through the item's client and answers error.
//!
//! One limit worth recording. Both killing halves of exec destroy panes: the
//! `-a` loop takes every sibling away through `window_remove_pane`, which
//! frees the pane's memory outright, and a plain kill hands the pane to
//! [`crate::server::server_kill_pane`], which either does the same or,
//! for the window's last pane, hands the whole window to
//! `server_kill_window` and the session teardown behind it. Fixtures own their
//! panes, so no test drives those frees; what the single-pane `-a` run pins is
//! the loop's skip-the-target guard instead.

use crate::arguments::args_has;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_kill_pane::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_AFTERHOOK, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND,
    MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS,
    MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_kill_pane_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::file::{file_find_ref, file_free};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{Args, Clients, Item, Target, globals, seen, zeroed};
use crate::types::*;
use crate::window::window_get_active;
use crate::window::{WINDOW_ZOOMED, window_count_panes, window_panes_first};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;

/// A peer for the fixture client, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session stays null and its flags stay clear, which
/// is what sends `cmdq_error` down the branch that files the message in the
/// server's message log before opening the client's error stream.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_kill_pane_entry;
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

#[test]
fn the_entry_advertises_kill_pane() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_kill_pane_entry;
        assert_eq!((*e).name.to_string_lossy(), "kill-pane");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "killp"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "at:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());
        assert_eq!((*e).usage.to_string_lossy(), "[-a] [-t target-pane]");
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!(CMD_AFTERHOOK, 0x4);
        let exec = (*e).exec as usize;
        assert_ne!(exec, 0);
    }
}

#[test]
fn the_protocol_message_constants_keep_their_wire_values() {
    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_TTYNAME, 102);
    assert_eq!(MSG_IDENTIFY_OLDCWD, 103);
    assert_eq!(MSG_IDENTIFY_STDIN, 104);
    assert_eq!(MSG_IDENTIFY_ENVIRON, 105);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    assert_eq!(MSG_IDENTIFY_CLIENTPID, 107);
    assert_eq!(MSG_IDENTIFY_CWD, 108);
    assert_eq!(MSG_IDENTIFY_FEATURES, 109);
    assert_eq!(MSG_IDENTIFY_STDOUT, 110);
    assert_eq!(MSG_IDENTIFY_LONGFLAGS, 111);
    assert_eq!(MSG_IDENTIFY_TERMINFO, 112);
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_DETACHKILL, 202);
    assert_eq!(MSG_EXIT, 203);
    assert_eq!(MSG_EXITED, 204);
    assert_eq!(MSG_EXITING, 205);
    assert_eq!(MSG_LOCK, 206);
    assert_eq!(MSG_READY, 207);
    assert_eq!(MSG_RESIZE, 208);
    assert_eq!(MSG_SHELL, 209);
    assert_eq!(MSG_SHUTDOWN, 210);
    assert_eq!(MSG_OLDSTDERR, 211);
    assert_eq!(MSG_OLDSTDIN, 212);
    assert_eq!(MSG_OLDSTDOUT, 213);
    assert_eq!(MSG_SUSPEND, 214);
    assert_eq!(MSG_UNLOCK, 215);
    assert_eq!(MSG_WAKEUP, 216);
    assert_eq!(MSG_EXEC, 217);
    assert_eq!(MSG_FLAGS, 218);
    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_READY, 305);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert_eq!(MSG_READ_CANCEL, 307);
}

#[test]
fn the_enumeration_constants_keep_their_values() {
    assert_eq!(PANE_LINES_SINGLE, 0);
    assert_eq!(PANE_LINES_DOUBLE, 1);
    assert_eq!(PANE_LINES_HEAVY, 2);
    assert_eq!(PANE_LINES_SIMPLE, 3);
    assert_eq!(PANE_LINES_NUMBER, 4);
    assert_eq!(PANE_LINES_SPACES, 5);

    assert_eq!(PROGRESS_BAR_HIDDEN, 0);
    assert_eq!(PROGRESS_BAR_NORMAL, 1);
    assert_eq!(PROGRESS_BAR_ERROR, 2);
    assert_eq!(PROGRESS_BAR_INDETERMINATE, 3);
    assert_eq!(PROGRESS_BAR_PAUSED, 4);

    assert_eq!(SCREEN_CURSOR_DEFAULT, 0);
    assert_eq!(SCREEN_CURSOR_BLOCK, 1);
    assert_eq!(SCREEN_CURSOR_UNDERLINE, 2);
    assert_eq!(SCREEN_CURSOR_BAR, 3);

    assert_eq!(STYLE_DEFAULT_BASE, 0);
    assert_eq!(STYLE_DEFAULT_PUSH, 1);
    assert_eq!(STYLE_DEFAULT_POP, 2);
    assert_eq!(STYLE_DEFAULT_SET, 3);

    assert_eq!(STYLE_RANGE_NONE, 0);
    assert_eq!(STYLE_RANGE_LEFT, 1);
    assert_eq!(STYLE_RANGE_RIGHT, 2);
    assert_eq!(STYLE_RANGE_PANE, 3);
    assert_eq!(STYLE_RANGE_WINDOW, 4);
    assert_eq!(STYLE_RANGE_SESSION, 5);
    assert_eq!(STYLE_RANGE_USER, 6);
    assert_eq!(STYLE_RANGE_CONTROL, 7);

    assert_eq!(STYLE_LIST_OFF, 0);
    assert_eq!(STYLE_LIST_ON, 1);
    assert_eq!(STYLE_LIST_FOCUS, 2);
    assert_eq!(STYLE_LIST_LEFT_MARKER, 3);
    assert_eq!(STYLE_LIST_RIGHT_MARKER, 4);

    assert_eq!(STYLE_ALIGN_DEFAULT, 0);
    assert_eq!(STYLE_ALIGN_LEFT, 1);
    assert_eq!(STYLE_ALIGN_CENTRE, 2);
    assert_eq!(STYLE_ALIGN_RIGHT, 3);
    assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);

    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_LIGHT, 1);
    assert_eq!(THEME_DARK, 2);

    assert_eq!(LAYOUT_LEFTRIGHT, 0);
    assert_eq!(LAYOUT_TOPBOTTOM, 1);
    assert_eq!(LAYOUT_WINDOWPANE, 2);

    assert_eq!(PROMPT_TYPE_COMMAND, 0);
    assert_eq!(PROMPT_TYPE_SEARCH, 1);
    assert_eq!(PROMPT_TYPE_TARGET, 2);
    assert_eq!(PROMPT_TYPE_WINDOW_TARGET, 3);
    assert_eq!(PROMPT_TYPE_INVALID, 255);
    assert_eq!(PROMPT_ENTRY, 0);
    assert_eq!(PROMPT_COMMAND, 1);

    assert_eq!(CLIENT_EXIT_RETURN, 0);
    assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
    assert_eq!(CLIENT_EXIT_DETACH, 2);

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
}

#[test]
fn parsing_resolves_both_spellings_of_the_command_to_this_entry() {
    let _guard = globals();
    unsafe {
        let plain = Args::parse(c"kill-pane");
        assert!(::core::ptr::eq((*plain.cmd()).entry, &cmd_kill_pane_entry));
        assert_eq!(args_has(&*plain.ptr(), b'a'), 0);
        assert_eq!(args_has(&*plain.ptr(), b't'), 0);

        let alias = Args::parse(c"killp");
        assert!(::core::ptr::eq((*alias.cmd()).entry, &cmd_kill_pane_entry));

        let all = Args::parse(c"kill-pane -a");
        assert!(::core::ptr::eq((*all.cmd()).entry, &cmd_kill_pane_entry));
        assert_eq!(args_has(&*all.ptr(), b'a'), 1);

        let targeted = Args::parse(c"kill-pane -t 0.1");
        assert_eq!(args_has(&*targeted.ptr(), b't'), 1);
        assert_eq!(args_has(&*targeted.ptr(), b'a'), 0);
    }
}

#[test]
fn the_all_flag_over_a_one_pane_window_answers_normal_and_touches_nothing() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let wl = t.winlink(0);
    let w = t.window(0);
    let wp = t.pane(0);

    let mut item = Item::new().with_args(c"kill-pane -a").targeting(&mut t);
    unsafe {
        assert!(::core::ptr::eq((*item.cmd()).entry, &cmd_kill_pane_entry));
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'a'), 1);
        assert_eq!((*wl).window(), w);
        assert_eq!(window_get_active(w), wp);

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!((*wl).window(), w, "the winlink still carries its window");
        assert_eq!(
            window_count_panes(w, 1),
            1,
            "the target pane was not its own sibling"
        );
        assert_eq!(window_panes_first(w), wp);
        assert_eq!(window_get_active(w), wp);
        assert_eq!(
            (*w).flags & WINDOW_ZOOMED,
            0,
            "nothing zoomed or unzoomed behind the command's back"
        );
    }
}

#[test]
fn with_no_active_pane_the_command_refuses_through_its_client() {
    let _guard = globals();
    let mut clients = Clients::new();
    let caller = clients.add("killpane-caller", 80, 24);
    unsafe { wire(caller) };

    let mut t = Target::new(80, 24);
    let mut item = Item::with_client().with_args(c"kill-pane");
    unsafe {
        let mut fs = t.state();
        fs.set_pane(null_mut());
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        (*p).target = fs;

        assert_eq!(run(&mut item), CMD_RETURN_ERROR);
        assert_eq!((*caller).retval, 1, "the refusal set the client's retval");

        let msgs = server_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("killpane-caller message: no active pane to kill")),
            "{msgs:?}"
        );

        let cf = file_find_ref(&raw mut (*caller).files, 2).expect("the error stream was opened");
        let cf_ptr = cf.as_ptr();
        let text = String::from_utf8_lossy((*cf_ptr).buffer.as_mut().as_slice()).into_owned();
        assert_eq!(text, "no active pane to kill\n");

        file_free(cf);
        assert!(file_find_ref(&raw mut (*caller).files, 2).is_none());
    }
}
