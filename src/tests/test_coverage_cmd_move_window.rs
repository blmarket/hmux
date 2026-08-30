//! Unit tests for [`crate::cmd::cmd_move_window`], the single exec hook behind
//! both the `move-window` and `link-window` commands.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! parsed command's own entry pointer with an item whose source find state has
//! already been resolved and whose current state says where an omitted `-t`
//! resolves to. Around that hook the tests pin both entries' metadata
//! (including that they share one function), the `-r` renumber turn with its
//! quiet refusal for an unknown session, the refusals that fire before
//! anything moves — a target that resolves to nothing, a destination index
//! already occupied by another window — and the moving half itself: a
//! cross-session move whose newcomer is selected in the destination session
//! while the emptied source slot disappears, the `link-window` spelling
//! keeping its source linked, `-b` and `-a` making room around the target
//! window with [`winlink_shuffle_up`], the `renumber-windows` option
//! tightening the source session once the move has landed, and the entry-name
//! dispatch that decides whether the source winlink is unlinked at all.
//!
//! One shape is deliberately left alone: a session whose last window moves
//! away hands over to `server_destroy_session_group`, which frees memory these
//! fixtures own, so every move here leaves its source session another window.

use crate::arguments::args_has;
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_move_window::{
    CMD_FIND_PANE, CMD_FIND_QUIET, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT, cmd_link_window_entry,
    cmd_move_window_entry,
};
use crate::cmd::cmdq_get_current;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::options::options_set_number;
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::session::session_get_curw;
use crate::session::winlink_of;
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, seen,
    unlink_all, zeroed,
};
use crate::types::*;
use crate::window::window_get_active;
use crate::window::{winlink_count, winlink_find_by_index};
use ::core::ffi::{c_char, c_int};

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 600_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 900_000;

/// A peer for the fixture client, marked bad so `proc_send` refuses any
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

/// Runs the item's parsed command through its own entry's exec hook, the way
/// the command queue would, so the `move-window` and `link-window` spellings
/// each arrive under their own name.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's source state at the resolved source and its current state
/// at where an omitted `-t` would land, as the command queue would have them
/// before the hook runs. The target state is filled in too, although this hook
/// builds its own.
unsafe fn aim(item: &mut Item, source: cmd_find_state, current: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).source = source;
        (*p).target = current.clone();
        *cmdq_get_current(p) = current;
    }
}

/// Points the states as [`aim`] does and makes `caller` the item's own client,
/// so `cmdq_error` files its message against that client.
unsafe fn aim_from(
    item: &mut Item,
    caller: *mut client,
    source: cmd_find_state,
    current: cmd_find_state,
) {
    unsafe {
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        aim(item, source, current);
    }
}

/// The find state of `wl`, as resolution would leave it: its session, its
/// window and that window's active pane.
unsafe fn fs_of(wl: *mut winlink) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
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

/// Registered sessions holding registered windows of one pane each, everything
/// in the server's trees the way [`cmd_find_target`] walks them. Winlinks the
/// chain itself made are unlinked again on the way out; winlinks the command
/// frees itself are told apart with [`Chain::forget`], because unlinking one
/// the command already took would walk freed memory.
struct Chain {
    registry: Registry,
    sessions: Vec<Session>,
    windows: Vec<Window>,
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
        c.add_session(name);
        c
    }

    fn add_session(&mut self, name: &str) -> usize {
        self.sessions
            .push(Session::new(self.sessions.len() as u_int, name));
        self.registry
            .add_session(self.sessions.last_mut().expect("a session"));
        self.sessions.len() - 1
    }

    /// Links a fresh window carrying one pane at index `idx` behind session
    /// `sidx`, answering its winlink, its window and its pane.
    fn add_window(
        &mut self,
        sidx: usize,
        idx: c_int,
    ) -> (*mut winlink, *mut window, *mut window_pane) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 7 + sidx as u_int;
        let mut w = Window::new(wid, "chain", 80, 24);
        let mut p = Pane::new(PANE_ID_BASE + self.panes.len() as u_int, 80, 24, 100);
        w.add_pane(&mut p);
        self.registry.add_window(&mut w);
        self.registry.add_pane(&mut p);
        let wp = p.ptr();
        let wptr = w.ptr();
        let wl = link(&mut self.sessions[sidx], &mut w, idx);
        self.tracked.push((sidx, wl));
        self.windows.push(w);
        self.panes.push(p);
        (wl, wptr, wp)
    }

    fn sptr(&mut self, i: usize) -> *mut session {
        self.sessions[i].ptr()
    }

    /// Drops a winlink from the cleanup list, for ones the command itself has
    /// taken back out of its session.
    fn forget(&mut self, wl: *mut winlink) {
        self.tracked.retain(|&(_, p)| p != wl);
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        self.tracked.clear();
        for s in &mut self.sessions {
            unlink_all(s);
        }
    }
}

#[test]
fn the_entries_advertise_their_commands_and_share_one_hook() {
    let _guard = globals();
    unsafe {
        let m = &raw const cmd_move_window_entry;
        let l = &raw const cmd_link_window_entry;

        assert_eq!((*m).name.to_string_lossy(), "move-window");
        assert_eq!(
            (*m).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "movew"
        );
        assert_eq!((*m).args.template.to_string_lossy(), "abdkrs:t:");
        assert_eq!((*m).args.lower, 0);
        assert_eq!((*m).args.upper, 0);
        assert!((*m).args.cb.is_none());
        assert_eq!(
            (*m).usage.to_string_lossy(),
            "[-abdkr] [-s src-window] [-t dst-window]"
        );
        assert_eq!((*m).source.flag, b's' as c_char);
        assert_eq!((*m).source.type_0, CMD_FIND_WINDOW);
        assert_eq!((*m).source.flags, 0);
        assert_eq!((*m).target.flag, 0 as c_char);
        assert_eq!((*m).target.type_0, CMD_FIND_PANE);
        assert_eq!((*m).target.flags, 0);
        assert_eq!((*m).flags, 0);

        assert_eq!((*l).name.to_string_lossy(), "link-window");
        assert_eq!(
            (*l).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "linkw"
        );
        assert_eq!((*l).args.template.to_string_lossy(), "abdks:t:");
        assert_eq!(
            (*l).usage.to_string_lossy(),
            "[-abdk] [-s src-window] [-t dst-window]"
        );
        assert_eq!((*l).source.flag, b's' as c_char);
        assert_eq!((*l).source.type_0, CMD_FIND_WINDOW);
        assert_eq!((*l).source.flags, 0);
        assert_eq!((*l).target.flag, 0 as c_char);
        assert_eq!((*l).target.type_0, CMD_FIND_PANE);
        assert_eq!((*l).flags, 0);

        let move_exec = (*m).exec as usize;
        let link_exec = (*l).exec as usize;
        assert_ne!(move_exec, 0);
        assert_eq!(move_exec, link_exec, "both entries dispatch one hook");

        assert_eq!(CMD_FIND_PANE, 0);
        assert_eq!(CMD_FIND_WINDOW, 1);
        assert_eq!(CMD_FIND_SESSION, 2);
        assert_eq!(CMD_RETURN_ERROR, -1);
        assert_eq!(CMD_RETURN_NORMAL, 0);
        assert_eq!(CMD_RETURN_WAIT, 1);
        assert_eq!(CMD_RETURN_STOP, 2);
        assert_eq!(CMD_FIND_QUIET, 0x2);
        assert_eq!(CMD_FIND_WINDOW_INDEX, 0x4);
    }
}

#[test]
fn the_r_flag_renumbers_the_current_session_from_base_index() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("renumber");
    let (wla, wa, _) = chain.add_window(0, 3);
    let (wlb, wb, _) = chain.add_window(0, 7);

    let mut item = Item::new().with_args(c"move-window -r");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wla));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wla);
        chain.forget(wlb);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
        let n0 = winlink_find_by_index(&raw mut (*s).windows, 0);
        let n1 = winlink_find_by_index(&raw mut (*s).windows, 1);
        assert!(!n0.is_null() && !n1.is_null());
        assert_ne!(n0, n1);
        assert_ne!(n0, wla, "the renumber rebuilt every winlink");
        assert_eq!((*n0).window(), wa);
        assert_eq!((*n1).window(), wb);
        assert_eq!((*n0).idx, 0);
        assert_eq!((*n1).idx, 1);
        assert!(winlink_find_by_index(&raw mut (*s).windows, 3).is_null());
        assert!(winlink_find_by_index(&raw mut (*s).windows, 7).is_null());
        assert_eq!(
            session_get_curw(s),
            n0,
            "the window that was current stayed current"
        );
        assert!((*s).lastw.is_empty());
    }
}

#[test]
fn the_r_flag_with_an_unknown_session_refuses_and_leaves_the_gaps() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("renumber");
    let (wla, _, _) = chain.add_window(0, 3);
    let (wlb, _, _) = chain.add_window(0, 7);

    let mut item = Item::new().with_args(c"move-window -r -t nosuchsession");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wla));

        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 3), wla);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 7), wlb);
        assert_eq!(session_get_curw(s), wla);
    }
}

#[test]
fn moving_to_another_session_selects_it_and_unlinks_the_source() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("home");
    let away = chain.add_session("away");
    let (wla, wa, pa) = chain.add_window(0, 0);
    let (wlb, _, _) = chain.add_window(0, 1);
    let (wlc, wc, pc) = chain.add_window(away, 0);

    let mut item = Item::new().with_args(c"move-window");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wlc));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wla);

        let home = chain.sptr(0);
        let dst = chain.sptr(away);
        assert_eq!(winlink_count(&raw mut (*home).windows), 1);
        assert!(winlink_find_by_index(&raw mut (*home).windows, 0).is_null());
        assert_eq!(winlink_find_by_index(&raw mut (*home).windows, 1), wlb);
        assert_eq!(
            session_get_curw(home),
            wlb,
            "the source session moved onto its other window"
        );
        assert!((*home).lastw.is_empty());

        assert_eq!(winlink_count(&raw mut (*dst).windows), 2);
        let moved = winlink_find_by_index(&raw mut (*dst).windows, 1);
        assert!(!moved.is_null());
        assert_ne!(moved, wla, "the destination link is a fresh winlink");
        assert_eq!((*moved).window(), wa);
        assert_eq!((*moved).session(), dst);
        assert_eq!(winlink_find_by_index(&raw mut (*dst).windows, 0), wlc);
        assert_eq!(
            session_get_curw(dst),
            moved,
            "without -d the destination selects the newcomer"
        );
        assert_eq!(
            winlink_of(dst, (*dst).lastw.first().copied()),
            wlc,
            "its old selection is stacked"
        );

        assert_eq!(
            window_get_active(wa),
            pa,
            "the window came across untouched"
        );
        assert_eq!(
            window_get_active(wc),
            pc,
            "and so did the destination's own"
        );
    }
}

#[test]
fn the_link_window_name_leaves_the_source_window_in_place() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("home");
    let away = chain.add_session("away");
    let (wla, wa, _) = chain.add_window(0, 0);
    let (_, wb, _) = chain.add_window(0, 1);
    let (wlc, _, _) = chain.add_window(away, 0);

    let mut item = Item::new().with_args(c"link-window");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wlc));
        assert_eq!(
            cmd_get_entry(&*item.cmd()).name.to_string_lossy(),
            "link-window",
            "the parser resolved the spelling to its own entry"
        );

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let home = chain.sptr(0);
        let dst = chain.sptr(away);
        assert_eq!(winlink_count(&raw mut (*home).windows), 2);
        assert_eq!(winlink_find_by_index(&raw mut (*home).windows, 0), wla);
        assert_eq!(
            session_get_curw(home),
            wla,
            "the source session was not moved off anything"
        );
        assert!((*home).lastw.is_empty());
        assert_eq!(
            (*winlink_find_by_index(&raw mut (*home).windows, 1)).window(),
            wb
        );

        assert_eq!(winlink_count(&raw mut (*dst).windows), 2);
        let added = winlink_find_by_index(&raw mut (*dst).windows, 1);
        assert!(!added.is_null());
        assert_eq!((*added).window(), wa, "the same window is now held twice");
        assert_eq!(session_get_curw(dst), added);
        assert_eq!(winlink_of(dst, (*dst).lastw.first().copied()), wlc);
    }
}

#[test]
fn an_occupied_destination_index_refuses_and_reports_a_cause() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let (wla, _, _) = chain.add_window(0, 0);
    let (wlb, _, _) = chain.add_window(0, 1);
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"move-window -d -t 1");
    unsafe {
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'd'), 1);
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b't'), 1);
        aim_from(&mut item, caller, fs_of(wla), fs_of(wla));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(msgs[before].contains("index in use: 1"), "{}", msgs[before]);
        assert_eq!((*caller).retval, 1);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 0), wla);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 1), wlb);
        assert_eq!(
            session_get_curw(s),
            wla,
            "nothing was disturbed by the refusal"
        );
    }
}

#[test]
fn before_shuffles_later_windows_up_to_make_room_at_the_target() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (wla, wa, _) = chain.add_window(0, 0);
    let (_, wb, _) = chain.add_window(0, 1);
    let (_, wc, _) = chain.add_window(0, 2);

    let mut item = Item::new().with_args(c"move-window -b -d -t 2");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wla));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wla);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 3);
        assert!(winlink_find_by_index(&raw mut (*s).windows, 0).is_null());
        let at_one = winlink_find_by_index(&raw mut (*s).windows, 1);
        let at_two = winlink_find_by_index(&raw mut (*s).windows, 2);
        let at_three = winlink_find_by_index(&raw mut (*s).windows, 3);
        assert_eq!((*at_one).window(), wb, "the window below the target stayed");
        assert_eq!(
            (*at_two).window(),
            wa,
            "-b made room exactly at the target index"
        );
        assert_eq!((*at_three).window(), wc, "the target itself shuffled up");
        assert_eq!(
            session_get_curw(s),
            at_three,
            "losing the current window landed on the last one"
        );
        assert!((*s).lastw.is_empty());
    }
}

#[test]
fn after_targets_the_slot_past_the_target_window() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (wla, wa, _) = chain.add_window(0, 0);
    let (_, wb, _) = chain.add_window(0, 1);

    let mut item = Item::new().with_args(c"move-window -a -d -t 1");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wla));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wla);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
        let at_two = winlink_find_by_index(&raw mut (*s).windows, 2);
        assert_eq!((*at_two).window(), wa, "-a picked the slot past the target");
        assert_eq!(
            (*winlink_find_by_index(&raw mut (*s).windows, 1)).window(),
            wb
        );
        assert!(winlink_find_by_index(&raw mut (*s).windows, 0).is_null());
        assert_eq!(session_get_curw(s), at_two);
    }
}

#[test]
fn renumber_windows_tightens_the_source_session_after_the_move() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("home");
    let away = chain.add_session("away");
    let (wla, wa, _) = chain.add_window(0, 5);
    let (wlb, wb, _) = chain.add_window(0, 9);
    let (wlc, _, _) = chain.add_window(away, 0);
    unsafe {
        options_set_number(chain.sessions[0].options(), c"renumber-windows".as_ptr(), 1);
    }

    let mut item = Item::new().with_args(c"move-window -d");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wlc));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wla);
        chain.forget(wlb);

        let home = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*home).windows), 1);
        let kept = winlink_find_by_index(&raw mut (*home).windows, 0);
        assert!(!kept.is_null());
        assert_ne!(kept, wlb, "the option rebuilt the remaining winlink");
        assert_eq!((*kept).window(), wb);
        assert_eq!((*kept).idx, 0);
        assert_eq!(session_get_curw(home), kept);
        assert!((*home).lastw.is_empty());

        let dst = chain.sptr(away);
        assert_eq!(winlink_count(&raw mut (*dst).windows), 2);
        let moved = winlink_find_by_index(&raw mut (*dst).windows, 1);
        assert_eq!((*moved).window(), wa);
        assert_eq!(session_get_curw(dst), wlc, "-d left the destination alone");
    }
}

#[test]
fn an_unresolvable_target_refuses_without_touching_anything() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (wla, _, _) = chain.add_window(0, 0);
    let (wlb, _, _) = chain.add_window(0, 1);

    let mut item = Item::new().with_args(c"move-window -d -t nosuchwindow");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wla));

        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&raw mut (*s).windows), 2);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 0), wla);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 1), wlb);
        assert_eq!(session_get_curw(s), wla);
        assert!((*s).lastw.is_empty());
    }
}
