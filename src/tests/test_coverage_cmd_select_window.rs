//! Unit tests for [`crate::cmd::cmd_select_window`] — the four command
//! entries `select-window`, `next-window`, `previous-window` and
//! `last-window`, which share one exec hook, together with the block of
//! message-protocol, style, layout, prompt and command constants the file
//! declares.
//!
//! Exec is reached through each parsed command's own entry pointer, exactly
//! as the command queue calls it, over items whose arguments come from the
//! real command parser and whose target is a registered session holding
//! linked windows of one pane each. Every deterministic branch is exercised:
//! a plain selection moving to the targeted winlink and stacking the old one;
//! selecting the window already current, which answers normal without
//! stacking or redrawing anything; walking forward by entry identity and
//! again by `-n`; walking backwards from the highest index; `-l` and
//! `last-window` returning to the window a previous selection stacked; `-T`
//! swapping back only when the target already is current, with the current
//! state refreshed only when it names this very session; and `-a` limiting a
//! walk to windows carrying an alert. The refusals — nothing to walk to, no
//! history behind `-l` or `-T` — file their cause against the item's client,
//! both in the server's message log and in the error stream the command
//! opens against it, and answer error with the selection untouched.
//!
//! Safety notes, like the other suites. Every fixture client reports to a
//! peer marked bad, so any message the command tries to send is refused
//! before a descriptor exists; errors land in the message log and in a
//! buffered stream file, which each test reads back and takes down again,
//! compensating the reference the file holds so the client never looks free
//!able. Successful selections end in `server_redraw_session`,
//! `cmdq_insert_hook` (a no-op while no `after-select-window` option exists)
//! and `recalculate_sizes`, plus a `session-window-changed` notification
//! that sits on the global queue nothing ever drains. Everything else these
//! tests touch is taken and given back under [`globals`].

use crate::arguments::{args_count, args_get, args_has};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_select_window::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_PANE, CMD_FIND_SESSION,
    CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT,
    CMD_TARGET_SESSION_USAGE, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND,
    MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS,
    MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
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
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_last_window_entry, cmd_next_window_entry,
    cmd_previous_window_entry, cmd_select_window_entry,
};
use crate::cmd::cmdq_get_current;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::file::{file_find_ref, file_free};
use crate::proc::PEER_BAD;
use crate::server::CLIENT_ALLREDRAWFLAGS;
use crate::server::message_log;
use crate::session::{session_attached, session_get_curw};
use crate::tests::test_fixtures::{
    Args, Clients, Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, seen,
    unlink, zeroed,
};
use crate::types::*;
use crate::window::window_get_latest;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;

/// The four entries under test, all pointing their exec at one hook.
const SELECTW: *const cmd_entry = &raw const cmd_select_window_entry;
const NEXTW: *const cmd_entry = &raw const cmd_next_window_entry;
const PREVW: *const cmd_entry = &raw const cmd_previous_window_entry;
const LASTW: *const cmd_entry = &raw const cmd_last_window_entry;

/// Where the fixture windows' ids start, far above anything production hands
/// out from its own counters.
const WINDOW_ID_BASE: u_int = 800_000;

/// Where the fixture panes' ids start; pane ids only ever show up in strings.
const PANE_ID_BASE: u_int = 850_000;

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session starts null and its flags clear, which is
/// what sends `cmdq_error` down the branch that logs the message and buffers
/// it against the client.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

/// Runs the parsed command an item carries through its own entry's exec hook,
/// the way the command queue calls it.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's target, source and current states at the resolved find
/// state `fs`, as resolution would leave them before the hook runs.
unsafe fn aim(item: &mut Item, fs: cmd_find_state) {
    unsafe {
        let p = item.ptr();
        (*p).target = fs.clone();
        (*p).source = fs.clone();
        *cmdq_get_current(p) = fs.clone();
    }
}

/// Points the states as [`aim`] does and makes `caller` the item's own client
/// and target client, so refusals report against that client.
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

/// The lines the server has recorded so far, oldest first. Entries
/// accumulate across the whole test binary, so assertions look for their own
/// wording rather than count lines from zero.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// What has been buffered as an error against `c`, taking the stream's file
/// down again so the next ask sees an empty client. The file held one
/// reference to the client, handed back here explicitly so the fixture never
/// crosses the threshold where it would be scheduled for destruction.
unsafe fn error_stream_text(c: *mut client) -> String {
    unsafe {
        let cf = file_find_ref(&raw mut (*c).files, 2).expect("the error stream was opened");
        let cf_ptr = cf.as_ptr();
        let text = String::from_utf8_lossy((*cf_ptr).buffer.as_mut().as_slice()).into_owned();
        file_free(cf);
        assert!(
            file_find_ref(&raw mut (*c).files, 2).is_none(),
            "the error stream was not taken down"
        );
        text
    }
}

/// A registered session holding linked windows of one pane each, everything
/// in the server's trees the way the target-taking commands expect to walk
/// them. Windows are linked in call order, so whichever goes in first stays
/// the session's current one until something selects otherwise. The winlinks
/// are unlinked again on the way out, which is safe because nothing here
/// ever spawns or destroys a window.
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

    /// Links a fresh window carrying one pane at index `idx`, answering its
    /// winlink. The first window linked becomes the session's current one.
    fn add_window(&mut self, idx: c_int) -> *mut winlink {
        let id = WINDOW_ID_BASE + self.windows.len() as u_int;
        let mut w = Window::new(id, "chain", 80, 24);
        let mut p = Pane::new(PANE_ID_BASE + self.panes.len() as u_int, 80, 24, 100);
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
fn the_four_entries_advertise_their_commands_and_share_one_hook() {
    let _guard = globals();
    unsafe {
        assert_eq!((*SELECTW).name.to_string_lossy(), "select-window");
        assert_eq!(
            (*SELECTW)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "selectw"
        );
        assert_eq!((*SELECTW).args.template.to_string_lossy(), "lnpTt:");
        assert_eq!((*SELECTW).args.lower, 0);
        assert_eq!((*SELECTW).args.upper, 0);
        assert!((*SELECTW).args.cb.is_none());
        assert_eq!(
            (*SELECTW).usage.to_string_lossy(),
            "[-lnpT] [-t target-window]"
        );
        assert_eq!((*SELECTW).source.flag, 0);
        assert_eq!((*SELECTW).source.type_0, CMD_FIND_PANE);
        assert_eq!((*SELECTW).source.flags, 0);
        assert_eq!((*SELECTW).target.flag, b't' as c_char);
        assert_eq!((*SELECTW).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*SELECTW).target.flags, 0);
        assert_eq!((*SELECTW).flags, 0);

        assert_eq!((*NEXTW).name.to_string_lossy(), "next-window");
        assert_eq!(
            (*NEXTW)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "next"
        );
        assert_eq!((*NEXTW).args.template.to_string_lossy(), "at:");
        assert_eq!((*NEXTW).args.lower, 0);
        assert_eq!((*NEXTW).args.upper, 0);
        assert!((*NEXTW).args.cb.is_none());
        assert_eq!((*NEXTW).usage.to_string_lossy(), "[-a] [-t target-session]");
        assert_eq!((*NEXTW).source.flag, 0);
        assert_eq!((*NEXTW).source.type_0, CMD_FIND_PANE);
        assert_eq!((*NEXTW).target.flag, b't' as c_char);
        assert_eq!((*NEXTW).target.type_0, CMD_FIND_SESSION);

        assert_eq!((*PREVW).name.to_string_lossy(), "previous-window");
        assert_eq!(
            (*PREVW)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "prev"
        );
        assert_eq!((*PREVW).args.template.to_string_lossy(), "at:");
        assert_eq!((*PREVW).usage.to_string_lossy(), "[-a] [-t target-session]");
        assert_eq!((*PREVW).target.type_0, CMD_FIND_SESSION);

        assert_eq!((*LASTW).name.to_string_lossy(), "last-window");
        assert_eq!(
            (*LASTW)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "last"
        );
        assert_eq!((*LASTW).args.template.to_string_lossy(), "t:");
        assert_eq!((*LASTW).target.type_0, CMD_FIND_SESSION);
        assert_eq!(
            (*LASTW).usage.to_string_lossy(),
            "[-t target-session]",
            "the usage string is the shared constant"
        );
        assert_eq!((*LASTW).usage, CMD_TARGET_SESSION_USAGE);

        for e in [SELECTW, NEXTW, PREVW, LASTW] {
            assert_eq!((*e).flags, 0);
        }
        let hook = |e: *const cmd_entry| (*e).exec;
        assert!(::core::ptr::fn_addr_eq(hook(NEXTW), hook(SELECTW)));
        assert!(::core::ptr::fn_addr_eq(hook(PREVW), hook(SELECTW)));
        assert!(::core::ptr::fn_addr_eq(hook(LASTW), hook(SELECTW)));

        assert_eq!(
            CMD_TARGET_SESSION_USAGE.to_bytes_with_nul(),
            b"[-t target-session]\0"
        );

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
        assert_eq!(PANE_LINES_SINGLE, 0);
        assert_eq!(PANE_LINES_DOUBLE, 1);
        assert_eq!(PANE_LINES_HEAVY, 2);
        assert_eq!(PANE_LINES_SIMPLE, 3);
        assert_eq!(PANE_LINES_NUMBER, 4);
        assert_eq!(PANE_LINES_SPACES, 5);
        assert_eq!(PROGRESS_BAR_HIDDEN, 0);
        assert_eq!(PROGRESS_BAR_NORMAL, 1);
        assert_eq!(PROGRESS_BAR_ERROR, 2);
        assert_eq!(PROGRESS_BAR_INDETERMINATE, 3);
        assert_eq!(PROGRESS_BAR_PAUSED, 4);
        assert_eq!(SCREEN_CURSOR_DEFAULT, 0);
        assert_eq!(SCREEN_CURSOR_BLOCK, 1);
        assert_eq!(SCREEN_CURSOR_UNDERLINE, 2);
        assert_eq!(SCREEN_CURSOR_BAR, 3);
        assert_eq!(STYLE_DEFAULT_BASE, 0);
        assert_eq!(STYLE_DEFAULT_PUSH, 1);
        assert_eq!(STYLE_DEFAULT_POP, 2);
        assert_eq!(STYLE_DEFAULT_SET, 3);
        assert_eq!(STYLE_RANGE_NONE, 0);
        assert_eq!(STYLE_RANGE_LEFT, 1);
        assert_eq!(STYLE_RANGE_RIGHT, 2);
        assert_eq!(STYLE_RANGE_PANE, 3);
        assert_eq!(STYLE_RANGE_WINDOW, 4);
        assert_eq!(STYLE_RANGE_SESSION, 5);
        assert_eq!(STYLE_RANGE_USER, 6);
        assert_eq!(STYLE_RANGE_CONTROL, 7);
        assert_eq!(STYLE_LIST_OFF, 0);
        assert_eq!(STYLE_LIST_ON, 1);
        assert_eq!(STYLE_LIST_FOCUS, 2);
        assert_eq!(STYLE_LIST_LEFT_MARKER, 3);
        assert_eq!(STYLE_LIST_RIGHT_MARKER, 4);
        assert_eq!(STYLE_ALIGN_DEFAULT, 0);
        assert_eq!(STYLE_ALIGN_LEFT, 1);
        assert_eq!(STYLE_ALIGN_CENTRE, 2);
        assert_eq!(STYLE_ALIGN_RIGHT, 3);
        assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);
        assert_eq!(THEME_UNKNOWN, 0);
        assert_eq!(THEME_LIGHT, 1);
        assert_eq!(THEME_DARK, 2);
        assert_eq!(LAYOUT_LEFTRIGHT, 0);
        assert_eq!(LAYOUT_TOPBOTTOM, 1);
        assert_eq!(LAYOUT_WINDOWPANE, 2);
        assert_eq!(PROMPT_TYPE_COMMAND, 0);
        assert_eq!(PROMPT_TYPE_SEARCH, 1);
        assert_eq!(PROMPT_TYPE_TARGET, 2);
        assert_eq!(PROMPT_TYPE_WINDOW_TARGET, 3);
        assert_eq!(PROMPT_TYPE_INVALID, 255);
        assert_eq!(PROMPT_ENTRY, 0);
        assert_eq!(PROMPT_COMMAND, 1);
        assert_eq!(CLIENT_EXIT_RETURN, 0);
        assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
        assert_eq!(CLIENT_EXIT_DETACH, 2);
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
    }
}

#[test]
fn parsing_resolves_the_four_names_their_aliases_and_their_letters() {
    let _guard = globals();
    unsafe {
        for (line, want) in [
            (c"select-window", SELECTW),
            (c"selectw", SELECTW),
            (c"next-window", NEXTW),
            (c"next", NEXTW),
            (c"previous-window", PREVW),
            (c"prev", PREVW),
            (c"last-window", LASTW),
            (c"last", LASTW),
        ] {
            let parsed = Args::parse(line);
            assert!(::core::ptr::eq((*parsed.cmd()).entry, want), "{line:?}");
        }

        let flagged = Args::parse(c"select-window -lnpT -t 3");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, SELECTW));
        let a = flagged.ptr();
        for letter in *b"lnpT" {
            assert_eq!(args_has(&*a, letter), 1, "{}", letter as char);
        }
        assert_eq!(args_has(&*a, b'a'), 0, "-a belongs to the walkers alone");
        assert_eq!(seen(args_get(&*a, b't')), "3");
        assert_eq!(args_count(&*a), 0);

        let activity = Args::parse(c"next -a -t up");
        assert!(::core::ptr::eq((*activity.cmd()).entry, NEXTW));
        assert_eq!(args_has(&*activity.ptr(), b'a'), 1);

        let mut unknown = cmd_parse_from_string(c"select-window -q".as_ptr(), null_mut());
        assert_eq!(unknown.status, CMD_PARSE_ERROR);
        let err = unknown.take_error();
        assert!(err.contains("unknown flag"), "{err}");
        assert!(err.contains("-q"), "{err}");

        let mut missing = cmd_parse_from_string(c"last-window -t".as_ptr(), null_mut());
        assert_eq!(missing.status, CMD_PARSE_ERROR);
        let err = missing.take_error();
        assert!(err.contains("expects an argument"), "{err}");
    }
}

#[test]
fn reselecting_the_current_window_answers_normal_and_touches_nothing() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("steady");
    let wl0 = chain.add_window(0);
    let mut clients = Clients::new();
    let viewer = clients.add("viewer", 80, 24);

    let mut item = Item::new().with_args(c"select-window");
    unsafe {
        wire(viewer);
        (*viewer).session = chain.sptr();
        aim_from(&mut item, viewer, fs_of(wl0));

        let before = server_messages().len();
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(server_messages().len(), before, "nothing was reported");

        let s = chain.sptr();
        assert_eq!(session_get_curw(s), wl0);
        assert!((*s).lastw.is_empty(), "re-selecting stacked nothing");
        assert_eq!(
            (*viewer).flags & CLIENT_ALLREDRAWFLAGS,
            0,
            "no redraw was raised without a change"
        );
        assert_eq!((*viewer).retval, 0, "no error touched the client");
        assert_eq!(
            window_get_latest((*wl0).window()),
            viewer,
            "the attached client became the window's latest"
        );
        assert_eq!(
            session_attached(s),
            1,
            "recalculate_sizes counted the client"
        );
    }
}

#[test]
fn selecting_different_window_next_previous_and_last_window() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("multi");
    let _wl0 = chain.add_window(0);
    let wl1 = chain.add_window(1);
    let wl2 = chain.add_window(2);
    unsafe {
        let mut item1 = Item::new().with_args(c"select-window");
        aim(&mut item1, fs_of(wl1));
        assert_eq!(run(&mut item1), CMD_RETURN_NORMAL);
        assert_eq!(session_get_curw(chain.sptr()), wl1);

        let mut item_next = Item::new().with_args(c"next-window");
        aim(&mut item_next, fs_of(wl1));
        assert_eq!(run(&mut item_next), CMD_RETURN_NORMAL);
        assert_eq!(session_get_curw(chain.sptr()), wl2);

        let mut item_prev = Item::new().with_args(c"previous-window");
        aim(&mut item_prev, fs_of(wl2));
        assert_eq!(run(&mut item_prev), CMD_RETURN_NORMAL);
        assert_eq!(session_get_curw(chain.sptr()), wl1);

        let mut item_last = Item::new().with_args(c"last-window");
        aim(&mut item_last, fs_of(wl1));
        assert_eq!(run(&mut item_last), CMD_RETURN_NORMAL);
        assert_eq!(session_get_curw(chain.sptr()), wl2);

        let mut item_t = Item::new().with_args(c"select-window -T");
        aim(&mut item_t, fs_of(wl2));
        assert_eq!(run(&mut item_t), CMD_RETURN_NORMAL);
        assert_eq!(session_get_curw(chain.sptr()), wl1);
    }
}

#[test]
fn single_window_session_navigation_refusals() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("single");
    let wl0 = chain.add_window(0);
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    unsafe {
        wire(caller);

        let mut item_next = Item::new().with_args(c"next-window");
        aim_from(&mut item_next, caller, fs_of(wl0));
        assert_eq!(run(&mut item_next), CMD_RETURN_ERROR);
        assert!(error_stream_text(caller).contains("no next window"));

        let mut item_prev = Item::new().with_args(c"previous-window");
        aim_from(&mut item_prev, caller, fs_of(wl0));
        assert_eq!(run(&mut item_prev), CMD_RETURN_ERROR);
        assert!(error_stream_text(caller).contains("no previous window"));

        let mut item_last = Item::new().with_args(c"last-window");
        aim_from(&mut item_last, caller, fs_of(wl0));
        assert_eq!(run(&mut item_last), CMD_RETURN_ERROR);
        assert!(error_stream_text(caller).contains("no last window"));

        let mut item_t = Item::new().with_args(c"select-window -T");
        aim_from(&mut item_t, caller, fs_of(wl0));
        assert_eq!(run(&mut item_t), CMD_RETURN_ERROR);
        assert!(error_stream_text(caller).contains("no last window"));
    }
}
