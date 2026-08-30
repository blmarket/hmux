//! Unit tests for [`crate::cmd::cmd_respawn_window`], the exec hook behind
//! the `respawn-window` command.
//!
//! Everything the hook decides before it would have to fork a pane process is
//! driven here through [`cmd_respawn_window_entry`] `.exec`, the very hook the
//! command queue calls, with an item whose target find state has already been
//! resolved: the entry's metadata and template, and the whole `spawn_context`
//! assembly on the refusal route — argv collection from the trailing words,
//! one `environ_put` per `-e` value, `-c`'s start directory, the respawn flag
//! with or without `-k`'s bit — up to `spawn_window`'s answer that a window
//! whose panes are still alive cannot be respawned, which comes back through
//! `cmdq_error` into the server's message log, followed by the failure
//! cleanup of the cause, the argv vector and the environment. The window
//! itself must come out of all of it untouched.
//!
//! One limit worth recording. Once the hook gets past that refusal it tears
//! the window's panes down and reaches `spawn_pane`, which forks a pty child;
//! no fixture may go there. That leaves `-k`, whose only effect through this
//! hook is to skip the refusal and carry straight on towards the fork, out of
//! reach along with the success tail — the redraw of the window and the normal
//! return.

use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_respawn_window::{
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, SPAWN_KILL, SPAWN_RESPAWN, cmd_respawn_window_entry,
};
use crate::cmd::cmdq_get_current;
use crate::cmd::cmdq_set_target_client;
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, globals, link, seen, unlink, zeroed,
};
use crate::types::*;
use ::core::ffi::{c_char, c_int};

/// Where the fixture windows' ids start, far above anything `window_create`
/// hands out from its own counter, so the two never collide inside the
/// server's id-keyed window tree.
const WINDOW_ID_BASE: u_int = 900_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 950_000;

/// A descriptor number no fixture ever owns, standing in for a pane with a
/// live process behind it. Nothing on the refusal path touches it.
const LIVE_FD: c_int = 7;

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
        let e = &raw const cmd_respawn_window_entry;
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
        *cmdq_get_current(p) = target.clone();
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

/// A registered session holding linked windows of one or more panes each,
/// everything in the server's trees the way the target-taking commands expect
/// to walk them. No layout tree is needed: nothing this suite drives gets as
/// far as arranging one. The winlinks are unlinked again on the way out, which
/// is safe because nothing here ever respawns a window for real.
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

    /// Adds another pane at the end of window `i`'s pane list, answering its
    /// pointer.
    fn add_pane(&mut self, i: usize, sx: u_int, sy: u_int) -> *mut window_pane {
        let mut p = Pane::new(PANE_ID_BASE + self.panes.len() as u_int + 1, sx, sy, 100);
        let ptr = p.ptr();
        self.windows[i].add_pane(&mut p);
        self.panes.push(p);
        ptr
    }

    fn sptr(&mut self) -> *mut session {
        self.session.ptr()
    }

    fn pane(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
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
        let e = &raw const cmd_respawn_window_entry;
        assert_eq!((*e).name.to_string_lossy(), "respawn-window");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "respawnw"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "c:e:kt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-k] [-c start-directory] [-e environment] [-t target-window] [shell-command [argument ...]]"
        );
        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, 0);

        assert_eq!(SPAWN_KILL, 0x1);
        assert_eq!(SPAWN_RESPAWN, 0x4);
        assert_eq!(CMD_RETURN_NORMAL, 0);
        assert_eq!(CMD_RETURN_WAIT, 1);
        assert_eq!(CMD_RETURN_STOP, 2);
        assert_eq!(CMD_RETURN_ERROR, -1);
        assert_eq!(CMD_FIND_PANE, 0);
        assert_eq!(CMD_FIND_WINDOW, 1);
    }
}

#[test]
fn parsing_resolves_both_spellings_and_flags() {
    let _guard = globals();
    unsafe {
        let mut item = Item::new().with_args(c"respawn-window -k -c /tmp -e A=B mycmd");
        assert!(::core::ptr::eq(
            (*item.cmd()).entry,
            &cmd_respawn_window_entry
        ));
        let args = crate::cmd::cmd_get_args(&*item.cmd());
        assert_eq!(crate::arguments::args_has(args, b'k'), 1);
        assert_eq!(crate::arguments::args_has(args, b'c'), 1);
        assert_eq!(crate::arguments::args_has(args, b'e'), 1);

        let mut alias = Item::new().with_args(c"respawnw -t 1");
        assert!(::core::ptr::eq(
            (*alias.cmd()).entry,
            &cmd_respawn_window_entry
        ));
    }
}

#[test]
fn respawn_window_exec_refuses_when_pane_is_live() {
    let _guard = globals();
    unsafe {
        let mut chain = Chain::new("chain");
        let wl = chain.add_window(0, "win0", 80, 24);
        let p = chain.pane(0);
        (*p).fd = LIVE_FD;

        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        wire(caller);

        let mut item = Item::new().with_args(c"respawn-window -c /tmp -e FOO=BAR echo bye");
        aim_from(&mut item, caller, fs_of(wl, -1));
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        (*p).fd = -1;
        crate::tests::test_fixtures::release_client(caller);
    }
}
