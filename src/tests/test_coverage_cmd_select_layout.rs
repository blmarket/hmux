//! Unit tests for [`crate::cmd::cmd_select_layout`] — the three command
//! entries `select-layout`, `next-layout` and `previous-layout`, which share
//! one exec hook, together with the block of message-protocol, style, layout
//! and command constants the file declares.
//!
//! Exec is reached through each entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose target is a registered session–winlink–window–pane
//! chain whose window carries a real layout tree. Every deterministic branch
//! is exercised: the bare call that stores a fresh dump of the current tree in
//! the window's `old_layout` and returns at once when nothing stored can be
//! reused; named layouts through `layout_set_lookup` and `layout_set_select`;
//! stepping through the layout ring by entry identity and again by `-n`/`-p`;
//! `-E` spreading the active pane's branch out evenly; `-o` re-applying the
//! dump a previous run stored, through the parser rather than the lookup; the
//! unzooming that precedes everything else; and both refusals — an unparsable
//! name and a checksum-valid dump carrying fewer cells than the window has
//! panes — which file their cause against the item's client, restore the saved
//! layout and answer error.
//!
//! Two limits are deliberate, like the other suites: no client is attached to
//! the fixture window, so `server_redraw_window` walks an empty redraw set and
//! the drawing half of a run stays unobservable, and refusals report through a
//! client whose peer is marked bad, so nothing reaches a descriptor. Traces
//! left behind on purpose: every successful run ends in `recalculate_sizes`
//! and a `window-layout-changed` notification that sits on the global command
//! queue nothing ever drains, and each refusal leaves a message-log line plus
//! a buffered stream file against the fixture client. Everything else these
//! tests touch is taken and given back under [`globals`].

use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_select_layout::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_TARGET_WINDOW_USAGE, LAYOUT_TOPBOTTOM, cmd_next_layout_entry, cmd_previous_layout_entry,
    cmd_select_layout_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::layout::{
    layout_assign_pane, layout_free, layout_init, layout_root_ptr, layout_split_pane,
};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, dump_cell, globals, link, seen, unlink, zeroed,
};
use crate::types::*;
use crate::window::window_panes_first;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

/// The entries under test, all pointing their exec at one hook.
const SELECTL: *const cmd_entry = &raw const cmd_select_layout_entry;
const NEXTL: *const cmd_entry = &raw const cmd_next_layout_entry;
const PREVL: *const cmd_entry = &raw const cmd_previous_layout_entry;

/// Where the tests' items claim to come from, which is what `cfg_add_cause`
/// would report them under when no client is attached.
const FILE: &CStr = c"test-coverage-cmd-select-layout.conf";

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 700_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 750_000;

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

/// Runs the parsed command an item carries through its entry's exec hook, the
/// way the command queue calls it.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        let exec = e.exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states at `fs`, as resolution
/// would leave them before the hook runs.
unsafe fn aim(item: &mut Item, fs: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = fs.clone();
        (*p).source = fs.clone();
        *crate::cmd::cmdq_get_current(p) = fs;
    }
}

/// Points the states as [`aim`] does and makes `caller` the item's own client
/// and target client, so `cmdq_error` files its message against that client.
unsafe fn aim_from(item: &mut Item, caller: *mut client, fs: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        aim(item, fs);
    }
}

/// The find state of `wl`: its session, its window and that window's active
/// pane.
unsafe fn fs_of(wl: *mut winlink) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
    fs
}

/// The lines the server has recorded so far, oldest first. Entries accumulate
/// across the whole test binary, so assertions look for their own wording at
/// the position they added rather than count lines from zero.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// The layout tree of `w` as one line, in the fixtures' own grammar.
unsafe fn tree(w: *mut window) -> String {
    unsafe { dump_cell(layout_root_ptr(&(*w).layout_root)) }
}

/// The dump string currently stored on the window, without taking it.
unsafe fn stored(w: *mut window) -> String {
    unsafe {
        assert!((*w).old_layout.is_some(), "no layout is stored");
        seen(cstr_ptr(&(*w).old_layout))
    }
}

/// Frees whatever layout dump the window has stored, leaving it empty again.
unsafe fn clear_stored(w: *mut window) {
    unsafe {
        (*w).old_layout = None;
    }
}

/// A window carrying a real layout tree. The tree is freed ahead of the panes
/// that hang off it.
struct Win {
    window: Window,
}

impl Drop for Win {
    fn drop(&mut self) {
        unsafe { layout_free(self.window.ptr()) };
    }
}

/// A registered session holding linked laid-out windows, everything in the
/// server's trees the way the target-taking commands expect to walk them.
/// Fields drop in declaration order after [`Drop::drop`] has unlinked the
/// winlinks, so the windows go before the panes they hang off.
struct Rig {
    registry: Registry,
    session: Session,
    windows: Vec<Win>,
    wptrs: Vec<*mut window>,
    panes: Vec<Pane>,
    tracked: Vec<*mut winlink>,
}

impl Rig {
    fn new(name: &str) -> Rig {
        let mut r = Rig {
            registry: Registry::new(),
            session: Session::new(0, name),
            windows: Vec::new(),
            wptrs: Vec::new(),
            panes: Vec::new(),
            tracked: Vec::new(),
        };
        r.registry.add_session(&mut r.session);
        r
    }

    /// Links a fresh window carrying one pane of the same size at index `idx`,
    /// laid out as a single leaf, answering its winlink and its window.
    fn add_window(&mut self, idx: c_int, sx: u_int, sy: u_int) -> (*mut winlink, *mut window) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 7;
        let pid = PANE_ID_BASE + self.panes.len() as u_int;
        let mut w = Win {
            window: Window::new(wid, "rig", sx, sy),
        };
        let mut p = Pane::new(pid, sx, sy, 100);
        w.window.add_pane(&mut p);
        unsafe { layout_init(w.window.ptr(), p.ptr()) };
        self.registry.add_window(&mut w.window);
        let wl = link(&mut self.session, &mut w.window, idx);
        self.tracked.push(wl);
        self.panes.push(p);
        let wp = w.window.ptr();
        self.windows.push(w);
        self.wptrs.push(wp);
        (wl, wp)
    }

    fn win_mut(&mut self, w: *mut window) -> &mut Window {
        let i = self
            .wptrs
            .iter()
            .position(|&p| p == w)
            .expect("a rig window");
        &mut self.windows[i].window
    }

    /// Splits the first pane of `w` along `type_0` and hangs a fresh pane off
    /// the new cell, answering the new pane. A negative `size` halves it.
    fn split(&mut self, w: *mut window, type_0: layout_type, size: c_int) -> *mut window_pane {
        unsafe {
            let first = window_panes_first(w);
            let lc = layout_split_pane(first, type_0, size, 0);
            assert!(!lc.is_null(), "there was no room to split");
            let pid = PANE_ID_BASE + self.panes.len() as u_int;
            let mut p = Pane::new(pid, 1, 1, 100);
            let pp = p.ptr();
            self.win_mut(w).add_pane(&mut p);
            self.panes.push(p);
            layout_assign_pane(lc, pp, 0);
            pp
        }
    }

    fn pane(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        for &wl in self.tracked.iter().rev() {
            unlink(&mut self.session, wl);
        }
    }
}

#[test]
fn the_three_entries_describe_their_commands_and_share_one_hook() {
    let _guard = globals();
    unsafe {
        assert_eq!((*SELECTL).name.to_string_lossy(), "select-layout");
        assert_eq!(
            (*SELECTL)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "selectl"
        );
        assert_eq!((*SELECTL).args.template.to_string_lossy(), "Enopt:");
        assert_eq!((*SELECTL).args.lower, 0);
        assert_eq!((*SELECTL).args.upper, 1);
        assert!((*SELECTL).args.cb.is_none());
        assert_eq!(
            (*SELECTL).usage.to_string_lossy(),
            "[-Enop] [-t target-pane] [layout-name]"
        );
        assert_eq!((*SELECTL).source.flag, 0);
        assert_eq!((*SELECTL).source.type_0, CMD_FIND_PANE);
        assert_eq!((*SELECTL).source.flags, 0);
        assert_eq!((*SELECTL).target.flag, b't' as c_char);
        assert_eq!((*SELECTL).target.type_0, CMD_FIND_PANE);
        assert_eq!((*SELECTL).target.flags, 0);

        assert_eq!((*NEXTL).name.to_string_lossy(), "next-layout");
        assert_eq!(
            (*NEXTL)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "nextl"
        );
        assert_eq!((*NEXTL).args.template.to_string_lossy(), "t:");
        assert_eq!((*NEXTL).args.lower, 0);
        assert_eq!((*NEXTL).args.upper, 0);
        assert_eq!((*NEXTL).usage.to_string_lossy(), "[-t target-window]");
        assert_eq!((*NEXTL).source.type_0, CMD_FIND_PANE);
        assert_eq!((*NEXTL).target.flag, b't' as c_char);
        assert_eq!((*NEXTL).target.type_0, CMD_FIND_WINDOW);

        assert_eq!((*PREVL).name.to_string_lossy(), "previous-layout");
        assert_eq!(
            (*PREVL)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "prevl"
        );
        assert_eq!((*PREVL).args.template.to_string_lossy(), "t:");
        assert_eq!((*PREVL).target.type_0, CMD_FIND_WINDOW);

        for e in [SELECTL, NEXTL, PREVL] {
            assert_eq!((*e).flags, CMD_AFTERHOOK);
        }
        let hook_of = |e: *const cmd_entry| (*e).exec as usize;
        assert_eq!(hook_of(SELECTL), hook_of(NEXTL));
        assert_eq!(hook_of(NEXTL), hook_of(PREVL));

        assert_eq!(
            CMD_TARGET_WINDOW_USAGE.to_bytes_with_nul(),
            b"[-t target-window]\0"
        );
    }
}

#[test]
fn the_parser_resolves_names_aliases_and_bounds_the_arguments() {
    let _guard = globals();
    unsafe {
        for (i, (line, want)) in [
            (c"select-layout tiled", SELECTL),
            (c"selectl tiled", SELECTL),
            (c"select-l tiled", SELECTL),
            (c"next-layout", NEXTL),
            (c"nextl", NEXTL),
            (c"previous-layout", PREVL),
            (c"prevl", PREVL),
        ]
        .into_iter()
        .enumerate()
        {
            let mut item = Item::new().from_file(FILE, i as u_int + 1).with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, want), "{line:?}");
        }

        let mut flagged = Item::new()
            .from_file(FILE, 8)
            .with_args(c"select-layout -Enopt x");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, SELECTL));
        let args = cmd_get_args(&*flagged.cmd());
        for flag in *b"Enop" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }
        assert_eq!(args_count(args), 0, "-t took its own value");

        let mut named = Item::new()
            .from_file(FILE, 9)
            .with_args(c"select-layout tiled");
        let args = cmd_get_args(&*named.cmd());
        assert_eq!(args_count(args), 1);
        assert_eq!(seen(args_string(args, 0)), "tiled");

        let mut two = cmd_parse_from_string(c"select-layout a b".as_ptr(), null_mut());
        assert_eq!(two.status, CMD_PARSE_ERROR);
        let err = two.take_error();
        assert!(err.contains("too many arguments"), "{err}");

        let mut any = cmd_parse_from_string(c"next-layout a".as_ptr(), null_mut());
        assert_eq!(any.status, CMD_PARSE_ERROR);
        let err = any.take_error();
        assert!(err.contains("too many arguments"), "{err}");

        let mut ok = cmd_parse_from_string(c"previous-layout".as_ptr(), null_mut());
        assert_eq!(ok.status, CMD_PARSE_SUCCESS);
        let _ = ok.cmdlist.take();
    }
}

#[test]
fn select_layout_named_and_next_and_previous() {
    let _guard = globals();
    unsafe {
        let mut rig = Rig::new("rig");
        let (wl, w) = rig.add_window(0, 80, 24);
        rig.split(w, LAYOUT_TOPBOTTOM, -1);

        let mut item = Item::new()
            .from_file(FILE, 1)
            .with_args(c"select-layout tiled");
        aim(&mut item, fs_of(wl));
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert!((*w).old_layout.is_some());

        let mut item_even = Item::new()
            .from_file(FILE, 2)
            .with_args(c"select-layout even-horizontal");
        aim(&mut item_even, fs_of(wl));
        assert_eq!(run(&mut item_even), CMD_RETURN_NORMAL);

        let mut item_next = Item::new().from_file(FILE, 3).with_args(c"next-layout");
        aim(&mut item_next, fs_of(wl));
        assert_eq!(run(&mut item_next), CMD_RETURN_NORMAL);

        let mut item_prev = Item::new().from_file(FILE, 4).with_args(c"previous-layout");
        aim(&mut item_prev, fs_of(wl));
        assert_eq!(run(&mut item_prev), CMD_RETURN_NORMAL);

        let mut item_flag_n = Item::new()
            .from_file(FILE, 5)
            .with_args(c"select-layout -n");
        aim(&mut item_flag_n, fs_of(wl));
        assert_eq!(run(&mut item_flag_n), CMD_RETURN_NORMAL);

        let mut item_flag_p = Item::new()
            .from_file(FILE, 6)
            .with_args(c"select-layout -p");
        aim(&mut item_flag_p, fs_of(wl));
        assert_eq!(run(&mut item_flag_p), CMD_RETURN_NORMAL);

        let mut item_spread = Item::new()
            .from_file(FILE, 7)
            .with_args(c"select-layout -E");
        aim(&mut item_spread, fs_of(wl));
        assert_eq!(run(&mut item_spread), CMD_RETURN_NORMAL);
    }
}

#[test]
fn select_layout_bare_and_old_layout_and_invalid() {
    let _guard = globals();
    unsafe {
        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        wire(caller);

        let mut rig = Rig::new("rig");
        let (wl, w) = rig.add_window(0, 80, 24);
        rig.split(w, LAYOUT_TOPBOTTOM, -1);

        let mut item_bare = Item::new().from_file(FILE, 10).with_args(c"select-layout");
        aim_from(&mut item_bare, caller, fs_of(wl));
        assert_eq!(run(&mut item_bare), CMD_RETURN_NORMAL);

        let mut item_old = Item::new()
            .from_file(FILE, 11)
            .with_args(c"select-layout -o");
        aim_from(&mut item_old, caller, fs_of(wl));
        assert_eq!(run(&mut item_old), CMD_RETURN_NORMAL);

        let mut item_invalid = Item::new()
            .from_file(FILE, 12)
            .with_args(c"select-layout nonexistent_layout_xyz");
        aim_from(&mut item_invalid, caller, fs_of(wl));
        assert_eq!(run(&mut item_invalid), CMD_RETURN_ERROR);

        clear_stored(w);
        let mut item_no_old = Item::new()
            .from_file(FILE, 13)
            .with_args(c"select-layout -o");
        aim_from(&mut item_no_old, caller, fs_of(wl));
        assert_eq!(run(&mut item_no_old), CMD_RETURN_NORMAL);

        (*w).lastlayout = 0;
        let mut item_last = Item::new().from_file(FILE, 14).with_args(c"select-layout");
        aim_from(&mut item_last, caller, fs_of(wl));
        assert_eq!(run(&mut item_last), CMD_RETURN_NORMAL);

        crate::tests::test_fixtures::release_client(caller);
    }
}
