//! Unit tests for [`crate::cmd::cmd_display_message`] — the
//! `display-message` entry metadata, its default template and constants, and
//! every branch of [`cmd_display_message_exec`] the fixtures can reach
//! without a terminal or a live daemon.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue would call it, over items whose arguments come from the real
//! command parser and whose target state is resolved against a registered
//! [`Target`]. Three output channels are exercised:
//!
//! * The status line. A client carrying a session receives the message
//!   through `status_message_set`, which parks the text in
//!   `message_string`, pushes a status screen and — for a non-zero delay —
//!   arms the client's message timer through ensure_reactor; nothing runs the loop,
//!   so the timer never fires, and each test deletes it again before its
//!   fixtures go.
//! * The control channel. A control client's writes land in the buffer event
//!   of [`ControlOut`], where they can be read back verbatim, which makes both
//!   the plain `%message` display and `-p`'s raw print observable.
//! * The queue error path. With no client behind the item at all,
//!   `cmdq_error` files a config cause; cfg.rs's cause list is private, so
//!   that test pins the return value and drains the list the way
//!   `cfg_print_causes` would. Error refusals on client-backed items report
//!   through the server's message log instead (the clients carry
//!   `CLIENT_ATTACHED`, because `file_error` would otherwise try to open a
//!   stream to the peer the fixture does not have).
//!
//! The deep half of `-I` is covered too, over a [`Peer`] of the test's own:
//! `file_read` asks the client's peer for standard input rather than reading
//! anything itself, so a socket pair behind a real `tmuxpeer` is all that path
//! needs — the request is left sitting in the peer's send buffer, since
//! nothing runs the loop. Not covered here: `-v`'s verbose format flag beyond
//! the bit it sets, whose effect lives inside a format tree exec frees before
//! returning.

use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_display_message::{
    __INT_MAX__, CLIENT_CONTROL, CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG,
    CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT,
    DISPLAY_MESSAGE_TEMPLATE, FORMAT_NONE, FORMAT_VERBOSE, UINT_MAX, cmd_display_message_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::control::control_state;
use crate::file::file_free;
use crate::proc::{proc_add_peer, proc_remove_peer};
use crate::reactor::Timer;
use crate::server::message_log;
use crate::server::{CLIENT_ATTACHED, CLIENT_DEAD, CLIENT_REDRAWSTATUS, TTY_FREEZE, TTY_NOCURSOR};
use crate::session::session_add_attached;
use crate::status::{status_init, status_message_clear};
use crate::tests::test_fixtures::{
    Clients, Item, StreamBuffer, Target, ensure_reactor, globals, seen,
};
use crate::types::*;
use crate::window::PANE_EMPTY;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

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

/// Hands cfg.rs's cause list to `cfg_print_causes`, which frees every entry.
/// With no client behind the item each cause only reaches `log_debug`.
unsafe fn drain_config_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
    }
}

/// Deletes the message timer if one was armed and clears the status line
/// again, popping the screen `status_message_set` pushed.
unsafe fn clear_status(c: *mut client) {
    unsafe {
        (*c).message_timer.disarm();
        status_message_clear(c);
        assert!((*c).message_string.is_none());
    }
}

/// A control client's write side: the state `control_write` reaches through
/// the client and the buffer event it writes into, read back with
/// [`ControlOut::written`]. Detaches itself from the client when it goes.
struct ControlOut {
    c: *mut client,
    bev: StreamBuffer,
}

impl ControlOut {
    /// Marks `c` a control client and gives it a fresh empty state,
    /// writing through `bev`.
    fn new(c: *mut client) -> ControlOut {
        let out = ControlOut {
            c,
            bev: StreamBuffer::new(),
        };
        unsafe {
            let state = (*c)
                .control_state
                .insert(Box::new(control_state::default()));
            state.write_event = out.bev.ptr();
            (*c).flags |= CLIENT_CONTROL as u64;
        }
        out
    }

    /// What has been written since the last time this was asked.
    fn written(&self) -> Vec<u8> {
        self.bev.written()
    }
}

impl Drop for ControlOut {
    fn drop(&mut self) {
        unsafe { (*self.c).control_state = None };
    }
}

/// A client's connection to its process: a `tmuxpeer` over one end of a socket
/// pair, hanging in a `tmuxproc` of the test's own. It is what `proc_send`
/// needs to compose a message at all; nothing runs the event loop, so what is
/// composed stays in the peer's send buffer. Takes itself back off the event
/// base, closes both ends and frees the peer when it goes.
struct Peer {
    proc: Box<tmuxproc>,
    peer: Option<Box<tmuxpeer>>,
    fds: [c_int; 2],
}

impl Peer {
    fn new() -> Peer {
        ensure_reactor();
        let mut proc = Box::new(tmuxproc::default());
        let mut fds = [-1 as c_int; 2];
        unsafe {
            assert_eq!(
                ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0,
                "no socket pair"
            );
            let peer = Some(proc_add_peer(&raw mut *proc, fds[0], None, None));
            Peer { proc, peer, fds }
        }
    }

    /// Gives `c` this peer, which owns it until [`Peer::take_back`].
    fn lend_to(&mut self, c: *mut client) {
        unsafe { (*c).peer = self.peer.take() };
    }

    fn take_back(&mut self, c: *mut client) {
        unsafe { self.peer = (*c).peer.take() };
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        unsafe {
            if let Some(peer) = self.peer.take() {
                proc_remove_peer(peer);
            }
            ::libc::close(self.fds[1]);
        }
        let _ = &self.proc;
    }
}

/// Runs the parsed command an item carries through the entry's exec hook,
/// the way the command queue calls it.
unsafe fn exec(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, &cmd_display_message_entry),
            "the item is not running display-message"
        );
        (cmd_display_message_entry.exec)(&*item.cmd(), item.ptr())
    }
}

/// An item carrying `line`'s parsed arguments, aimed at a registered target
/// by the client `who`: its target state names the current winlink and the
/// client doubles as the target client, so exec formats against whoever the
/// caller attached.
fn aimed(line: &'static CStr, who: *mut client, t: &mut Target) -> Item {
    let mut item = Item::with_client().with_args(line);
    item.set_client(who);
    item.targeting(t)
}

#[test]
fn the_entry_advertises_the_display_message_command() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_display_message_entry;
        assert_eq!((*e).name.to_string_lossy(), "display-message");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "display"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-aCIlNpv] [-c target-client] [-d delay] [-F format] [-t target-pane] [message]"
        );

        assert_eq!((*e).args.template.to_string_lossy(), "aCc:d:lINpt:F:v");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);

        assert_eq!(
            (*e).flags,
            CMD_AFTERHOOK | CMD_CLIENT_CFLAG | CMD_CLIENT_CANFAIL
        );
    }
}

#[test]
fn the_default_template_matches_upstream() {
    let want: &[u8] =
        b"[#{session_name}] #{window_index}:#{window_name}, current pane #{pane_index} - (%H:%M %d-%b-%y)\0";
    assert_eq!(DISPLAY_MESSAGE_TEMPLATE.len(), 96);
    assert_eq!(want.len(), 96);
    assert!(
        DISPLAY_MESSAGE_TEMPLATE
            .iter()
            .zip(want)
            .all(|(a, b)| *a == *b as c_char)
    );
}

#[test]
fn constants_used_by_the_message_paths_keep_their_values() {
    assert_eq!(FORMAT_VERBOSE, 0x8);
    assert_eq!(FORMAT_NONE, 0);

    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_CANFAIL, 0x40);
    assert_eq!(CMD_AFTERHOOK, 0x4);
    assert_eq!(CMD_CLIENT_CFLAG, 0x8);
    assert_eq!(CMD_CLIENT_CANFAIL, 0x20);
    assert_eq!(CLIENT_CONTROL, 0x2000);

    assert_eq!(UINT_MAX, u32::MAX);
    assert_eq!(__INT_MAX__, i32::MAX);
}

/// `-I` with no pane behind the item gives up at once: nothing to read into.
#[test]
fn input_without_a_target_pane_answers_normal() {
    let _guard = globals();
    let mut item = Item::new().with_args(c"display-message -I");
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);
}

/// A pane still holding content refuses `-I` outright: error, and the cause
/// reported through the queue's error path.
#[test]
fn input_is_refused_on_a_pane_that_already_has_content() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("input", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let mut item = aimed(c"display-message -I", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("pane is not empty")),
            "{msgs:?}"
        );
    }
}

/// A dead client has nowhere to take input, so `-I` answers normal without
/// starting the read.
#[test]
fn input_skips_a_dead_client_on_an_input_ready_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("dead-input", 80, 24);
    unsafe {
        (*t.pane(0)).flags |= PANE_EMPTY;
        (*c).flags |= CLIENT_DEAD as u64;
        let mut item = aimed(c"display-message -I", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
    }
}

/// While the client still owns a session, `-I` declines quietly too: reading
/// pane input is only for clients between sessions.
#[test]
fn input_skips_while_the_client_still_has_a_session() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("busy-input", 80, 24);
    unsafe {
        (*t.pane(0)).flags |= PANE_EMPTY;
        (*c).session = t.session();
        let mut item = aimed(c"display-message -I", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
    }
}

/// A client between sessions on an empty pane is what `-I` is for: the read is
/// started and the command answers wait, leaving the item for the read's own
/// callback to continue. The read itself is a `MSG_READ_OPEN` to the client's
/// peer, which is left sitting in its send buffer, and the client picks up a
/// reference from the read and another from the file it now carries.
///
/// Nothing here runs the event loop, so that callback never fires and the
/// teardown stands in for the closed branch of it: the data the read was
/// handed goes, and so does the reference it took.
#[test]
fn input_on_a_session_less_client_starts_the_read_and_waits() {
    let _guard = globals();
    let mut peer = Peer::new();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("reader", 80, 24);
    unsafe {
        (*t.pane(0)).flags |= PANE_EMPTY;
        peer.lend_to(c);

        let rv;
        {
            let mut item = aimed(c"display-message -I", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_WAIT);

        let cf = (*c)
            .files
            .values()
            .next()
            .cloned()
            .expect("the client is not reading anything");
        let cf_ptr = cf.as_ptr();
        assert!(
            matches!((*cf_ptr).data, ClientFileData::PaneInput(_)),
            "display-message callback data is not pane-input data"
        );
        drop(::std::mem::take(&mut (*cf_ptr).data));
        file_free(cf);
        assert!((*c).files.is_empty());
        peer.take_back(c);
    }
}

/// `-F` and a message argument are mutually exclusive.
#[test]
fn F_and_an_argument_together_are_refused() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("clash", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let mut item = aimed(c"display-message -F x hello", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("only one of -F or argument must be given")),
            "{msgs:?}"
        );
    }
}

/// A delay the number parser turns down becomes a queue error prefixed with
/// the option it came from.
#[test]
fn a_delay_that_is_not_a_number_is_reported() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("slowcoach", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let mut item = aimed(c"display-message -d soon hello", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(msgs.iter().any(|m| m.contains("delay")), "{msgs:?}");
    }
}

/// The ordinary path: the argument expands formats and lands in the target
/// client's status line, freezing the terminal and asking for a redraw until
/// it goes away.
#[test]
fn a_formatted_argument_reaches_the_status_line() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("shown", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message \"#{session_name}\"", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "0");
        assert_eq!(
            (*c).tty.flags & (TTY_FREEZE | TTY_NOCURSOR),
            TTY_FREEZE | TTY_NOCURSOR
        );
        assert_eq!(
            (*c).flags & CLIENT_REDRAWSTATUS as u64,
            CLIENT_REDRAWSTATUS as u64
        );
        assert_eq!((*c).message_ignore_keys, 0);

        clear_status(c);
    }
}

/// `-l` sends the template exactly as given: no format expansion happens.
#[test]
fn the_literal_flag_bypasses_expansion() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("listeral", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message -l \"#{session_name}\"", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "#{session_name}");

        clear_status(c);
    }
}

/// With neither an argument nor `-F`, the upstream default template is shown.
/// Its clock part varies, so only the fixed frame is pinned.
#[test]
fn the_default_template_is_used_when_nothing_is_given() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("defaulted", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let msg = seen((*c).message_string_ptr());
        assert!(
            msg.starts_with("[0] 0:target, current pane 0 - ("),
            "{msg:?}"
        );
        assert!(msg.ends_with(')'), "{msg:?}");

        clear_status(c);
    }
}

/// `-F` supplies the template whenever no argument is given.
#[test]
fn F_supplies_the_template_when_no_argument_is_given() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("effed", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message -F \"x#{session_name}\"", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "x0");

        clear_status(c);
    }
}

/// `-C` asks for no freeze, and `-d 0` keeps the message up indefinitely
/// without arming anything: the timer is left untouched and the message
/// ignores no keys because there is no delay window.
#[test]
fn C_leaves_the_terminal_unfrozen_and_d_zero_never_arms_the_timer() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("unfrozen", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message -C -N -d 0 stuck", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "stuck");
        assert_eq!((*c).tty.flags & TTY_FREEZE, 0);
        assert_eq!((*c).tty.flags & TTY_NOCURSOR, TTY_NOCURSOR);
        assert!(!(*c).message_timer.is_set());
        assert_eq!((*c).message_ignore_keys, 0);

        clear_status(c);
    }
}

/// `-d 5000` keeps the message for five seconds and lets `-N` mark it as one
/// keys may not dismiss while the delay runs.
#[test]
fn N_marks_the_message_as_ignoring_keys_when_a_delay_runs() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("patient", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message -N -d 5000 wait", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "wait");
        assert_eq!((*c).message_ignore_keys, 1);
        assert!((*c).message_timer.is_set());

        clear_status(c);
    }
}

/// A control client gets `%message <text>` lines rather than a status-line
/// popup, whatever the message says.
#[test]
fn a_control_client_receives_percent_message_output() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("controlled", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let rv;
        {
            let mut item = aimed(c"display-message hello", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(out.written(), b"%message hello\n".to_vec());
    }
}

/// `-p` prints the message itself to the control channel, newline included,
/// without the `%message` dressing.
#[test]
fn print_writes_the_message_verbatim_to_the_control_client() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("printed", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let rv;
        {
            let mut item = aimed(c"display-message -p hello", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(out.written(), b"hello\n".to_vec());
    }
}

/// When the target client is not on the target session, exec picks the best
/// client of the session itself for the formats — the one most recently
/// active among those carrying the session — and still delivers through the
/// control channel.
#[test]
fn the_best_client_of_the_session_formats_the_message() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        session_add_attached(t.session());
    }
    let mut clients = Clients::new();
    let old = clients.add("old", 80, 24);
    let new = clients.add("new", 80, 24);
    let ctrl = clients.add("ctrl", 80, 24);
    let out = ControlOut::new(ctrl);
    unsafe {
        (*old).session = t.session();
        (*old).activity_time.tv_sec = 100;
        (*new).session = t.session();
        (*new).activity_time.tv_sec = 200;

        let rv;
        {
            let mut item = aimed(c"display-message \"#{client_name}\"", ctrl, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(out.written(), b"%message new\n".to_vec());
    }
}

/// `-a` walks every format entry printing key=value pairs; without any
/// client behind the item those prints have nowhere to go, but the command
/// still answers normal.
#[test]
fn listing_every_format_entry_answers_normal_without_a_client() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::new()
        .with_args(c"display-message -a")
        .targeting(&mut t);
    assert_eq!(unsafe { exec(&mut item) }, CMD_RETURN_NORMAL);
}

/// No client anywhere: the message cannot be shown, so it is filed as a
/// config cause and the command still answers normal. The cause list is
/// drained the way the config loader does once the assertion is pinned.
#[test]
fn with_no_client_anywhere_the_message_becomes_a_config_cause() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::new()
        .from_file(c"fixture.conf", 11)
        .with_args(c"display-message unreachable")
        .targeting(&mut t);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        drain_config_causes();
    }
}

/// With no target state at all — the shape a `-t` the finder gave up on
/// leaves, since the entry carries `CMD_FIND_CANFAIL` — there is no session to
/// pick a best client from and the target client is on a session of its own,
/// so the formats run against no client whatever: `#{session_name}` comes out
/// empty. The message still reaches that client's status line.
#[test]
fn with_no_target_session_the_formats_run_against_no_client() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("untargeted", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = Item::with_client().with_args(c"display-message \"[#{session_name}]\"");
            item.set_client(c);
            cmdq_set_target_client(item.ptr(), c);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "[]");

        clear_status(c);
    }
}

/// A message with nowhere to go is thrown away. The item carries a client, so
/// it is not filed as a config cause; there is no target client, so there is
/// neither a status line nor a control channel to take it; and without `-p`
/// nothing prints it either. The command still answers normal, and the
/// expanded text is simply freed.
#[test]
fn a_message_with_no_target_client_is_dropped() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .with_args(c"display-message dropped")
        .targeting(&mut t);
    unsafe {
        cmdq_set_target_client(item.ptr(), null_mut());
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert!((*item.client()).message_string.is_none());
    }
}

/// `-v` asks the format engine for its verbose reporting. The expansion it
/// produces is the same either way, and the tree it logs into is freed before
/// exec returns, so what is pinned here is that the flag changes nothing the
/// caller sees.
#[test]
fn the_verbose_flag_still_delivers_the_message() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("verbose", 80, 24);
    unsafe {
        status_init(c);
        (*c).session = t.session();

        let rv;
        {
            let mut item = aimed(c"display-message -v \"#{session_name}\"", c, &mut t);
            rv = exec(&mut item);
        }
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen((*c).message_string_ptr()), "0");

        clear_status(c);
    }
}
