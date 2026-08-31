//! Unit tests for [`crate::cmd::cmd_swap_window`] — the `swap-window` entry
//! (name, alias, argument template, usage, find flags and exec hook) and the
//! block of message-protocol, pane-line, progress-bar, cursor, style, theme,
//! layout, prompt, exit, parse, find and return constants the file declares.
//!
//! The exec hook is reached through the entry's own function pointer over an
//! item whose source and target find states have been resolved by hand,
//! exactly as the command queue would leave them. The runs here are the
//! conservative ones: registered sessions holding linked windows drive every
//! decision the hook makes — a window swapped with itself returning early,
//! the exchange of two windows between their winlinks within one session and
//! across two, `-d` selecting the swapped-in windows in both sessions, a
//! marked source winlink handing its mark to its destination, and a window
//! held by a second winlink keeping its holder list linked when the first one
//! is swapped away. Every side effect past the exchange is a no-op by
//! construction: no session here carries a group until the refusal test joins
//! two into one, so the synchronize and redraw sweeps walk empty lists, and
//! with no attached client the final `recalculate_sizes` finds no size to
//! change.
//!
//! One shape is deliberately left alone: a grouped pair whose swap is allowed
//! would run `session_group_synchronize_from`, which rebuilds whole window
//! trees and takes the old winlinks back out from under these fixtures, so a
//! group only ever reaches the refusal.

use crate::arguments::args_has;
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_swap_window::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_DEFAULT_MARKED,
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
    MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING,
    MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_swap_window_entry,
};
use crate::cmd::cmdq_get_current;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::proc::PEER_BAD;
use crate::server::{marked_pane, message_log};
use crate::session::session_get_curw;
use crate::session::{
    session_group_add, session_group_name, session_group_new, session_group_remove, session_groups,
};
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, seen, unlink, zeroed,
};
use crate::types::*;
use crate::window::winlink_count;
use crate::window::winlinks_into;
use ::core::ffi::{CStr, c_char, c_int, c_void};
use ::core::ptr::{null, null_mut};

/// The entry under test.
const ENTRY: *const cmd_entry = &raw const cmd_swap_window_entry;

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 640_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 930_000;

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, ENTRY),
            "the item is not running swap-window"
        );
        let exec = (*ENTRY).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// Points the item's source state at the resolved source and its target state
/// at the resolved destination, as the command queue would leave them before
/// the hook runs. The current state follows the target, although this hook
/// never reads it.
unsafe fn aim(item: &mut Item, source: cmd_find_state, target: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).source = source;
        (*p).target = target.clone();
        *cmdq_get_current(p) = target.clone();
    }
}

/// Points the states as [`aim`] does and makes `caller` the item's own
/// client, so `cmdq_error` files its message against that client.
unsafe fn aim_from(
    item: &mut Item,
    caller: *mut client,
    source: cmd_find_state,
    target: cmd_find_state,
) {
    unsafe {
        let p = item.ptr();
        item.set_client(caller);
        cmdq_set_target_client(p, caller);
        aim(item, source, target)
    }
}

/// The find state of `wl`: its session, its window and that window's active
/// pane.
unsafe fn fs_of(wl: *mut winlink) -> cmd_find_state {
    let mut fs = *Box::new(cmd_find_state::default());
    unsafe { cmd_find_from_winlink(&mut fs, wl, 0) };
    fs
}

/// A peer for the fixture client, marked bad so any message the error path
/// tries to send is refused before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session stays null, which sends `cmdq_error` down
/// the branch that files the message in the server's message log.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
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

/// Registered sessions holding registered windows of one pane each,
/// everything in the server's trees the way target-taking commands walk
/// them. Winlinks the chain itself made are unlinked again on the way out; a
/// window shared by a second session through [`link`] is unlinked once per
/// holding session, releasing exactly the typed handles linking took.
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
}

impl Drop for Chain {
    fn drop(&mut self) {
        for (si, wl) in ::std::mem::take(&mut self.tracked).into_iter().rev() {
            unlink(&mut self.sessions[si], wl);
        }
    }
}

/// Joins fixture sessions into one session group for the length of a test.
/// The group hangs off the global tree `session_groups`, so the teardown takes
/// it apart the way the server does: each member leaves through
/// `session_group_remove`, which gives the emptied group up. Declared after
/// the chain a test builds, it is dropped while the member sessions are still
/// alive.
struct Group {
    sg: *mut session_group,
    members: Vec<*mut session>,
}

impl Group {
    /// An empty group named `name`, inserted into the global tree.
    fn new(name: &CStr) -> Group {
        let sg = unsafe { session_group_new(name.as_ptr()) };
        assert!(!sg.is_null(), "no session group");
        Group {
            sg,
            members: Vec::new(),
        }
    }

    /// Adds `s` as a member, unless it already belongs to some group.
    fn add(&mut self, s: *mut session) {
        unsafe { session_group_add(self.sg, s) };
        self.members.push(s);
    }

    fn ptr(&self) -> *mut session_group {
        self.sg
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        unsafe {
            let name = session_group_name(self.sg).to_owned();
            for s in ::std::mem::take(&mut self.members) {
                session_group_remove(s);
            }
            let _ = session_groups.map().remove(&name);
        }
    }
}

#[test]
fn the_entry_describes_the_swap_window_command() {
    let _guard = globals();
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "swap-window");
        assert_eq!(
            (*ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "swapw"
        );
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "ds:t:");
        assert_eq!((*ENTRY).args.lower, 0);
        assert_eq!((*ENTRY).args.upper, 0);
        assert!(
            (*ENTRY).args.cb.is_none(),
            "swap-window takes no args callback"
        );
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-d] [-s src-window] [-t dst-window]"
        );

        assert_eq!((*ENTRY).source.flag, b's' as c_char);
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_WINDOW);
        assert_eq!((*ENTRY).source.flags, CMD_FIND_DEFAULT_MARKED);
        assert_eq!((*ENTRY).target.flag, b't' as c_char);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*ENTRY).target.flags, 0);

        assert_eq!((*ENTRY).flags, 0);

        assert_eq!(CMD_FIND_PANE, 0);
        assert_eq!(CMD_FIND_WINDOW, 1);
        assert_eq!(CMD_FIND_SESSION, 2);
        assert_eq!(CMD_FIND_DEFAULT_MARKED, 0x8);
        assert_eq!(CMD_RETURN_ERROR, -1);
        assert_eq!(CMD_RETURN_NORMAL, 0);
        assert_eq!(CMD_RETURN_WAIT, 1);
        assert_eq!(CMD_RETURN_STOP, 2);

        assert_eq!(ARGS_PARSE_INVALID, 0);
        assert_eq!(ARGS_PARSE_STRING, 1);
        assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
        assert_eq!(ARGS_PARSE_COMMANDS, 3);

        assert_eq!(CLIENT_EXIT_RETURN, 0);
        assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
        assert_eq!(CLIENT_EXIT_DETACH, 2);

        assert_eq!(PROMPT_ENTRY, 0);
        assert_eq!(PROMPT_COMMAND, 1);
        assert_eq!(PROMPT_TYPE_COMMAND, 0);
        assert_eq!(PROMPT_TYPE_SEARCH, 1);
        assert_eq!(PROMPT_TYPE_TARGET, 2);
        assert_eq!(PROMPT_TYPE_WINDOW_TARGET, 3);
        assert_eq!(PROMPT_TYPE_INVALID, 255);

        assert_eq!(LAYOUT_LEFTRIGHT, 0);
        assert_eq!(LAYOUT_TOPBOTTOM, 1);
        assert_eq!(LAYOUT_WINDOWPANE, 2);

        assert_eq!(THEME_UNKNOWN, 0);
        assert_eq!(THEME_LIGHT, 1);
        assert_eq!(THEME_DARK, 2);

        assert_eq!(STYLE_ALIGN_DEFAULT, 0);
        assert_eq!(STYLE_ALIGN_LEFT, 1);
        assert_eq!(STYLE_ALIGN_CENTRE, 2);
        assert_eq!(STYLE_ALIGN_RIGHT, 3);
        assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);

        assert_eq!(STYLE_LIST_OFF, 0);
        assert_eq!(STYLE_LIST_ON, 1);
        assert_eq!(STYLE_LIST_FOCUS, 2);
        assert_eq!(STYLE_LIST_LEFT_MARKER, 3);
        assert_eq!(STYLE_LIST_RIGHT_MARKER, 4);

        assert_eq!(STYLE_RANGE_NONE, 0);
        assert_eq!(STYLE_RANGE_LEFT, 1);
        assert_eq!(STYLE_RANGE_RIGHT, 2);
        assert_eq!(STYLE_RANGE_PANE, 3);
        assert_eq!(STYLE_RANGE_WINDOW, 4);
        assert_eq!(STYLE_RANGE_SESSION, 5);
        assert_eq!(STYLE_RANGE_USER, 6);
        assert_eq!(STYLE_RANGE_CONTROL, 7);

        assert_eq!(STYLE_DEFAULT_BASE, 0);
        assert_eq!(STYLE_DEFAULT_PUSH, 1);
        assert_eq!(STYLE_DEFAULT_POP, 2);
        assert_eq!(STYLE_DEFAULT_SET, 3);

        assert_eq!(SCREEN_CURSOR_DEFAULT, 0);
        assert_eq!(SCREEN_CURSOR_BLOCK, 1);
        assert_eq!(SCREEN_CURSOR_UNDERLINE, 2);
        assert_eq!(SCREEN_CURSOR_BAR, 3);

        assert_eq!(PROGRESS_BAR_HIDDEN, 0);
        assert_eq!(PROGRESS_BAR_NORMAL, 1);
        assert_eq!(PROGRESS_BAR_ERROR, 2);
        assert_eq!(PROGRESS_BAR_INDETERMINATE, 3);
        assert_eq!(PROGRESS_BAR_PAUSED, 4);

        assert_eq!(PANE_LINES_SINGLE, 0);
        assert_eq!(PANE_LINES_DOUBLE, 1);
        assert_eq!(PANE_LINES_HEAVY, 2);
        assert_eq!(PANE_LINES_SIMPLE, 3);
        assert_eq!(PANE_LINES_NUMBER, 4);
        assert_eq!(PANE_LINES_SPACES, 5);

        assert_eq!(MSG_VERSION, 12);
        assert_eq!(MSG_IDENTIFY_FLAGS, 100);
        assert_eq!(MSG_IDENTIFY_TERM, 101);
        assert_eq!(MSG_IDENTIFY_TTYNAME, 102);
        assert_eq!(MSG_IDENTIFY_OLDCWD, 103);
        assert_eq!(MSG_IDENTIFY_STDIN, 104);
        assert_eq!(MSG_IDENTIFY_ENVIRON, 105);
        assert_eq!(MSG_IDENTIFY_DONE, 106);
        assert_eq!(MSG_IDENTIFY_CLIENTPID, 107);
        assert_eq!(MSG_IDENTIFY_CWD, 108);
        assert_eq!(MSG_IDENTIFY_FEATURES, 109);
        assert_eq!(MSG_IDENTIFY_STDOUT, 110);
        assert_eq!(MSG_IDENTIFY_LONGFLAGS, 111);
        assert_eq!(MSG_IDENTIFY_TERMINFO, 112);
        assert_eq!(MSG_COMMAND, 200);
        assert_eq!(MSG_DETACH, 201);
        assert_eq!(MSG_DETACHKILL, 202);
        assert_eq!(MSG_EXIT, 203);
        assert_eq!(MSG_EXITED, 204);
        assert_eq!(MSG_EXITING, 205);
        assert_eq!(MSG_LOCK, 206);
        assert_eq!(MSG_READY, 207);
        assert_eq!(MSG_RESIZE, 208);
        assert_eq!(MSG_SHELL, 209);
        assert_eq!(MSG_SHUTDOWN, 210);
        assert_eq!(MSG_OLDSTDERR, 211);
        assert_eq!(MSG_OLDSTDIN, 212);
        assert_eq!(MSG_OLDSTDOUT, 213);
        assert_eq!(MSG_SUSPEND, 214);
        assert_eq!(MSG_UNLOCK, 215);
        assert_eq!(MSG_WAKEUP, 216);
        assert_eq!(MSG_EXEC, 217);
        assert_eq!(MSG_FLAGS, 218);
        assert_eq!(MSG_READ_OPEN, 300);
        assert_eq!(MSG_READ, 301);
        assert_eq!(MSG_READ_DONE, 302);
        assert_eq!(MSG_WRITE_OPEN, 303);
        assert_eq!(MSG_WRITE, 304);
        assert_eq!(MSG_WRITE_READY, 305);
        assert_eq!(MSG_WRITE_CLOSE, 306);
        assert_eq!(MSG_READ_CANCEL, 307);

        assert!(null::<c_void>().is_null());
    }
}

#[test]
fn the_parser_resolves_the_name_the_alias_and_a_prefix() {
    let _guard = globals();
    unsafe {
        for line in [c"swap-window -d", c"swapw -d", c"swap-w -d"] {
            let mut item = Item::new().with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, ENTRY), "{line:?}");
            assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'd'), 1);
        }

        let mut flagged = Item::new().with_args(c"swap-window -d -s @1 -t @2");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, ENTRY));
        let args = cmd_get_args(&*flagged.cmd());
        for flag in *b"dst" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }

        let mut none = cmd_parse_from_string(c"swap-window".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"swap-window surplus".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
    }
}

#[test]
fn swapping_a_window_with_itself_returns_without_touching_anything() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("self");
    let (wla, wa, _) = chain.add_window(0, 0);
    let (wlb, wb, _) = chain.add_window(0, 1);

    let mut item = Item::new().with_args(c"swap-window");
    unsafe {
        aim(&mut item, fs_of(wla), fs_of(wla));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s = chain.sptr(0);
        assert_eq!(winlink_count(&(*s).windows), 2);
        assert_eq!((*wla).window(), wa);
        assert_eq!((*wlb).window(), wb);
        assert_eq!(
            winlinks_into(wa).next().unwrap_or(::core::ptr::null_mut()),
            wla,
            "nothing was relinked"
        );
        assert_eq!(
            winlinks_into(wb).next().unwrap_or(::core::ptr::null_mut()),
            wlb
        );
        assert_eq!(session_get_curw(s), wla);
        assert!(marked_pane.winlink().is_null(), "the mark stayed alone");
    }
}

#[test]
fn swapping_windows_in_same_session_and_cross_sessions() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        let mut chain = Chain::new("s0");
        let (wl0, w0, _) = chain.add_window(0, 0);
        let (wl1, w1, _) = chain.add_window(0, 1);

        let s1_idx = chain.add_session("s1");
        let (wl2, w2, _) = chain.add_window(s1_idx, 0);

        let mut item_intra = Item::new().with_args(c"swap-window -d");
        aim(&mut item_intra, fs_of(wl0), fs_of(wl1));
        assert_eq!(run(&mut item_intra), CMD_RETURN_NORMAL);
        assert_eq!((*wl0).window(), w1);
        assert_eq!((*wl1).window(), w0);

        let mut item_cross = Item::new().with_args(c"swap-window -d");
        aim(&mut item_cross, fs_of(wl0), fs_of(wl2));
        assert_eq!(run(&mut item_cross), CMD_RETURN_NORMAL);
        assert_eq!((*wl0).window(), w2);
        assert_eq!((*wl2).window(), w1);
    }
}

#[test]
fn swapping_windows_in_same_session_group_is_refused() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        let mut chain = Chain::new("sg0");
        let s0 = chain.sptr(0);
        let s1_idx = chain.add_session("sg1");
        let s1 = chain.sptr(s1_idx);

        let (wl0, _, _) = chain.add_window(0, 0);
        let (wl1, _, _) = chain.add_window(s1_idx, 0);

        let mut group = Group::new(c"mygroup");
        group.add(s0);
        group.add(s1);

        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        wire(caller);

        let mut item = Item::new().with_args(c"swap-window");
        aim_from(&mut item, caller, fs_of(wl0), fs_of(wl1));
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);

        crate::tests::test_fixtures::release_client(caller);
    }
}
