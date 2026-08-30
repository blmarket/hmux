//! Unit tests for [`crate::cmd::cmd_select_pane`], the exec hook shared by the
//! `select-pane` and `last-pane` commands.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entry the parsed command resolved to, over an item whose target find state
//! has already been filled from a registered session, winlink and window.
//! Around it the tests pin both entries' metadata, every constant this file
//! declares for its own compilation, and the parser's handling of the
//! `DdegLlMmP:RT:t:UZ` and `det:Z` templates together with the `selectp` and
//! `lastp` aliases.
//!
//! Each deterministic branch of exec is then driven once: `last-pane` with no
//! history refuses through the item's client; with two panes it falls back to
//! the only other pane via the window's own list; after a real selection it
//! prefers the stack that selection left behind; `-d` and `-e` inside it only
//! switch input on the last pane. A plain selection moves focus, stacks the
//! old active pane and updates the item's current state; selecting the active
//! pane again answers normal untouched; `-L`/`-R` walk to a neighbouring
//! pane across real geometry while every direction without a neighbour
//! answers normal at once. The mark branch marks, toggles off, clears with
//! `-M` and moves the mark between panes, restyling both ends; `-P` sets both
//! style options and flags the pane, `-g` prints the style back through the
//! client's output stream; `-T` sets the title through the format engine;
//! and a client carrying `CLIENT_ACTIVEPANE` keeps its own pane without the
//! window's active pane moving.
//!
//! Safety notes. Every fixture client reports to a peer marked bad, so any
//! message the command tries to send is refused before a descriptor exists;
//! prints and errors land in the client's file buffers and the server's
//! message log instead, where they are read back and freed again. Selections
//! raise notifications that sit on the global command queue nothing ever
//! drains, like the other suites. The mark tests clear [`marked_pane`] on
//! entry and exit so no pointer into one test's fixtures outlives them, and
//! the per-client pane tree the `CLIENT_ACTIVEPANE` path builds is taken
//! down again explicitly.

use crate::arguments::{args_count, args_get, args_has};
use crate::cmd::cmd_find_from_winlink_pane;
use crate::cmd::cmd_select_pane::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_ACTIVEPANE, CLIENT_CONTROL, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN,
    CLIENT_EXIT_SHUTDOWN, CLIENT_REDRAWBORDERS, CLIENT_REDRAWSTATUS, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR as SUBJ_ERROR,
    CMD_RETURN_NORMAL as SUBJ_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC,
    MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT,
    MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR,
    MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN,
    MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION,
    MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_INPUTOFF,
    PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE,
    PANE_LINES_SPACES, PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH,
    PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK,
    SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE,
    STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT,
    STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH, STYLE_DEFAULT_SET, STYLE_LIST_FOCUS,
    STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE, STYLE_RANGE_RIGHT,
    STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW, THEME_DARK, THEME_LIGHT,
    THEME_UNKNOWN, cmd_last_pane_entry, cmd_select_pane_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::cmd::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmdq_get_current};
use crate::file::{file_find_ref, file_free};
use crate::proc::PEER_BAD;
use crate::server::CLIENT_ALLREDRAWFLAGS;
use crate::server::{marked_pane, message_log};
use crate::tests::test_fixtures::{
    Args, Clients, Item, Pane, Registry, Session, Window, globals, link, seen, unlink, zeroed,
};
use crate::types::*;
use crate::window::window_get_active;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;

/// The flags exec raises on a pane whose marking or styling changed.
const RESTYLED: c_int = PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;

/// The `select-pane` entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn select_entry() -> *const cmd_entry {
    &raw const cmd_select_pane_entry
}

/// The `last-pane` entry likewise.
fn last_entry() -> *const cmd_entry {
    &raw const cmd_last_pane_entry
}

/// Runs the parsed command an item carries through its own entry's exec hook,
/// the way the command queue calls it.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer, its session — which may be null — exactly `flags`, and
/// its terminal's back-pointer to itself, which `tty_window_bigger` follows.
unsafe fn wire(c: *mut client, session: *mut session, flags: uint64_t) {
    unsafe {
        (*c).peer = Some(bad_peer());
        (*c).session = session;
        (*c).flags = flags;
        (*c).tty.owner = crate::server::client_ref_from_ptr(c).map(|c| c.downgrade());
    }
}

/// Points the item's target, source and current states at `wl` and `wp`, as a
/// resolved target would have left them for the hook to pick up.
unsafe fn aim(item: &mut Item, wl: *mut winlink, wp: *mut window_pane) {
    unsafe {
        let mut fs = Box::new(cmd_find_state::default());
        cmd_find_from_winlink_pane(&mut fs, wl, wp, 0);
        let p = item.ptr();
        (*p).target = (*fs).clone();
        (*p).source = (*fs).clone();
        *cmdq_get_current(p) = (*fs).clone();
    }
}

/// The lines the server has recorded so far, oldest first. Entries
/// accumulate across the whole test binary, so assertions look for their own
/// wording rather than exact contents.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// A copy of the server's marked-pane state, read through a raw pointer so
/// that no shared reference into the `static mut` is ever formed.
fn marked_state() -> cmd_find_state {
    unsafe { ::core::ptr::read(&raw const marked_pane) }
}

/// What has been printed to `c`'s output or error stream, freeing the file
/// entry afterwards so the stream is empty for the next ask. An empty answer
/// means nothing was printed.
unsafe fn stream_text(c: *mut client, stream: c_int) -> String {
    unsafe {
        let Some(cf) = file_find_ref(&raw mut (*c).files, stream) else {
            return String::new();
        };
        let cf_ptr = cf.as_ptr();
        let text = String::from_utf8_lossy((*cf_ptr).buffer.as_mut().as_slice()).into_owned();
        file_free(cf);
        assert!(
            file_find_ref(&raw mut (*c).files, stream).is_none(),
            "the stream was not taken down"
        );
        text
    }
}

/// Counts the per-window sizes recorded against `c`, dropping them on the way
/// out, since nothing here tears the client down through `server_destroy`.
unsafe fn take_client_windows(c: *mut client) -> u_int {
    unsafe { ::core::mem::take(&mut (*c).windows).len() as u_int }
}

#[test]
fn the_entries_advertise_both_spellings_and_the_file_declares_its_constants() {
    unsafe {
        let e = select_entry();
        assert_eq!((*e).name.to_string_lossy(), "select-pane");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "selectp"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "DdegLlMmP:RT:t:UZ");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-DdeLlMmRUZ] [-T title] [-t target-pane]"
        );
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, 0);

        let l = last_entry();
        assert_eq!((*l).name.to_string_lossy(), "last-pane");
        assert_eq!(
            (*l).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "lastp"
        );
        assert_eq!((*l).args.template.to_string_lossy(), "det:Z");
        assert_eq!((*l).args.lower, 0);
        assert_eq!((*l).args.upper, 0);
        assert!((*l).args.cb.is_none());
        assert_eq!((*l).usage.to_string_lossy(), "[-deZ] [-t target-window]");
        assert_eq!((*l).source.flag, 0);
        assert_eq!((*l).source.type_0, CMD_FIND_PANE);
        assert_eq!((*l).source.flags, 0);
        assert_eq!((*l).target.flag, b't' as c_char);
        assert_eq!((*l).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*l).target.flags, 0);
        assert_eq!((*l).flags, 0);
        assert!(::core::ptr::fn_addr_eq((*l).exec, (*e).exec));

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
        assert_eq!(MSG_READY, 207);
        assert_eq!(MSG_RESIZE, 208);
        assert_eq!(MSG_SHELL, 209);
        assert_eq!(MSG_SHUTDOWN, 210);
        assert_eq!(MSG_LOCK, 206);
        assert_eq!(MSG_UNLOCK, 215);
        assert_eq!(MSG_SUSPEND, 214);
        assert_eq!(MSG_WAKEUP, 216);
        assert_eq!(MSG_EXEC, 217);
        assert_eq!(MSG_FLAGS, 218);
        assert_eq!(MSG_OLDSTDERR, 211);
        assert_eq!(MSG_OLDSTDIN, 212);
        assert_eq!(MSG_OLDSTDOUT, 213);
        assert_eq!(MSG_READ_OPEN, 300);
        assert_eq!(MSG_READ, 301);
        assert_eq!(MSG_READ_DONE, 302);
        assert_eq!(MSG_READ_CANCEL, 307);
        assert_eq!(MSG_WRITE_OPEN, 303);
        assert_eq!(MSG_WRITE_READY, 305);
        assert_eq!(MSG_WRITE_CLOSE, 306);
        assert_eq!(MSG_WRITE, 304);
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
        assert_eq!(SUBJ_NORMAL, CMD_RETURN_NORMAL);
        assert_eq!(CMD_RETURN_WAIT, 1);
        assert_eq!(CMD_RETURN_STOP, 2);
        assert_eq!(SUBJ_ERROR, CMD_RETURN_ERROR);
        assert_eq!(PANE_REDRAW, 0x1);
        assert_eq!(PANE_INPUTOFF, 0x40);
        assert_eq!(PANE_STYLECHANGED, 0x1000);
        assert_eq!(PANE_THEMECHANGED, 0x2000);
        assert_eq!(CLIENT_REDRAWSTATUS, 0x10);
        assert_eq!(CLIENT_REDRAWBORDERS, 0x400);
        assert_eq!(CLIENT_CONTROL, 0x2000);
        assert_eq!(CLIENT_ACTIVEPANE, 0x80000000u64);
    }
}

#[test]
fn parsing_resolves_both_names_their_aliases_and_their_letters() {
    let _guard = globals();
    unsafe {
        let flagged = Args::parse(c"select-pane -P bg=red -T new -t 0.1");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, select_entry()));
        let a = flagged.ptr();
        assert_eq!(args_has(&*a, b'P'), 1);
        assert_eq!(seen(args_get(&*a, b'P')), "bg=red");
        assert_eq!(seen(args_get(&*a, b'T')), "new");
        assert_eq!(seen(args_get(&*a, b't')), "0.1");

        let letters = Args::parse(c"selectp -DdegLlMmRUZ");
        assert!(::core::ptr::eq((*letters.cmd()).entry, select_entry()));
        let a = letters.ptr();
        for flag in *b"DdegLlMmRUZ" {
            assert_eq!(args_has(&*a, flag), 1, "-{flag} missing");
        }
        assert_eq!(args_count(&*a), 0);

        let last = Args::parse(c"last-pane");
        assert!(::core::ptr::eq((*last.cmd()).entry, last_entry()));
        let alias = Args::parse(c"lastp -Z");
        assert!(::core::ptr::eq((*alias.cmd()).entry, last_entry()));
        assert_eq!(args_has(&*alias.ptr(), b'Z'), 1);

        let mut unknown = cmd_parse_from_string(c"select-pane -q".as_ptr(), null_mut());
        assert_eq!(unknown.status, CMD_PARSE_ERROR);
        let err = unknown.take_error();
        assert!(err.contains("unknown flag"), "{err}");
        assert!(err.contains("-q"), "{err}");

        let mut hungry = cmd_parse_from_string(c"select-pane -P".as_ptr(), null_mut());
        assert_eq!(hungry.status, CMD_PARSE_ERROR);
        let err = hungry.take_error();
        assert!(err.contains("expects an argument"), "{err}");
        assert!(err.contains("-P"), "{err}");
    }
}

#[test]
fn selecting_the_active_pane_again_answers_normal_and_touches_nothing() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(4, "steady");
    registry.add_session(&mut s);
    let mut w = Window::new(4, "four", 80, 24);
    let mut first = Pane::new(0, 80, 24, 100);
    w.add_pane(&mut first);
    let wl = link(&mut s, &mut w, 0);
    let mut clients = Clients::new();
    let viewer = clients.add("viewer", 100, 40);
    unsafe {
        wire(viewer, s.ptr(), 0);

        let mut item = Item::new().with_args(c"select-pane");
        aim(&mut item, wl, first.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(window_get_active(w.ptr()), first.ptr());
        assert!(
            (*w.ptr()).last_panes.is_empty(),
            "a no-op selection stacked something"
        );
        assert_eq!(
            (*viewer).flags & CLIENT_ALLREDRAWFLAGS,
            0,
            "a no-op selection drew something"
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn direction_flags_without_a_neighbour_answer_normal_at_once() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(6, "boxed");
    registry.add_session(&mut s);
    let mut w = Window::new(6, "one", 80, 24);
    let mut only = Pane::new(0, 80, 24, 100);
    w.add_pane(&mut only);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        for line in [
            c"select-pane -L",
            c"select-pane -R",
            c"select-pane -U",
            c"select-pane -D",
        ] {
            let mut item = Item::new().with_args(line);
            aim(&mut item, wl, only.ptr());
            assert_eq!(run(&mut item), CMD_RETURN_NORMAL, "{line:?}");
            assert_eq!(
                window_get_active(w.ptr()),
                only.ptr(),
                "{line:?} moved focus without a neighbour"
            );
            assert!(
                (*w.ptr()).last_panes.is_empty(),
                "{line:?} stacked something"
            );
        }
    }
    unlink(&mut s, wl);
}

#[test]
fn last_pane_with_no_history_and_one_pane_returns_error() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(10, "solo");
    registry.add_session(&mut s);
    let mut w = Window::new(10, "one", 80, 24);
    let mut only = Pane::new(0, 80, 24, 100);
    w.add_pane(&mut only);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item = Item::new().with_args(c"last-pane");
        aim(&mut item, wl, only.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);
    }
    unlink(&mut s, wl);
}

#[test]
fn last_pane_with_two_panes_selects_the_other_and_handles_d_and_e() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(11, "pair");
    registry.add_session(&mut s);
    let mut w = Window::new(11, "two", 80, 24);
    let mut first = Pane::new(0, 80, 24, 100);
    let mut second = Pane::new(1, 80, 24, 100);
    w.add_pane(&mut first);
    w.add_pane(&mut second);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item_d = Item::new().with_args(c"last-pane -d");
        aim(&mut item_d, wl, first.ptr());
        assert_eq!(run(&mut item_d), CMD_RETURN_NORMAL);
        assert_ne!((*second.ptr()).flags & PANE_INPUTOFF, 0);

        let mut item_e = Item::new().with_args(c"last-pane -e");
        aim(&mut item_e, wl, first.ptr());
        assert_eq!(run(&mut item_e), CMD_RETURN_NORMAL);
        assert_eq!((*second.ptr()).flags & PANE_INPUTOFF, 0);

        let mut item = Item::new().with_args(c"last-pane");
        aim(&mut item, wl, first.ptr());
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(window_get_active(w.ptr()), second.ptr());
    }
    unlink(&mut s, wl);
}

#[test]
fn select_pane_input_off_and_title_and_style() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(12, "styles");
    registry.add_session(&mut s);
    let mut w = Window::new(12, "win", 80, 24);
    let mut p = Pane::new(0, 80, 24, 100);
    w.add_pane(&mut p);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item_d = Item::new().with_args(c"select-pane -d");
        aim(&mut item_d, wl, p.ptr());
        assert_eq!(run(&mut item_d), CMD_RETURN_NORMAL);
        assert_ne!((*p.ptr()).flags & PANE_INPUTOFF, 0);

        let mut item_e = Item::new().with_args(c"select-pane -e");
        aim(&mut item_e, wl, p.ptr());
        assert_eq!(run(&mut item_e), CMD_RETURN_NORMAL);
        assert_eq!((*p.ptr()).flags & PANE_INPUTOFF, 0);

        let mut item_t = Item::new().with_args(c"select-pane -T my-title");
        aim(&mut item_t, wl, p.ptr());
        assert_eq!(run(&mut item_t), CMD_RETURN_NORMAL);

        let mut item_p = Item::new().with_args(c"select-pane -P fg=blue");
        aim(&mut item_p, wl, p.ptr());
        assert_eq!(run(&mut item_p), CMD_RETURN_NORMAL);
        assert_ne!((*p.ptr()).flags & RESTYLED, 0);

        let mut item_g = Item::with_client().with_args(c"select-pane -g");
        wire(item_g.client(), null_mut(), 0);
        aim(&mut item_g, wl, p.ptr());
        assert_eq!(run(&mut item_g), CMD_RETURN_NORMAL);

        let mut item_bad = Item::new().with_args(c"select-pane -P bg=green");
        aim(&mut item_bad, wl, p.ptr());
        assert_eq!(run(&mut item_bad), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl);
}

#[test]
fn select_pane_marking_and_clearing() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(13, "marks");
    registry.add_session(&mut s);
    let mut w = Window::new(13, "win", 80, 24);
    let mut first = Pane::new(0, 80, 24, 100);
    let mut second = Pane::new(1, 80, 24, 100);
    w.add_pane(&mut first);
    w.add_pane(&mut second);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        crate::server::server_clear_marked();

        let mut item_m = Item::new().with_args(c"select-pane -m");
        aim(&mut item_m, wl, first.ptr());
        assert_eq!(run(&mut item_m), CMD_RETURN_NORMAL);
        assert_eq!(crate::server::server_is_marked(s.ptr(), wl, first.ptr()), 1);

        let mut item_m2 = Item::new().with_args(c"select-pane -m");
        aim(&mut item_m2, wl, second.ptr());
        assert_eq!(run(&mut item_m2), CMD_RETURN_NORMAL);
        assert_eq!(
            crate::server::server_is_marked(s.ptr(), wl, second.ptr()),
            1
        );

        let mut item_clear = Item::new().with_args(c"select-pane -M");
        aim(&mut item_clear, wl, second.ptr());
        assert_eq!(run(&mut item_clear), CMD_RETURN_NORMAL);
        assert_eq!(crate::server::server_check_marked(), 0);
    }
    unlink(&mut s, wl);
}

#[test]
fn select_pane_switches_between_panes_with_activepane_client() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(14, "switch");
    registry.add_session(&mut s);
    let mut w = Window::new(14, "win", 80, 24);
    let mut first = Pane::new(0, 80, 24, 100);
    let mut second = Pane::new(1, 80, 24, 100);
    w.add_pane(&mut first);
    w.add_pane(&mut second);
    let wl = link(&mut s, &mut w, 0);
    let mut clients = Clients::new();
    let viewer = clients.add("viewer", 100, 40);
    unsafe {
        wire(viewer, s.ptr(), CLIENT_ACTIVEPANE);

        let mut item = Item::new().with_args(c"select-pane -Z");
        aim(&mut item, wl, second.ptr());
        item.set_client(viewer);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let mut item_normal = Item::new().with_args(c"select-pane");
        aim(&mut item_normal, wl, second.ptr());
        assert_eq!(run(&mut item_normal), CMD_RETURN_NORMAL);
        assert_eq!(window_get_active(w.ptr()), second.ptr());
    }
    unlink(&mut s, wl);
}
