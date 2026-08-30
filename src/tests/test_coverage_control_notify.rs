//! Unit tests for [`crate::control`] — the notification fan-out
//! that walks the server's client list and hands one `%`-line to every
//! control-mode client, plus the module's message-type and style constants.
//!
//! Every notify function is driven over the real [`Clients`] list: a client
//! only receives a line when it carries [`CLIENT_CONTROL`] **and** a
//! [`control_state`], and the session-dependent functions filter further by
//! whether the window sits in that client's own session. Writes land in the
//! buffer event of a [`ControlOut`], where they can be read back verbatim,
//! newline included — `control_write` goes straight through `control_vwrite`
//! because the queued-block path belongs to control.rs, and `log_debug` is
//! inert here with no log file open. Nothing runs the event loop, so what is
//! written stays put until the test reads it.
//!
//! Not exercised: the queued-block path behind a full `all_blocks` queue,
//! which is control.rs's flush machinery, and any terminal or process side —
//! the buffer events ride socket pairs nobody reads.

use crate::control::control_state;
use crate::control::*;
use crate::server::CLIENT_CONTROL;
use crate::tests::test_fixtures::{
    Clients, Layout, Pane, Session, StreamBuffer, Window, globals, link, unlink,
};
use ::core::ptr::null_mut;

/// A control-mode client's write side: the state [`control_write`] reaches
/// through the client and the buffer event it writes into, read back with
/// [`ControlOut::written`]. Detaches itself from the client when it goes.
struct ControlOut {
    c: *mut client,
    bev: StreamBuffer,
}

impl ControlOut {
    /// Marks `c` a control client and gives it a fresh empty state,
    /// writing through the buffer event.
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

#[test]
fn the_read_and_write_message_types_keep_their_values() {
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
fn the_server_message_types_keep_their_values() {
    assert_eq!(MSG_VERSION, 12);
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
}

#[test]
fn the_identify_message_types_keep_their_values() {
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
}

#[test]
fn the_pane_progress_bar_and_cursor_enumerations_keep_their_values() {
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
}

#[test]
fn the_style_enumerations_keep_their_values() {
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
}

#[test]
fn the_theme_layout_prompt_and_exit_enumerations_keep_their_values() {
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
}

#[test]
fn the_control_marker_keeps_its_value() {
    assert_eq!(CLIENT_CONTROL, 0x2000);
}

#[test]
fn notifications_reach_only_control_clients_carrying_a_state() {
    let _guard = globals();
    let mut list = Clients::new();
    let bare = list.add("bare", 80, 24);
    let ctrl = list.add("ctrl", 80, 24);
    unsafe {
        (*bare).flags |= CLIENT_CONTROL as u64;
    }
    let out = ControlOut::new(ctrl);
    unsafe {
        control_notify_pane_mode_changed(5);

        assert_eq!(
            out.written(),
            b"%pane-mode-changed %5\n",
            "the flag and the state together let the line through"
        );

        (*ctrl).flags &= !(CLIENT_CONTROL as u64);
        control_notify_pane_mode_changed(5);
        assert_eq!(
            out.written(),
            Vec::<u8>::new(),
            "the flag decides, not the state alone"
        );
        (*ctrl).flags |= CLIENT_CONTROL as u64;

        assert_eq!(out.written(), Vec::<u8>::new());
        assert!(
            (*bare).control_state.is_none(),
            "the walk leaves a stateless client alone"
        );
    }
}

#[test]
fn layout_changes_are_skipped_until_a_laid_out_window_is_held_by_ones_own_session() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    let mut bare = Window::new(20, "unlaid", 80, 24);
    let mut holding = Session::new(2, "holding");
    let mut elsewhere = Session::new(3, "elsewhere");
    let mut list = Clients::new();
    let watcher = list.add("watcher", 80, 24);
    let out = ControlOut::new(watcher);
    unsafe {
        (*watcher).session = elsewhere.ptr();
        control_notify_window_layout_changed(bare.ptr());
        assert_eq!(
            out.written(),
            Vec::<u8>::new(),
            "no winlink on the window ends the walk early"
        );

        (*watcher).session = holding.ptr();
        let bare_wl = link(&mut holding, &mut bare, 1);
        control_notify_window_layout_changed(bare.ptr());
        assert_eq!(
            out.written(),
            Vec::<u8>::new(),
            "a linked window without a layout tree ends the walk early"
        );
        unlink(&mut holding, bare_wl);

        (*watcher).session = elsewhere.ptr();
        let laid_wl = link(&mut holding, l.window(), 0);
        control_notify_window_layout_changed(l.w());
        assert_eq!(
            out.written(),
            Vec::<u8>::new(),
            "another session's client is not told"
        );

        (*watcher).session = null_mut();
        control_notify_window_layout_changed(l.w());
        assert_eq!(
            out.written(),
            Vec::<u8>::new(),
            "a sessionless client is not told"
        );

        (*watcher).session = holding.ptr();
        control_notify_window_layout_changed(l.w());
        assert_eq!(
            out.written(),
            b"%layout-change @1 b25e,80x24,0,0,1 b25e,80x24,0,0,1 *\n"
        );

        unlink(&mut holding, laid_wl);
    }
}

#[test]
fn pane_changes_need_an_active_pane_and_then_name_window_and_pane() {
    let _guard = globals();
    let mut w = Window::new(9, "panes", 80, 24);
    let mut p = Pane::new(3, 80, 24, 100);
    let mut list = Clients::new();
    let ctrl = list.add("ctrl", 80, 24);
    let out = ControlOut::new(ctrl);
    unsafe {
        control_notify_window_pane_changed(w.ptr());
        assert!(
            out.written().is_empty(),
            "a window without an active pane is skipped"
        );

        w.add_pane(&mut p);
        control_notify_window_pane_changed(w.ptr());
        assert_eq!(out.written(), b"%window-pane-changed @9 %3\n");
    }
}

#[test]
fn add_close_and_rename_lines_follow_each_clients_own_session() {
    let _guard = globals();
    let mut home = Session::new(4, "home");
    let mut away = Session::new(5, "away");
    let mut w = Window::new(4, "shared", 80, 24);
    let mut list = Clients::new();
    let watcher = list.add("watcher", 80, 24);
    let out = ControlOut::new(watcher);
    unsafe {
        let wl = link(&mut home, &mut w, 0);

        (*watcher).session = home.ptr();
        control_notify_window_linked(home.ptr(), w.ptr());
        assert_eq!(out.written(), b"%window-add @4\n");

        control_notify_window_unlinked(home.ptr(), w.ptr());
        assert_eq!(out.written(), b"%window-close @4\n");

        control_notify_window_renamed(w.ptr());
        assert_eq!(out.written(), b"%window-renamed @4 shared\n");

        (*watcher).session = away.ptr();
        control_notify_window_linked(home.ptr(), w.ptr());
        assert_eq!(
            out.written(),
            b"%unlinked-window-add @4\n",
            "away has no winlink for the window"
        );

        control_notify_window_unlinked(home.ptr(), w.ptr());
        assert_eq!(out.written(), b"%unlinked-window-close @4\n");

        control_notify_window_renamed(w.ptr());
        assert_eq!(out.written(), b"%unlinked-window-renamed @4 shared\n");

        (*watcher).session = null_mut();
        control_notify_window_linked(home.ptr(), w.ptr());
        assert_eq!(
            out.written(),
            Vec::<u8>::new(),
            "a sessionless client is skipped"
        );

        unlink(&mut home, wl);
    }
}

#[test]
fn session_changes_speak_to_the_moved_client_differently_from_the_rest() {
    let _guard = globals();
    let mut moved_to = Session::new(6, "six");
    let mut list = Clients::new();
    let mover = list.add("mover", 80, 24);
    let watcher = list.add("watcher", 80, 24);
    unsafe {
        (*watcher).session = moved_to.ptr();
    }
    let out = ControlOut::new(watcher);
    unsafe {
        control_notify_client_session_changed(mover);
        assert!(
            out.written().is_empty(),
            "a client moving from nowhere is not announced"
        );

        (*mover).session = moved_to.ptr();
        control_notify_client_session_changed(mover);
        assert_eq!(out.written(), b"%client-session-changed mover $6 six\n");

        control_notify_client_session_changed(watcher);
        assert_eq!(
            out.written(),
            b"%session-changed $6 six\n",
            "the mover hears its own shorter line"
        );
    }
}

#[test]
fn detach_rename_and_lifecycle_lines_name_their_subjects() {
    let _guard = globals();
    let mut s = Session::new(7, "seven");
    let mut list = Clients::new();
    let leaver = list.add("gone", 80, 24);
    let ctrl = list.add("ctrl", 80, 24);
    let out = ControlOut::new(ctrl);
    unsafe {
        control_notify_client_detached(leaver);
        assert_eq!(out.written(), b"%client-detached gone\n");

        control_notify_session_created(s.ptr());
        assert_eq!(out.written(), b"%sessions-changed\n");

        control_notify_session_closed(s.ptr());
        assert_eq!(out.written(), b"%sessions-changed\n");

        control_notify_session_renamed(s.ptr());
        assert_eq!(out.written(), b"%session-renamed $7 seven\n");
    }
}

#[test]
fn session_window_changes_report_the_current_window() {
    let _guard = globals();
    let mut s = Session::new(8, "windows");
    let mut w = Window::new(5, "current", 80, 24);
    let mut list = Clients::new();
    let ctrl = list.add("ctrl", 80, 24);
    let out = ControlOut::new(ctrl);
    unsafe {
        let wl = link(&mut s, &mut w, 0);

        control_notify_session_window_changed(s.ptr());
        assert_eq!(out.written(), b"%session-window-changed $8 @5\n");

        unlink(&mut s, wl);
    }
}

#[test]
fn paste_buffer_lines_carry_the_buffer_name() {
    let _guard = globals();
    let mut list = Clients::new();
    let ctrl = list.add("ctrl", 80, 24);
    let out = ControlOut::new(ctrl);
    unsafe {
        control_notify_paste_buffer_changed(c"buf-one".as_ptr());
        assert_eq!(out.written(), b"%paste-buffer-changed buf-one\n");

        control_notify_paste_buffer_deleted(c"buf-two".as_ptr());
        assert_eq!(out.written(), b"%paste-buffer-deleted buf-two\n");
    }
}
