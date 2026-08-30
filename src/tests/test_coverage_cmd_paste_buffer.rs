//! Unit tests for [`crate::cmd::cmd_paste_buffer`], the `paste-buffer`
//! command: the metadata its [`cmd_entry`] static publishes, the protocol and
//! visibility constants it declares, and every deterministic branch of its
//! exec routine that can be reached without a live server — the exited-pane
//! refusal, buffer lookup by name and by top-of-store, line splitting with
//! separators from `-r` and `-s`, raw lines under `-S`, visible-form escaping,
//! bracketed-paste wrapping when the pane's screen has the mode on, deletion
//! under `-d`, and the quiet paths an empty store or a pane with input off
//! take.
//!
//! The private helpers (`cmd_paste_buffer_exec`, `_paste`) are exercised only
//! through the entry's own function pointer, exactly as the command queue
//! calls them, so no test-only visibility changes were needed. Items carry
//! arguments from the real command parser and targets from the [`Target`]
//! fixture; the bytes the command sends to the pane go into a [`StreamBuffer`]
//! hung off `wp.event` over a local socket pair, read back without running any
//! event loop, and buffers go into a [`Paste`] store emptied again afterwards.
//! A fixture pane starts out "exited" (`fd == -1`), which is how
//! `window_pane_exited` answers: the paste tests mark their pane live by
//! giving it a descriptor number nothing ever opens, since this exec path only
//! ever reads that field. Not covered here: writing into a pane owned by a
//! running server process, which wants a real terminal.

use crate::cmd::cmd_paste_buffer::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, MODE_BRACKETPASTE,
    MSG_COMMAND, MSG_DETACHKILL, MSG_EXEC, MSG_EXITED, MSG_FLAGS, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_TERMINFO, MSG_LOCK, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_VERSION, MSG_WAKEUP, PANE_INPUTOFF, VIS_NOSLASH, VIS_SAFE,
    cmd_paste_buffer_entry,
};
use crate::paste::{paste_buffer_data, paste_get_name, paste_get_top, paste_set};
use crate::tests::test_fixtures::{Item, Paste, StreamBuffer, Target, globals};
use crate::types::*;
use crate::window::PANE_EXITED;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::{null, null_mut};

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-paste-buffer.conf";

/// A named buffer the tests put in the store.
const BUF: &CStr = c"buf";

/// Runs the exec routine through the entry's own function pointer, the way the
/// command queue calls it, and answers what it answers.
unsafe fn exec(item: &mut Item) -> cmd_retval {
    unsafe {
        let run = cmd_paste_buffer_entry.exec;
        run(&*item.cmd(), item.ptr())
    }
}

/// An item carrying a parsed command line, sourced from [`FILE`] and aimed at
/// `target`'s active pane.
fn item(line: &'static CStr, number: u_int, target: &mut Target) -> Item {
    Item::new()
        .from_file(FILE, number)
        .with_args(line)
        .targeting(target)
}

/// Marks `target`'s active pane as not exited and hands its writes to `bev`.
/// The descriptor number is only read here; nothing opens it.
unsafe fn attach(wp: *mut window_pane, bev: &StreamBuffer) {
    unsafe {
        (*wp).fd = 1000;
        (*wp).event = bev.ptr();
    }
}

/// Whether a buffer called `name` exists in the store.
fn exists(name: &CStr) -> bool {
    !unsafe { paste_get_name(name.as_ptr()) }.is_null()
}

/// Adds an automatic buffer holding `data`, as `set-buffer` given no name
/// would. Automatic ones are exactly what an unnamed `paste-buffer` finds.
unsafe fn add_automatic(data: &str) {
    unsafe {
        assert!(
            paste_set(data.as_bytes().to_vec(), null()).is_ok(),
            "{data:?} was not set"
        );
    }
}

/// The bytes of the newest automatic buffer, which `paste_get_top` answers.
unsafe fn automatic_bytes() -> Vec<u8> {
    unsafe {
        let pb = paste_get_top(null_mut());
        assert!(!pb.is_null(), "no top buffer");
        paste_buffer_data(&*pb).to_vec()
    }
}

#[test]
fn the_paste_buffer_entry_describes_the_paste_buffer_command() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_paste_buffer_entry;
        assert_eq!((*e).name.to_string_lossy(), "paste-buffer");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "pasteb"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-dprS] [-s separator] [-b buffer-name] [-t target-pane]"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "db:prSs:t:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 't' as i32 as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_module_declares_the_protocol_and_visibility_constants_it_uses() {
    for &(got, want) in &[
        (MSG_VERSION, 12),
        (MSG_IDENTIFY_LONGFLAGS, 111),
        (MSG_IDENTIFY_TERMINFO, 112),
        (MSG_IDENTIFY_FLAGS, 100),
        (MSG_IDENTIFY_CWD, 108),
        (MSG_COMMAND, 200),
        (MSG_DETACHKILL, 202),
        (MSG_EXITED, 204),
        (MSG_LOCK, 206),
        (MSG_READY, 207),
        (MSG_RESIZE, 208),
        (MSG_SHELL, 209),
        (MSG_SHUTDOWN, 210),
        (MSG_SUSPEND, 214),
        (MSG_WAKEUP, 216),
        (MSG_EXEC, 217),
        (MSG_FLAGS, 218),
        (MSG_READ_OPEN, 300),
        (MSG_READ, 301),
        (MSG_READ_DONE, 302),
        (MSG_READ_CANCEL, 307),
    ] {
        assert_eq!(got, want);
    }
    assert_eq!(VIS_SAFE, 0x20);
    assert_eq!(VIS_NOSLASH, 0x40);
    assert_eq!(MODE_BRACKETPASTE, 0x400);
    assert_eq!(PANE_INPUTOFF, 0x40);
    assert_eq!(CMD_AFTERHOOK, 0x4);
}

#[test]
fn pasting_a_named_buffer_sends_each_line_with_the_default_separator() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(BUF, "ab\r\ncd");
        attach(t.pane(0), &bev);

        let mut it = item(c"paste-buffer -b buf", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            bev.written(),
            b"ab\r\rcd",
            "one separator after each newline-closed line, none after the tail"
        );
        assert!(exists(BUF));
    }
}

#[test]
fn an_unnamed_buffer_pastes_the_top_of_the_store() {
    let _guard = globals();
    let _store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        add_automatic("older\nline\n");
        add_automatic("newer\nline\n");
        attach(t.pane(0), &bev);

        assert_eq!(automatic_bytes(), b"newer\nline\n");
        let mut it = item(c"paste-buffer", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            bev.written(),
            b"newer\rline\r",
            "the most recently added buffer is the one on top"
        );
    }
}

#[test]
fn pasting_a_missing_named_buffer_is_an_error() {
    let _guard = globals();
    let _store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        add_automatic("something");
        attach(t.pane(0), &bev);

        let mut it = item(c"paste-buffer -b nosuch", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_ERROR);
        assert!(bev.written().is_empty(), "nothing reached the pane");
    }
}

#[test]
fn pasting_into_a_pane_that_has_exited_is_an_error() {
    let _guard = globals();
    let _store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        let mut gone = item(c"paste-buffer", 1, &mut t);
        assert_eq!(
            exec(&mut gone),
            CMD_RETURN_ERROR,
            "the fixture pane starts with fd == -1, which counts as exited"
        );

        attach(t.pane(0), &bev);
        (*t.pane(0)).flags |= PANE_EXITED;
        let mut flagged = item(c"paste-buffer", 2, &mut t);
        assert_eq!(exec(&mut flagged), CMD_RETURN_ERROR);
        assert!(bev.written().is_empty());
    }
}

#[test]
fn an_empty_store_pastes_nothing_but_still_answers_normal() {
    let _guard = globals();
    let _store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        attach(t.pane(0), &bev);

        let mut it = item(c"paste-buffer -d", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert!(bev.written().is_empty(), "no buffer, nothing to send");
    }
}

#[test]
fn the_S_flag_sends_lines_raw_without_escaping() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(BUF, "a\x01b\ncd");
        attach(t.pane(0), &bev);

        let mut raw = item(c"paste-buffer -S -b buf", 1, &mut t);
        assert_eq!(exec(&mut raw), CMD_RETURN_NORMAL);
        assert_eq!(
            bev.written(),
            b"a\x01b\rcd",
            "the control byte goes over exactly as stored"
        );

        let mut shown = item(c"paste-buffer -b buf", 2, &mut t);
        assert_eq!(exec(&mut shown), CMD_RETURN_NORMAL);
        assert_eq!(
            bev.written(),
            b"a^Ab\rcd",
            "without -S the control byte is made visible first"
        );
    }
}

#[test]
fn the_r_flag_makes_the_separator_a_newline_and_s_supplies_its_own() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(BUF, "ab\ncd\n");
        attach(t.pane(0), &bev);

        let mut r = item(c"paste-buffer -r -b buf", 1, &mut t);
        assert_eq!(exec(&mut r), CMD_RETURN_NORMAL);
        assert_eq!(bev.written(), b"ab\ncd\n");

        let mut s = item(c"paste-buffer -s XY -b buf", 2, &mut t);
        assert_eq!(exec(&mut s), CMD_RETURN_NORMAL);
        assert_eq!(bev.written(), b"abXYcdXY");
    }
}

#[test]
fn bracketed_paste_wraps_the_stream_only_when_the_mode_is_on() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(BUF, "ab\ncd");
        attach(t.pane(0), &bev);
        let wp = t.pane(0);
        let sc = (*wp).screen();
        (*sc).mode |= MODE_BRACKETPASTE;

        let mut wrapped = item(c"paste-buffer -p -b buf", 1, &mut t);
        assert_eq!(exec(&mut wrapped), CMD_RETURN_NORMAL);
        assert_eq!(
            bev.written(),
            b"\x1B[200~ab\rcd\x1B[201~",
            "the stream opens and closes inside the bracketed-paste markers"
        );

        let mut naked = item(c"paste-buffer -b buf", 2, &mut t);
        assert_eq!(exec(&mut naked), CMD_RETURN_NORMAL);
        assert_eq!(bev.written(), b"ab\rcd");

        (*sc).mode &= !MODE_BRACKETPASTE;
        let mut unmoded = item(c"paste-buffer -p -b buf", 3, &mut t);
        assert_eq!(exec(&mut unmoded), CMD_RETURN_NORMAL);
        assert_eq!(bev.written(), b"ab\rcd");
    }
}

#[test]
fn the_d_flag_deletes_the_buffer_once_it_has_been_sent() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(BUF, "xy");
        attach(t.pane(0), &bev);

        let mut keep = item(c"paste-buffer -b buf", 1, &mut t);
        assert_eq!(exec(&mut keep), CMD_RETURN_NORMAL);
        assert_eq!(bev.written(), b"xy");
        assert!(exists(BUF));

        let mut drop_0 = item(c"paste-buffer -d -b buf", 2, &mut t);
        assert_eq!(exec(&mut drop_0), CMD_RETURN_NORMAL);
        assert!(!exists(BUF), "with -d the buffer is gone afterwards");
    }
}

#[test]
fn a_pane_with_input_off_receives_nothing_but_d_still_applies() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(BUF, "zz");
        attach(t.pane(0), &bev);
        (*t.pane(0)).flags |= PANE_INPUTOFF;

        let mut it = item(c"paste-buffer -d -b buf", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert!(
            bev.written().is_empty(),
            "input-off panes are not written to"
        );
        assert!(!exists(BUF), "deletion happens even when sending did not");
    }
}
