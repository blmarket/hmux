use super::*;
use crate::cmd::cmd_find_from_winlink;
use crate::layout::{LAYOUT_TOPBOTTOM, layout_free, layout_init, layout_split_pane};
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, unlink,
};
use crate::window::window_get_active;
use crate::window::window_pane_of_id;
use crate::window::{
    window_count_panes, window_panes_first, window_panes_last, window_panes_next, window_panes_prev,
};
use ::core::ffi::c_int;

/// Where the fixture windows' ids start, clear of anything the server's
/// own counters hand out.
const WINDOW_ID_BASE: u_int = 710_000;

/// Where the fixture panes' ids start.
const PANE_ID_BASE: u_int = 810_000;

/// A registered window whose layout tree is freed ahead of its panes, so
/// the leaves write their pane's `layout_cell` back to null as they go.
struct Win {
    window: Window,
}

impl Drop for Win {
    fn drop(&mut self) {
        unsafe { layout_free(self.window.ptr()) };
    }
}

/// One session holding linked windows of laid-out panes, in the server's
/// trees the way a prepared command queue item's find states expect to
/// walk them. Fields drop in declaration order, so the windows go before
/// the panes they hang off.
struct Fixture {
    registry: Registry,
    session: Session,
    windows: Vec<Win>,
    panes: Vec<Pane>,
    links: Vec<*mut winlink>,
}

impl Fixture {
    fn new() -> Fixture {
        let mut f = Fixture {
            registry: Registry::new(),
            session: Session::new(0, "0"),
            windows: Vec::new(),
            panes: Vec::new(),
            links: Vec::new(),
        };
        f.registry.add_session(&mut f.session);
        f
    }

    /// Links a fresh window of `panes` laid-out panes at index `idx`,
    /// answering its winlink and its panes in creation order. The first
    /// pane is the active one and each further pane is laid out by halving
    /// the one before it.
    fn add_window(
        &mut self,
        idx: c_int,
        panes: usize,
        sx: u_int,
        sy: u_int,
    ) -> (*mut winlink, Vec<*mut window_pane>) {
        let id = WINDOW_ID_BASE + self.windows.len() as u_int;
        let mut w = Win {
            window: Window::new(id, "fixture", sx, sy),
        };
        let mut made: Vec<*mut window_pane> = Vec::new();
        for i in 0..panes {
            let pid = PANE_ID_BASE + self.panes.len() as u_int + 1;
            let mut p = Pane::new(pid, sx, sy, 100);
            let ptr = p.ptr();
            w.window.add_pane(&mut p);
            self.panes.push(p);
            unsafe {
                if i == 0 {
                    layout_init(w.window.ptr(), ptr);
                } else {
                    let lc = layout_split_pane(made[i - 1], LAYOUT_TOPBOTTOM, -1, 0);
                    assert!(!lc.is_null(), "there was no room to lay out");
                    layout_assign_pane(lc, ptr, 0);
                }
            }
            made.push(ptr);
        }
        self.registry.add_window(&mut w.window);
        let wl = link(&mut self.session, &mut w.window, idx);
        self.links.push(wl);
        self.windows.push(w);
        (wl, made)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for wl in ::core::mem::take(&mut self.links).into_iter().rev() {
            unlink(&mut self.session, wl);
        }
    }
}

/// The find state of `wl`, as the command queue would have resolved it:
/// the window's active pane is what a bare target names.
unsafe fn fs_of(wl: *mut winlink) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
    fs
}

/// The `i`th pane of `w` in window order.
unsafe fn pane_at(w: *mut window, i: usize) -> *mut window_pane {
    unsafe {
        let mut p = window_panes_first(w);
        for _ in 0..i {
            p = window_panes_next(w, p);
        }
        p
    }
}

/// The `i`th pane of `w` in z order.
unsafe fn z_at(w: *mut window, i: usize) -> *mut window_pane {
    unsafe { window_pane_of_id(w, (*w).z_index[i]) }
}

#[test]
fn joining_in_front_of_a_later_pane_relinks_the_pane_behind_it() {
    let _guard = globals();
    ensure_reactor();
    let mut fx = Fixture::new();
    let (wl_src, src_panes) = fx.add_window(0, 2, 80, 24);
    let (wl_dst, dst_panes) = fx.add_window(1, 2, 80, 24);
    let moved = src_panes[0];
    let (front, back) = (dst_panes[0], dst_panes[1]);
    let (w_src, w_dst) = unsafe { ((*wl_src).window(), (*wl_dst).window()) };

    let mut item = Item::new().with_args(c"join-pane -d");
    unsafe {
        let p = item.ptr();
        (*p).target = fs_of(wl_dst);
        (*p).source = fs_of(wl_src);
        *cmdq_get_current(p) = (*p).target.clone();
        assert_eq!(
            (*p).target.pane(),
            front,
            "the destination window's active pane is the one in front"
        );
        assert!(
            !window_panes_next(w_dst, front).is_null(),
            "and it has a pane behind it"
        );

        assert_eq!(cmd_join_pane_exec(&*item.cmd(), p), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_dst, 1), 3);
        assert_eq!(pane_at(w_dst, 0), front, "the target pane stays in front");
        assert_eq!(pane_at(w_dst, 1), moved, "the joined pane follows it");
        assert_eq!(pane_at(w_dst, 2), back, "and the pane that was behind it");
        assert_eq!(z_at(w_dst, 0), front);
        assert_eq!(z_at(w_dst, 1), moved);
        assert_eq!(z_at(w_dst, 2), back);

        assert_eq!(
            window_panes_prev(w_dst, back),
            moved,
            "the pane behind was relinked onto the joined pane"
        );
        assert_eq!(
            (*w_dst).z_index,
            vec![(*front).id, (*moved).id, (*back).id],
            "and so was its z link"
        );
        assert_eq!(window_panes_prev(w_dst, moved), front);
        assert_eq!(
            window_panes_last(w_dst),
            back,
            "the list's tail is left alone, since nothing landed at the end"
        );

        assert_eq!((*moved).window, w_dst);
        assert_eq!(
            window_get_active(w_dst),
            front,
            "-d leaves the active pane alone"
        );
        assert_eq!(window_count_panes(w_src, 1), 1);
        assert_eq!(pane_at(w_src, 0), src_panes[1]);
    }
}
