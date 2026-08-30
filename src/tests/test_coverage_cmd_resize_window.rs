//! Unit tests for [`crate::cmd::cmd_resize_window`] — the `resize-window`
//! entry (name, alias, argument template, usage, find flags and exec hook),
//! the block of message-protocol, style and command constants the file
//! declares, and the deterministic behaviour of [`cmd_resize_window_exec`] as
//! reached through the entry's own function pointer over items built by the
//! real command parser.
//!
//! Exec's job is one pass over the arguments: take the adjustment from
//! argument 0 (defaulting to one) and refuse one `strtonum` cannot parse,
//! read explicit sizes from `-x` and `-y` and refuse out-of-range ones with
//! the cause `args_strtonum` hands back, move the width or height by the
//! adjustment for `-L`/`-R`/`-U`/`-D` without ever stepping below zero, let
//! `-A`/`-a` ask `default_window_size` for the largest or smallest size the
//! clients settle on, then pin the answer as the window's manual sizes, flip
//! its `window-size` option to manual and recalculate at once. Each step is
//! pinned here by behaviour: a run with nothing asked leaves the size alone
//! but pins it manual; `-R` grows by a parsed adjustment while `-L` both
//! shrinks and clamps; `-D` and `-U` do the same vertically, `-U` never
//! going above the top edge; `-x`/`-y` set either dimension outright;
//! `-A` and `-a`, with no client able to help, fall back on the session's
//! `default-size` option. The refusals — an unparsable adjustment and an
//! out-of-range width or height — leave the window exactly as it was while
//! the wording lands in the server's message log: the item carries a client
//! whose peer is marked bad, so `cmdq_error` files each message there
//! without ever reaching a descriptor.
//!
//! Traces left behind on purpose, like the other suites: every successful run
//! ends in `recalculate_size`, which raises `window-layout-changed` and
//! `window-resized` notifications that sit on the global command queue
//! nothing ever drains, and each refusal files a message-log line plus a
//! stream file against the fixture client. Everything else these tests touch
//! is taken and given back under [`globals`].

use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_resize_window::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, cmd_resize_window_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::layout::{layout_free, layout_init, layout_root_ptr};
use crate::options::{options_get_number, options_ptr};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, globals, link, seen, unlink, zeroed,
};
use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

/// The entry under test.
const ENTRY: *const cmd_entry = &raw const cmd_resize_window_entry;

/// Where the tests' items claim to come from.
const FILE: &CStr = c"test-coverage-cmd-resize-window.conf";

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 800_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 950_000;

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

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, ENTRY),
            "the item is not running resize-window"
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
        *crate::cmd::cmdq_get_current(p) = target;
    }
}

/// Points the states as [`aim`] does and makes `caller` the item's own client
/// and target client, so `cmdq_error` files its message against that client.
unsafe fn aim_from(item: &mut Item, caller: *mut client, target: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        aim(item, target);
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

/// The window's own size.
unsafe fn window_size(w: *mut window) -> (u_int, u_int) {
    unsafe { ((*w).sx, (*w).sy) }
}

/// The size of the window's layout tree.
unsafe fn layout_size(w: *mut window) -> (u_int, u_int) {
    unsafe {
        (
            (*layout_root_ptr(&(*w).layout_root)).sx,
            (*layout_root_ptr(&(*w).layout_root)).sy,
        )
    }
}

/// The manual sizes pinned on the window.
unsafe fn manual_size(w: *mut window) -> (u_int, u_int) {
    unsafe { ((*w).manual_sx, (*w).manual_sy) }
}

/// The window's `window-size` option.
unsafe fn window_size_option(w: *mut window) -> i64 {
    unsafe { options_get_number(options_ptr(&(*w).options), c"window-size".as_ptr()) }
}

/// A window whose single pane hangs off a real layout tree, because the hook
/// ends in `recalculate_size`, which reshapes the tree when it applies the
/// new size. The tree is freed ahead of the pane it points at.
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
/// Fields are dropped in declaration order, so the windows go before the
/// panes they hang off.
struct Chain {
    registry: Registry,
    session: Session,
    windows: Vec<Win>,
    panes: Vec<Pane>,
    tracked: Vec<*mut winlink>,
}

impl Chain {
    fn new(name: &str) -> Chain {
        let mut c = Chain {
            registry: Registry::new(),
            session: Session::new(0, name),
            windows: Vec::new(),
            panes: Vec::new(),
            tracked: Vec::new(),
        };
        c.registry.add_session(&mut c.session);
        c
    }

    /// Links a fresh window carrying one pane of the same size at index
    /// `idx`, laid out as a single leaf, answering its winlink, its window
    /// and that pane.
    fn add_window(
        &mut self,
        idx: c_int,
        sx: u_int,
        sy: u_int,
    ) -> (*mut winlink, *mut window, *mut window_pane) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 7;
        let pid = PANE_ID_BASE + self.panes.len() as u_int;
        let mut w = Win {
            window: Window::new(wid, "chain", sx, sy),
        };
        let mut p = Pane::new(pid, sx, sy, 100);
        w.window.add_pane(&mut p);
        self.panes.push(p);
        unsafe { layout_init(w.window.ptr(), self.panes.last_mut().expect("a pane").ptr()) };
        self.registry.add_window(&mut w.window);
        let wl = link(&mut self.session, &mut w.window, idx);
        self.tracked.push(wl);
        let wp = self.panes.last_mut().expect("a pane").ptr();
        let wptr = w.window.ptr();
        self.windows.push(w);
        (wl, wptr, wp)
    }

    fn sptr(&mut self) -> *mut session {
        self.session.ptr()
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        for &wl in self.tracked.iter().rev() {
            unlink(&mut self.session, wl);
        }
    }
}

#[test]
fn the_entry_describes_the_resize_window_command() {
    let _guard = globals();
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "resize-window");
        assert_eq!(
            (*ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "resizew"
        );
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "aADLRt:Ux:y:");
        assert_eq!((*ENTRY).args.lower, 0);
        assert_eq!((*ENTRY).args.upper, 1);
        assert!(
            (*ENTRY).args.cb.is_none(),
            "resize-window takes no args callback"
        );
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-aADLRU] [-x width] [-y height] [-t target-window] [adjustment]"
        );

        assert_eq!((*ENTRY).source.flag, 0);
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).source.flags, 0);
        assert_eq!((*ENTRY).target.flag, b't' as c_char);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*ENTRY).target.flags, 0);

        assert_eq!((*ENTRY).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_parser_resolves_the_name_the_alias_and_a_prefix() {
    let _guard = globals();
    unsafe {
        for (i, line) in [c"resize-window -R", c"resizew -R", c"resize-w -R"]
            .into_iter()
            .enumerate()
        {
            let mut item = Item::new().from_file(FILE, i as u_int + 1).with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, ENTRY), "{line:?}");
        }

        let mut flagged = Item::new()
            .from_file(FILE, 9)
            .with_args(c"resize-window -A -a -D -L -R -U -x 11 -y 22");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, ENTRY));
        let args = cmd_get_args(&*flagged.cmd());
        for flag in *b"AaDLRUxy" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }
        assert_eq!(args_count(args), 0, "every flag took its own argument");

        let mut adjusted = Item::new()
            .from_file(FILE, 10)
            .with_args(c"resize-window -R 5");
        assert!(::core::ptr::eq((*adjusted.cmd()).entry, ENTRY));
        let args = cmd_get_args(&*adjusted.cmd());
        assert_eq!(args_count(args), 1);
        assert_eq!(seen(args_string(args, 0)), "5");
    }
}

#[test]
fn the_template_bounds_allow_at_most_one_adjustment() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"resize-window".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut one = cmd_parse_from_string(c"resize-window 3".as_ptr(), null_mut());
        assert_eq!(one.status, CMD_PARSE_SUCCESS);
        let _ = one.cmdlist.take();

        let mut two = cmd_parse_from_string(c"resize-window 3 4".as_ptr(), null_mut());
        assert_eq!(two.status, CMD_PARSE_ERROR);
        let err = two.take_error();
        assert!(err.contains("resize-window"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");
    }
}
