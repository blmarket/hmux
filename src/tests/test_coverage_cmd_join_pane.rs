//! Unit tests for [`crate::cmd::cmd_join_pane`], the single exec hook behind
//! both the `join-pane` and `move-pane` commands.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entries' `.exec` pointers with an item whose target and source find states
//! have already been resolved. Around that hook the tests pin both entries'
//! metadata (including that they share one function), the refusals that fire
//! before anything moves — a source equal to the target, an unparsable `-l`
//! size, a floating destination pane, a destination too small to split — and
//! the moving half itself: insertion behind the target pane with its
//! top-bottom split, `-h`'s sideways split, `-d`'s hold on the destination's
//! active pane and current state, the reparenting of the moved pane's options
//! onto the destination window's, the style/theme flags, the mark following
//! or not following the lost pane, the emptied source window being unlinked
//! from its session by [`server_kill_window`], a cross-session move whose
//! select and current-state updates land in the destination session, and a
//! source window carrying no layout tree at all.
//!
//! Two findings worth recording. With `-b`, [`layout_get_tiled_cell`] takes
//! the before flag into account when it shapes the new cell, but the hook's
//! own copy of `flags` never hears about it, so the pane still lands
//! **behind** the target in the window list — the test pins both halves of
//! that behaviour. And when a session loses its **last** window,
//! [`server_kill_window`] hands over to `server_destroy_session_group` and
//! `session_destroy`, which would free memory these fixtures own; no test
//! drives that path, so the kill covered here always leaves its session
//! another window.

use crate::arguments::args_has;
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_join_pane::{
    CMD_FIND_DEFAULT_MARKED, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, PANE_STYLECHANGED,
    PANE_THEMECHANGED, SPAWN_BEFORE, cmd_join_pane_entry, cmd_move_pane_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::layout::layout_cell_pane;
use crate::layout::{
    LAYOUT_CELL_FLOATING, LAYOUT_TOPBOTTOM, layout_assign_pane, layout_free, layout_init,
    layout_split_pane,
};
use crate::options::options_get_parent;
use crate::proc::PEER_BAD;
use crate::server::{marked_pane, message_log, server_clear_marked, server_set_marked};
use crate::session::session_get_curw;
use crate::session::session_select;
use crate::session::winlink_of;
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Window, dump_cell, ensure_reactor, globals, link, seen,
    unlink, zeroed,
};
use crate::types::*;
use crate::window::PaneStack;
use crate::window::window_get_active;
use crate::window::window_pane_of_id;
use crate::window::{
    window_count_panes, window_pane_stack_first, window_panes_first, window_panes_next,
    winlink_count, winlink_find_by_index,
};
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 700_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 800_000;

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session stays null and its flags stay clear, which
/// is what sends `cmdq_error` down the branch that files the message in the
/// server's message log.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

/// Runs the item's parsed command through the join-pane entry's exec hook, the
/// way the command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_join_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Runs the item's parsed command through the move-pane entry's exec hook,
/// which is the same function under a second name.
unsafe fn run_move(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_move_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states where the test wants
/// them, as the resolved find states of a prepared command queue item would be.
unsafe fn aim(item: &mut Item, target: cmd_find_state, source: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = target.clone();
        (*p).source = source;
        *crate::cmd::cmdq_get_current(p) = target;
    }
}

/// Points the states as [`aim`] does and makes `caller` the item's own client
/// and target client, so `cmdq_error` files its message against that client.
unsafe fn aim_from(
    item: &mut Item,
    caller: *mut client,
    target: cmd_find_state,
    source: cmd_find_state,
) {
    unsafe {
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        aim(item, target, source);
    }
}

/// The find state of `wl` with `idx` filled in by hand, since resolution is
/// the command queue's job and this hook reads the states as given.
unsafe fn fs_of(wl: *mut winlink, idx: c_int) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
    fs.idx = idx;
    fs
}

/// The lines the server has recorded so far, oldest first.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
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

/// A registered window. Its layout tree is freed ahead of its panes, which is
/// what [`Chain`]'s field order buys: the leaves write their pane's
/// `layout_cell` back to null as they go.
struct Win {
    window: Window,
}

impl Drop for Win {
    fn drop(&mut self) {
        unsafe { layout_free(self.window.ptr()) };
    }
}

/// A registered session holding linked windows of laid-out panes, everything
/// in the server's trees the way the target-taking commands expect to walk
/// them. Fields are dropped in declaration order, so the windows go before
/// the panes they hang off. Winlinks the chain itself made are unlinked again
/// on the way out; winlinks the command frees itself are told apart with
/// [`Chain::forget`], because unlinking one the command already freed would
/// walk freed memory.
struct Chain {
    registry: Registry,
    sessions: Vec<Session>,
    windows: Vec<Win>,
    panes: Vec<Pane>,
    tracked: Vec<(usize, *mut winlink)>,
}

impl Chain {
    fn new(name: &str) -> Chain {
        let mut c = Chain {
            registry: Registry::new(),
            sessions: Vec::new(),
            windows: Vec::new(),
            panes: Vec::new(),
            tracked: Vec::new(),
        };
        c.sessions
            .push(Session::new(c.sessions.len() as u_int, name));
        c.registry
            .add_session(c.sessions.last_mut().expect("a session"));
        c
    }

    fn add_session(&mut self, name: &str) -> usize {
        self.sessions
            .push(Session::new(self.sessions.len() as u_int, name));
        self.registry
            .add_session(self.sessions.last_mut().expect("a session"));
        self.sessions.len() - 1
    }

    /// A fresh pane of the chain's own, not yet hanging off any window.
    fn new_pane(&mut self, sx: u_int, sy: u_int) -> *mut window_pane {
        let id = PANE_ID_BASE + self.panes.len() as u_int + 1;
        let mut p = Pane::new(id, sx, sy, 100);
        let ptr = p.ptr();
        self.panes.push(p);
        ptr
    }

    fn push_window(&mut self, sidx: usize, idx: c_int, mut w: Win) -> *mut winlink {
        self.registry.add_window(&mut w.window);
        let wl = link(&mut self.sessions[sidx], &mut w.window, idx);
        self.tracked.push((sidx, wl));
        self.windows.push(w);
        wl
    }

    /// Links a fresh window carrying `panes` laid-out panes at index `idx`
    /// behind session `sidx`, answering its position, its winlink and its
    /// pane pointers in creation order (the first pane is the active one).
    /// Further panes are laid out by halving below the one before them.
    fn add_window(
        &mut self,
        sidx: usize,
        idx: c_int,
        panes: usize,
        sx: u_int,
        sy: u_int,
    ) -> (usize, *mut winlink, Vec<*mut window_pane>) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 13 + sidx as u_int;
        let mut w = Win {
            window: Window::new(wid, "chain", sx, sy),
        };
        let first = self.new_pane(sx, sy);
        unsafe {
            w.window.add_pane(self.panes.last_mut().expect("a pane"));
            layout_init(w.window.ptr(), first);
        }
        let mut made = vec![first];
        for _ in 1..panes {
            let p = self.new_pane(1, 1);
            unsafe {
                w.window.add_pane(self.panes.last_mut().expect("a pane"));
                let lc = layout_split_pane(*made.last().expect("a pane"), LAYOUT_TOPBOTTOM, -1, 0);
                assert!(!lc.is_null(), "there was no room to lay out");
                layout_assign_pane(lc, p, 0);
            }
            made.push(p);
        }
        let wl = self.push_window(sidx, idx, w);
        (self.windows.len() - 1, wl, made)
    }

    /// Links a fresh window whose panes have no layout cells at all, the
    /// transient state the `layout_close_pane` guard exists for.
    fn add_bare_window(
        &mut self,
        sidx: usize,
        idx: c_int,
        panes: usize,
        sx: u_int,
        sy: u_int,
    ) -> (usize, *mut winlink, Vec<*mut window_pane>) {
        let wid = WINDOW_ID_BASE + 500 + self.windows.len() as u_int;
        let mut w = Win {
            window: Window::new(wid, "bare", sx, sy),
        };
        let mut made = Vec::new();
        for _ in 0..panes {
            let p = self.new_pane(sx, sy);
            w.window.add_pane(self.panes.last_mut().expect("a pane"));
            made.push(p);
        }
        let wl = self.push_window(sidx, idx, w);
        (self.windows.len() - 1, wl, made)
    }

    fn sptr(&mut self, i: usize) -> *mut session {
        self.sessions[i].ptr()
    }

    fn wptr(&mut self, i: usize) -> *mut window {
        self.windows[i].window.ptr()
    }

    /// Drops a winlink from the cleanup list, for ones the command itself has
    /// taken back out of the session.
    fn forget(&mut self, wl: *mut winlink) {
        self.tracked.retain(|&(_, p)| p != wl);
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        for (si, wl) in ::std::mem::take(&mut self.tracked).into_iter().rev() {
            unlink(&mut self.sessions[si], wl);
        }
    }
}

#[test]
fn the_entries_advertise_their_commands_and_share_one_hook() {
    let _guard = globals();
    unsafe {
        let j = &raw const cmd_join_pane_entry;
        let m = &raw const cmd_move_pane_entry;

        assert_eq!((*j).name.to_string_lossy(), "join-pane");
        assert_eq!(
            (*j).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "joinp"
        );
        assert_eq!((*j).args.template.to_string_lossy(), "bdfhvp:l:s:t:");
        assert_eq!((*j).args.lower, 0);
        assert_eq!((*j).args.upper, 0);
        assert!((*j).args.cb.is_none());
        assert_eq!(
            (*j).usage.to_string_lossy(),
            "[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]"
        );
        assert_eq!((*j).source.flag, b's' as c_char);
        assert_eq!((*j).source.type_0, CMD_FIND_PANE);
        assert_eq!((*j).source.flags, CMD_FIND_DEFAULT_MARKED);
        assert_eq!((*j).target.flag, b't' as c_char);
        assert_eq!((*j).target.type_0, CMD_FIND_PANE);
        assert_eq!((*j).target.flags, 0);
        assert_eq!((*j).flags, 0);

        assert_eq!((*m).name.to_string_lossy(), "move-pane");
        assert_eq!(
            (*m).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "movep"
        );
        assert_eq!((*m).args.template.to_string_lossy(), "bdfhvp:l:s:t:");
        assert_eq!((*m).usage.to_string_lossy(), (*j).usage.to_string_lossy());
        assert_eq!((*m).source.flag, b's' as c_char);
        assert_eq!((*m).source.type_0, CMD_FIND_PANE);
        assert_eq!((*m).source.flags, CMD_FIND_DEFAULT_MARKED);
        assert_eq!((*m).target.flag, b't' as c_char);
        assert_eq!((*m).target.flags, 0);
        assert_eq!((*m).flags, 0);

        let join_exec = (*j).exec as usize;
        let move_exec = (*m).exec as usize;
        assert_ne!(join_exec, 0);
        assert_eq!(join_exec, move_exec, "both entries dispatch one hook");

        assert_eq!(CMD_FIND_DEFAULT_MARKED, 0x8);
        assert_eq!(SPAWN_BEFORE, 0x8);
        assert_eq!(PANE_STYLECHANGED, 0x1000);
        assert_eq!(PANE_THEMECHANGED, 0x2000);
    }
}

#[test]
fn joining_a_pane_to_itself_is_refused_and_touches_nothing() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 2, 80, 24);
    let w0 = unsafe { (*wl0).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"join-pane");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl0, -1), fs_of(wl0, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("source and target panes must be different"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        assert_eq!(winlink_count(&(*chain.sptr(0)).windows), 1);
        assert_eq!(window_count_panes(w0, 1), 2);
        assert_eq!(pane_at(w0, 0), panes[0]);
        assert_eq!(pane_at(w0, 1), panes[1]);
        assert_eq!(z_at(w0, 0), panes[0]);
        assert_eq!(window_get_active(w0), panes[0]);
        assert_eq!((*panes[0]).window, w0);
        assert_eq!(
            options_get_parent((*panes[0]).options_ptr()),
            null_mut::<options>(),
            "no reparenting happened"
        );
    }
}

#[test]
fn an_invalid_l_size_refuses_the_command_before_anything_moves() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let (w_src, w_dst) = unsafe { ((*wl_src).window(), (*wl_dst).window()) };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"join-pane -l garbage");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("size or position invalid tiled geometry"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        assert_eq!(window_count_panes(w_src, 1), 2);
        assert_eq!(window_count_panes(w_dst, 1), 1);
        assert_eq!((*src_panes[0]).window, w_src);
        assert_eq!((*dst_panes[0]).window, w_dst);
        assert_eq!(window_get_active(w_dst), dst_panes[0]);
        assert_eq!(winlink_count(&(*chain.sptr(0)).windows), 2);
    }
}

#[test]
fn a_floating_destination_pane_refuses_the_split() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl_src, _) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let w_dst = unsafe { (*wl_dst).window() };
    let dst_wp = dst_panes[0];
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"join-pane");
    unsafe {
        (*(*dst_wp).layout_cell).flags |= LAYOUT_CELL_FLOATING;
        aim_from(&mut item, caller, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("size or position can't split a floating pane"),
            "{}",
            msgs[before]
        );

        assert_eq!(window_count_panes(w_dst, 1), 1);
        assert_eq!((*dst_wp).window, w_dst);
        assert_eq!(
            (*(*dst_wp).layout_cell).flags & LAYOUT_CELL_FLOATING,
            LAYOUT_CELL_FLOATING,
            "the flag the refusal read is left alone"
        );
    }
}

#[test]
fn a_destination_without_room_for_a_second_pane_refuses_the_move() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_tiny, tiny_panes) = chain.add_window(0, 1, 1, 20, 2);
    let w_tiny = unsafe { (*wl_tiny).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"join-pane");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl_tiny, -1), fs_of(wl_src, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("size or position no space for a new pane"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        assert_eq!((*tiny_panes[0]).window, w_tiny);
        assert_eq!((*tiny_panes[0]).sy, 2, "nothing about the target moved");
        assert_eq!((*src_panes[0]).window, (*wl_src).window());
    }
}

#[test]
fn joining_below_the_target_moves_activates_and_selects_it() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let (moved, stay) = (src_panes[0], src_panes[1]);
    let kept = dst_panes[0];
    let (w_src, w_dst) = unsafe { ((*wl_src).window(), (*wl_dst).window()) };

    let mut item = Item::new().with_args(c"join-pane");
    unsafe {
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_dst, 1), 2);
        assert_eq!(pane_at(w_dst, 0), kept, "the target pane stays in front");
        assert_eq!(pane_at(w_dst, 1), moved, "the joined pane follows it");
        assert_eq!(z_at(w_dst, 0), kept);
        assert_eq!(z_at(w_dst, 1), moved);
        assert_eq!((*moved).window, w_dst);
        assert_eq!((*moved).layout_cell, (*pane_at(w_dst, 1)).layout_cell);

        assert_eq!(
            dump_cell((*w_dst).layout_root_ptr()),
            format!(
                "TB 80x24+0+0 [%{} 80x12+0+0 | %{} 80x11+0+13]",
                (*kept).id,
                (*moved).id
            )
        );
        assert_eq!((*kept).sx, 80);
        assert_eq!((*kept).sy, 12);
        assert_eq!((*moved).sx, 80);
        assert_eq!((*moved).sy, 11);
        assert_eq!((*moved).yoff, 13);

        assert_eq!(
            window_get_active(w_dst),
            moved,
            "without -d the joined pane takes over"
        );
        assert_eq!(
            window_pane_stack_first(w_dst, PaneStack::LastUsed),
            kept,
            "the old active pane is remembered"
        );

        assert_eq!(
            options_get_parent((*moved).options_ptr()),
            (*w_dst).options_ptr(),
            "the pane now inherits the destination window's options"
        );
        assert_eq!(
            (*moved).flags & (PANE_STYLECHANGED | PANE_THEMECHANGED),
            PANE_STYLECHANGED | PANE_THEMECHANGED
        );

        assert_eq!(window_count_panes(w_src, 1), 1);
        assert_eq!(pane_at(w_src, 0), stay);
        assert_eq!(
            window_get_active(w_src),
            stay,
            "the next pane took over at the source"
        );
        assert_eq!(
            (*stay).flags & (PANE_STYLECHANGED | PANE_THEMECHANGED),
            0,
            "only the mover is flagged"
        );

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&(*s).windows), 2);
        assert_eq!(
            session_get_curw(s),
            wl_dst,
            "the destination index was selected"
        );
        assert_eq!(
            winlink_of(s, (*s).lastw.first().copied()),
            wl_src,
            "the old selection went onto the stack"
        );

        let cur = crate::cmd::cmdq_get_current(item.ptr());
        assert_eq!((*cur).session(), s);
        assert_eq!((*cur).winlink(), wl_dst);
        assert_eq!((*cur).window(), w_dst);
        assert_eq!((*cur).pane(), moved);
    }
}

#[test]
fn with_d_the_destination_keeps_its_active_pane_and_current_alone() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let moved = src_panes[0];
    let kept = dst_panes[0];
    let (w_src, w_dst) = unsafe { ((*wl_src).window(), (*wl_dst).window()) };

    let mut item = Item::new().with_args(c"join-pane -d");
    unsafe {
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'd'), 1);

        let cur = crate::cmd::cmdq_get_current(item.ptr());
        let (bs, bwl, bw, bwp) = (
            (*cur).session(),
            (*cur).winlink(),
            (*cur).window(),
            (*cur).pane(),
        );

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_dst, 1), 2);
        assert_eq!(pane_at(w_dst, 1), moved);
        assert_eq!(
            window_get_active(w_dst),
            kept,
            "-d leaves the active pane alone"
        );
        assert_eq!(
            window_pane_stack_first(w_dst, PaneStack::LastUsed),
            null_mut::<window_pane>(),
            "nothing was pushed onto the destination's stack"
        );
        assert_eq!((*cur).session(), bs, "the current state was not re-found");
        assert_eq!((*cur).winlink(), bwl);
        assert_eq!((*cur).window(), bw);
        assert_eq!((*cur).pane(), bwp);

        assert_eq!(
            options_get_parent((*moved).options_ptr()),
            (*w_dst).options_ptr()
        );

        assert_eq!(window_count_panes(w_src, 1), 1);
        assert_eq!(pane_at(w_src, 0), src_panes[1]);

        let s = chain.sptr(0);
        assert_eq!(session_get_curw(s), wl_src, "no selection happened either");
        assert!((*s).lastw.is_empty());
        assert_eq!(winlink_count(&(*s).windows), 2);
    }
}

#[test]
fn before_flips_which_half_the_joined_pane_takes_but_still_lands_behind() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let moved = src_panes[0];
    let kept = dst_panes[0];
    let (w_src, w_dst) = unsafe { ((*wl_src).window(), (*wl_dst).window()) };

    let mut item = Item::new().with_args(c"join-pane -b -h");
    unsafe {
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_dst, 1), 2);
        assert_eq!(
            pane_at(w_dst, 0),
            kept,
            "the hook never sees -b, so the pane lands behind the target"
        );
        assert_eq!(pane_at(w_dst, 1), moved);
        assert_eq!(z_at(w_dst, 0), kept);
        assert_eq!(z_at(w_dst, 1), moved);

        assert_eq!(
            dump_cell((*w_dst).layout_root_ptr()),
            format!(
                "LR 80x24+0+0 [%{} 40x24+0+0 | %{} 39x24+41+0]",
                (*moved).id,
                (*kept).id
            ),
            "the split itself did honour -b -h"
        );
        assert_eq!((*moved).sx, 40);
        assert_eq!((*moved).sy, 24);
        assert_eq!((*moved).yoff, 0);
        assert_eq!((*kept).sx, 39);
        assert_eq!((*kept).xoff, 41);

        assert_eq!(window_count_panes(w_src, 1), 1);
        assert_eq!(window_get_active(w_dst), moved);
    }
}

#[test]
fn emptying_the_source_window_unlinks_it_from_its_session() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 1, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let moved = src_panes[0];
    let kept = dst_panes[0];
    let w_src = unsafe { (*wl_src).window() };
    let w_dst = unsafe { (*wl_dst).window() };

    unsafe {
        assert_eq!(
            session_select(chain.sptr(0), 1),
            0,
            "picked the other window"
        );
    }

    let mut item = Item::new().with_args(c"join-pane -d");
    unsafe {
        server_set_marked(chain.sptr(0), wl_src, moved);
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wl_src);

        assert!(
            marked_pane.pane().is_null(),
            "losing the marked pane cleared the mark"
        );
        assert!(marked_pane.session().is_null());

        assert_eq!(window_count_panes(w_src, 1), 0);
        assert!(window_get_active(w_src).is_null());

        let s = chain.sptr(0);
        assert_eq!(
            winlink_count(&(*s).windows),
            1,
            "the emptied window was unlinked"
        );
        assert!(winlink_find_by_index(&mut (*s).windows, 0).is_null());
        assert_eq!(winlink_find_by_index(&mut (*s).windows, 1), wl_dst);
        assert_eq!(
            session_get_curw(s),
            wl_dst,
            "what was selected beforehand stays selected"
        );
        assert!((*s).lastw.is_empty(), "its stack entry went with it");

        assert_eq!(window_count_panes(w_dst, 1), 2);
        assert_eq!(pane_at(w_dst, 0), kept);
        assert_eq!(pane_at(w_dst, 1), moved);
        assert_eq!(window_get_active(w_dst), kept, "-d still holds here");
    }
}

#[test]
fn moving_across_sessions_follows_the_destination_session() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("home");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let away = chain.add_session("away");
    let (_, wl_dst, dst_panes) = chain.add_window(away, 0, 1, 80, 24);
    let moved = src_panes[0];
    let kept = dst_panes[0];
    let w_src = unsafe { (*wl_src).window() };

    let mut item = Item::new().with_args(c"join-pane");
    unsafe {
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s_home = chain.sptr(0);
        let s_away = chain.sptr(away);
        assert_eq!(winlink_count(&(*s_home).windows), 1);
        assert_eq!(window_count_panes(w_src, 1), 1, "one pane stayed home");
        assert_eq!(
            session_get_curw(s_home),
            wl_src,
            "the home session was not touched"
        );

        assert_eq!(winlink_count(&(*s_away).windows), 1);
        assert_eq!(winlink_find_by_index(&mut (*s_away).windows, 0), wl_dst);
        let w_dst = (*wl_dst).window();
        assert_eq!(pane_at(w_dst, 0), kept);
        assert_eq!(pane_at(w_dst, 1), moved);
        assert_eq!((*moved).window, w_dst);
        assert_eq!(window_get_active(w_dst), moved);
        assert_eq!(
            options_get_parent((*moved).options_ptr()),
            (*w_dst).options_ptr()
        );

        assert_eq!(
            session_get_curw(s_away),
            wl_dst,
            "the destination session selects its window"
        );
        assert!(
            (*s_away).lastw.is_empty(),
            "it had nothing else to remember"
        );

        let cur = crate::cmd::cmdq_get_current(item.ptr());
        assert_eq!((*cur).session(), s_away);
        assert_eq!((*cur).winlink(), wl_dst);
        assert_eq!((*cur).window(), w_dst);
        assert_eq!((*cur).pane(), moved);
    }
}

#[test]
fn the_mark_survives_when_it_is_not_the_pane_that_leaves() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let moved = src_panes[0];
    let kept = dst_panes[0];

    let mut item = Item::new().with_args(c"join-pane -d");
    unsafe {
        server_set_marked(chain.sptr(0), wl_dst, kept);
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let mark = &raw const marked_pane;
        assert_eq!((*mark).pane(), kept, "a mark elsewhere is untouched");
        assert_eq!((*mark).winlink(), wl_dst);
        assert_eq!((*moved).flags & PANE_STYLECHANGED, PANE_STYLECHANGED);

        server_clear_marked();
        assert!(
            marked_pane.pane().is_null(),
            "cleanup leaves no mark behind"
        );
    }
}

#[test]
fn a_source_window_without_a_layout_tree_still_gives_up_its_pane() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (src_i, wl_bare, bare_panes) = chain.add_bare_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let moved = bare_panes[0];
    let stay = bare_panes[1];
    let kept = dst_panes[0];
    let w_src = chain.wptr(src_i);

    let mut item = Item::new().with_args(c"join-pane -d");
    unsafe {
        assert!((*moved).layout_cell.is_null(), "the fixture starts bare");
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_bare, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_src, 1), 1);
        assert_eq!(pane_at(w_src, 0), stay);

        let w_dst = (*wl_dst).window();
        assert_eq!(window_count_panes(w_dst, 1), 2);
        assert_eq!(pane_at(w_dst, 1), moved);
        assert_ne!((*moved).layout_cell, null_mut(), "it was given a cell");
        assert_eq!(
            layout_cell_pane(w_dst, (*moved).layout_cell),
            moved,
            "and the cell points back at it"
        );
        assert_eq!((*kept).sy, 12);
        assert_eq!((*moved).sy, 11);
        assert_eq!(
            options_get_parent((*moved).options_ptr()),
            (*w_dst).options_ptr()
        );
    }
}

#[test]
fn the_move_pane_name_runs_the_same_hook() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_src, src_panes) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_dst, dst_panes) = chain.add_window(0, 1, 1, 80, 24);
    let moved = src_panes[0];
    let kept = dst_panes[0];
    let (w_src, w_dst) = unsafe { ((*wl_src).window(), (*wl_dst).window()) };

    let mut item = Item::new().with_args(c"move-pane -d");
    unsafe {
        aim(&mut item, fs_of(wl_dst, -1), fs_of(wl_src, -1));
        assert_eq!(
            cmd_get_entry(&*item.cmd()).name.to_string_lossy(),
            "move-pane",
            "the parser resolved the alias spelling to its own entry"
        );

        assert_eq!(run_move(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_dst, 1), 2);
        assert_eq!(pane_at(w_dst, 1), moved);
        assert_eq!(window_get_active(w_dst), kept, "-d holds for move-pane too");
        assert_eq!(window_count_panes(w_src, 1), 1);
    }
}
