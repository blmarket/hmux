use super::*;
use crate::cmd::cmd_find_from_winlink;
use crate::layout::layout_free_cell;
use crate::session::session_get_curw;
use crate::session::winlink_of;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, seen, unlink,
};
use crate::window::window_get_active;
use crate::window::{window_panes_first, window_panes_last, window_panes_next, winlink_count};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// Where the fixture windows' ids start, clear of anything
/// `window_create` hands out from the server's own counter.
const WINDOW_ID_BASE: u_int = 700_000;

/// Where the fixture panes' ids start.
const PANE_ID_BASE: u_int = 800_000;

/// Windows the exec hook built with `window_create` and left in the
/// server's tree, held until the test ends.
struct Created(Vec<WindowRef>);

impl Created {
    fn new() -> Created {
        Created(Vec::new())
    }

    fn keep(&mut self, w: *mut window) {
        let Some(reference) = crate::window::window_ref_from_ptr(w) else {
            panic!("created window has no owner");
        };
        self.0.push(reference);
    }
}

/// Clears the pane back-pointers out of the tree under `lc`, so freeing it
/// stays inside the layout and never reaches a pane its [`Pane`] owns.
unsafe fn forget_panes(lc: *mut layout_cell) {
    unsafe {
        if lc.is_null() {
            return;
        }
        for lcchild in crate::list::foreach_owned(&raw mut (*lc).cells) {
            forget_panes(lcchild);
        }
        (*lc).wp_id = None;
    }
}

impl Drop for Created {
    fn drop(&mut self) {
        unsafe {
            for w_ref in &self.0 {
                let w = w_ref.as_ptr();
                crate::window::windows.map().remove(&(*w).id);
                forget_panes((*w).layout_root_ptr());
                layout_free_cell(w, (*w).layout_root.take());
                layout_free_cell(w, (*w).saved_layout_root.take());
                w_ref.mark_unmanaged();
            }
        }
    }
}

/// Registered sessions holding linked windows of real panes, in the
/// server's trees the way a prepared command queue item's find states
/// expect to walk them. Every pane gets a shell string so
/// `default_window_name` reads something deterministic when the pane
/// becomes the active pane of a fresh window. Winlinks the fixture linked
/// are unlinked again on the way out, minus the ones the command freed
/// ([`World::forget`]) and plus the ones it made ([`World::owns`]).
struct World {
    registry: Registry,
    sessions: Vec<Session>,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    shells: Vec<CString>,
    tracked: Vec<(usize, *mut winlink)>,
}

impl World {
    fn new(name: &str) -> World {
        let mut w = World {
            registry: Registry::new(),
            sessions: Vec::new(),
            windows: Vec::new(),
            panes: Vec::new(),
            shells: Vec::new(),
            tracked: Vec::new(),
        };
        w.add_session(name);
        w
    }

    fn add_session(&mut self, name: &str) -> usize {
        self.sessions
            .push(Session::new(self.sessions.len() as u_int, name));
        self.registry
            .add_session(self.sessions.last_mut().expect("a session"));
        self.sessions.len() - 1
    }

    /// Links a fresh window of `panes` panes at index `idx` behind session
    /// `sidx`, answering its winlink and its panes in creation order; the
    /// first pane is the window's active one.
    fn add_window(
        &mut self,
        sidx: usize,
        idx: c_int,
        panes: usize,
    ) -> (*mut winlink, Vec<*mut window_pane>) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 17 + sidx as u_int;
        let mut w = Window::new(wid, "world", 80, 24);
        let mut made = Vec::new();
        for _ in 0..panes {
            let pid = PANE_ID_BASE + self.panes.len() as u_int + 1;
            let mut p = Pane::new(pid, 80, 24, 100);
            self.shells
                .push(CString::new("/bin/sh").expect("a shell path has no NUL"));
            unsafe { (*p.ptr()).shell = Some(self.shells.last().expect("a shell").clone()) };
            w.add_pane(&mut p);
            made.push(p.ptr());
            self.panes.push(p);
        }
        self.registry.add_window(&mut w);
        let wl = link(&mut self.sessions[sidx], &mut w, idx);
        self.tracked.push((sidx, wl));
        self.windows.push(w);
        (wl, made)
    }

    fn sptr(&mut self, i: usize) -> *mut session {
        self.sessions[i].ptr()
    }

    /// Drops a winlink from the cleanup list, for ones the command took
    /// out of the session itself.
    fn forget(&mut self, wl: *mut winlink) {
        self.tracked.retain(|&(_, p)| p != wl);
    }

    /// Adds a winlink the command made to the cleanup list.
    fn owns(&mut self, sidx: usize, wl: *mut winlink) {
        self.tracked.push((sidx, wl));
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for (si, wl) in ::std::mem::take(&mut self.tracked).into_iter().rev() {
            unlink(&mut self.sessions[si], wl);
        }
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way
/// the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_break_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states where the test
/// wants them, as a prepared item's resolved find states would be.
fn aim(item: &mut Item, target: cmd_find_state, source: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = target.clone();
        (*p).source = source;
        *cmdq_get_current(p) = target.clone();
    }
}

/// The find state of `wl` with `idx` filled in by hand, since resolution
/// is the command queue's job and this hook reads the states as given.
unsafe fn fs_of(wl: *mut winlink, idx: c_int) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
    fs.idx = idx;
    fs
}

/// A target state naming only a session and an index, which is what
/// `cmd_find_target` leaves behind for a `-t` window index that no window
/// holds — `CMD_FIND_WINDOW_INDEX` fills in `idx` and keeps `wl` null.
unsafe fn fs_index(s: *mut session, idx: c_int) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    fs.set_session(s);
    fs.idx = idx;
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

#[test]
fn an_index_only_target_shuffles_up_from_the_sessions_current_window() {
    let _guard = globals();
    ensure_reactor();
    let mut created = Created::new();
    let mut world = World::new("0");
    let (wl_cur, panes) = world.add_window(0, 0, 2);
    let (wl_next, _) = world.add_window(0, 1, 1);
    let w_cur = unsafe { (*wl_cur).window() };
    let moved = panes[0];

    let mut item = Item::new().with_args(c"break-pane -a");
    unsafe {
        let s = world.sptr(0);
        assert_eq!(
            session_get_curw(s),
            wl_cur,
            "the first linked window is current"
        );
        aim(&mut item, fs_index(s, 9), fs_of(wl_cur, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            (*wl_next).idx,
            2,
            "the window above the current one shuffled up"
        );
        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert!(!wl_new.is_null(), "the freed index took the new window");
        let nw = (*wl_new).window();
        created.keep(nw);
        world.owns(0, wl_new);
        assert_eq!((*moved).window, nw, "the pane moved into the new window");
        assert_ne!(nw, w_cur);
        assert_eq!(
            winlink_find_by_index(&raw mut (*s).windows, 0),
            wl_cur,
            "the current window kept its own index"
        );
        assert_eq!(winlink_count(&raw mut (*s).windows), 3);
        assert_eq!(
            session_get_curw(s),
            wl_new,
            "without -d the new window is selected"
        );
        assert_eq!(window_count_panes(w_cur, 1), 1);
    }
}

#[test]
fn a_target_at_the_last_index_refuses_to_shuffle_up() {
    let _guard = globals();
    ensure_reactor();
    let mut world = World::new("0");
    let (wl_src, panes) = world.add_window(0, 0, 2);
    let (wl_last, _) = world.add_window(0, c_int::MAX, 1);
    let w_src = unsafe { (*wl_src).window() };

    let mut item = Item::new().with_args(c"break-pane -b");
    unsafe {
        aim(&mut item, fs_of(wl_last, c_int::MAX), fs_of(wl_src, -1));

        assert_eq!(
            run(&mut item),
            CMD_RETURN_ERROR,
            "there is no index above the last one to shuffle into"
        );

        let s = world.sptr(0);
        assert_eq!(
            winlink_count(&raw mut (*s).windows),
            2,
            "nothing was linked"
        );
        assert_eq!(
            winlink_find_by_index(&raw mut (*s).windows, c_int::MAX),
            wl_last,
            "the target stayed at the last index"
        );
        assert_eq!(window_count_panes(w_src, 1), 2, "the source kept its panes");
        assert_eq!(pane_at(w_src, 0), panes[0]);
        assert_eq!(pane_at(w_src, 1), panes[1]);
        assert_eq!(session_get_curw(s), wl_src);
    }
}

#[test]
fn a_single_pane_window_relinked_without_n_keeps_the_name_it_had() {
    let _guard = globals();
    ensure_reactor();
    let mut world = World::new("src");
    let (wl_src, _) = world.add_window(0, 0, 1);
    world.add_window(0, 1, 1);
    let w_src = unsafe { (*wl_src).window() };
    let dst = world.add_session("dst");
    let (wl_dst, _) = world.add_window(dst, 0, 1);

    let mut item = Item::new().with_args(c"break-pane -d");
    unsafe {
        aim(&mut item, fs_of(wl_dst, 4), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        world.forget(wl_src);

        let s_dst = world.sptr(dst);
        let wl_new = winlink_find_by_index(&raw mut (*s_dst).windows, 4);
        assert!(!wl_new.is_null());
        world.owns(dst, wl_new);
        assert_eq!((*wl_new).window(), w_src);
        assert_eq!(
            session_get_curw(s_dst),
            wl_dst,
            "-d keeps the destination's current window"
        );
        assert_eq!(
            seen(cstr_ptr(&(*w_src).name)),
            "world",
            "the relinked window keeps the name it came with"
        );
        assert_eq!(
            options_get_number((*w_src).options_ptr(), c"automatic-rename".as_ptr()),
            1,
            "and its automatic renaming is left alone"
        );
    }
}

#[test]
fn p_takes_the_format_from_f_instead_of_the_default_template() {
    let _guard = globals();
    ensure_reactor();
    let mut created = Created::new();
    let mut world = World::new("0");
    let (wl0, panes) = world.add_window(0, 0, 2);
    let w0 = unsafe { (*wl0).window() };
    let moved = panes[0];

    let mut item = Item::new().with_args(c"break-pane -d -P -F '#{window_name}'");
    unsafe {
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(args_has(args, b'P'), 1);
        assert_eq!(seen(args_get(args, b'F')), "#{window_name}");
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w0, 1), 1);
        let nw = (*moved).window;
        created.keep(nw);
        let s = world.sptr(0);
        let wl_new = winlink_find_by_window(&raw mut (*s).windows, nw);
        assert!(!wl_new.is_null());
        world.owns(0, wl_new);
        assert_ne!(nw, w0);
        assert_eq!(seen(cstr_ptr(&(*nw).name)), "sh");
    }
}

#[test]
fn a_single_pane_window_is_relinked_into_the_destination_session_and_n_renames_it() {
    let _guard = globals();
    ensure_reactor();
    let mut world = World::new("src");
    let (wl_src, panes) = world.add_window(0, 0, 1);
    let (wl_keep, _) = world.add_window(0, 1, 1);
    let w_src = unsafe { (*wl_src).window() };
    let dst = world.add_session("dst");
    let (wl_dst, _) = world.add_window(dst, 0, 1);

    let mut item = Item::new().with_args(c"break-pane -n moved");
    unsafe {
        aim(&mut item, fs_of(wl_dst, 5), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        world.forget(wl_src);

        let s_src = world.sptr(0);
        let s_dst = world.sptr(dst);
        assert_eq!(
            winlink_count(&raw mut (*s_src).windows),
            1,
            "the source session gave the window up"
        );
        assert_eq!(
            winlink_find_by_index(&raw mut (*s_src).windows, 0),
            null_mut()
        );
        assert_eq!(
            session_get_curw(s_src),
            wl_keep,
            "the source moved on to what is left"
        );

        let wl_new = winlink_find_by_index(&raw mut (*s_dst).windows, 5);
        assert!(!wl_new.is_null(), "the window landed on the given index");
        world.owns(dst, wl_new);
        assert_eq!((*wl_new).window(), w_src, "it is the very same window");
        assert_eq!(
            winlink_find_by_window(&raw mut (*s_dst).windows, w_src),
            wl_new
        );
        assert_eq!(winlink_count(&raw mut (*s_dst).windows), 2);
        assert_eq!(
            session_get_curw(s_dst),
            wl_new,
            "without -d the destination selects it"
        );
        assert_eq!(winlink_of(s_dst, (*s_dst).lastw.first().copied()), wl_dst);

        assert_eq!(window_count_panes(w_src, 1), 1, "the window kept its pane");
        assert_eq!(pane_at(w_src, 0), panes[0]);
        assert_eq!((*panes[0]).window, w_src, "no new window was built");
        assert_eq!(seen(cstr_ptr(&(*w_src).name)), "moved");
        assert_eq!(
            options_get_number((*w_src).options_ptr(), c"automatic-rename".as_ptr()),
            0,
            "-n switches automatic renaming off"
        );
    }
}

#[test]
fn breaking_the_last_pane_hands_both_list_tails_to_the_one_in_front() {
    let _guard = globals();
    ensure_reactor();
    let mut created = Created::new();
    let mut world = World::new("0");
    let (wl0, panes) = world.add_window(0, 0, 3);
    let w0 = unsafe { (*wl0).window() };
    let (first, front, moved) = (panes[0], panes[1], panes[2]);

    let mut item = Item::new().with_args(c"break-pane -d");
    unsafe {
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));
        (*item.ptr()).source.set_pane(moved);

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w0, 1), 2);
        assert_eq!(pane_at(w0, 0), first);
        assert_eq!(pane_at(w0, 1), front);
        assert_eq!(
            window_panes_last(w0),
            front,
            "the pane in front took over the pane list's tail"
        );
        assert_eq!(
            (*w0).z_index,
            vec![(*first).id, (*front).id],
            "and the z-order list's tail"
        );
        assert!(window_panes_next(w0, front).is_null());

        let nw = (*moved).window;
        created.keep(nw);
        assert_ne!(nw, w0);
        assert_eq!(window_panes_first(nw), moved);
        assert_eq!(window_panes_last(nw), moved);
        assert_eq!((*nw).z_index, vec![(*moved).id]);
        assert_eq!(window_get_active(nw), moved);
        assert_eq!(window_count_panes(nw, 1), 1);

        let s = world.sptr(0);
        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert!(!wl_new.is_null());
        world.owns(0, wl_new);
        assert_eq!((*wl_new).window(), nw);
        assert_eq!(
            session_get_curw(s),
            wl0,
            "-d keeps the current window selected"
        );
    }
}
