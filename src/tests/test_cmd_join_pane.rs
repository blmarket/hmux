use super::*;
use crate::WindowPane;
use crate::cmd::cmd_find_from_winlink;
use crate::layout::LAYOUT_TOPBOTTOM;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, unlink_all,
};
use crate::window::{WinlinkRef, window_count_panes, window_panes_position};
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
        (self.window.reference()).free_layout();
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
}

impl Fixture {
    fn new() -> Fixture {
        let mut f = Fixture {
            registry: Registry::new(),
            session: Session::new(0, "0"),
            windows: Vec::new(),
            panes: Vec::new(),
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
    ) -> (WinlinkRef, Vec<RustWindowPaneWeak>) {
        let id = WINDOW_ID_BASE + self.windows.len() as u_int;
        let mut w = Win {
            window: Window::new(id, "fixture", sx, sy),
        };
        let mut made: Vec<RustWindowPaneWeak> = Vec::new();
        for i in 0..panes {
            let pid = PANE_ID_BASE + self.panes.len() as u_int + 1;
            let mut p = Pane::new(pid, sx, sy, 100);
            w.window.add_pane(&mut p);
            self.panes.push(p);
            let window = w.window.reference();
            let pane = {
                window
                    .as_window()
                    .panes
                    .last()
                    .map(|pane| pane.downgrade())
                    .unwrap()
            };
            unsafe {
                if i == 0 {
                    window.init_layout(
                        &crate::window::window_pane_find_by_id(pane.pane_id())
                            .expect("the layout pane exists"),
                    );
                } else {
                    let pane_id = made[i - 1].pane_id();
                    let lc = window.split_pane_layout(
                        &crate::window::window_pane_find_by_id(pane_id)
                            .expect("the layout pane exists"),
                        LAYOUT_TOPBOTTOM,
                        -1,
                        0,
                    );
                    assert!(lc.is_some(), "there was no room to lay out");
                    window.assign_pane_layout(
                        lc.as_ref().unwrap(),
                        &crate::window::window_pane_find_by_id(pane.pane_id())
                            .expect("the layout pane exists"),
                        0,
                    );
                }
            }
            made.push(pane);
        }
        self.registry.add_window(&mut w.window);
        link(&mut self.session, &mut w.window, idx);
        let wl = WinlinkRef::new(self.session.reference(), idx).unwrap();
        self.windows.push(w);
        (wl, made)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        unlink_all(&mut self.session);
    }
}

/// The find state of `wl`, as the command queue would have resolved it:
/// the window's active pane is what a bare target names.
unsafe fn fs_of(wl: &WinlinkRef) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl.get().unwrap(), 0) };
    fs
}

/// The `i`th pane of `w` in z order.
unsafe fn z_at(w: &WindowRef, i: usize) -> RustWindowPaneWeak {
    w.as_window().z_index[i].clone()
}

#[test]
fn joining_in_front_of_a_later_pane_relinks_the_pane_behind_it() {
    let _guard = globals();
    ensure_reactor();
    let mut fx = Fixture::new();
    let (wl_src, src_panes) = fx.add_window(0, 2, 80, 24);
    let (wl_dst, dst_panes) = fx.add_window(1, 2, 80, 24);
    let moved = src_panes[0].clone();
    let (front, back) = (dst_panes[0].clone(), dst_panes[1].clone());
    let (w_src, w_dst) = (
        wl_src.get().unwrap().window_handle().unwrap().clone(),
        wl_dst.get().unwrap().window_handle().unwrap().clone(),
    );

    let item = Item::new().with_args(c"join-pane -d");
    unsafe {
        let mut state = item.item_mut();
        state.target = fs_of(&wl_dst);
        state.source = fs_of(&wl_src);
        *state.current() = state.target.clone();
        assert!(
            state.target.pane_list_ref().unwrap().ptr_eq(&front),
            "the destination window's active pane is the one in front"
        );
        assert!(
            w_dst.as_window().panes.get(1).is_some(),
            "and it has a pane behind it"
        );

        drop(state);
        assert_eq!(
            item.with_command(|command, item| cmd_join_pane_exec(command, item)),
            CMD_RETURN_NORMAL
        );

        assert_eq!(window_count_panes(&w_dst.as_window(), 1), 3);
        assert!(
            w_dst
                .as_window()
                .panes
                .get(0)
                .is_some_and(|owner| owner.downgrade().ptr_eq(&front)),
            "the target pane stays in front"
        );
        assert!(
            w_dst
                .as_window()
                .panes
                .get(1)
                .is_some_and(|owner| owner.downgrade().ptr_eq(&moved)),
            "the joined pane follows it"
        );
        assert!(
            w_dst
                .as_window()
                .panes
                .get(2)
                .is_some_and(|owner| owner.downgrade().ptr_eq(&back)),
            "and the pane that was behind it"
        );
        assert!(z_at(&w_dst, 0).ptr_eq(&front));
        assert!(z_at(&w_dst, 1).ptr_eq(&moved));
        assert!(z_at(&w_dst, 2).ptr_eq(&back));

        assert_eq!(
            window_panes_position(&w_dst.as_window(), back.get()),
            Some(2),
            "the pane behind was relinked onto the joined pane"
        );
        assert_eq!(
            w_dst
                .as_window()
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            vec![front.pane_id(), moved.pane_id(), back.pane_id()],
            "and so was its z link"
        );
        assert_eq!(
            window_panes_position(&w_dst.as_window(), moved.get()),
            Some(1)
        );
        assert!(
            w_dst
                .as_window()
                .panes
                .last()
                .is_some_and(|owner| owner.downgrade().ptr_eq(&back)),
            "the list's tail is left alone, since nothing landed at the end"
        );

        assert!(moved.as_pane().window_context().unwrap().ptr_eq(&w_dst));
        assert!(
            w_dst.active_pane().is_some_and(|pane| pane.ptr_eq(&front)),
            "-d leaves the active pane alone"
        );
        assert_eq!(window_count_panes(&w_src.as_window(), 1), 1);
        assert!(
            w_src
                .as_window()
                .panes
                .get(0)
                .is_some_and(|owner| { owner.downgrade().ptr_eq(&src_panes[1]) })
        );
    }
}
