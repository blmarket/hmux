//! Coverage for [`crate::window`] – additional helpers not covered by beta
//! (window/pane helpers, zoom bookkeeping, resize, floating, search etc).

use crate::WindowPane;
use crate::pane_identity::PaneIdentity;

use crate::layout::{LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM};
use crate::tests::test_fixtures::{Layout, Pane, Window, globals};
use crate::types::layout_type;
use crate::window::{
    PANE_EXITED, WINDOW_ZOOMED, window_find_string, window_has_floating_panes, window_pane_exited,
    window_pane_find_down, window_pane_find_left, window_pane_find_right, window_pane_find_up,
    window_pane_is_floating, window_pane_search, window_pane_visible, window_panes_position,
};

fn split(l: &mut Layout, i: usize, ty: layout_type) -> usize {
    unsafe {
        let pane_id = (*l.pane(i)).pane_id();
        let lc = (l.reference()).split_pane_layout(
            &crate::window::window_pane_find_by_id(pane_id).expect("the layout pane exists"),
            ty,
            -1,
            0,
        );
        assert!(lc.is_some(), "split failed");
        let j = l.add_pane(1, 1);
        (l.reference()).assign_pane_layout(
            lc.as_ref().unwrap(),
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        j
    }
}

#[test]
fn window_resize_updates_dimensions() {
    let _guard = globals();
    let fixture = Window::new(1, "r", 80, 24);
    let window = fixture.handle().clone();
    window.resize(100, 40, 13, 17);
    assert_eq!(fixture.handle().dimensions().size.width, 100);
    assert_eq!(fixture.handle().dimensions().size.height, 40);
    window.resize(120, 50, -1, -1);
    assert_eq!(window.dimensions().pixels.width, 13);
    assert_eq!(window.dimensions().pixels.height, 17);
    window.resize(120, 50, -1, 0);
    assert_eq!(window.dimensions().pixels.width, 13);
    assert_eq!(
        window.dimensions().pixels.height,
        crate::window::DEFAULT_YPIXEL as u32
    );
    window.resize(120, 50, 0, -1);
    assert_eq!(
        window.dimensions().pixels.width,
        crate::window::DEFAULT_XPIXEL as u32
    );
    assert_eq!(
        window.dimensions().pixels.height,
        crate::window::DEFAULT_YPIXEL as u32
    );
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
        assert_eq!(window_has_floating_panes(&mut *w), 0);
        assert_eq!(
            window_pane_is_floating(
                &*w,
                &{ crate::window::window_pane_ref_of(&(*p0)) }.expect("the pane allocation exists")
            ),
            0
        );
        assert_eq!(
            window_pane_is_floating(
                &*w,
                &{ crate::window::window_pane_ref_of(&(*p1)) }.expect("the pane allocation exists")
            ),
            0
        );
        // mark p1 floating via its layout cell
        crate::tests::test_fixtures::set_pane_floating(&mut *w, (*p1).pane_id(), true);
        assert_ne!(
            window_pane_is_floating(
                &*w,
                &{ crate::window::window_pane_ref_of(&(*p1)) }.expect("the pane allocation exists")
            ),
            0
        );
        assert_eq!(window_has_floating_panes(&mut *w), 1);
        // unmark
        crate::tests::test_fixtures::set_pane_floating(&mut *w, (*p1).pane_id(), false);
        assert_eq!(window_has_floating_panes(&mut *w), 0);
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
        assert_ne!(window_pane_visible(&*w, &*p0), 0);
        assert_ne!(window_pane_visible(&*w, &*p1), 0);
        // zoom p1: window gets WINDOW_ZOOMED, only active visible if we set active
        // Layout starts active = first pane; make p1 active then zoom
        // Use window_zoom on p1 (needs two panes)
        let id = (*p1).pane_id();
        let rc = (l.reference())
            .zoom(&crate::window::window_pane_find_by_id(id).expect("the selected pane exists"));
        assert_eq!(rc, 0);
        assert_ne!((*w).flags & WINDOW_ZOOMED, 0);
        // p1 is active after zoom (window_zoom sets active if needed)
        assert_ne!(window_pane_visible(&*w, &*p1), 0);
        // p0 should be hidden while zoomed
        assert_eq!(window_pane_visible(&*w, &*p0), 0);
        // unzoom restores
        let rc2 = (l.reference()).unzoom(0);
        assert_eq!(rc2, 0);
        assert_eq!((*w).flags & WINDOW_ZOOMED, 0);
        assert_ne!(window_pane_visible(&*w, &*p0), 0);
    }
}

#[test]
fn window_pane_exited_reports_fd_and_flag() {
    let _guard = globals();
    let mut p = Pane::new(1, 80, 24, 100);
    unsafe {
        let wp = p.ptr();
        // fd == -1 counts as exited
        assert_ne!(window_pane_exited(&*wp), 0);
        // give it a fake fd and clear exited flag -> not exited
        *(*wp).fd_mut() = 99;
        *(*wp).flags_mut() &= !PANE_EXITED;
        assert_eq!(window_pane_exited(&*wp), 0);
        // exited flag alone counts too
        *(*wp).flags_mut() |= PANE_EXITED;
        assert_ne!(window_pane_exited(&*wp), 0);
        *(*wp).fd_mut() = -1;
        *(*wp).flags_mut() &= !PANE_EXITED;
        // back to exited via fd
        assert_ne!(window_pane_exited(&*wp), 0);
        // avoid close on drop
        *(*wp).fd_mut() = -1;
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
        assert_eq!(
            window_find_string(&*w, c"top").map(|pane| pane.id()),
            Some((*p0).pane_id())
        );
        assert_eq!(
            window_find_string(&*w, c"bottom").map(|pane| pane.id()),
            Some((*p0).pane_id())
        );
        assert_eq!(
            window_find_string(&*w, c"left").map(|pane| pane.id()),
            Some((*p0).pane_id())
        );
        assert_eq!(
            window_find_string(&*w, c"right").map(|pane| pane.id()),
            Some((*p0).pane_id())
        );
        assert_eq!(
            window_find_string(&*w, c"top-left").map(|pane| pane.id()),
            Some((*p0).pane_id())
        );
        assert_eq!(
            window_find_string(&*w, c"bottom-right").map(|pane| pane.id()),
            Some((*p0).pane_id())
        );
        assert!(window_find_string(&*w, c"centre").is_none());
        assert!(window_find_string(&*w, c"bogus").is_none());
    }
}

#[test]
fn window_pane_search_finds_written_text() {
    let _guard = globals();
    let mut p = Pane::new(10, 80, 24, 100);
    unsafe {
        let wp = p.ptr();
        // write "hello world" on first line via grid
        let grid = RustScreen::grid_mut((*wp).base_mut());
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
        let n = window_pane_search(&mut *wp, c"hello", 0, 0);
        assert_ne!(n, 0, "should find hello");
        let m = window_pane_search(&mut *wp, c"notpresent", 0, 0);
        assert_eq!(m, 0);
        // regex mode, case-insensitive
        let r = window_pane_search(&mut *wp, c"hello.*", 1, 0);
        assert_ne!(r, 0);
        let ri = window_pane_search(&mut *wp, c"HELLO", 0, 1);
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
        assert_eq!(
            window_pane_find_right(p0.as_ref()).map(|pane| pane.id()),
            Some(crate::PaneIdentity::pane_id(&*p1))
        );
        assert_eq!(
            window_pane_find_left(p1.as_ref()).map(|pane| pane.id()),
            Some(crate::PaneIdentity::pane_id(&*p0))
        );
        let _ = window_pane_find_up(p0.as_ref());
        let _ = window_pane_find_down(p0.as_ref());
        let _ = window_pane_find_up(p1.as_ref());
        let _ = window_pane_find_down(p1.as_ref());
    }
    drop(l);
    let mut l2 = Layout::new(80, 24);
    let k = split(&mut l2, 0, LAYOUT_TOPBOTTOM);
    unsafe {
        let q0 = l2.pane(0);
        let q1 = l2.pane(k);
        assert_eq!(
            window_pane_find_down(q0.as_ref()).map(|pane| pane.id()),
            Some(crate::PaneIdentity::pane_id(&*q1))
        );
        assert_eq!(
            window_pane_find_up(q1.as_ref()).map(|pane| pane.id()),
            Some(crate::PaneIdentity::pane_id(&*q0))
        );
        let _ = window_pane_find_left(q0.as_ref());
        let _ = window_pane_find_right(q1.as_ref());
    }
}

#[test]
fn window_panes_position_reports_membership() {
    let _guard = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    let mut outsider = Pane::new(99, 80, 24, 100);
    unsafe {
        let w = l.w();
        assert!(window_panes_position(&*w, l.pane(0).as_ref()).is_some());
        assert!(window_panes_position(&*w, l.pane(j).as_ref()).is_some());
        assert!(window_panes_position(&*w, outsider.ptr().as_ref()).is_none());
    }
}
use crate::screen::RustScreen;
