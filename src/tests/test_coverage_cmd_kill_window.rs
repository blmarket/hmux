//! Unit tests for [`crate::cmd::cmd_kill_window`], the one exec hook behind
//! both the `kill-window` and `unlink-window` commands.
//!
//! The hook picks its behaviour by comparing the running command's entry
//! against the `unlink-window` entry, so every branch of it is reachable by
//! calling [`cmd_kill_window_entry`] or [`cmd_unlink_window_entry`] through
//! their `.exec` pointers with an item whose target find state has already
//! been resolved. Around that hook the tests pin both entries' metadata
//! (including that they share one function), the parsing of their names,
//! aliases and flags, the unlink refusal that fires when nothing beyond its
//! own session holds the window, `-k`'s unlink of a window the session gives
//! up anyway, an unlink that passes its guard because a second session still
//! holds the window, `-a`'s early answer on a session whose only window is
//! the target, `-a`'s letting go of every other window while keeping the
//! target's own, and the plain kill that detaches the target everywhere.
//!
//! Three safety rules shape what these tests drive. Every kill leaves its
//! session another window, since a session losing its last one hands over to
//! `server_destroy_session_group`, which would free memory the fixtures own.
//! Fixture windows retain their [`Window`] owner while winlinks are detached,
//! so the command can be checked without leaving a raw pointer dangling.
//! Production windows are owned by their winlinks and deferred alert or
//! notification entries. Winlinks the command frees itself are told apart with
//! [`Chain::forget`], because unlinking one the command already took back out
//! of its session would walk freed memory.

use crate::arguments::args_has;
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_kill_window::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, MSG_COMMAND, MSG_READ_OPEN, MSG_SHUTDOWN, MSG_VERSION,
    RB_NEGINF, cmd_kill_window_entry, cmd_unlink_window_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::session::session_get_curw;
use crate::session::{session_is_linked, session_select};
use crate::tests::test_fixtures::{
    Args, Clients, Item, Registry, Session, Window, ensure_reactor, globals, link, seen, unlink,
    zeroed,
};
use crate::types::*;
use crate::window::{winlink_count, winlink_find_by_index};
use ::core::ffi::{c_char, c_int};

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 700_000;

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

/// Runs the item's parsed command through the kill-window entry's exec hook,
/// the way the command queue would.
unsafe fn run_kill(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_kill_window_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Runs the item's parsed command through the unlink-window entry's exec
/// hook, which is the same function under a second name reached by way of the
/// other entry.
unsafe fn run_unlink(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_unlink_window_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states where the test wants
/// them, as the resolved find states of a prepared command queue item would
/// be. `caller`, when given, becomes the item's own client and its target
/// client, since [`Item::with_client`] carries only an anonymous one of its
/// own.
unsafe fn aim(item: &mut Item, target: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = target.clone();
        (*p).source = target.clone();
        *crate::cmd::cmdq_get_current(p) = target;
    }
}

unsafe fn aim_from(item: &mut Item, caller: *mut client, target: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        aim(item, target);
    }
}

/// The find state of `wl`, since resolution is the command queue's job and
/// this hook reads the states as given.
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

/// Registered sessions holding linked windows, everything in the server's
/// trees the way the target-taking commands expect to walk them. Winlinks the
/// chain itself made are unlinked again on the way out; winlinks the command
/// frees itself are told apart with [`Chain::forget`].
struct Chain {
    registry: Registry,
    sessions: Vec<Session>,
    windows: Vec<Window>,
    tracked: Vec<(usize, *mut winlink)>,
}

impl Chain {
    fn new(name: &str) -> Chain {
        let mut c = Chain {
            registry: Registry::new(),
            sessions: Vec::new(),
            windows: Vec::new(),
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

    /// Links a fresh window at index `idx` behind session `sidx`, answering
    /// its winlink.
    fn add_window(&mut self, sidx: usize, idx: c_int) -> *mut winlink {
        let mut w = Window::new(
            WINDOW_ID_BASE + self.windows.len() as u_int * 13 + sidx as u_int,
            "chain",
            80,
            24,
        );
        self.registry.add_window(&mut w);
        let wl = link(&mut self.sessions[sidx], &mut w, idx);
        self.tracked.push((sidx, wl));
        self.windows.push(w);
        wl
    }

    /// Links an existing window of the chain into session `sidx` as well, as
    /// `link-window` would share it across sessions.
    fn share_window(&mut self, sidx: usize, widx: usize, idx: c_int) -> *mut winlink {
        let wl = link(&mut self.sessions[sidx], &mut self.windows[widx], idx);
        self.tracked.push((sidx, wl));
        wl
    }

    fn sptr(&mut self, i: usize) -> *mut session {
        self.sessions[i].ptr()
    }

    /// Drops a winlink from the cleanup list, for ones the command itself has
    /// taken back out of its session and freed.
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
fn the_entries_advertise_two_commands_sharing_one_hook() {
    unsafe {
        let k = &raw const cmd_kill_window_entry;
        let u = &raw const cmd_unlink_window_entry;
        assert_ne!(k, u);

        assert_eq!((*k).name.to_string_lossy(), "kill-window");
        assert_eq!(
            (*k).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "killw"
        );
        assert_eq!((*u).name.to_string_lossy(), "unlink-window");
        assert_eq!(
            (*u).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "unlinkw"
        );

        for e in [k, u] {
            assert_eq!((*e).args.lower, 0);
            assert_eq!((*e).args.upper, 0);
            assert!((*e).args.cb.is_none());

            assert_eq!((*e).source.flag, 0);
            assert_eq!((*e).source.type_0, CMD_FIND_PANE);
            assert_eq!((*e).source.flags, 0);

            assert_eq!((*e).target.flag, b't' as c_char);
            assert_eq!((*e).target.type_0, CMD_FIND_WINDOW);
            assert_eq!((*e).target.flags, 0);

            assert_eq!((*e).flags, 0);
        }

        assert_eq!((*k).args.template.to_string_lossy(), "at:");
        assert_eq!((*k).usage.to_string_lossy(), "[-a] [-t target-window]");

        assert_eq!((*u).args.template.to_string_lossy(), "kt:");
        assert_eq!((*u).usage.to_string_lossy(), "[-k] [-t target-window]");

        assert!(
            ::core::ptr::fn_addr_eq((*k).exec, (*u).exec),
            "both entries dispatch one hook"
        );
    }
}

#[test]
fn the_constants_pin_the_values_the_exec_and_entries_read_back() {
    assert_eq!(ARGS_PARSE_INVALID, 0);
    assert_eq!(ARGS_PARSE_STRING, 1);
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(ARGS_PARSE_COMMANDS, 3);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);

    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);

    assert_eq!(
        RB_NEGINF, -1,
        "the walk over a session's windows starts here"
    );

    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_SHUTDOWN, 210);
    assert_eq!(MSG_READ_OPEN, 300);
}

#[test]
fn parsing_resolves_both_names_and_their_aliases_to_these_entries() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        let k = &raw const cmd_kill_window_entry;
        let u = &raw const cmd_unlink_window_entry;

        for (line, e) in [
            (c"kill-window", k),
            (c"killw", k),
            (c"unlink-window", u),
            (c"unlinkw", u),
        ] {
            let args = Args::parse(line);
            assert!(
                ::core::ptr::eq((*args.cmd()).entry, e),
                "{line:?} went to the wrong entry"
            );
        }

        let mut item = Item::new().with_args(c"kill-window -a");
        assert!(::core::ptr::eq(cmd_get_entry(&*item.cmd()), k));
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'a'), 1);

        let mut item = Item::new().with_args(c"unlink-window -k");
        assert!(::core::ptr::eq(cmd_get_entry(&*item.cmd()), u));
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'k'), 1);
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'a'), 0);
    }
}

#[test]
fn unlinking_a_solely_linked_window_without_k_is_refused() {
    let _guard = globals();
    ensure_reactor();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let mut chain = Chain::new("0");
    let wl = chain.add_window(0, 0);
    let w = unsafe { (*wl).window() };
    unsafe { wire(caller) };

    let mut item = Item::with_client().with_args(c"unlink-window");
    unsafe {
        aim_from(&mut item, caller, fs_of(wl));

        let before = server_messages().len();

        assert_eq!(session_is_linked(chain.sptr(0), w), 0);
        let rv = run_unlink(&mut item);

        assert_eq!(rv, CMD_RETURN_ERROR);

        let msgs = server_messages();
        assert_eq!(msgs.len(), before + 1, "{msgs:?}");
        assert!(
            msgs[before].contains("window only linked to one session"),
            "{}",
            msgs[before]
        );
        assert_eq!((*caller).retval, 1);

        assert_eq!(winlink_count(&(*chain.sptr(0)).windows), 1);
        assert_eq!(winlink_find_by_index(&mut (*chain.sptr(0)).windows, 0), wl);
        assert_eq!(session_get_curw(chain.sptr(0)), wl, "nothing was detached");
    }
}

#[test]
fn with_k_the_session_gives_up_a_window_it_wants_gone() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let wl_t = chain.add_window(0, 0);
    let wl_keep = chain.add_window(0, 1);
    let w_t = unsafe { (*wl_t).window() };

    unsafe {
        assert_eq!(session_select(chain.sptr(0), 1), 0, "moved off the target");

        let mut item = Item::new().with_args(c"unlink-window -k");
        aim(&mut item, fs_of(wl_t));

        assert_eq!(run_unlink(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wl_t);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&(*s).windows), 1);
        assert!(winlink_find_by_index(&mut (*s).windows, 0).is_null());
        assert_eq!(winlink_find_by_index(&mut (*s).windows, 1), wl_keep);
        assert_eq!(session_get_curw(s), wl_keep, "the selection was left alone");

        assert!((*w_t).winlinks.is_empty(), "no session lists it any more");
        assert!(!(*wl_keep).window().is_null());
    }
}

#[test]
fn a_window_another_session_still_holds_unlinks_without_k() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("home");
    let wl_shared = chain.add_window(0, 0);
    let wl_keep = chain.add_window(0, 1);
    let away = chain.add_session("away");
    let wl_away = chain.share_window(away, 0, 0);
    let w_shared = unsafe { (*wl_shared).window() };

    unsafe {
        assert_eq!(session_is_linked(chain.sptr(0), w_shared), 1);
        assert_eq!(session_select(chain.sptr(0), 1), 0, "moved off the target");

        let mut item = Item::new().with_args(c"unlink-window");
        aim(&mut item, fs_of(wl_shared));

        assert_eq!(run_unlink(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wl_shared);

        let home = chain.sptr(0);
        assert_eq!(winlink_count(&(*home).windows), 1);
        assert_eq!(winlink_find_by_index(&mut (*home).windows, 1), wl_keep);
        assert_eq!(session_get_curw(home), wl_keep);

        let away_s = chain.sptr(away);
        assert_eq!(winlink_count(&(*away_s).windows), 1);
        assert_eq!(winlink_find_by_index(&mut (*away_s).windows, 0), wl_away);
        assert_eq!(
            session_get_curw(away_s),
            wl_away,
            "the away session was untouched"
        );

        assert_eq!((*wl_away).window(), w_shared);
    }
}

#[test]
fn with_a_on_the_only_window_nothing_is_asked_to_go() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let wl = chain.add_window(0, 0);
    let w = unsafe { (*wl).window() };

    let mut item = Item::new().with_args(c"kill-window -a");
    unsafe {
        aim(&mut item, fs_of(wl));

        assert_eq!(run_kill(&mut item), CMD_RETURN_NORMAL);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&(*s).windows), 1);
        assert_eq!(winlink_find_by_index(&mut (*s).windows, 0), wl);
        assert_eq!(session_get_curw(s), wl);
        assert!(!(*w).winlinks.is_empty());
    }
}

#[test]
fn with_a_the_session_lets_go_of_every_other_window() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let wl_target = chain.add_window(0, 0);
    let wl_other = chain.add_window(0, 1);
    let w_target = unsafe { (*wl_target).window() };
    let w_other = unsafe { (*wl_other).window() };

    let mut item = Item::new().with_args(c"kill-window -a");
    unsafe {
        assert_eq!(session_get_curw(chain.sptr(0)), wl_target);

        aim(&mut item, fs_of(wl_target));
        assert_eq!(run_kill(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wl_other);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&(*s).windows), 1);
        assert!(winlink_find_by_index(&mut (*s).windows, 1).is_null());
        assert_eq!(winlink_find_by_index(&mut (*s).windows, 0), wl_target);
        assert_eq!(session_get_curw(s), wl_target, "the target kept its place");

        assert!(!(*w_target).winlinks.is_empty());
        assert!(
            (*w_other).winlinks.is_empty(),
            "its last winlink went with it"
        );
    }
}

#[test]
fn without_flags_the_target_window_leaves_every_session() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let wl_target = chain.add_window(0, 0);
    let wl_keep = chain.add_window(0, 1);
    let w_target = unsafe { (*wl_target).window() };

    unsafe {
        assert_eq!(session_select(chain.sptr(0), 1), 0, "moved off the target");

        let mut item = Item::new().with_args(c"kill-window");
        aim(&mut item, fs_of(wl_target));

        assert_eq!(run_kill(&mut item), CMD_RETURN_NORMAL);
        chain.forget(wl_target);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&(*s).windows), 1);
        assert!(winlink_find_by_index(&mut (*s).windows, 0).is_null());
        assert_eq!(winlink_find_by_index(&mut (*s).windows, 1), wl_keep);
        assert_eq!(session_get_curw(s), wl_keep, "where the selection was left");

        assert!((*w_target).winlinks.is_empty());
        assert!(!(*wl_keep).window().is_null());
    }
}
