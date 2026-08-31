//! Unit tests for [`crate::cmd::cmd_rotate_window`] — the `rotate-window`
//! entry (name, alias, argument template, usage, find flags and exec hook),
//! and the block of message-protocol, pane-line, progress-bar, cursor, style,
//! theme, layout, prompt, exit, parse, find and return constants the file
//! declares.
//!
//! The exec hook is reached through the entry's own function pointer over an
//! item whose target state has been resolved by hand, exactly as the command
//! queue would leave it. The runs here are deliberately the conservative ones:
//! a window carrying a single pane drives both rotation branches end to end —
//! the unlink/relink of the pane list, the layout-cell hand-back, the active
//! pane choice on either side of the `-D` split, the re-found current state
//! and the redraw sweep over the client list — while every side effect stays
//! a no-op by construction. With one pane [`window_pane_resize`] is never
//! reached with a size change, so no resize entry is queued; the rotation
//! lands the same pane back as active, so [`window_set_active_pane`] returns
//! before raising `window-pane-changed`, leaving nothing on the global
//! command queue nothing drains; and the zoom push/pop bracket an unzoomed
//! window, so no layout is rebuilt. A multi-pane run would change the active
//! pane and queue that notification, so it is not driven here.

use crate::arguments::{args_count, args_has};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_rotate_window::{
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_NORMAL, cmd_rotate_window_entry,
};
use crate::cmd::cmdq_get_current;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::layout::layout_cell_pane;
use crate::layout::{layout_free, layout_init};
use crate::server::CLIENT_ALLREDRAWFLAGS;
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, unlink,
};
use crate::types::*;
use crate::window::window_get_active;
use crate::window::{
    WINDOW_WASZOOMED, WINDOW_ZOOMED, window_count_panes, window_panes_first, window_panes_last,
};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// The entry under test.
const ENTRY: *const cmd_entry = &raw const cmd_rotate_window_entry;

/// Where the tests' items claim to come from.
const FILE: &CStr = c"test-coverage-cmd-rotate-window.conf";

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 860_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 970_000;

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, ENTRY),
            "the item is not running rotate-window"
        );
        let exec = (*ENTRY).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target state where the test wants it, as resolution would
/// leave it before the hook runs. The source and current states are filled in
/// too, although this hook reads only the target.
unsafe fn aim(item: &mut Item, target: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = target.clone();
        (*p).source = target.clone();
        *cmdq_get_current(p) = target.clone();
    }
}

/// The find state of `wl`: its session, its window and that window's active
/// pane.
unsafe fn fs_of(wl: *mut winlink) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
    fs
}

/// A registered session holding one linked window with one pane laid out as a
/// single leaf, everything in the server's trees the way target-taking
/// commands expect to walk them. Fields are dropped in declaration order: the
/// teardown unlinks the winlink first, then the registry empties the trees,
/// then the window's layout tree is freed ahead of the pane it points at.
struct Rig {
    registry: Registry,
    session: Session,
    window: Win,
    pane: Pane,
    wl: *mut winlink,
}

/// A window owning a real layout tree, freed ahead of its panes.
struct Win {
    window: Window,
}

impl Drop for Win {
    fn drop(&mut self) {
        unsafe { layout_free(self.window.ptr()) };
    }
}

impl Rig {
    /// An 80x24 window at index 0 whose single pane fills it.
    fn new() -> Rig {
        let mut rig = Rig {
            registry: Registry::new(),
            session: Session::new(0, "rot"),
            window: Win {
                window: Window::new(WINDOW_ID_BASE, "rot", 80, 24),
            },
            pane: Pane::new(PANE_ID_BASE, 80, 24, 100),
            wl: null_mut::<winlink>(),
        };
        rig.registry.add_session(&mut rig.session);
        rig.window.window.add_pane(&mut rig.pane);
        unsafe { layout_init(rig.w(), rig.pane.ptr()) };
        rig.registry.add_window(&mut rig.window.window);
        rig.wl = link(&mut rig.session, &mut rig.window.window, 0);
        rig
    }

    fn w(&mut self) -> *mut window {
        self.window.window.ptr()
    }

    fn s(&mut self) -> *mut session {
        self.session.ptr()
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        unlink(&mut self.session, self.wl);
    }
}

#[test]
fn the_entry_describes_the_rotate_window_command() {
    let _guard = globals();
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "rotate-window");
        assert_eq!(
            (*ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "rotatew"
        );
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "Dt:UZ");
        assert_eq!((*ENTRY).args.lower, 0);
        assert_eq!((*ENTRY).args.upper, 0);
        assert!(
            (*ENTRY).args.cb.is_none(),
            "rotate-window takes no args callback"
        );
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-DUZ] [-t target-window]"
        );

        assert_eq!((*ENTRY).source.flag, 0);
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).source.flags, 0);
        assert_eq!((*ENTRY).target.flag, b't' as c_char);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*ENTRY).target.flags, 0);

        assert_eq!((*ENTRY).flags, 0);
    }
}

#[test]
fn the_parser_resolves_the_name_the_alias_and_a_prefix() {
    let _guard = globals();
    unsafe {
        for (i, line) in [c"rotate-window -D", c"rotatew -D", c"rotate-w -D"]
            .into_iter()
            .enumerate()
        {
            let mut item = Item::new().from_file(FILE, i as u_int + 1).with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, ENTRY), "{line:?}");
        }

        let mut flagged = Item::new()
            .from_file(FILE, 9)
            .with_args(c"rotate-window -D -U -Z -t @3");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, ENTRY));
        let args = cmd_get_args(&*flagged.cmd());
        for flag in *b"DUZt" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }
        assert_eq!(args_count(args), 0, "-t took its argument");
    }
}

#[test]
fn the_template_bounds_reject_a_free_argument() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"rotate-window".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"rotate-window surplus".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
    }
}

#[test]
fn rotating_down_leaves_a_one_pane_window_whole_and_redraws_its_clients() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut rig = Rig::new();

    unsafe {
        (*caller).session = rig.s();

        let mut item = Item::new().with_args(c"rotate-window -D");
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'D'), 1);
        aim(&mut item, fs_of(rig.wl));

        let wp = rig.pane.ptr();
        let w = rig.w();
        let root = (*w).layout_root_ptr();
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w, 1), 1);
        assert_eq!(window_panes_first(w), wp);
        assert_eq!(window_panes_last(w), wp, "the list came back well formed");
        assert_eq!(window_get_active(w), wp, "the only pane stays active");

        assert_eq!((*w).layout_root_ptr(), root);
        assert_eq!(
            layout_cell_pane(w, (*wp).layout_cell),
            wp,
            "the cell points back at its pane"
        );

        assert_eq!(
            (*w).flags & (WINDOW_ZOOMED | WINDOW_WASZOOMED),
            0,
            "the zoom bracket left nothing behind"
        );
        assert!((*wp).resize_queue.is_empty(), "no resize was queued");

        let cur = cmdq_get_current(item.ptr());
        assert_eq!((*cur).session(), rig.s());
        assert_eq!((*cur).winlink(), rig.wl);
        assert_eq!((*cur).window(), w);
        assert_eq!((*cur).pane(), wp);

        assert_eq!((*caller).flags, CLIENT_ALLREDRAWFLAGS);
    }
}

#[test]
fn rotating_up_without_d_does_the_same_over_an_empty_client_list() {
    let _guard = globals();
    ensure_reactor();
    let mut rig = Rig::new();

    unsafe {
        assert!(
            crate::server::clients.is_empty(),
            "no client is watching this run"
        );

        let mut item = Item::new().with_args(c"rotate-window -Z");
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'Z'), 1);
        aim(&mut item, fs_of(rig.wl));

        let wp = rig.pane.ptr();
        let w = rig.w();
        let root = (*w).layout_root_ptr();
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w, 1), 1);
        assert_eq!(window_panes_first(w), wp);
        assert_eq!(window_panes_last(w), wp);
        assert_eq!(window_get_active(w), wp);
        assert_eq!((*w).layout_root_ptr(), root);
        assert_eq!(layout_cell_pane(w, (*wp).layout_cell), wp);
        assert_eq!((*w).flags & (WINDOW_ZOOMED | WINDOW_WASZOOMED), 0);
        assert!((*wp).resize_queue.is_empty());

        let cur = cmdq_get_current(item.ptr());
        assert_eq!((*cur).session(), rig.s());
        assert_eq!((*cur).winlink(), rig.wl);
        assert_eq!((*cur).window(), w);
        assert_eq!((*cur).pane(), wp);
    }
}
