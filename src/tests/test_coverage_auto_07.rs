//! Coverage for [`crate::modes`] constants/mode checks and
//! [`crate::window`] pane/window/winlink helpers.
//!
//! All helpers are deterministic, avoid fatal/daemon paths and use
//! [`globals`] when touching globals or option trees.

use crate::modes::{
    CURSORDRAG_ENDSEL, CURSORDRAG_NONE, CURSORDRAG_SEL, LINE_SEL_LEFT_RIGHT, LINE_SEL_NONE,
    LINE_SEL_RIGHT_LEFT, RECENTRE_BOTTOM, RECENTRE_MIDDLE, RECENTRE_TOP, SEL_CHAR, SEL_LINE,
    SEL_WORD, WINDOW_COPY_CMD_CANCEL, WINDOW_COPY_CMD_CLEAR_ALWAYS,
    WINDOW_COPY_CMD_CLEAR_EMACS_ONLY, WINDOW_COPY_CMD_CLEAR_NEVER, WINDOW_COPY_CMD_NOTHING,
    WINDOW_COPY_CMD_REDRAW, WINDOW_COPY_DRAG_REPEAT_TIME, WINDOW_COPY_JUMPBACKWARD,
    WINDOW_COPY_JUMPFORWARD, WINDOW_COPY_JUMPTOBACKWARD, WINDOW_COPY_JUMPTOFORWARD,
    WINDOW_COPY_LINE_NUMBERS_ABSOLUTE, WINDOW_COPY_LINE_NUMBERS_DEFAULT,
    WINDOW_COPY_LINE_NUMBERS_HYBRID, WINDOW_COPY_LINE_NUMBERS_OFF,
    WINDOW_COPY_LINE_NUMBERS_RELATIVE, WINDOW_COPY_OFF, WINDOW_COPY_SEARCH_ALL_TIMEOUT,
    WINDOW_COPY_SEARCH_MAX_LINE, WINDOW_COPY_SEARCH_TIMEOUT, WINDOW_COPY_SEARCHDOWN,
    WINDOW_COPY_SEARCHUP, window_copy_get_current_offset,
};
use crate::tests::test_fixtures::{Pane, Target, Window, globals, zeroed};
use crate::types::WindowMode;
use crate::window::window_get_active;
use crate::window::{
    window_count_panes, window_find_by_id, window_has_pane, window_pane_exited,
    window_pane_find_by_id, window_pane_index, window_pane_visible, winlink_count,
    winlink_find_by_index,
};

// ---------------------------------------------------------------------------
// window_copy constants
// ---------------------------------------------------------------------------

#[test]
fn window_copy_cursor_and_selection_constants() {
    assert_eq!(CURSORDRAG_NONE, 0);
    assert_eq!(CURSORDRAG_ENDSEL, 1);
    assert_eq!(CURSORDRAG_SEL, 2);

    assert_eq!(LINE_SEL_NONE, 0);
    assert_eq!(LINE_SEL_LEFT_RIGHT, 1);
    assert_eq!(LINE_SEL_RIGHT_LEFT, 2);

    assert_eq!(SEL_CHAR, 0);
    assert_eq!(SEL_WORD, 1);
    assert_eq!(SEL_LINE, 2);

    assert_eq!(RECENTRE_TOP, 0);
    assert_eq!(RECENTRE_MIDDLE, 1);
    assert_eq!(RECENTRE_BOTTOM, 2);
}

#[test]
fn window_copy_search_and_line_number_constants() {
    assert_eq!(WINDOW_COPY_OFF, 0);
    assert_eq!(WINDOW_COPY_SEARCHUP, 1);
    assert_eq!(WINDOW_COPY_SEARCHDOWN, 2);
    assert_eq!(WINDOW_COPY_JUMPFORWARD, 3);
    assert_eq!(WINDOW_COPY_JUMPBACKWARD, 4);
    assert_eq!(WINDOW_COPY_JUMPTOFORWARD, 5);
    assert_eq!(WINDOW_COPY_JUMPTOBACKWARD, 6);

    assert_eq!(WINDOW_COPY_LINE_NUMBERS_OFF, 0);
    assert_eq!(WINDOW_COPY_LINE_NUMBERS_DEFAULT, 1);
    assert_eq!(WINDOW_COPY_LINE_NUMBERS_ABSOLUTE, 2);
    assert_eq!(WINDOW_COPY_LINE_NUMBERS_RELATIVE, 3);
    assert_eq!(WINDOW_COPY_LINE_NUMBERS_HYBRID, 4);

    assert_eq!(WINDOW_COPY_CMD_NOTHING, 0);
    assert_eq!(WINDOW_COPY_CMD_REDRAW, 1);
    assert_eq!(WINDOW_COPY_CMD_CANCEL, 2);

    assert_eq!(WINDOW_COPY_CMD_CLEAR_ALWAYS, 0);
    assert_eq!(WINDOW_COPY_CMD_CLEAR_NEVER, 1);
    assert_eq!(WINDOW_COPY_CMD_CLEAR_EMACS_ONLY, 2);

    // timeouts from window_copy.rs:309-362
    assert_eq!(WINDOW_COPY_SEARCH_TIMEOUT, 10000);
    assert_eq!(WINDOW_COPY_SEARCH_ALL_TIMEOUT, 200);
    assert_eq!(WINDOW_COPY_SEARCH_MAX_LINE, 2000);
    assert_eq!(WINDOW_COPY_DRAG_REPEAT_TIME, 50000);
}

#[test]
fn window_copy_mode_names_match_expected() {
    unsafe {
        let n1 = WindowMode::Copy.name();
        let n2 = WindowMode::View.name();
        assert_eq!(n1, c"copy-mode");
        assert_eq!(n2, c"view-mode");
        assert_ne!(n1.as_ptr(), n2.as_ptr());
    }
}

// ---------------------------------------------------------------------------
// window helpers with Pane/Window fixtures
// ---------------------------------------------------------------------------

#[test]
fn window_helpers_count_and_has_pane() {
    let _guard = globals();
    let mut w = Window::new(10, "win-helpers", 80, 24);
    let mut p1 = Pane::new(101, 80, 24, 100);
    let mut p2 = Pane::new(102, 80, 24, 100);
    let mut outsider = Pane::new(999, 80, 24, 100);
    unsafe {
        assert_eq!(window_count_panes(w.ptr(), 1), 0);
        assert_eq!(window_count_panes(w.ptr(), 0), 0);
        w.add_pane(&mut p1);
        assert_eq!(window_count_panes(w.ptr(), 1), 1);
        assert_eq!(window_has_pane(w.ptr(), p1.ptr()), 1);
        assert_eq!(window_has_pane(w.ptr(), outsider.ptr()), 0);
        w.add_pane(&mut p2);
        assert_eq!(window_count_panes(w.ptr(), 1), 2);
        assert_eq!(window_has_pane(w.ptr(), p2.ptr()), 1);
        // first pane is active
        assert_eq!(window_get_active(w.ptr()), p1.ptr());
    }
}

#[test]
fn window_pane_visible_and_exited_with_fixtures() {
    let _guard = globals();
    let mut w = Window::new(11, "vis", 80, 24);
    let mut p = Pane::new(201, 80, 24, 100);
    w.add_pane(&mut p);
    unsafe {
        // no zoom -> visible
        assert_eq!(window_pane_visible(p.ptr()), 1);
        // fd == -1 means exited (no process)
        assert_eq!(window_pane_exited(p.ptr()), 1);
        // give it a fake fd -> not exited unless PANE_EXITED flag
        (*p.ptr()).fd = 5;
        assert_eq!(window_pane_exited(p.ptr()), 0);
        (*p.ptr()).flags |= crate::window::PANE_EXITED;
        assert_eq!(window_pane_exited(p.ptr()), 1);
        (*p.ptr()).flags &= !crate::window::PANE_EXITED;
        (*p.ptr()).fd = -1;

        // zoomed: only active pane visible
        (*w.ptr()).flags |= crate::window::WINDOW_ZOOMED;
        assert_eq!(window_pane_visible(p.ptr()), 1);
        let mut p2 = Pane::new(202, 80, 24, 100);
        w.add_pane(&mut p2);
        // p2 is not active, so not visible when zoomed
        assert_eq!(window_pane_visible(p2.ptr()), 0);
        assert_eq!(window_pane_visible(p.ptr()), 1);
        (*w.ptr()).flags &= !crate::window::WINDOW_ZOOMED;
    }
}

#[test]
fn window_pane_index_respects_pane_base_index() {
    let _guard = globals();
    let mut w = Window::new(12, "idx", 80, 24);
    let mut p1 = Pane::new(301, 80, 24, 100);
    let mut p2 = Pane::new(302, 80, 24, 100);
    w.add_pane(&mut p1);
    w.add_pane(&mut p2);
    unsafe {
        // default pane-base-index is 0
        assert_eq!(window_pane_index(p1.ptr()), (0, 0));
        assert_eq!(window_pane_index(p2.ptr()), (0, 1));

        // change base to 1
        crate::options::options_set_number(
            (*w.ptr()).options_ptr(),
            c"pane-base-index".as_ptr(),
            1,
        );
        assert_eq!(window_pane_index(p1.ptr()), (0, 1));
        assert_eq!(window_pane_index(p2.ptr()), (0, 2));
    }
}

#[test]
fn winlink_and_window_find_via_target() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let w = target.window(0);
        let wl = target.winlink(0);
        // winlink_count for session's window list
        let s = target.session();
        assert_eq!(winlink_count(&raw mut (*s).windows), 1);
        let found = winlink_find_by_index(&raw mut (*s).windows, 0);
        assert_eq!(found, wl);
        assert!(winlink_find_by_index(&raw mut (*s).windows, 99).is_null());

        // window_find_by_id uses global window tree (Registry)
        assert_eq!(window_find_by_id((*w).id), w);
        assert!(window_find_by_id(99999).is_null());
        assert_eq!(window_pane_find_by_id((*target.pane(0)).id), target.pane(0));
        assert!(window_pane_find_by_id(99999).is_null());
    }
}

#[test]
fn window_copy_get_current_offset_returns_zero_without_copy_mode() {
    let _guard = globals();
    let mut w = Window::new(13, "copy-off", 80, 24);
    let mut p = Pane::new(401, 80, 24, 100);
    w.add_pane(&mut p);
    unsafe {
        // pane with a wme whose data is null -> returns 0 (null-data guard)
        (*p.ptr())
            .modes
            .push(zeroed::<crate::types::window_mode_entry>());
        // state is None by zeroed
        assert!(matches!(
            (*p.ptr()).modes[0].state,
            crate::types::WindowModeState::None
        ));
        assert!(window_copy_get_current_offset(p.ptr()).is_none());
        (*p.ptr()).modes.clear();
    }
}
