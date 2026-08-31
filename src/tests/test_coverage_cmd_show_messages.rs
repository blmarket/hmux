//! Unit tests for [`crate::cmd::cmd_show_messages`], the `show-messages`
//! command: the metadata its [`cmd_entry`] publishes, the default message
//! template, the protocol and style constants the module carries, and every
//! deterministic branch of [`cmd_show_messages_exec`] its fixtures can reach
//! without a terminal, a live server or a forked job — the plain walk over the
//! server's message log (newest first), the `-J` job summary call with no jobs
//! to print, and `-T`, whose terminal listing is driven both without a target
//! flag and with one that skips or matches the target client's own terminal.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue would call it, over items whose arguments come from the real
//! command parser. Output is made observable through a control-mode client:
//! `cmdq_print` routes a client with no session through `control_write`,
//! which lands each line in the buffer event of a [`StreamBuffer`] over a
//! local socket pair, readable without any event loop. The message log is a
//! process global, so each test takes it aside ([`MessageLog`]) and puts the
//! original back, dropping what it added. The `-T` listing needs a terminal in
//! the
//! global `tty_terms` list: [`Term`] builds a real one through
//! `tty_term_create` from just the `clear` and `cup` capabilities, and gives
//! it back with `tty_term_free`. Not covered here: the job summary's own
//! loop, whose only writer (`job_run`) forks a real process, so `-J` is
//! exercised over an empty job list; and the blank-line bookkeeping between
//! sections, which only matters once something prints there.

use crate::cmd::cmd_show_messages::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_AFTERHOOK,
    CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC,
    MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_DONE, MSG_IDENTIFY_TERM, MSG_LOCK,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, PANE_LINES_DOUBLE,
    PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES,
    PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL,
    PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID,
    PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE, SHOW_MESSAGES_TEMPLATE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_show_messages_entry,
};
use crate::control::control_state;
use crate::fmt_args;
use crate::server::{CLIENT_CONTROL, CLIENT_UTF8};
use crate::server::{message_log, server_add_message};
use crate::terminfo::{tty_term_create, tty_term_free, tty_term_ncodes, tty_term_opt, tty_terms};
use crate::tests::test_fixtures::{
    Environ, Item, StreamBuffer, Target, globals, zeroed_client, zeroed_tty,
};
use crate::types::*;
use ::core::cell::Cell;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;
use ::std::collections::VecDeque;
use ::std::ffi::CString;

/// Runs the parsed command an item carries through the entry's exec hook,
/// the way the command queue calls it.
unsafe fn exec(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_show_messages_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// A control-mode client's write side: the state `control_write` reaches
/// through the item's client and the buffer event it lands in, read back
/// with [`Self::written`]. Detaches itself from the client when it goes.
struct Control {
    bev: StreamBuffer,
    attached: Cell<*mut client>,
}

impl Control {
    fn new() -> Control {
        Control {
            bev: StreamBuffer::new(),
            attached: Cell::new(null_mut()),
        }
    }

    /// Marks the item's client a control client writing through here. Its
    /// empty block list keeps `control_write` on the direct path, and the
    /// UTF-8 flag spares each line the sanitizing copy.
    fn attach_to(&mut self, item: &mut Item) {
        let c = item.client();
        unsafe {
            let state = (*c)
                .control_state
                .insert(Box::new(control_state::default()));
            state.write_event = self.bev.ptr();
            (*c).flags |= CLIENT_CONTROL as uint64_t | CLIENT_UTF8 as uint64_t;
        }
        self.attached.set(c);
    }

    /// What has been written since the last time this was asked.
    fn written(&self) -> Vec<u8> {
        self.bev.written()
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        let c = self.attached.get();
        if !c.is_null() {
            unsafe { (*c).control_state = None };
        }
    }
}

/// A turn at the server's message log: the test gets an empty one, and what
/// it recorded goes away before the original state is put back.
struct MessageLog {
    saved: VecDeque<message_entry>,
}

impl MessageLog {
    fn take() -> MessageLog {
        MessageLog {
            saved: ::core::mem::take(message_log.queue()),
        }
    }
}

impl Drop for MessageLog {
    fn drop(&mut self) {
        *message_log.queue() = ::core::mem::take(&mut self.saved);
    }
}

/// A real terminal registered in the global `tty_terms` list, built by
/// `tty_term_create` from just the two capabilities the creation path insists
/// on. Its name deliberately matches none of the default override or feature
/// patterns. `tty_term_free` unlinks and frees it again; the environment the
/// creation path peeks into belongs to the client and goes with it.
struct Term {
    tty: Box<tty>,
    who: ClientRef,
    name: CString,
    caps: Vec<CString>,
    term: Option<Box<tty_term>>,
}

impl Term {
    /// A terminal named `name` belonging to a fixture client of its own, as
    /// `tty_open` would leave one for a client that just connected.
    fn new(name: &str) -> Term {
        let mut t = Term {
            tty: zeroed_tty(),
            who: zeroed_client(),
            name: CString::new(name).expect("a term name has no NUL"),
            caps: vec![
                CString::new("clear=\x1b[H\x1b[2J").expect("no NUL"),
                CString::new("cup=\x1b[%i%p1%d;%p2%dH").expect("no NUL"),
            ],
            term: None,
        };
        t.tty.owner = crate::server::client_ref_from_ptr(&raw mut *t.who).map(|c| c.downgrade());
        t.who.environ = Some(Environ::new().owned());
        t.who.name = Some(CString::new("showmsgs-client").expect("no NUL"));
        unsafe {
            let mut feat: c_int = 0;
            t.term = Some(
                tty_term_create(
                    &mut t.tty,
                    t.name.as_ptr() as *mut c_char,
                    &t.caps,
                    &mut feat,
                )
                .expect("no terminal was created"),
            );
        }
        t
    }

    fn ptr(&self) -> *mut tty_term {
        tty_term_opt(&self.term).map_or(null_mut(), |t| t as *const tty_term as *mut tty_term)
    }

    /// Hands the description to `target`, which owns it the way an attached
    /// client's tty owns the one `tty_open` left it, until [`Term::take_back`].
    fn lend_to(&mut self, target: &mut tty) {
        target.term = self.term.take();
    }

    fn take_back(&mut self, target: &mut tty) {
        self.term = target.term.take();
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        if let Some(term) = self.term.take() {
            unsafe { tty_term_free(term) };
        }
    }
}

#[test]
fn the_entry_advertises_the_show_messages_command() {
    let _guard = globals();
    unsafe {
        let e: *const cmd_entry = &raw const cmd_show_messages_entry;
        assert_eq!((*e).name.to_string_lossy(), "show-messages");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "showmsgs"
        );
        assert_eq!((*e).usage.to_string_lossy(), "[-JT] [-t target-client]");

        assert_eq!((*e).args.template.to_string_lossy(), "JTt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 0);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!(
            (*e).flags,
            CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL
        );
    }
}

#[test]
fn the_default_template_matches_upstream() {
    assert_eq!(
        SHOW_MESSAGES_TEMPLATE,
        c"#{t/p:message_time}: #{message_text}"
    );
}

#[test]
fn the_protocol_message_constants_keep_their_wire_values() {
    assert_eq!(MSG_VERSION, 12);

    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_DONE, 106);

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

    assert_eq!(MSG_FLAGS, 218);
    assert_eq!(MSG_EXEC, 217);
    assert_eq!(MSG_WAKEUP, 216);

    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_READ_CANCEL, 307);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_CLOSE, 306);
}

#[test]
fn the_command_style_constants_keep_their_values() {
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);

    assert_eq!(ARGS_PARSE_INVALID, 0);
    assert_eq!(ARGS_PARSE_STRING, 1);
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(ARGS_PARSE_COMMANDS, 3);

    assert_eq!(CMD_AFTERHOOK, 0x4);
    assert_eq!(CMD_CLIENT_TFLAG, 0x10);
    assert_eq!(CMD_CLIENT_CANFAIL, 0x20);

    assert_eq!(CLIENT_EXIT_RETURN, 0);
    assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
    assert_eq!(CLIENT_EXIT_DETACH, 2);

    assert_eq!(PROMPT_ENTRY, 0);
    assert_eq!(PROMPT_COMMAND, 1);
    assert_eq!(PROMPT_TYPE_COMMAND, 0);
    assert_eq!(PROMPT_TYPE_SEARCH, 1);
    assert_eq!(PROMPT_TYPE_TARGET, 2);
    assert_eq!(PROMPT_TYPE_WINDOW_TARGET, 3);
    assert_eq!(PROMPT_TYPE_INVALID, 255);

    assert_eq!(LAYOUT_LEFTRIGHT, 0);
    assert_eq!(LAYOUT_TOPBOTTOM, 1);
    assert_eq!(LAYOUT_WINDOWPANE, 2);

    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_LIGHT, 1);
    assert_eq!(THEME_DARK, 2);

    assert_eq!(SCREEN_CURSOR_DEFAULT, 0);
    assert_eq!(SCREEN_CURSOR_BLOCK, 1);
    assert_eq!(SCREEN_CURSOR_UNDERLINE, 2);
    assert_eq!(SCREEN_CURSOR_BAR, 3);

    assert_eq!(PROGRESS_BAR_HIDDEN, 0);
    assert_eq!(PROGRESS_BAR_NORMAL, 1);
    assert_eq!(PROGRESS_BAR_ERROR, 2);
    assert_eq!(PROGRESS_BAR_INDETERMINATE, 3);
    assert_eq!(PROGRESS_BAR_PAUSED, 4);

    assert_eq!(PANE_LINES_SINGLE, 0);
    assert_eq!(PANE_LINES_DOUBLE, 1);
    assert_eq!(PANE_LINES_HEAVY, 2);
    assert_eq!(PANE_LINES_SIMPLE, 3);
    assert_eq!(PANE_LINES_NUMBER, 4);
    assert_eq!(PANE_LINES_SPACES, 5);

    assert_eq!(STYLE_ALIGN_DEFAULT, 0);
    assert_eq!(STYLE_ALIGN_LEFT, 1);
    assert_eq!(STYLE_ALIGN_CENTRE, 2);
    assert_eq!(STYLE_ALIGN_RIGHT, 3);
    assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);

    assert_eq!(STYLE_LIST_OFF, 0);
    assert_eq!(STYLE_LIST_ON, 1);
    assert_eq!(STYLE_LIST_FOCUS, 2);
    assert_eq!(STYLE_LIST_LEFT_MARKER, 3);
    assert_eq!(STYLE_LIST_RIGHT_MARKER, 4);

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
}

/// With neither `-T` nor `-J`, exec walks the message log. An empty one has
/// nothing to show, so nothing reaches the control channel either.
#[test]
fn an_empty_log_answers_normal_and_prints_nothing() {
    let _guard = globals();
    let _log = MessageLog::take();
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .with_args(c"show-messages")
        .targeting(&mut t);
    let mut control = Control::new();
    control.attach_to(&mut item);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);
    assert!(control.written().is_empty());
}

/// Each logged line comes back through the template newest first, whatever
/// the pretty-printed time in front of it says.
#[test]
fn logged_messages_print_newest_first_to_the_control_client() {
    let _guard = globals();
    let log = MessageLog::take();
    unsafe {
        server_add_message(c"first message".as_ptr(), fmt_args![]);
        server_add_message(c"second message".as_ptr(), fmt_args![]);
    }
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .with_args(c"show-messages")
        .targeting(&mut t);
    let mut control = Control::new();
    control.attach_to(&mut item);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);

    let written = control.written();
    let out = String::from_utf8_lossy(&written);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out:?}");
    assert!(lines[0].ends_with(": second message"), "{out:?}");
    assert!(lines[1].ends_with(": first message"), "{out:?}");
    drop(log);
}

/// `-J` asks for a job summary. With no job on the list there is nothing to
/// say, but the branch still counts as done and answers normal.
#[test]
fn J_with_no_jobs_answers_normal_and_prints_nothing() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .with_args(c"show-messages -J")
        .targeting(&mut t);
    let mut control = Control::new();
    control.attach_to(&mut item);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);
    assert!(control.written().is_empty());
}

/// `-T` lists the terminals the server knows. With none registered there is
/// nothing to describe and nothing to separate.
#[test]
fn T_without_terminals_prints_nothing() {
    let _guard = globals();
    unsafe {
        assert!(
            tty_terms.queue().is_empty(),
            "another terminal outlived its test"
        );
    }
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .with_args(c"show-messages -T")
        .targeting(&mut t);
    let mut control = Control::new();
    control.attach_to(&mut item);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);
    assert!(control.written().is_empty());
}

/// Without `-t`, every registered terminal is described whatever else is
/// true — even when the item has no client at all to show the listing to,
/// each term still has all of its capability slots walked before exec
/// answers normal.
#[test]
fn T_lists_a_registered_terminal_without_a_client() {
    let _guard = globals();
    let term = Term::new("showmsgs-term");
    assert_eq!(
        unsafe { tty_terms.queue()[0].term },
        term.ptr(),
        "the term is registered"
    );
    let mut t = Target::new(80, 24);
    let mut item = Item::new().with_args(c"show-messages -T").targeting(&mut t);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);
}

/// With `-t`, a terminal is only described when it is the target client's
/// own. One pass over two registered terms exercises both sides at once: the
/// foreign one is skipped silently, the match gets the full listing, and the
/// printed one is numbered by its place among the shown, not the registered.
#[test]
fn T_honours_the_target_clients_own_terminal() {
    let _guard = globals();
    let _foreign = Term::new("showmsgs-foreign");
    let mut own = Term::new("showmsgs-own");
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .with_args(c"show-messages -T -t dummy")
        .targeting(&mut t);
    unsafe { own.lend_to(&mut (*item.client()).tty) };
    let mut control = Control::new();
    control.attach_to(&mut item);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);

    let written = control.written();
    let out = String::from_utf8_lossy(&written);
    assert!(
        out.starts_with("Terminal 0: showmsgs-own for showmsgs-client, flags=0x"),
        "{out:?}"
    );
    assert_eq!(
        out.matches('\n').count(),
        1 + tty_term_ncodes() as usize,
        "{out:?}"
    );
    assert!(!out.contains("showmsgs-foreign"), "{out:?}");
    unsafe { own.take_back(&mut (*item.client()).tty) };
}
