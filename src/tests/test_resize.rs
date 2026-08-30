use super::*;
use crate::layout::{LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, layout_root_ptr};
use crate::layout::{layout_assign_pane, layout_free, layout_init, layout_split_pane};
use crate::options::{options_set_number, options_set_string};
use crate::server::server_client_add_client_window;
use crate::session::{session_attached, session_set_curw};
use crate::tests::test_fixtures::{
    Clients, Pane, Registry, Session, Window, globals, link, unlink,
};
use crate::window::window_set_active;
use crate::window::window_set_latest;
use crate::window::{PANE_ZOOMED, window_zoom};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;

/// A window carrying a layout tree and the panes that hang off it, the way
/// a freshly created window is left. The window and its panes are the
/// server-free fixtures; the tree is real, and is freed before the panes
/// go.
struct Win {
    window: Window,
    panes: Vec<Pane>,
    next_id: u_int,
}

impl Win {
    fn new(sx: u_int, sy: u_int) -> Win {
        let mut w = Win {
            window: Window::new(1, "resize", sx, sy),
            panes: Vec::new(),
            next_id: 0,
        };
        w.add_pane(sx, sy);
        unsafe { layout_init(w.ptr(), w.pane(0)) };
        w
    }

    fn ptr(&mut self) -> *mut window {
        self.window.ptr()
    }

    fn pane(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
    }

    fn add_pane(&mut self, sx: u_int, sy: u_int) -> usize {
        self.next_id += 1;
        let mut pane = Pane::new(self.next_id, sx, sy, 100);
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    /// Splits pane `i` and gives the new cell a pane of its own.
    fn split(&mut self, i: usize, type_0: layout_type) -> usize {
        unsafe {
            let lc = layout_split_pane(self.pane(i), type_0, -1, 0);
            assert!(!lc.is_null(), "there was no room to split");
            let j = self.add_pane(1, 1);
            layout_assign_pane(lc, self.pane(j), 0);
            j
        }
    }

    /// The window's size and the size its layout tree settled on.
    fn size(&mut self) -> (u_int, u_int, u_int, u_int) {
        unsafe {
            let w = self.ptr();
            (
                (*w).sx,
                (*w).sy,
                (*layout_root_ptr(&(*w).layout_root)).sx,
                (*layout_root_ptr(&(*w).layout_root)).sy,
            )
        }
    }
}

impl Drop for Win {
    fn drop(&mut self) {
        unsafe { layout_free(self.window.ptr()) };
    }
}

/// What `ignore_client_size` answers, as the C's zero or one.
unsafe fn ignores(c: *mut client) -> c_int {
    unsafe { ignore_client_size(c) as c_int }
}

/// What `clients_calculate_size` works out for `w` under `type_0`, and
/// whether it found a size at all.
unsafe fn calculate(
    type_0: c_int,
    current: c_int,
    c: *mut client,
    s: *mut session,
    w: *mut window,
) -> (c_int, u_int, u_int, u_int, u_int) {
    unsafe {
        let mut sx = 0;
        let mut sy = 0;
        let mut xpixel = 0;
        let mut ypixel = 0;
        let size =
            clients_calculate_size(type_0, current, c, s, w, default_window_size_skip_client);
        sx = size.sx;
        sy = size.sy;
        xpixel = size.xpixel;
        ypixel = size.ypixel;
        (size.found as c_int, sx, sy, xpixel, ypixel)
    }
}

/// The size `default_window_size` settles on.
unsafe fn default_size(
    c: *mut client,
    s: *mut session,
    w: *mut window,
    type_0: c_int,
) -> (u_int, u_int, u_int, u_int) {
    unsafe { default_window_size(c, s, w, type_0) }
}

#[test]
fn resizing_a_window_moves_it_and_its_layout() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    unsafe { (*w.ptr()).flags |= WINDOW_RESIZE };
    unsafe { resize_window(w.ptr(), 40, 10, -1, -1) };
    assert_eq!(w.size(), (40, 10, 40, 10));
    unsafe {
        assert_eq!((*w.pane(0)).sx, 40);
        assert_eq!((*w.pane(0)).sy, 10);
        assert_eq!((*w.ptr()).flags & WINDOW_RESIZE, 0);
    }
}

#[test]
fn a_window_size_is_clamped_to_the_window_minimum_and_maximum() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    unsafe { resize_window(w.ptr(), 0, 0, -1, -1) };
    assert_eq!(w.size(), (1, 1, 1, 1));
    unsafe { resize_window(w.ptr(), 20000, 20000, -1, -1) };
    assert_eq!(w.size(), (10000, 10000, 10000, 10000));
}

#[test]
fn a_window_is_never_smaller_than_its_layout_needs() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    w.split(0, LAYOUT_LEFTRIGHT);
    unsafe { resize_window(w.ptr(), 1, 1, -1, -1) };
    assert_eq!(w.size(), (3, 1, 3, 1));

    let mut tall = Win::new(80, 24);
    tall.split(0, LAYOUT_TOPBOTTOM);
    unsafe { resize_window(tall.ptr(), 1, 1, -1, -1) };
    assert_eq!(tall.size(), (1, 3, 1, 3));
}

#[test]
fn a_zoomed_window_is_unzoomed_and_zoomed_again_around_the_resize() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    w.split(0, LAYOUT_LEFTRIGHT);
    assert_eq!(unsafe { window_zoom(w.pane(0)) }, 0);
    unsafe {
        assert_ne!((*w.ptr()).flags & WINDOW_ZOOMED, 0);
        resize_window(w.ptr(), 40, 10, -1, -1);
        assert_ne!((*w.ptr()).flags & WINDOW_ZOOMED, 0);
        assert_ne!((*w.pane(0)).flags & PANE_ZOOMED, 0);
        assert_eq!((*w.pane(0)).sx, 40);
    }
    assert_eq!(w.size().0, 40);
}

#[test]
fn a_client_with_no_session_or_on_its_way_out_is_ignored() {
    let _guard = globals();
    let mut s = Session::new(1, "ignore");
    let mut list = Clients::new();
    let c = list.add("c", 80, 24);
    assert_eq!(unsafe { ignores(c) }, 1);
    unsafe {
        (*c).session = s.ptr();
        assert_eq!(ignores(c), 0);
        for flag in [CLIENT_DEAD, CLIENT_SUSPENDED, CLIENT_EXIT] {
            (*c).flags = flag as uint64_t;
            assert_eq!(ignores(c), 1, "{flag}");
        }
    }
}

#[test]
fn a_client_ignoring_size_gives_way_to_one_that_does_not() {
    let _guard = globals();
    let mut s = Session::new(1, "ignore");
    let mut list = Clients::new();
    let first = list.add("first", 80, 24);
    unsafe {
        (*first).session = s.ptr();
        (*first).flags = CLIENT_IGNORESIZE as uint64_t;
        assert_eq!(ignores(first), 0);
        let second = list.add("second", 80, 24);
        (*second).session = s.ptr();
        assert_eq!(ignores(first), 1);
        (*second).flags = CLIENT_DEAD as uint64_t;
        assert_eq!(ignores(first), 0);
        (*second).flags = CLIENT_IGNORESIZE as uint64_t;
        assert_eq!(ignores(first), 0);
        (*second).session = null_mut::<session>();
        assert_eq!(ignores(first), 0);
    }
}

#[test]
fn a_control_client_is_ignored_until_it_reports_a_size() {
    let _guard = globals();
    let mut s = Session::new(1, "control");
    let mut list = Clients::new();
    let c = list.add("c", 80, 24);
    unsafe {
        (*c).session = s.ptr();
        (*c).flags = CLIENT_CONTROL as uint64_t;
        assert_eq!(ignores(c), 1);
        (*c).flags = (CLIENT_CONTROL | CLIENT_SIZECHANGED) as uint64_t;
        assert_eq!(ignores(c), 0);
        (*c).flags = CLIENT_CONTROL as uint64_t | CLIENT_WINDOWSIZECHANGED as uint64_t;
        assert_eq!(ignores(c), 0);
    }
}

#[test]
fn the_clients_showing_a_window_are_counted_up_to_two() {
    let _guard = globals();
    let mut s = Session::new(1, "count");
    let mut w = Win::new(80, 24);
    let mut list = Clients::new();
    assert_eq!(unsafe { clients_with_window(w.ptr()) }, 0);
    let first = list.add("first", 80, 24);
    unsafe { (*first).session = s.ptr() };
    assert_eq!(unsafe { clients_with_window(w.ptr()) }, 0);
    let wl = link(&mut s, &mut w.window, 0);
    assert_eq!(unsafe { clients_with_window(w.ptr()) }, 1);
    let second = list.add("second", 80, 24);
    unsafe { (*second).session = s.ptr() };
    let third = list.add("third", 80, 24);
    unsafe { (*third).session = s.ptr() };
    assert_eq!(unsafe { clients_with_window(w.ptr()) }, 2);
    unlink(&mut s, wl);
}

#[test]
fn the_largest_size_is_the_biggest_terminal_of_any_client() {
    let _guard = globals();
    let mut s = Session::new(1, "largest");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    assert_eq!(
        unsafe { calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()) },
        (0, 0, 0, 0, 0)
    );
    let first = list.add("first", 80, 24);
    let second = list.add("second", 100, 20);
    unsafe {
        (*first).session = s.ptr();
        (*second).session = s.ptr();
        (*second).tty.xpixel = 8;
        (*second).tty.ypixel = 16;
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 100, 24, 8, 16)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn the_smallest_size_is_the_smallest_terminal_of_any_client() {
    let _guard = globals();
    let mut s = Session::new(1, "smallest");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    assert_eq!(
        unsafe { calculate(0x7f, 0, null_mut(), s.ptr(), w.ptr()) },
        (0, UINT_MAX, UINT_MAX, 0, 0)
    );
    let first = list.add("first", 80, 24);
    let second = list.add("second", 100, 20);
    unsafe {
        (*first).session = s.ptr();
        (*second).session = s.ptr();
        assert_eq!(
            calculate(0x7f, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 80, 20, 0, 0)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn a_client_not_showing_the_window_is_skipped() {
    let _guard = globals();
    let mut s = Session::new(1, "skip");
    let mut other = Session::new(2, "other");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let inside = list.add("inside", 80, 24);
    let outside = list.add("outside", 20, 10);
    unsafe {
        (*inside).session = s.ptr();
        (*outside).session = other.ptr();
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 80, 24, 0, 0)
        );
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), other.ptr(), null_mut()),
            (1, 20, 10, 0, 0)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn a_manual_size_is_the_windows_own_and_needs_no_client() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    unsafe {
        (*w.ptr()).manual_sx = 33;
        (*w.ptr()).manual_sy = 11;
        assert_eq!(
            calculate(WINDOW_SIZE_MANUAL, 0, null_mut(), null_mut(), w.ptr()),
            (1, 33, 11, 0, 0)
        );
        assert_eq!(
            calculate(WINDOW_SIZE_MANUAL, 0, null_mut(), null_mut(), null_mut()),
            (0, UINT_MAX, UINT_MAX, 0, 0)
        );
    }
}

#[test]
fn the_latest_size_follows_the_window_s_latest_client() {
    let _guard = globals();
    let mut s = Session::new(1, "latest");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let first = list.add("first", 80, 24);
    let second = list.add("second", 100, 20);
    unsafe {
        (*first).session = s.ptr();
        (*second).session = s.ptr();
        window_set_latest(w.ptr(), first);
        assert_eq!(
            calculate(WINDOW_SIZE_LATEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 80, 24, 0, 0)
        );
        window_set_latest(w.ptr(), second);
        assert_eq!(
            calculate(WINDOW_SIZE_LATEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 100, 20, 0, 0)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn a_client_that_reported_a_size_for_the_window_is_read_from_that() {
    let _guard = globals();
    let mut s = Session::new(1, "reported");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let c = list.add("c", 80, 24);
    unsafe {
        (*c).session = s.ptr();
        let cw = server_client_add_client_window(c, (*w.ptr()).id);
        (*cw).sx = 40;
        (*cw).sy = 12;
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 40, 12, 0, 0)
        );
        (*server_client_add_client_window(c, (*w.ptr()).id)).sx = 0;
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 80, 24, 0, 0)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn a_client_that_changed_a_window_size_holds_the_answer_down() {
    let _guard = globals();
    let mut s = Session::new(1, "held");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let c = list.add("c", 80, 24);
    unsafe {
        (*c).session = s.ptr();
        (*c).flags = CLIENT_WINDOWSIZECHANGED as uint64_t;
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 80, 24, 0, 0)
        );
        let cw = server_client_add_client_window(c, (*w.ptr()).id);
        (*cw).sx = 40;
        (*cw).sy = 0;
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 40, 24, 0, 0)
        );
        let cw = server_client_add_client_window(c, (*w.ptr()).id);
        (*cw).sx = 0;
        (*cw).sy = 12;
        assert_eq!(
            calculate(WINDOW_SIZE_LARGEST, 0, null_mut(), s.ptr(), w.ptr()),
            (1, 80, 12, 0, 0)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn the_default_window_size_falls_back_on_the_session_option() {
    let _guard = globals();
    let mut s = Session::new(1, "default");
    let list = Clients::new();
    assert_eq!(
        unsafe { default_size(null_mut(), s.ptr(), null_mut(), WINDOW_SIZE_LARGEST) },
        (80, 24, 0, 0)
    );
    unsafe {
        options_set_string(
            s.options(),
            c"default-size".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"120x40".as_ptr()],
        );
        assert_eq!(
            default_size(null_mut(), s.ptr(), null_mut(), WINDOW_SIZE_LARGEST),
            (120, 40, 0, 0)
        );
        options_set_string(
            s.options(),
            c"default-size".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"nonsense".as_ptr()],
        );
        assert_eq!(
            default_size(null_mut(), s.ptr(), null_mut(), WINDOW_SIZE_LARGEST),
            (80, 24, 0, 0)
        );
    }
    drop(list);
}

#[test]
fn the_default_window_size_reads_the_window_size_option_when_asked() {
    let _guard = globals();
    let mut s = Session::new(1, "option");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let c = list.add("c", 90, 30);
    unsafe {
        (*c).session = s.ptr();
        window_set_latest(w.ptr(), c);
        assert_eq!(
            default_size(null_mut(), s.ptr(), w.ptr(), -1),
            (90, 30, 0, 0)
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn the_latest_size_of_a_client_that_asked_is_taken_straight_from_it() {
    let _guard = globals();
    let mut s = Session::new(1, "asked");
    let mut list = Clients::new();
    let c = list.add("c", 90, 30);
    unsafe {
        (*c).session = s.ptr();
        (*c).tty.xpixel = 9;
        (*c).tty.ypixel = 18;
        assert_eq!(
            default_size(c, s.ptr(), null_mut(), WINDOW_SIZE_LATEST),
            (90, 30, 9, 18)
        );
        (*c).flags = CLIENT_CONTROL as uint64_t;
        assert_eq!(
            default_size(c, s.ptr(), null_mut(), WINDOW_SIZE_LATEST),
            (80, 24, 0, 0)
        );
    }
}

#[test]
fn a_default_window_size_is_clamped_the_way_a_resize_is() {
    let _guard = globals();
    let mut s = Session::new(1, "clamped");
    let list = Clients::new();
    unsafe {
        options_set_string(
            s.options(),
            c"default-size".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"0x0".as_ptr()],
        );
        assert_eq!(
            default_size(null_mut(), s.ptr(), null_mut(), WINDOW_SIZE_LARGEST),
            (1, 1, 0, 0)
        );
        options_set_string(
            s.options(),
            c"default-size".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"20000x20000".as_ptr()],
        );
        assert_eq!(
            default_size(null_mut(), s.ptr(), null_mut(), WINDOW_SIZE_LARGEST),
            (10000, 10000, 0, 0)
        );
    }
    drop(list);
}

#[test]
fn a_window_with_no_active_pane_is_not_resized() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    unsafe {
        window_set_active(w.ptr(), null_mut::<window_pane>());
        recalculate_size(w.ptr(), 1);
    }
    assert_eq!(w.size(), (80, 24, 80, 24));
    unsafe { window_set_active(w.ptr(), w.pane(0)) };
}

#[test]
fn a_window_no_client_can_size_is_left_where_it_is() {
    let _guard = globals();
    let mut w = Win::new(80, 24);
    let list = Clients::new();
    unsafe { recalculate_size(w.ptr(), 0) };
    assert_eq!(w.size(), (80, 24, 80, 24));
    drop(list);
}

#[test]
fn a_recalculated_size_is_held_until_the_next_redraw_unless_it_is_wanted_now() {
    let _guard = globals();
    let mut s = Session::new(1, "recalculate");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let c = list.add("c", 40, 12);
    unsafe {
        (*c).session = s.ptr();
        recalculate_size(w.ptr(), 0);
        assert_eq!(w.size(), (80, 24, 80, 24));
        assert_ne!((*w.ptr()).flags & WINDOW_RESIZE, 0);
        assert_eq!(((*w.ptr()).new_sx, (*w.ptr()).new_sy), (40, 12));
        recalculate_size(w.ptr(), 0);
        assert_eq!(w.size(), (80, 24, 80, 24));
        recalculate_size(w.ptr(), 1);
        assert_eq!(w.size(), (40, 12, 40, 12));
        recalculate_size(w.ptr(), 0);
        assert_eq!(w.size(), (40, 12, 40, 12));
    }
    unlink(&mut s, wl);
}

#[test]
fn a_manual_window_is_resized_at_once() {
    let _guard = globals();
    let mut s = Session::new(1, "manual");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let list = Clients::new();
    unsafe {
        options_set_number(
            w.window.options(),
            c"window-size".as_ptr(),
            WINDOW_SIZE_MANUAL as i64,
        );
        (*w.ptr()).manual_sx = 50;
        (*w.ptr()).manual_sy = 15;
        recalculate_size(w.ptr(), 0);
        assert_eq!(w.size(), (50, 15, 50, 15));
    }
    drop(list);
    unlink(&mut s, wl);
}

#[test]
fn an_aggressive_resize_only_counts_the_clients_showing_the_window() {
    let _guard = globals();
    let mut s = Session::new(1, "aggressive");
    let mut shown = Win::new(80, 24);
    let mut hidden = Win::new(80, 24);
    unsafe { (*hidden.ptr()).id = 2 };
    let first = link(&mut s, &mut shown.window, 0);
    let second = link(&mut s, &mut hidden.window, 1);
    let mut list = Clients::new();
    let c = list.add("c", 40, 12);
    unsafe {
        (*c).session = s.ptr();
        options_set_number(hidden.window.options(), c"aggressive-resize".as_ptr(), 1);
        recalculate_size(hidden.ptr(), 1);
        assert_eq!(hidden.size(), (80, 24, 80, 24));
        recalculate_size(shown.ptr(), 1);
        assert_eq!(shown.size(), (40, 12, 40, 12));
    }
    unlink(&mut s, second);
    unlink(&mut s, first);
}

#[test]
fn a_client_whose_session_shows_no_window_at_all_is_skipped() {
    let _guard = globals();
    let mut s = Session::new(1, "nowhere");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    let mut list = Clients::new();
    let c = list.add("c", 40, 12);
    unsafe {
        (*c).session = s.ptr();
        session_set_curw(s.ptr(), null_mut::<winlink>());
        recalculate_size(w.ptr(), 1);
        assert_eq!(w.size(), (80, 24, 80, 24));
        session_set_curw(s.ptr(), wl);
        recalculate_size(w.ptr(), 1);
        assert_eq!(w.size(), (40, 12, 40, 12));
    }
    unlink(&mut s, wl);
}

#[test]
fn recalculating_every_size_counts_the_attached_clients_and_the_status_lines() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(1, "every");
    let mut w = Win::new(80, 24);
    let wl = link(&mut s, &mut w.window, 0);
    registry.add_session(&mut s);
    registry.add_window(&mut w.window);
    let mut list = Clients::new();
    let attached = list.add("attached", 40, 12);
    let bare = list.add("bare", 40, 12);
    unsafe {
        (*attached).session = s.ptr();
        options_set_number(s.options(), c"status".as_ptr(), 1);
        recalculate_sizes_now(1);
        assert_eq!(session_attached(s.ptr()), 1);
        assert_eq!((*s.ptr()).statuslines, 1);
        assert_eq!((*attached).flags & CLIENT_STATUSOFF as uint64_t, 0);
        assert_eq!((*bare).flags & CLIENT_STATUSOFF as uint64_t, 0);
        assert_eq!(w.size(), (40, 11, 40, 11));
        (*attached).tty.sy = 1;
        recalculate_sizes();
        assert_ne!((*attached).flags & CLIENT_STATUSOFF as uint64_t, 0);
    }
    unlink(&mut s, wl);
}
