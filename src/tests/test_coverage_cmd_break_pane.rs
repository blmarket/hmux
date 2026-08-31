//! Unit tests for [`crate::cmd::cmd_break_pane`], the exec hook behind the
//! `break-pane` command.
//!
//! The command's two halves are both reachable without a live server and both
//! are exercised here through [`cmd_break_pane_entry`] `.exec`, the very hook
//! the command queue calls: the single-pane half relinks the window into the
//! destination session ([`server_link_window`] and friends), while the
//! multi-pane half detaches the source pane from its window, builds a real
//! window for it with [`window_create`], lays it out, names it and attaches it.
//! Around those, the tests pin the entry's metadata and template, the `-n`
//! validity refusal, the two "index" refusals, the `-a`/`-b` shuffling of
//! destination indices, `-d`'s hold on the current window, `-n`'s rename plus
//! `automatic-rename` switch-off, and a cross-session break whose select and
//! current-state updates land in the destination session.
//!
//! Two limits worth recording. With `-P` the command prints the expanded
//! template to the item's client; the fixtures run against an item with no
//! client, so `cmdq_print` only reaches `log_debug` and that test pins the
//! decision and the movement rather than the printed line. And the shuffle
//! failure return of `winlink_shuffle_up` needs every index up to `INT_MAX`
//! taken, which no fixture can arrange.

use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_break_pane::{
    BREAK_PANE_TEMPLATE, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX, PANE_CHANGED,
    PANE_STYLECHANGED, PANE_THEMECHANGED, cmd_break_pane_entry,
};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL};
use crate::layout::layout_free_cell;
use crate::options::{options_get_number, options_get_parent};
use crate::proc::PEER_BAD;
use crate::server::{marked_pane, message_log, server_set_marked};
use crate::session::session_get_curw;
use crate::session::winlink_of;
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Window, dump_cell, ensure_reactor, globals, link, seen,
    unlink, zeroed,
};
use crate::types::*;
use crate::window::window_get_active;
use crate::window::window_get_latest;
use crate::window::{
    window_count_panes, window_find_by_id, window_panes_first, window_panes_next, winlink_count,
    winlink_find_by_index, winlink_find_by_window,
};
use ::core::ffi::{c_char, c_int};
use ::std::ffi::CString;

/// Where the fixture windows' ids start, far above anything `window_create`
/// hands out from its own counter, so the two never collide inside the
/// server's id-keyed window tree.
const WINDOW_ID_BASE: u_int = 500_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 600_000;

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

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_break_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states where the test wants
/// them, as the resolved find states of a prepared command queue item would be.
/// `caller`, when given, becomes the item's own client and its target client,
/// since [`Item::with_client`] carries only an anonymous one of its own.
unsafe fn aim(item: &mut Item, target: cmd_find_state, source: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = target.clone();
        (*p).source = source;
        *crate::cmd::cmdq_get_current(p) = target;
    }
}

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

/// Windows the exec hook built with `window_create` and left in the server's
/// tree, removed and cleaned up again when the test ends. The moved pane stays
/// owned by the [`Pane`] fixture.
struct CreatedWindows(Vec<WindowRef>);

impl CreatedWindows {
    fn new() -> CreatedWindows {
        CreatedWindows(Vec::new())
    }

    fn keep(&mut self, w: *mut window) {
        let Some(reference) = crate::window::window_ref_from_ptr(w) else {
            panic!("created window has no owner");
        };
        self.0.push(reference);
    }
}

impl Drop for CreatedWindows {
    fn drop(&mut self) {
        unsafe {
            for w_ref in &self.0 {
                let w = w_ref.as_ptr();
                crate::window::windows.map().remove(&(*w).id);
                layout_free_cell(w, (*w).layout_root.take());
                layout_free_cell(w, (*w).saved_layout_root.take());
                w_ref.mark_unmanaged();
            }
        }
    }
}

/// A registered session holding linked windows of real panes. Every pane gets
/// a shell string so `default_window_name` has something deterministic to read
/// if the pane becomes the active pane of a fresh window. Winlinks the chain
/// itself made are unlinked again on the way out; winlinks the command frees
/// or creates itself are told apart with [`Chain::forget`], because unlinking
/// one the command already freed would walk freed memory.
struct Chain {
    registry: Registry,
    sessions: Vec<Session>,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    shells: Vec<CString>,
    tracked: Vec<(usize, *mut winlink)>,
}

impl Chain {
    fn new(name: &str) -> Chain {
        let mut c = Chain {
            registry: Registry::new(),
            sessions: Vec::new(),
            windows: Vec::new(),
            panes: Vec::new(),
            shells: Vec::new(),
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

    /// Links a fresh window carrying `panes` panes at index `idx` behind
    /// session `sidx`, answering its position, its winlink and its pane
    /// pointers in creation order (the first pane is the active one).
    fn add_window(
        &mut self,
        sidx: usize,
        idx: c_int,
        panes: usize,
        sx: u_int,
        sy: u_int,
    ) -> (usize, *mut winlink, Vec<*mut window_pane>) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 13 + sidx as u_int;
        let mut w = Window::new(wid, "chain", sx, sy);
        let mut made = Vec::new();
        for _ in 0..panes {
            let pid = PANE_ID_BASE + self.panes.len() as u_int + 1;
            let mut p = Pane::new(pid, sx, sy, 100);
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
        (self.windows.len() - 1, wl, made)
    }

    fn sptr(&mut self, i: usize) -> *mut session {
        self.sessions[i].ptr()
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
fn the_entry_advertises_the_command_its_flags_and_the_constants_it_runs_with() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_break_pane_entry;
        assert_eq!((*e).name.to_string_lossy(), "break-pane");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "breakp"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "abdPF:n:s:t:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-abdP] [-F format] [-n window-name] [-s src-pane] [-t dst-window]"
        );
        assert_eq!((*e).source.flag, b's' as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*e).target.flags, CMD_FIND_WINDOW_INDEX);
        assert_eq!((*e).flags, 0);

        assert_eq!(PANE_CHANGED, 0x80);
        assert_eq!(PANE_STYLECHANGED, 0x1000);
        assert_eq!(PANE_THEMECHANGED, 0x2000);
        assert_eq!(CMD_FIND_WINDOW_INDEX, 0x4);
    }
}

#[test]
fn the_default_template_is_the_upstream_one() {
    let expected: &[u8] = b"#{session_name}:#{window_index}.#{pane_index}\0";
    let got: Vec<u8> = BREAK_PANE_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(BREAK_PANE_TEMPLATE.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );
}

#[test]
fn an_invalid_n_refuses_the_command_and_touches_nothing() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 1, 80, 24);
    let w0 = unsafe { (*wl0).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"break-pane -n a\\200b");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl0, -1), fs_of(wl0, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("invalid window name"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        assert_eq!(
            winlink_find_by_index(&raw mut (*chain.sptr(0)).windows, 0),
            wl0
        );
        assert_eq!((*wl0).window(), w0);
        assert_eq!(winlink_count(&raw mut (*chain.sptr(0)).windows), 1);
        assert_eq!(window_count_panes(w0, 1), 1);
        assert_eq!(pane_at(w0, 0), panes[0]);
        assert_eq!(seen(cstr_ptr(&(*w0).name)), "chain");
        assert!(marked_pane.pane().is_null());
    }
}

#[test]
fn relinking_a_single_pane_window_over_its_own_index_reports_same_index() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 1, 80, 24);
    let w0 = unsafe { (*wl0).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"break-pane");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl0, 0), fs_of(wl0, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(msgs[before].contains("same index: 0"), "{}", msgs[before]);
        assert_eq!((*caller).retval, 1);

        assert_eq!(
            winlink_find_by_index(&raw mut (*chain.sptr(0)).windows, 0),
            wl0
        );
        assert_eq!(session_get_curw(chain.sptr(0)), wl0);
        assert_eq!(winlink_count(&raw mut (*chain.sptr(0)).windows), 1);
        assert_eq!(window_count_panes(w0, 1), 1);
        assert_eq!(pane_at(w0, 0), panes[0]);
    }
}

#[test]
fn breaking_a_multi_pane_window_onto_a_taken_explicit_index_refuses_first() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 2, 80, 24);
    let w0 = unsafe { (*wl0).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"break-pane");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl0, 0), fs_of(wl0, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(msgs[before].contains("index in use: 0"), "{}", msgs[before]);
        assert_eq!((*caller).retval, 1);

        assert_eq!(winlink_count(&raw mut (*chain.sptr(0)).windows), 1);
        assert_eq!(window_count_panes(w0, 1), 2);
        assert_eq!(pane_at(w0, 0), panes[0]);
        assert_eq!(pane_at(w0, 1), panes[1]);
        assert_eq!(window_get_active(w0), panes[0]);
        assert_eq!((*w0).z_index.first().copied(), Some((*panes[0]).id));
    }
}

#[test]
fn breaking_the_active_pane_builds_it_a_window_and_selects_it() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 2, 80, 24);
    let w0 = unsafe { (*wl0).window() };
    let (moved, left) = (panes[0], panes[1]);

    let mut created = CreatedWindows::new();
    let mut item = Item::new().with_args(c"break-pane");
    unsafe {
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));
        server_set_marked(chain.sptr(0), wl0, moved);

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert!(
            marked_pane.pane().is_null(),
            "the mark followed the pane away"
        );

        assert_eq!(window_count_panes(w0, 1), 1);
        assert_eq!(pane_at(w0, 0), left);
        assert_eq!(window_get_active(w0), left);

        let nw = (*moved).window;
        assert_ne!(nw, w0, "the pane did not move");
        created.keep(nw);
        assert_eq!(window_find_by_id((*nw).id), nw, "the new window is unknown");
        assert_eq!((*nw).sx, 80);
        assert_eq!((*nw).sy, 24);
        assert_eq!(window_panes_first(nw), moved);
        assert_eq!((*nw).z_index.first().copied(), Some((*moved).id));
        assert_eq!(window_get_active(nw), moved);
        assert_eq!(window_count_panes(nw, 1), 1);
        assert_eq!(
            (*moved).flags & (PANE_CHANGED | PANE_STYLECHANGED | PANE_THEMECHANGED),
            PANE_CHANGED | PANE_STYLECHANGED | PANE_THEMECHANGED
        );

        assert_eq!(
            dump_cell((*nw).layout_root_ptr()),
            format!("%{} 80x24+0+0", (*moved).id)
        );
        assert_eq!((*moved).layout_cell, (*nw).layout_root_ptr());
        assert_eq!(
            seen(cstr_ptr(&(*nw).name)),
            "sh",
            "the name comes from the pane's shell"
        );
        assert_eq!(
            crate::options::options_get_number((*nw).options_ptr(), c"automatic-rename".as_ptr()),
            1,
            "a defaulted name leaves automatic renaming alone"
        );
        assert_eq!(
            options_get_parent((*moved).options_ptr()),
            (*nw).options_ptr()
        );
        assert!(window_get_latest(nw).is_null());

        let s = chain.sptr(0);
        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert!(!wl_new.is_null());
        assert_eq!((*wl_new).idx, 1, "the next free index was chosen");
        assert_eq!((*wl_new).window(), nw);
        assert_eq!(winlink_find_by_window(&raw mut (*s).windows, nw), wl_new);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 0), wl0);
        assert_eq!((*wl0).window(), w0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
        assert_eq!(
            session_get_curw(s),
            wl_new,
            "without -d the new window is selected"
        );
        assert_eq!(
            winlink_of(s, (*s).lastw.first().copied()),
            wl0,
            "the old selection went onto the stack"
        );
    }
}

#[test]
fn with_d_the_new_window_is_not_selected_and_n_names_it() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 2, 80, 24);
    let w0 = unsafe { (*wl0).window() };
    let moved = panes[0];

    let mut created = CreatedWindows::new();
    let mut item = Item::new().with_args(c"break-pane -d -n mined");
    unsafe {
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(args_has(args, b'd'), 1);
        assert_eq!(seen(args_get(args, b'n')), "mined");

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w0, 1), 1);

        let s = chain.sptr(0);
        let nw = (*moved).window;
        created.keep(nw);
        assert_ne!(nw, w0);
        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert_eq!((*wl_new).window(), nw);
        assert_eq!(seen(cstr_ptr(&(*nw).name)), "mined");
        assert_eq!(
            options_get_number((*nw).options_ptr(), c"automatic-rename".as_ptr()),
            0,
            "-n switches automatic renaming off"
        );
        assert_eq!(
            session_get_curw(s),
            wl0,
            "-d keeps the current window selected"
        );
        assert!((*s).lastw.is_empty(), "nothing was pushed onto the stack");
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
    }
}

#[test]
fn before_shuffles_every_winlink_from_the_target_up() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_source, _) = chain.add_window(0, 0, 2, 80, 24);
    let w_source = unsafe { (*wl_source).window() };
    let (_, wl_target, _) = chain.add_window(0, 1, 1, 80, 24);

    let mut created = CreatedWindows::new();
    let mut item = Item::new().with_args(c"break-pane -b");
    unsafe {
        aim(&mut item, fs_of(wl_target, -1), fs_of(wl_source, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s = chain.sptr(0);
        let wl_before = winlink_find_by_index(&raw mut (*s).windows, 0);
        assert_eq!(
            wl_before, wl_source,
            "the source below the target stayed put"
        );
        assert_eq!((*wl_before).window(), w_source);
        assert_eq!(window_count_panes((*wl_before).window(), 1), 1);

        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 1);
        let nw = (*wl_new).window();
        created.keep(nw);
        assert_ne!(nw, w_source);
        assert_eq!(
            (*wl_new).idx,
            1,
            "the new window took the target's old place"
        );

        let wl_after = winlink_find_by_index(&raw mut (*s).windows, 2);
        assert_eq!(wl_after, wl_target, "the target shuffled up above it");
        assert_eq!((*wl_target).idx, 2);
        assert_eq!(winlink_count(&raw mut (*s).windows), 3);
        assert_eq!(
            session_get_curw(s),
            wl_new,
            "without -d the new window is selected"
        );
    }
}

#[test]
fn after_inserts_above_the_target_without_disturbing_it() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl_source, _) = chain.add_window(0, 0, 2, 80, 24);
    let (_, wl_target, _) = chain.add_window(0, 1, 1, 80, 24);

    let mut created = CreatedWindows::new();
    let mut item = Item::new().with_args(c"break-pane -a");
    unsafe {
        aim(&mut item, fs_of(wl_target, -1), fs_of(wl_source, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s = chain.sptr(0);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 0), wl_source);
        let wl_kept = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert_eq!(wl_kept, wl_target, "the target stayed where it was");
        assert_eq!((*wl_target).idx, 1);

        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 2);
        created.keep((*wl_new).window());
        assert_eq!((*wl_new).idx, 2, "the new window went just past the target");
        assert_eq!(winlink_count(&raw mut (*s).windows), 3);
        assert_eq!(session_get_curw(s), wl_new);
    }
}

#[test]
fn p_prints_through_the_default_template_without_a_client() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (_, wl0, panes) = chain.add_window(0, 0, 2, 80, 24);
    let w0 = unsafe { (*wl0).window() };

    let mut created = CreatedWindows::new();
    let mut item = Item::new().with_args(c"break-pane -P");
    unsafe {
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w0, 1), 1);
        let nw = (*panes[0]).window;
        created.keep(nw);
        let s = chain.sptr(0);
        let wl_new = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert_eq!((*wl_new).window(), nw);
        assert_eq!(session_get_curw(s), wl_new);
    }
}

#[test]
fn breaking_across_sessions_selects_in_the_destination_session() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("home");
    let (_, wl_source, panes) = chain.add_window(0, 0, 2, 80, 24);
    let w_source = unsafe { (*wl_source).window() };
    let away = chain.add_session("away");
    let (_, wl_dst_home, _) = chain.add_window(away, 0, 1, 80, 24);

    let mut created = CreatedWindows::new();
    let mut item = Item::new().with_args(c"break-pane");
    unsafe {
        aim(&mut item, fs_of(wl_dst_home, -1), fs_of(wl_source, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w_source, 1), 1, "one pane stayed home");

        let s_away = chain.sptr(away);
        let wl_new = winlink_find_by_index(&raw mut (*s_away).windows, 1);
        let nw = (*wl_new).window();
        created.keep(nw);
        assert_eq!((*wl_new).idx, 1, "the destination chose its own free index");
        assert_eq!((*wl_new).window(), nw);
        assert_eq!(winlink_count(&raw mut (*s_away).windows), 2);
        assert_eq!(
            session_get_curw(s_away),
            wl_new,
            "the destination session follows the new window"
        );
        assert_eq!(
            winlink_count(&raw mut (*chain.sptr(0)).windows),
            1,
            "the home session kept exactly its own window"
        );

        let cur = crate::cmd::cmdq_get_current(item.ptr());
        assert_eq!((*cur).session(), s_away);
        assert_eq!((*cur).winlink(), wl_new);
        assert_eq!((*cur).window(), nw);
        assert_eq!(
            (*cur).pane(),
            panes[0],
            "the moved pane is the state's active pane"
        );
    }
}
