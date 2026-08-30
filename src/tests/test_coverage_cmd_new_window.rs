//! Unit tests for [`crate::cmd::cmd_new_window`], the exec hook behind the
//! `new-window` command.
//!
//! Everything the hook decides before it would have to fork a pane process is
//! driven here through [`cmd_new_window_entry`] `.exec`, the very hook the
//! command queue calls, with an item whose target find state has already been
//! resolved: the entry's metadata and template, the `-n` validity refusal,
//! the `-S` search that selects an existing window of the same name, `-d`'s
//! hold on that selection, the refusal when two windows share the name, and
//! the spawn refusal for an explicit index that is already linked. The last
//! of these walks the whole `spawn_context` assembly — argv collection, the
//! `-e` environment copy, `-c`, `-n`, the spawn attempt itself — and its
//! failure cleanup.
//!
//! One limit worth recording. Every route to a *successful* [`spawn_window`]
//! reaches `spawn_pane`, which forks a pty child; no fixture may go there.
//! That keeps four branches out of reach: the success half of the hook (the
//! current-state re-find, redraws or status updates, the `-P` template print
//! and the `after-new-window` hook), the `-a`/`-b` shuffles whose index
//! always comes back free, a `-S` search that finds nothing and falls through
//! to the spawn, and `-k`, which with an index in use would unlink the window
//! and carry on towards the same fork.

use crate::arguments::{args_has, args_string};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_new_window::{
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    NEW_WINDOW_TEMPLATE, SPAWN_DETACHED, SPAWN_KILL, cmd_new_window_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::session::session_get_curw;
use crate::session::winlink_of;
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, seen, unlink,
    zeroed,
};
use crate::types::*;
use crate::window::window_get_latest;
use crate::window::{window_count_panes, window_panes_first, winlink_count, winlink_find_by_index};
use ::core::ffi::{c_char, c_int};

/// Where the fixture windows' ids start, far above anything `window_create`
/// hands out from its own counter, so the two never collide inside the
/// server's id-keyed window tree.
const WINDOW_ID_BASE: u_int = 900_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 950_000;

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
        let e = &raw const cmd_new_window_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states where the test wants
/// them, as the resolved find states of a prepared command queue item would
/// be.
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

/// The find state of `wl` with `idx` filled in by hand, since resolution is
/// the command queue's job and this hook reads the states as given. `-1` is
/// what "no index asked for" looks like to the hook and to `spawn_window`.
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

/// A registered session holding linked windows of one pane each, everything
/// in the server's trees the way the target-taking commands expect to walk
/// them. No layout tree is needed: nothing this suite drives gets as far as
/// arranging one. The winlinks are unlinked again on the way out, which is
/// safe because nothing here ever spawns a window for real.
struct Chain {
    registry: Registry,
    session: Session,
    windows: Vec<Window>,
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

    /// Links a fresh window named `name`, holding one pane, at index `idx`,
    /// answering its winlink. The first window linked becomes the session's
    /// current one.
    fn add_window(&mut self, idx: c_int, name: &str, sx: u_int, sy: u_int) -> *mut winlink {
        let id = WINDOW_ID_BASE + self.windows.len() as u_int;
        let mut w = Window::new(id, name, sx, sy);
        let mut p = Pane::new(PANE_ID_BASE + self.panes.len() as u_int + 1, sx, sy, 100);
        w.add_pane(&mut p);
        self.registry.add_window(&mut w);
        let wl = link(&mut self.session, &mut w, idx);
        self.tracked.push(wl);
        self.windows.push(w);
        self.panes.push(p);
        wl
    }

    fn sptr(&mut self) -> *mut session {
        self.session.ptr()
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        for wl in ::std::mem::take(&mut self.tracked).into_iter().rev() {
            unlink(&mut self.session, wl);
        }
    }
}

#[test]
fn the_entry_advertises_the_command_its_flags_and_the_constants_it_runs_with() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_new_window_entry;
        assert_eq!((*e).name.to_string_lossy(), "new-window");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "neww"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "abc:de:F:kn:PSt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-abdkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command [argument ...]]"
        );
        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*e).target.flags, CMD_FIND_WINDOW_INDEX);
        assert_eq!((*e).flags, 0);

        assert_eq!(CMD_FIND_WINDOW_INDEX, 0x4);
        assert_eq!(SPAWN_KILL, 0x1);
        assert_eq!(SPAWN_DETACHED, 0x2);
        assert_eq!(CMD_RETURN_NORMAL, 0);
        assert_eq!(CMD_RETURN_ERROR, -1);
    }
}

#[test]
fn the_default_template_is_the_upstream_one() {
    let expected: &[u8] = b"#{session_name}:#{window_index}.#{pane_index}\0";
    let got: Vec<u8> = NEW_WINDOW_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(NEW_WINDOW_TEMPLATE.len(), expected.len());
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
    let wl0 = chain.add_window(0, "keep", 80, 24);
    let w0 = unsafe { (*wl0).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"new-window -n a\\200b");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl0, -1));

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

        let s = chain.sptr();
        assert_eq!(winlink_count(&raw mut (*s).windows), 1);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 0), wl0);
        assert_eq!((*wl0).window(), w0);
        assert_eq!(window_count_panes(w0, 1), 1);
        assert_eq!(session_get_curw(s), wl0, "the refusal selected nothing");
    }
}

#[test]
fn searching_for_an_existing_name_selects_that_window_without_spawning() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let wl0 = chain.add_window(0, "keep", 80, 24);
    let wl1 = chain.add_window(1, "dupe", 80, 24);
    let w1 = unsafe { (*wl1).window() };

    let mut item = Item::with_client().with_args(c"neww -S -n dupe");
    unsafe {
        assert_eq!(
            cmd_get_entry(&*item.cmd()).name.to_string_lossy(),
            "new-window",
            "the alias spelling resolves to this entry"
        );
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'S'), 1);

        let s = chain.sptr();
        (*item.client()).session = s;
        aim(&mut item, fs_of(wl0, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            session_get_curw(s),
            wl1,
            "the same-named window was selected"
        );
        assert_eq!(
            winlink_of(s, (*s).lastw.first().copied()),
            wl0,
            "the old selection was stacked"
        );
        assert_eq!(
            winlink_count(&raw mut (*s).windows),
            2,
            "nothing was spawned"
        );
        assert!(
            winlink_find_by_index(&raw mut (*s).windows, 2).is_null(),
            "no third window appeared"
        );
        assert_eq!(
            window_get_latest(w1),
            item.client(),
            "the item's client became the new window's latest"
        );
        assert_ne!(
            (*w1).activity_time.tv_sec,
            0,
            "its activity time was refreshed"
        );
        assert_eq!(
            (*s).statuslines,
            1,
            "recalculate_sizes refreshed the status cache"
        );
    }
}

#[test]
fn with_d_the_search_returns_without_touching_the_selection() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let wl0 = chain.add_window(0, "keep", 80, 24);
    let wl1 = chain.add_window(1, "other", 80, 24);
    let w1 = unsafe { (*wl1).window() };

    let mut item = Item::with_client().with_args(c"new-window -S -d -n other");
    unsafe {
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'd'), 1);
        let s = chain.sptr();
        aim(&mut item, fs_of(wl0, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(server_messages().len(), before, "nothing was reported");
        assert_eq!(
            session_get_curw(s),
            wl0,
            "-d leaves the current window selected"
        );
        assert!((*s).lastw.is_empty(), "nothing was pushed onto the stack");
        assert_eq!(
            winlink_count(&raw mut (*s).windows),
            2,
            "nothing was spawned"
        );
        assert!(
            window_get_latest(w1).is_null(),
            "-d never got as far as the latest update"
        );
        assert_eq!((*s).statuslines, 0, "recalculate_sizes never ran either");
    }
}

#[test]
fn two_windows_sharing_the_name_refuse_the_search() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let wl0 = chain.add_window(0, "dupe", 80, 24);
    chain.add_window(1, "dupe", 80, 24);
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"new-window -S -n dupe");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl0, -1));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("multiple windows named dupe"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        let s = chain.sptr();
        assert_eq!(
            winlink_count(&raw mut (*s).windows),
            2,
            "nothing was spawned"
        );
        assert_eq!(session_get_curw(s), wl0, "the refusal selected nothing");
    }
}

#[test]
fn an_explicit_index_in_use_fails_the_spawn_and_cleans_up() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let wl0 = chain.add_window(0, "keep", 80, 24);
    let w0 = unsafe { (*wl0).window() };
    let pane0 = unsafe { window_panes_first(w0) };
    unsafe { wire(caller) };

    let mut item =
        Item::with_client().with_args(c"new-window -n mined -e FOO=bar -c / somescript arg1");
    unsafe {
        assert_eq!(
            seen(args_string(cmd_get_args(&*item.cmd()), 0)),
            "somescript",
            "the shell-command words were parsed"
        );
        aim_from(&mut item, caller, fs_of(wl0, 0));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("create window failed: index 0 in use"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        let s = chain.sptr();
        assert_eq!(winlink_count(&raw mut (*s).windows), 1);
        assert_eq!(winlink_find_by_index(&raw mut (*s).windows, 0), wl0);
        assert_eq!(session_get_curw(s), wl0);
        assert_eq!((*wl0).window(), w0);
        assert_eq!(window_count_panes(w0, 1), 1);
        assert_eq!(window_panes_first(w0), pane0, "the pane was left alone");
        assert_eq!(seen(cstr_ptr(&(*w0).name)), "keep", "no rename happened");
    }
}
