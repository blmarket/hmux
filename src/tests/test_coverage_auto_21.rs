//! Coverage for [`crate::window`] – additional helpers not covered by beta
//! (window/pane helpers, zoom bookkeeping, resize, floating, search etc).

use crate::layout::{
    LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, layout_assign_pane, layout_split_pane,
};
use crate::screen::screen_grid_ptr;
use crate::tests::test_fixtures::{Layout, Pane, Window, globals};
use crate::types::layout_type;
use crate::window::{
    PANE_EXITED, WINDOW_ZOOMED, window_find_string, window_has_floating_panes, window_has_pane,
    window_pane_exited, window_pane_find_down, window_pane_find_left, window_pane_find_right,
    window_pane_find_up, window_pane_is_floating, window_pane_search, window_pane_visible,
    window_resize,
};

fn split(l: &mut Layout, i: usize, ty: layout_type) -> usize {
    unsafe {
        let lc = layout_split_pane(l.pane(i), ty, -1, 0);
        assert!(!lc.is_null(), "split failed");
        let j = l.add_pane(1, 1);
        layout_assign_pane(lc, l.pane(j), 0);
        j
    }
}

#[test]
fn window_resize_updates_dimensions() {
    let _guard = globals();
    let mut w = Window::new(1, "r", 80, 24);
    unsafe {
        assert_eq!((*w.ptr()).sx, 80);
        assert_eq!((*w.ptr()).sy, 24);
        window_resize(w.ptr(), 100, 40, -1, -1);
        assert_eq!((*w.ptr()).sx, 100);
        assert_eq!((*w.ptr()).sy, 40);
        let xpixel_before = (*w.ptr()).xpixel;
        window_resize(w.ptr(), 100, 40, 0, 0);
        // 0 resets to default
        assert_ne!((*w.ptr()).xpixel, 0);
        assert_ne!((*w.ptr()).ypixel, 0);
        let _ = xpixel_before;
    }
}

#[test]
fn window_has_floating_panes_and_is_floating() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        let w = l.w();
        let p0 = l.pane(0);
        let p1 = l.pane(j);
        assert_eq!(window_has_floating_panes(w), 0);
        assert_eq!(window_pane_is_floating(p0), 0);
        assert_eq!(window_pane_is_floating(p1), 0);
        // mark p1 floating via its layout cell
        let cell = (*p1).layout_cell;
        assert!(!cell.is_null());
        (*cell).flags |= LAYOUT_CELL_FLOATING;
        assert_ne!(window_pane_is_floating(p1), 0);
        assert_eq!(window_has_floating_panes(w), 1);
        // unmark
        (*cell).flags &= !LAYOUT_CELL_FLOATING;
        assert_eq!(window_has_floating_panes(w), 0);
    }
}

#[test]
fn window_pane_visible_respects_zoom() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        let w = l.w();
        let p0 = l.pane(0);
        let p1 = l.pane(j);
        // without zoom every pane visible
        assert_ne!(window_pane_visible(p0), 0);
        assert_ne!(window_pane_visible(p1), 0);
        // zoom p1: window gets WINDOW_ZOOMED, only active visible if we set active
        // Layout starts active = first pane; make p1 active then zoom
        // Use window_zoom on p1 (needs two panes)
        let rc = crate::window::window_zoom(p1);
        assert_eq!(rc, 0);
        assert_ne!((*w).flags & WINDOW_ZOOMED, 0);
        // p1 is active after zoom (window_zoom sets active if needed)
        assert_ne!(window_pane_visible(p1), 0);
        // p0 should be hidden while zoomed
        assert_eq!(window_pane_visible(p0), 0);
        // unzoom restores
        let rc2 = crate::window::window_unzoom(w, 0);
        assert_eq!(rc2, 0);
        assert_eq!((*w).flags & WINDOW_ZOOMED, 0);
        assert_ne!(window_pane_visible(p0), 0);
    }
}

#[test]
fn window_pane_exited_reports_fd_and_flag() {
    let _guard = globals();
    let mut p = Pane::new(1, 80, 24, 100);
    unsafe {
        let wp = p.ptr();
        // fd == -1 counts as exited
        assert_ne!(window_pane_exited(wp), 0);
        // give it a fake fd and clear exited flag -> not exited
        (*wp).fd = 99;
        (*wp).flags &= !PANE_EXITED;
        assert_eq!(window_pane_exited(wp), 0);
        // exited flag alone counts too
        (*wp).flags |= PANE_EXITED;
        assert_ne!(window_pane_exited(wp), 0);
        (*wp).fd = -1;
        (*wp).flags &= !PANE_EXITED;
        // back to exited via fd
        assert_ne!(window_pane_exited(wp), 0);
        // avoid close on drop
        (*wp).fd = -1;
    }
}

#[test]
fn window_weak_handles_follow_the_window_lifetime() {
    let _guard = globals();
    let w = Window::new(2, "ref", 80, 24);
    let weak = w.weak();
    assert!(weak.upgrade().is_some());
    drop(w);
    assert!(weak.upgrade().is_none());
}

#[test]
fn window_find_string_maps_positions() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    // single pane fills whole window: every position resolves to that pane
    unsafe {
        let w = l.w();
        let p0 = l.pane(0);
        assert_eq!(window_find_string(w, c"top".as_ptr()), p0);
        assert_eq!(window_find_string(w, c"bottom".as_ptr()), p0);
        assert_eq!(window_find_string(w, c"left".as_ptr()), p0);
        assert_eq!(window_find_string(w, c"right".as_ptr()), p0);
        assert_eq!(window_find_string(w, c"top-left".as_ptr()), p0);
        assert_eq!(window_find_string(w, c"bottom-right".as_ptr()), p0);
        assert!(window_find_string(w, c"centre".as_ptr()).is_null());
        assert!(window_find_string(w, c"bogus".as_ptr()).is_null());
    }
}

#[test]
fn window_pane_search_finds_written_text() {
    let _guard = globals();
    let mut p = Pane::new(10, 80, 24, 100);
    unsafe {
        let wp = p.ptr();
        // write "hello world" on first line via grid
        let grid = screen_grid_ptr(&mut (*wp).base);
        assert!(!grid.is_null());
        let hello = b"hello world";
        for (i, &ch) in hello.iter().enumerate() {
            let mut gc = crate::grid::grid_default_cell;
            gc.data.data[0] = ch;
            gc.data.have = 1;
            gc.data.size = 1;
            gc.data.width = 1;
            crate::grid::grid_set_cell(&mut *grid, i as u32, 0, &gc);
        }
        // fnmatch search (regex=0) with ignore=0
        let n = window_pane_search(wp, c"hello".as_ptr(), 0, 0);
        assert_ne!(n, 0, "should find hello");
        let m = window_pane_search(wp, c"notpresent".as_ptr(), 0, 0);
        assert_eq!(m, 0);
        // regex mode, case-insensitive
        let r = window_pane_search(wp, c"hello.*".as_ptr(), 1, 0);
        assert_ne!(r, 0);
        let ri = window_pane_search(wp, c"HELLO".as_ptr(), 0, 1);
        assert_ne!(ri, 0, "ignore case should find HELLO");
    }
}

#[test]
fn window_pane_find_directional() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        let p0 = l.pane(0);
        let p1 = l.pane(j);
        assert_eq!(window_pane_find_right(p0), p1);
        assert_eq!(window_pane_find_left(p1), p0);
        let _ = window_pane_find_up(p0);
        let _ = window_pane_find_down(p0);
        let _ = window_pane_find_up(p1);
        let _ = window_pane_find_down(p1);
    }
    drop(l);
    let mut l2 = Layout::new(80, 24);
    let k = split(&mut l2, 0, LAYOUT_TOPBOTTOM);
    unsafe {
        let q0 = l2.pane(0);
        let q1 = l2.pane(k);
        assert_eq!(window_pane_find_down(q0), q1);
        assert_eq!(window_pane_find_up(q1), q0);
        let _ = window_pane_find_left(q0);
        let _ = window_pane_find_right(q1);
    }
}

#[test]
fn window_has_pane_reports_membership() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    let mut outsider = Pane::new(99, 80, 24, 100);
    unsafe {
        let w = l.w();
        assert_eq!(window_has_pane(w, l.pane(0)), 1);
        assert_eq!(window_has_pane(w, l.pane(j)), 1);
        assert_eq!(window_has_pane(w, outsider.ptr()), 0);
    }
}
