//! Unit tests for [`crate::cmd::cmd_copy_mode`] — the two command entries it
//! publishes (`copy-mode` and `clock-mode`, sharing one exec hook), the
//! message-protocol, display and key constants it declares, and every branch
//! of `cmd_copy_mode_exec` the fixtures can reach without a live daemon or a
//! terminal.
//!
//! Exec is reached through each entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose target and source find states are resolved against
//! a registered [`Target`] (or, for the mouse branches, against a `key_event`
//! whose mouse payload points at that registered state). The mode-opening
//! branches run the real `init`/`free` of window_copy and window_clock over
//! that registered state — history lines are grown in a pane's base grid with
//! `grid_scroll_history` so page-up and slider scrolling land somewhere
//! observable — so every test closes the modes it opened with
//! [`window_pane_reset_mode_all`] before its fixtures go away.
//!
//! Three limits worth recording. First, `-M`'s happy path ends in
//! `window_copy_drag_update`, which arms the mode's drag timer through
//! ensure_reactor, so those tests take [`ensure_reactor`]; nothing runs the loop, so the
//! timer never fires and `window_pane_reset_mode_all` takes it down again.
//! Second, like the other suites: the mode-change notifications are appended
//! to the global command queue, which no unit test ever drains. Third, the
//! `-S` slider arithmetic divides by the pane height; with a zero-height pane
//! that would be float infinity, but no fixture or real pane is zero high, so
//! the branch is pinned at ordinary heights only.

use crate::cmd::cmd_copy_mode::{
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_AFTERHOOK, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, CMD_TARGET_PANE_USAGE, KEYC_ANY, KEYC_DRAGGING,
    KEYC_MASK_KEY, KEYC_MASK_TYPE, KEYC_MOUSE, KEYC_NONE, KEYC_TYPE_DOUBLECLICK,
    KEYC_TYPE_MOUSEDOWN, KEYC_TYPE_MOUSEDRAG, KEYC_TYPE_MOUSEDRAGEND, KEYC_TYPE_MOUSEMOVE,
    KEYC_TYPE_MOUSEUP, KEYC_TYPE_NOTYPE, KEYC_TYPE_SECONDCLICK, KEYC_TYPE_TRIPLECLICK,
    KEYC_UNKNOWN, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH,
    MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN,
    MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK,
    MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK,
    MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY,
    PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE,
    PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE,
    PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND,
    PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_clock_mode_entry, cmd_copy_mode_entry,
};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::{cmdq_get_current, cmdq_get_event};
use crate::grid::{
    grid_clear_history, grid_default_cell, grid_get_cell, grid_scroll_history, grid_set_cell,
};
use crate::modes::{
    CURSORDRAG_ENDSEL, window_copy_backing, window_copy_mode_data, window_copy_set_line_numbers,
};
use crate::screen::{screen_grid, screen_grid_mut};
use crate::tests::test_fixtures::{Item, Target, ascii, ensure_reactor, globals, seen};
use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::{PANE_CHANGED, PANE_REDRAW, PANE_REDRAWSCROLLBAR, window_pane_reset_mode_all};
use ::core::ffi::{c_char, c_int};

/// How many history lines the tests grow into a pane's base grid when they
/// want something to scroll back through.
const HISTORY_LINES: u_int = 30;

/// The entries whose exec hook is under test.
const COPY_MODE: *const cmd_entry = &raw const cmd_copy_mode_entry;
const CLOCK_MODE: *const cmd_entry = &raw const cmd_clock_mode_entry;

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it. The item must be running that entry.
unsafe fn exec_via(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, entry),
            "the item is not running this entry"
        );
        let exec = (*entry).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item carrying a parsed command line, aimed at the registered target:
/// its target, source and current states all point at the current winlink.
unsafe fn item_on(line: &'static ::core::ffi::CStr, t: &mut Target) -> Item {
    unsafe {
        let mut item = Item::new().with_args(line);
        let fs = t.state();
        (*item.ptr()).target = fs.clone();
        (*item.ptr()).source = fs.clone();
        *cmdq_get_current(item.ptr()) = fs;
        item
    }
}

/// Points the item's target state at the target's winlink but its source state
/// at `source`, which is what a resolved `-s src-pane` item carries.
unsafe fn aim_source(item: &mut Item, t: &mut Target, source: cmd_find_state) {
    unsafe {
        let fs = t.state();
        (*item.ptr()).target = fs.clone();
        (*item.ptr()).source = source;
        *cmdq_get_current(item.ptr()) = fs;
    }
}

/// The find state of `wl`, since resolution is the command queue's job and the
/// exec hook reads the states as given.
unsafe fn fs_of(wl: *mut winlink) -> cmd_find_state {
    unsafe {
        let mut fs = *Box::new(cmd_find_state::default());
        cmd_find_from_winlink(&mut fs, wl, 0);
        fs
    }
}

/// Fills the item's queue-state event with a mouse report carrying
/// [`KEYC_MOUSE`] as its key — which is what suppresses line numbers — and
/// claiming `valid`, session `s`, window `w` and pane `wp`, which is how a
/// real mouse event arrives annotated by the tty layer.
unsafe fn mouse_event_of(
    item: &mut Item,
    valid: c_int,
    s: c_int,
    w: c_int,
    wp: c_int,
) -> *mut key_event {
    unsafe {
        let ev = cmdq_get_event(item.ptr());
        (*ev).key = KEYC_MOUSE as key_code;
        (*ev).m.valid = valid;
        (*ev).m.s = s;
        (*ev).m.w = w;
        (*ev).m.wp = wp;
        ev
    }
}

/// The pane's first mode entry, failing if it opened none.
unsafe fn first_mode(wp: *mut window_pane) -> *mut window_mode_entry {
    unsafe {
        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null(), "the pane opened no mode");
        wme
    }
}

/// How many mode entries the pane carries.
unsafe fn mode_count(wp: *mut window_pane) -> usize {
    unsafe { (*wp).modes.len() }
}

/// Takes every mode off `wp`, leaving it back on its base screen.
unsafe fn close_modes(wp: *mut window_pane) {
    unsafe {
        window_pane_reset_mode_all(wp);
        assert!((*wp).modes.is_empty(), "modes were not closed");
        assert_eq!((*wp).screen(), &raw mut (*wp).base);
    }
}

/// The copy-mode data behind the pane's topmost mode entry, failing if the
/// pane is not running copy mode.
unsafe fn copy_data(wp: *mut window_pane) -> *mut window_copy_mode_data {
    unsafe {
        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Copy);
        (*wme).state.copy()
    }
}

/// Grows [`HISTORY_LINES`] lines of empty history into the pane's base grid,
/// the way a full screen of output scrolled off the top would leave it.
fn grow_history(wp: *mut window_pane) {
    for _ in 0..HISTORY_LINES {
        unsafe { grid_scroll_history(screen_grid_mut(&mut (*wp).base), 8) };
    }
}

#[test]
fn the_entries_advertise_their_commands_templates_and_flags() {
    unsafe {
        let e = COPY_MODE;
        assert_eq!((*e).name.to_string_lossy(), "copy-mode");
        assert!((*e).alias.is_none(), "copy-mode has no alias");
        assert_eq!((*e).args.template.to_string_lossy(), "deHMqSs:t:u");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none(), "copy-mode takes no args callback");
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-deHMqSu] [-s src-pane] [-t target-pane]"
        );
        assert_eq!((*e).source.flag, b's' as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, CMD_AFTERHOOK | CMD_READONLY);

        let e = CLOCK_MODE;
        assert_eq!((*e).name.to_string_lossy(), "clock-mode");
        assert!((*e).alias.is_none(), "clock-mode has no alias");
        assert_eq!((*e).args.template.to_string_lossy(), "t:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none(), "clock-mode takes no args callback");
        assert_eq!((*e).usage.to_string_lossy(), "[-t target-pane]");
        assert_eq!((*e).usage, CMD_TARGET_PANE_USAGE);
        assert_eq!((*e).source.flag, 0, "clock-mode has no source flag");
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!(
            (*e).flags,
            CMD_AFTERHOOK,
            "clock-mode may run after hooks only"
        );

        assert_eq!(
            CMD_TARGET_PANE_USAGE.to_bytes_with_nul(),
            b"[-t target-pane]\0"
        );
    }
}

#[test]
fn the_message_and_display_constants_keep_their_values() {
    for (constant, value) in [
        (MSG_READ_CANCEL, 307),
        (MSG_WRITE_CLOSE, 306),
        (MSG_WRITE_READY, 305),
        (MSG_WRITE, 304),
        (MSG_WRITE_OPEN, 303),
        (MSG_READ_DONE, 302),
        (MSG_READ, 301),
        (MSG_READ_OPEN, 300),
        (MSG_FLAGS, 218),
        (MSG_EXEC, 217),
        (MSG_WAKEUP, 216),
        (MSG_UNLOCK, 215),
        (MSG_SUSPEND, 214),
        (MSG_OLDSTDOUT, 213),
        (MSG_OLDSTDIN, 212),
        (MSG_OLDSTDERR, 211),
        (MSG_SHUTDOWN, 210),
        (MSG_SHELL, 209),
        (MSG_RESIZE, 208),
        (MSG_READY, 207),
        (MSG_LOCK, 206),
        (MSG_EXITING, 205),
        (MSG_EXITED, 204),
        (MSG_EXIT, 203),
        (MSG_DETACHKILL, 202),
        (MSG_DETACH, 201),
        (MSG_COMMAND, 200),
        (MSG_IDENTIFY_TERMINFO, 112),
        (MSG_IDENTIFY_LONGFLAGS, 111),
        (MSG_IDENTIFY_STDOUT, 110),
        (MSG_IDENTIFY_FEATURES, 109),
        (MSG_IDENTIFY_CWD, 108),
        (MSG_IDENTIFY_CLIENTPID, 107),
        (MSG_IDENTIFY_DONE, 106),
        (MSG_IDENTIFY_ENVIRON, 105),
        (MSG_IDENTIFY_STDIN, 104),
        (MSG_IDENTIFY_OLDCWD, 103),
        (MSG_IDENTIFY_TTYNAME, 102),
        (MSG_IDENTIFY_TERM, 101),
        (MSG_IDENTIFY_FLAGS, 100),
        (MSG_VERSION, 12),
        (PANE_LINES_SINGLE, 0),
        (PANE_LINES_DOUBLE, 1),
        (PANE_LINES_HEAVY, 2),
        (PANE_LINES_SIMPLE, 3),
        (PANE_LINES_NUMBER, 4),
        (PANE_LINES_SPACES, 5),
        (PROGRESS_BAR_HIDDEN, 0),
        (PROGRESS_BAR_NORMAL, 1),
        (PROGRESS_BAR_ERROR, 2),
        (PROGRESS_BAR_INDETERMINATE, 3),
        (PROGRESS_BAR_PAUSED, 4),
        (SCREEN_CURSOR_DEFAULT, 0),
        (SCREEN_CURSOR_BLOCK, 1),
        (SCREEN_CURSOR_UNDERLINE, 2),
        (SCREEN_CURSOR_BAR, 3),
        (STYLE_ALIGN_DEFAULT, 0),
        (STYLE_ALIGN_LEFT, 1),
        (STYLE_ALIGN_CENTRE, 2),
        (STYLE_ALIGN_RIGHT, 3),
        (STYLE_ALIGN_ABSOLUTE_CENTRE, 4),
        (STYLE_LIST_OFF, 0),
        (STYLE_LIST_ON, 1),
        (STYLE_LIST_FOCUS, 2),
        (STYLE_LIST_LEFT_MARKER, 3),
        (STYLE_LIST_RIGHT_MARKER, 4),
        (STYLE_RANGE_NONE, 0),
        (STYLE_RANGE_LEFT, 1),
        (STYLE_RANGE_RIGHT, 2),
        (STYLE_RANGE_PANE, 3),
        (STYLE_RANGE_WINDOW, 4),
        (STYLE_RANGE_SESSION, 5),
        (STYLE_RANGE_USER, 6),
        (STYLE_RANGE_CONTROL, 7),
        (STYLE_DEFAULT_BASE, 0),
        (STYLE_DEFAULT_PUSH, 1),
        (STYLE_DEFAULT_POP, 2),
        (STYLE_DEFAULT_SET, 3),
        (THEME_UNKNOWN, 0),
        (THEME_LIGHT, 1),
        (THEME_DARK, 2),
        (LAYOUT_LEFTRIGHT, 0),
        (LAYOUT_TOPBOTTOM, 1),
        (LAYOUT_WINDOWPANE, 2),
        (PROMPT_TYPE_COMMAND, 0),
        (PROMPT_TYPE_SEARCH, 1),
        (PROMPT_TYPE_TARGET, 2),
        (PROMPT_TYPE_WINDOW_TARGET, 3),
        (PROMPT_TYPE_INVALID, 255),
        (PROMPT_ENTRY, 0),
        (PROMPT_COMMAND, 1),
        (CLIENT_EXIT_RETURN, 0),
        (CLIENT_EXIT_SHUTDOWN, 1),
        (CLIENT_EXIT_DETACH, 2),
        (CMD_FIND_PANE, 0),
        (CMD_FIND_WINDOW, 1),
        (CMD_FIND_SESSION, 2),
    ] {
        assert_eq!(constant, value);
    }
    for (constant, value) in [
        (CMD_RETURN_ERROR, -1),
        (CMD_RETURN_NORMAL, 0),
        (CMD_RETURN_WAIT, 1),
        (CMD_RETURN_STOP, 2),
        (CMD_READONLY, 0x2),
        (CMD_AFTERHOOK, 0x4),
    ] {
        assert_eq!(constant, value as cmd_retval);
    }
}

#[test]
fn the_key_constants_the_exec_predicate_depends_on_keep_their_values() {
    assert_eq!(KEYC_MASK_KEY, 0xffffffffff_u64);
    assert_eq!(KEYC_MASK_TYPE, 0xff00000000u64);
    assert_eq!(KEYC_MOUSE, 8589934641);
    assert_eq!(KEYC_DRAGGING, 8589934642);
    assert_eq!(KEYC_NONE, 8589934592);
    assert_eq!(KEYC_UNKNOWN, 8589934593);
    assert_eq!(KEYC_ANY, 8589934596);
    assert_eq!(KEYC_TYPE_MOUSEMOVE, 3);
    assert_eq!(KEYC_TYPE_MOUSEDOWN, 4);
    assert_eq!(KEYC_TYPE_MOUSEUP, 5);
    assert_eq!(KEYC_TYPE_MOUSEDRAG, 6);
    assert_eq!(KEYC_TYPE_MOUSEDRAGEND, 7);
    assert_eq!(KEYC_TYPE_SECONDCLICK, 10);
    assert_eq!(KEYC_TYPE_DOUBLECLICK, 11);
    assert_eq!(KEYC_TYPE_TRIPLECLICK, 12);
    assert_eq!(KEYC_TYPE_NOTYPE, 13);
    assert!(
        KEYC_TYPE_MOUSEMOVE < KEYC_TYPE_TRIPLECLICK,
        "the mouse type range the line-number predicate scans is non-empty"
    );
}

#[test]
fn the_parser_resolves_the_commands_to_these_entries() {
    let _guard = globals();
    unsafe {
        for (line, want) in [(c"copy-mode", COPY_MODE), (c"clock-mode", CLOCK_MODE)] {
            let mut item = Item::new().with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, want), "{line:?}");
        }
    }
}

#[test]
fn a_plain_invocation_opens_copy_mode_on_the_target_pane() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Copy);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "copy-mode");
        assert_eq!((*wme).wp, wp);
        assert_eq!(
            (*wme).swp,
            wp,
            "without -s the mode backs onto its own pane"
        );
        assert_eq!((*wme).prefix, 1);
        assert_eq!((*wp).screen(), (*wme).screen);
        assert_ne!((*wp).screen(), &raw mut (*wp).base);
        assert_eq!(
            (*wp).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED),
            PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED
        );

        let data = copy_data(wp);
        assert_eq!(
            (*data).line_numbers,
            1,
            "a plain event keeps line numbers on"
        );
        assert_eq!((*data).scroll_exit, 0, "-e was not given");
        assert_eq!((*data).hide_position, 0, "-H was not given");

        close_modes(wp);
    }
}

#[test]
fn e_and_h_are_copied_into_the_mode_state() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode -He", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        let data = copy_data(wp);
        assert_ne!((*data).scroll_exit, 0, "-e asks to exit at the top");
        assert_ne!((*data).hide_position, 0, "-H hides the position indicator");
        assert_eq!(mode_count(wp), 1);

        close_modes(wp);
    }
}

#[test]
fn running_copy_mode_again_keeps_one_entry_and_restores_line_numbers() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut first = item_on(c"copy-mode", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut first), CMD_RETURN_NORMAL);
        assert_eq!(mode_count(wp), 1);

        window_copy_set_line_numbers(wp, 0);
        assert_eq!(
            (*copy_data(wp)).line_numbers,
            0,
            "the helper turned line numbers off"
        );

        let mut second = item_on(c"copy-mode", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut second), CMD_RETURN_NORMAL);
        assert_eq!(mode_count(wp), 1, "the second run stacked another mode");
        assert_eq!(
            (*copy_data(wp)).line_numbers,
            1,
            "the already-open branch still applies the event's line-number choice"
        );

        close_modes(wp);
    }
}

#[test]
fn q_clears_every_mode_on_the_pane_and_reports_normal() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut opener = item_on(c"copy-mode", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut opener), CMD_RETURN_NORMAL);
        assert!(!(*wp).modes.is_empty());

        let mut quitter = item_on(c"copy-mode -q", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut quitter), CMD_RETURN_NORMAL);
        assert!((*wp).modes.is_empty(), "-q left a mode open");
        assert_eq!(
            (*wp).screen(),
            &raw mut (*wp).base,
            "-q did not restore the base screen"
        );
        assert_eq!(
            (*wp).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED),
            PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED
        );
    }
}

#[test]
fn q_on_a_plain_pane_is_a_harmless_normal() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode -q", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        assert!((*wp).modes.is_empty());
        assert_eq!((*wp).flags, 0, "the pane was untouched");
    }
}

#[test]
fn an_invalid_mouse_event_reports_normal_without_touching_the_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode -M", &mut t);
        mouse_event_of(&mut item, 0, 0, -1, -1);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        assert!(
            (*wp).modes.is_empty(),
            "an invalid mouse report opened a mode"
        );
        assert_eq!((*wp).flags, 0);
    }
}

#[test]
fn a_mouse_event_with_no_client_reports_normal_without_a_mode() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode -M", &mut t);
        mouse_event_of(&mut item, 1, 0, -1, -1);
        assert!(
            crate::cmd::cmdq_get_client(&*item.ptr()).is_null(),
            "the fixture has a client"
        );
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        assert!((*wp).modes.is_empty());
        assert_eq!((*wp).flags, 0);
    }
}

#[test]
fn a_mouse_event_for_another_clients_session_reports_normal() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = Item::with_client().with_args(c"copy-mode -M");
        let fs = t.state();
        (*item.ptr()).target = fs.clone();
        (*item.ptr()).source = fs.clone();
        *cmdq_get_current(item.ptr()) = fs;
        assert_ne!(
            (*item.client()).session,
            t.session(),
            "the client follows the session"
        );
        mouse_event_of(&mut item, 1, 0, -1, -1);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        assert!((*wp).modes.is_empty());
        assert_eq!((*wp).flags, 0);
    }
}

#[test]
fn a_mouse_pane_outside_the_current_window_finds_no_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let second = t.add_window(1, 80, 24);
    unsafe {
        let wp0 = t.pane(0);
        let wp1 = t.pane(second);
        let outside_id = (*wp1).id as c_int;

        let mut item = item_on(c"copy-mode -M", &mut t);
        mouse_event_of(&mut item, 1, 0, -1, outside_id);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        assert!(
            (*wp0).modes.is_empty(),
            "a pane from another window opened a mode"
        );
        assert!((*wp1).modes.is_empty());
    }
}

#[test]
fn m_opens_copy_mode_on_the_mouse_pane_and_starts_a_drag() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = Item::with_client().with_args(c"copy-mode -M");
        let fs = t.state();
        (*item.ptr()).target = fs.clone();
        (*item.ptr()).source = fs.clone();
        *cmdq_get_current(item.ptr()) = fs;
        (*item.client()).session = t.session();
        mouse_event_of(&mut item, 1, 0, -1, -1);

        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        let data = copy_data(wp);
        assert_eq!(
            (*data).line_numbers,
            0,
            "mouse events suppress line numbers"
        );
        assert_eq!(
            (*data).cursordrag,
            CURSORDRAG_ENDSEL,
            "the drag began a selection"
        );
        assert!((*data).screen.sel.is_some(), "no selection was started");
        assert!(
            (*item.client()).tty.mouse_drag_update.is_some(),
            "the drag update callback was not installed"
        );
        assert!(
            (*item.client()).tty.mouse_drag_release.is_some(),
            "the drag release callback was not installed"
        );

        close_modes(wp);
    }
}

#[test]
fn s_opens_the_mode_on_the_target_backed_by_the_source_pane() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let second = t.add_window(1, 80, 24);
    unsafe {
        let wp = t.pane(0);
        let swp = t.pane(second);

        let marked = ascii(b'X');
        grid_set_cell(screen_grid_mut(&mut (*swp).base), 5, 5, &marked);

        let mut item = item_on(c"copy-mode -s whatever", &mut t);
        let source = fs_of(t.winlink(second));
        aim_source(&mut item, &mut t, source);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).wp, wp, "the mode sits on the target pane");
        assert_eq!((*wme).swp, swp, "the source pane was handed to the mode");
        let data = (*wme).state.copy();
        assert_ne!(
            window_copy_backing(&mut *data),
            &raw mut (*swp).base,
            "the backing is a clone of the source screen"
        );
        let got: grid_cell = {
            let mut c = grid_default_cell;
            c = grid_get_cell(screen_grid(&*window_copy_backing(&mut *data)), 5, 5);
            c
        };
        assert_eq!(got.data.data[0], b'X', "the backing lost the source's text");
        let own: grid_cell = {
            let mut c = grid_default_cell;
            c = grid_get_cell(screen_grid(&(*wp).base), 5, 5);
            c
        };
        assert_ne!(own.data.data[0], b'X', "the target pane was written to");

        close_modes(wp);
    }
}

#[test]
fn u_pages_up_a_screenful_and_clamps_at_the_oldest_history() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);

        grow_history(wp);
        let mut item = item_on(c"copy-mode -u", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        let data = copy_data(wp);
        assert_eq!(
            (*data).oy,
            22,
            "page up did not move back one page of 80x24"
        );
        assert_eq!(
            (*data).cy,
            0,
            "the cursor did not stay at the oldest position"
        );
        assert_eq!(mode_count(wp), 1, "-u kept the mode open");
        close_modes(wp);

        grid_clear_history(screen_grid_mut(&mut (*wp).base));
        for _ in 0..10 {
            grid_scroll_history(screen_grid_mut(&mut (*wp).base), 8);
        }
        let mut short = item_on(c"copy-mode -u", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut short), CMD_RETURN_NORMAL);
        let data = copy_data(wp);
        assert_eq!(
            (*data).oy,
            10,
            "page up ran past what little history there was"
        );
        assert_eq!((*data).cy, 0);

        close_modes(wp);
    }
}

#[test]
fn d_pages_down_through_a_fresh_view() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode -d", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        let data = copy_data(wp);
        assert_eq!((*data).oy, 0, "there was nothing to scroll back from");
        assert_eq!((*data).cy, 22, "the cursor did not move down a page");
        assert_eq!(mode_count(wp), 1, "-d without -e keeps the mode open");

        close_modes(wp);
    }
}

#[test]
fn de_opens_then_exits_when_there_is_nothing_to_scroll_back() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode -de", &mut t);
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);
        assert!(
            (*wp).modes.is_empty(),
            "-d with -e on a fresh view did not exit copy mode"
        );
        assert_eq!((*wp).screen(), &raw mut (*wp).base);
    }
}

#[test]
fn big_s_scrolls_by_the_scrollbar_slider_position() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        grow_history(wp);

        let mut item = Item::with_client().with_args(c"copy-mode -S");
        let fs = t.state();
        (*item.ptr()).target = fs.clone();
        (*item.ptr()).source = fs.clone();
        *cmdq_get_current(item.ptr()) = fs;
        (*item.client()).tty.mouse_slider_mpos = 3;
        let ev = mouse_event_of(&mut item, 1, 0, -1, -1);
        (*ev).m.y = 8;

        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        let data = copy_data(wp);
        assert_eq!((*data).oy, 19, "the slider did not move the view");
        assert_eq!(mode_count(wp), 1, "-S kept the mode open");

        close_modes(wp);
    }
}

#[test]
fn a_key_typed_inside_the_mouse_range_suppresses_line_numbers() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode", &mut t);
        let ev = cmdq_get_event(item.ptr());
        (*ev).key = (KEYC_TYPE_MOUSEDRAG as key_code) << 32;
        assert_ne!(
            (*ev).key & KEYC_MASK_KEY,
            KEYC_MOUSE as key_code,
            "the key is not the mouse key itself"
        );
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            (*copy_data(wp)).line_numbers,
            0,
            "a key whose type sits in the mouse range still suppresses line numbers"
        );

        close_modes(wp);
    }
}

#[test]
fn a_key_typed_above_the_mouse_range_keeps_line_numbers() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"copy-mode", &mut t);
        let ev = cmdq_get_event(item.ptr());
        (*ev).key = (KEYC_TYPE_NOTYPE as key_code) << 32;
        assert!(
            (*ev).key & KEYC_MASK_TYPE > (KEYC_TYPE_TRIPLECLICK as key_code) << 32,
            "the key's type is past the end of the mouse range"
        );
        assert_eq!(exec_via(COPY_MODE, &mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            (*copy_data(wp)).line_numbers,
            1,
            "a key typed past the mouse range is not a mouse key"
        );

        close_modes(wp);
    }
}

#[test]
fn clock_mode_opens_clock_mode_with_no_source_pane() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut item = item_on(c"clock-mode", &mut t);
        assert_eq!(exec_via(CLOCK_MODE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Clock);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "clock-mode");
        assert_eq!((*wme).wp, wp);
        assert!(
            (*wme).swp.is_null(),
            "clock mode is never handed a source pane"
        );
        assert_eq!((*wp).screen(), (*wme).screen);
        assert_ne!((*wp).screen(), &raw mut (*wp).base);

        close_modes(wp);
    }
}
