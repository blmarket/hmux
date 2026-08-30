//! Unit tests for [`crate::control`] — the control-mode plumbing.
//!
//! What is reachable without a live server is the metadata the module carries
//! and a handful of buffer helpers that `control_write`/`control_all_done`
//! expose through a fake client. Every test stays off the event loop and off
//! any fork: `StreamBuffer` backs the client's `write_event`, `control_state::default`
//! builds the map heads, and the `control_*` helpers are driven with zeroed
//! structs the way the fixtures do. Nothing here calls `fatal` and nothing
//! touches a real pane's data path.

use crate::control::{
    BUFFER_EOL_ANY, BUFFER_EOL_CRLF, BUFFER_EOL_CRLF_STRICT, BUFFER_EOL_LF, BUFFER_EOL_NUL,
    CLIENT_CONTROL_NOOUTPUT, CLIENT_CONTROL_PAUSEAFTER, CLIENT_DEAD, CLIENT_EXIT, CLIENT_SUSPENDED,
    CONTROL_BUFFER_HIGH, CONTROL_BUFFER_LOW, CONTROL_IGNORE_FLAGS, CONTROL_MAXIMUM_AGE,
    CONTROL_PANE_OFF, CONTROL_PANE_PAUSED, CONTROL_SUB_ALL_PANES, CONTROL_SUB_ALL_WINDOWS,
    CONTROL_SUB_PANE, CONTROL_SUB_SESSION, CONTROL_SUB_WINDOW, CONTROL_WRITE_MINIMUM, MSG_COMMAND,
    MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS,
    MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, control_add_sub, control_all_done,
    control_continue_pane, control_discard, control_pane_offset, control_pause_pane, control_ready,
    control_remove_sub, control_reset_offsets, control_set_pane_off, control_set_pane_on,
    control_state, control_write,
};
use crate::reactor::Timer;
use crate::tests::test_fixtures::{StreamBuffer, globals, zeroed_client, zeroed_pane};
use crate::types::*;
use ::std::ffi::CString;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// A fake control client whose `write_event` is a [`StreamBuffer`]. The client
/// owns the `control_state`, so `control_*` reaches it through
/// `c.control_state` and it goes when the client does.
struct FakeControl {
    client: ClientRef,
    _bev: StreamBuffer,
    _guard: ::std::sync::MutexGuard<'static, ()>,
}

impl FakeControl {
    fn new() -> FakeControl {
        let guard = globals();
        let bev = StreamBuffer::new();
        let mut state = Box::new(control_state::default());
        state.write_event = bev.ptr();
        state.read_event = crate::reactor::Stream::NONE;
        let mut client = zeroed_client();
        client.control_state = Some(state);
        // give the client a name so log_debug has something to print
        client.name = Some(CString::new("fake").unwrap());
        FakeControl {
            client,
            _bev: bev,
            _guard: guard,
        }
    }

    fn ptr(&mut self) -> *mut client {
        &raw mut *self.client
    }

    /// The control state the client owns.
    fn state(&mut self) -> *mut control_state {
        &raw mut **self
            .client
            .control_state
            .as_mut()
            .expect("the client keeps its control state")
    }

    fn written(&self) -> Vec<u8> {
        self._bev.written()
    }
}

impl Drop for FakeControl {
    fn drop(&mut self) {
        unsafe {
            // disarm timer if armed – the reactor base is global
            let cs = self.state();
            (*cs).subs_timer.disarm();
            // free any queued control_blocks that a test may have left
            (*cs).all_blocks.clear();
            // free any manually inserted panes (control_reset_offsets would
            // normally do this, but tests may not call it)
            for (_, cp) in ::core::mem::take(&mut (*cs).panes) {
                drop(cp);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn the_buffer_eol_constants_keep_their_values() {
    assert_eq!(BUFFER_EOL_ANY, 0);
    assert_eq!(BUFFER_EOL_CRLF, 1);
    assert_eq!(BUFFER_EOL_CRLF_STRICT, 2);
    assert_eq!(BUFFER_EOL_LF, 3);
    assert_eq!(BUFFER_EOL_NUL, 4);
}

#[test]
fn the_message_constants_keep_their_upstream_values() {
    assert_eq!(MSG_VERSION, 12);
    // identify 100..112
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
    // command 200..218
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
    // read/write 300..307
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
fn the_enumerated_control_constants_keep_their_orderings() {
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

    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_LIGHT, 1);
    assert_eq!(THEME_DARK, 2);

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

    assert_eq!(CONTROL_SUB_SESSION, 0);
    assert_eq!(CONTROL_SUB_PANE, 1);
    assert_eq!(CONTROL_SUB_ALL_PANES, 2);
    assert_eq!(CONTROL_SUB_WINDOW, 3);
    assert_eq!(CONTROL_SUB_ALL_WINDOWS, 4);
}

#[test]
fn the_control_buffer_and_flag_constants_keep_their_values() {
    assert_eq!(CONTROL_BUFFER_LOW, 512);
    assert_eq!(CONTROL_BUFFER_HIGH, 8192);
    assert_eq!(CONTROL_WRITE_MINIMUM, 32);
    assert_eq!(CONTROL_MAXIMUM_AGE, 300000);
    assert_eq!(CONTROL_PANE_OFF, 0x1);
    assert_eq!(CONTROL_PANE_PAUSED, 0x2);
    assert_eq!(CLIENT_EXIT, 0x4);
    assert_eq!(CLIENT_SUSPENDED, 0x40);
    assert_eq!(CLIENT_DEAD, 0x200);
    assert_eq!(CLIENT_CONTROL_NOOUTPUT, 0x4000000);
    assert_eq!(CLIENT_CONTROL_PAUSEAFTER, 0x100000000u64);
    assert_eq!(
        CONTROL_IGNORE_FLAGS as u64,
        CLIENT_CONTROL_NOOUTPUT as u64
            | CLIENT_DEAD as u64
            | CLIENT_SUSPENDED as u64
            | CLIENT_EXIT as u64
    );
}

// ---------------------------------------------------------------------------
// control_state::default
// ---------------------------------------------------------------------------

#[test]
fn empty_control_state_starts_with_no_panes_blocks_or_subs() {
    let cs = control_state::default();
    assert!(cs.panes.is_empty());
    assert!(cs.subs.is_empty());
    assert_eq!(cs.pending_count, 0);
    assert!(cs.pending_list.is_empty());
    assert!(cs.all_blocks.is_empty());
    // timer starts disarmed
    assert!(!cs.subs_timer.is_set());
}

// ---------------------------------------------------------------------------
// control_all_done
// ---------------------------------------------------------------------------

#[test]
fn all_done_is_true_when_nothing_is_queued_and_false_otherwise() {
    let mut fc = FakeControl::new();
    unsafe {
        assert_eq!(control_all_done(fc.ptr()), 1, "empty state is done");
        // write a line – goes straight to the evbuffer because all_blocks is empty
        control_write(fc.ptr(), c"hello".as_ptr(), &[]);
        // now the output buffer is non-empty, so not done
        assert_eq!(control_all_done(fc.ptr()), 0);
        // draining is done by reading via written(), but the evbuffer still
        // holds the data until the loop drains it; control_all_done still sees it
        let data = fc.written();
        assert_eq!(data, b"hello\n");
        // evbuffer still reports the length as seen by control_all_done?
        // StreamBuffer keeps the evbuffer length; written() only tracks a cursor,
        // so control_all_done still returns 0
        assert_eq!(control_all_done(fc.ptr()), 0);
    }
}

// ---------------------------------------------------------------------------
// control_write – immediate vs queued
// ---------------------------------------------------------------------------

#[test]
fn control_write_goes_immediately_when_no_blocks_are_queued() {
    let mut fc = FakeControl::new();
    unsafe {
        control_write(fc.ptr(), c"%%hello %u".as_ptr(), crate::fmt_args![42u32]);
        let data = fc.written();
        assert_eq!(data, b"%hello 42\n");
        // no block was queued
        assert!((*fc.state()).all_blocks.is_empty());
    }
}

#[test]
fn control_write_queues_when_blocks_are_pending() {
    let mut fc = FakeControl::new();
    unsafe {
        // fabricate a pending block so the next write is queued
        let cs = fc.state();
        let cb_owned = Box::new(crate::control::control_block {
            size: 0,
            line: Some(c"queued".to_owned()),
            t: 0,
        });
        let cb = &raw const *cb_owned;
        (*cs).all_blocks.push(cb_owned);

        // now a write should be queued, not emitted
        control_write(fc.ptr(), c"second".as_ptr(), &[]);
        assert_eq!(
            fc.written(),
            Vec::<u8>::new(),
            "queued write does not hit the buffer"
        );
        assert_eq!(
            (*cs).all_blocks.len(),
            2,
            "the second block is behind the first"
        );
        assert!(::core::ptr::eq(&raw const *(*cs).all_blocks[0], cb));
        // all_done must be false while blocks are queued
        assert_eq!(control_all_done(fc.ptr()), 0);
    }
}

// ---------------------------------------------------------------------------
// control_reset_offsets
// ---------------------------------------------------------------------------

#[test]
fn reset_offsets_clears_panes_and_pending_list() {
    let mut fc = FakeControl::new();
    unsafe {
        let cs = fc.state();
        // insert a dummy pane directly into the map
        let mut cp_box = Box::new(crate::control::control_pane {
            pane: 99,
            offset: window_pane_offset { used: 0 },
            queued: window_pane_offset { used: 0 },
            flags: 0,
            pending_flag: 0,
            blocks: crate::control::control_pane_blocks::new(),
        });
        let cp = &raw mut *cp_box;
        (*cs).panes.insert(99, cp_box);
        (*cs).pending_count = 1;
        (*cs).pending_list.push(cp);

        control_reset_offsets(fc.ptr());

        assert!((*cs).panes.is_empty(), "panes cleared");
        assert!((*cs).pending_list.is_empty(), "pending list cleared");
        assert_eq!((*cs).pending_count, 0);
    }
}

// ---------------------------------------------------------------------------
// control_pane_offset
// ---------------------------------------------------------------------------

#[test]
fn pane_offset_respects_nooutput_and_missing_pane() {
    let mut fc = FakeControl::new();
    let mut wp = zeroed_pane();
    wp.id = 1;
    unsafe {
        // NOOUTPUT flag forces off=0 and null return
        (*fc.ptr()).flags |= CLIENT_CONTROL_NOOUTPUT as u64;
        let (ret, off) = control_pane_offset(fc.ptr(), &raw mut *wp);
        assert!(ret.is_null());
        assert_eq!(off, 0);

        // without the flag but with no pane registered, same result
        (*fc.ptr()).flags &= !(CLIENT_CONTROL_NOOUTPUT as u64);
        let (ret, off) = control_pane_offset(fc.ptr(), &raw mut *wp);
        assert!(ret.is_null());
        assert_eq!(off, 0);
    }
}

#[test]
fn control_pane_state_transitions() {
    let mut fc = FakeControl::new();
    let mut wp = zeroed_pane();
    wp.id = 5;
    wp.offset = window_pane_offset { used: 100 };
    unsafe {
        control_set_pane_off(fc.ptr(), &raw mut *wp);
        control_set_pane_on(fc.ptr(), &raw mut *wp);

        control_pause_pane(fc.ptr(), &raw mut *wp);
        control_continue_pane(fc.ptr(), &raw mut *wp);
    }
}

#[test]
fn control_subs_add_and_remove() {
    let mut fc = FakeControl::new();
    unsafe {
        control_add_sub(
            fc.ptr(),
            c"sub1".as_ptr(),
            crate::control::CONTROL_SUB_SESSION,
            0,
            c"#{session_name}".as_ptr(),
        );

        control_ready(fc.ptr());
        control_discard(fc.ptr());

        control_remove_sub(fc.ptr(), c"sub1".as_ptr());
    }
}
