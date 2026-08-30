//! Unit tests for [`crate::spawn`], the engine shared by `new-session`,
//! `new-window`, `split-window` and the respawn commands.
//!
//! Two kinds of thing are reachable here without owning a process. The first
//! is metadata: the spawn flags, the message protocol numbers and the
//! enumerated families this module carries for the rest of the tree, checked
//! against their upstream values and orderings.
//!
//! The second is the pair of entry points' deterministic refusals — the
//! branches answered entirely from the caller's memory before any descriptor
//! is touched. [`spawn_window`] refuses a respawn whose window still holds an
//! attached pane ("window … still active") and a fresh spawn whose explicit
//! index is already linked ("index … in use"); [`spawn_pane`], entered
//! directly, refuses a respawn whose own pane is still attached ("pane …
//! still active") after reading nothing but the session's `history-limit`.
//! Each test pins the exact cause string and the absence of side effects on
//! the session, winlink, window and pane behind it.
//!
//! One limit worth recording. Every route past these checks ends in
//! `fdforkpty`, which forks a real pty child and chdirs this process on the
//! way; the `-k` routes close live descriptors before that. No fixture may go
//! there, so everything downstream of the refusals stays out of reach on
//! purpose.

use crate::session::session_get_curw;
use crate::spawn::{
    _PATH_BSHELL, _PATH_DEFPATH, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN,
    IUTF8, LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MODE_CRLF,
    MODE_CURSOR, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED,
    MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE,
    MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_EMPTY, PANE_EXITED, PANE_LINES_DOUBLE,
    PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES,
    PANE_STATUSDRAWN, PANE_STATUSREADY, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, SIG_BLOCK, SIG_SETMASK, SIGCHLD, SPAWN_DETACHED, SPAWN_EMPTY,
    SPAWN_FLOATING, SPAWN_KILL, SPAWN_NONOTIFY, SPAWN_RESPAWN, SPAWN_ZOOM, STDERR_FILENO,
    STDIN_FILENO, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    TCSANOW, THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, VERASE, WINDOW_ZOOMED, WINLINK_ACTIVITY,
    WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE, spawn_pane, spawn_window,
};
use crate::tests::test_fixtures::{Item, Pane, Session, Window, globals, link, seen, unlink_all};
use crate::types::*;
use crate::window::{window_panes_first, winlink_count, winlink_find_by_index};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;

/// A descriptor number parked in the fixture pane's `fd`, so that both entry
/// points see a pane that is still attached. Nothing ever closes it: every
/// branch under test returns before any descriptor work.
const FAKE_FD: c_int = 10;

/// A session `$0` named "0" holding one window `@0` named "keep" at index
/// `idx`, whose single pane has [`FAKE_FD`] in its `fd`. Nothing registers
/// with the server's trees: the refusal branches walk only the session's own
/// winlink tree and the window's own pane list, so ordinary fixture memory is
/// enough.
struct Rig {
    s: *mut session,
    w: *mut window,
    p: *mut window_pane,
    wl: *mut winlink,
    _session: Session,
    _window: Window,
    _pane: Pane,
}

impl Drop for Rig {
    fn drop(&mut self) {
        unlink_all(&mut self._session);
    }
}

impl Rig {
    fn new(idx: c_int) -> Rig {
        let mut session = Session::new(0, "0");
        let mut window = Window::new(0, "keep", 80, 24);
        let mut pane = Pane::new(1, 80, 24, 100);
        unsafe { (*pane.ptr()).fd = FAKE_FD };
        window.add_pane(&mut pane);
        let wl = link(&mut session, &mut window, idx);
        Rig {
            s: session.ptr(),
            w: window.ptr(),
            p: pane.ptr(),
            wl,
            _session: session,
            _window: window,
            _pane: pane,
        }
    }
}

/// A spawn context pointing at `rig` through a fresh command-queue item with
/// no client, no name, no argv, no environment and no working directory — the
/// shape a hook assembles when none of its flags were given.
fn context(
    item: &mut Item,
    rig: &Rig,
    wp0: *mut window_pane,
    flags: c_int,
    idx: c_int,
) -> Box<spawn_context<'static>> {
    let mut sc = Box::new(spawn_context::default());
    sc.item = crate::cmd::cmdq_item_weak_from_ptr(item.ptr());
    sc.s = rig.s;
    sc.wl = rig.wl;
    sc.wp0 = wp0;
    sc.idx = idx;
    sc.flags = flags;
    sc
}

/// Checks that a family of constants numbers its members consecutively from
/// `base`, which pins both each value and the absence of collisions.
fn consecutive(family: &[msgtype], base: msgtype) {
    for (i, v) in family.iter().enumerate() {
        assert_eq!(*v, base + i as msgtype);
    }
}

#[test]
fn the_spawn_flags_and_their_neighbouring_bits_are_the_upstream_values() {
    assert_eq!(SPAWN_KILL, 0x1);
    assert_eq!(SPAWN_DETACHED, 0x2);
    assert_eq!(SPAWN_RESPAWN, 0x4);
    assert_eq!(SPAWN_NONOTIFY, 0x10);
    assert_eq!(SPAWN_EMPTY, 0x40);
    assert_eq!(SPAWN_ZOOM, 0x80);
    assert_eq!(SPAWN_FLOATING, 0x100);

    assert_eq!(MODE_CURSOR, 0x1);
    assert_eq!(MODE_CRLF, 0x4000);
    assert_eq!(PANE_EXITED, 0x100);
    assert_eq!(PANE_STATUSREADY, 0x200);
    assert_eq!(PANE_STATUSDRAWN, 0x400);
    assert_eq!(PANE_EMPTY, 0x800);
    assert_eq!(WINDOW_ZOOMED, 0x8);
    assert_eq!(LAYOUT_CELL_FLOATING, 0x1);

    assert_eq!(WINLINK_BELL, 0x1);
    assert_eq!(WINLINK_ACTIVITY, 0x2);
    assert_eq!(WINLINK_SILENCE, 0x4);
    assert_eq!(
        WINLINK_ALERTFLAGS,
        WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE
    );
}

#[test]
fn the_message_constants_number_their_protocols_contiguously() {
    assert_eq!(MSG_VERSION, 12);

    consecutive(
        &[
            MSG_IDENTIFY_FLAGS,
            MSG_IDENTIFY_TERM,
            MSG_IDENTIFY_TTYNAME,
            MSG_IDENTIFY_OLDCWD,
            MSG_IDENTIFY_STDIN,
            MSG_IDENTIFY_ENVIRON,
            MSG_IDENTIFY_DONE,
            MSG_IDENTIFY_CLIENTPID,
            MSG_IDENTIFY_CWD,
            MSG_IDENTIFY_FEATURES,
            MSG_IDENTIFY_STDOUT,
            MSG_IDENTIFY_LONGFLAGS,
            MSG_IDENTIFY_TERMINFO,
        ],
        100,
    );
    consecutive(
        &[
            MSG_COMMAND,
            MSG_DETACH,
            MSG_DETACHKILL,
            MSG_EXIT,
            MSG_EXITED,
            MSG_EXITING,
            MSG_LOCK,
            MSG_READY,
            MSG_RESIZE,
            MSG_SHELL,
            MSG_SHUTDOWN,
            MSG_OLDSTDERR,
            MSG_OLDSTDIN,
            MSG_OLDSTDOUT,
            MSG_SUSPEND,
            MSG_UNLOCK,
            MSG_WAKEUP,
            MSG_EXEC,
            MSG_FLAGS,
        ],
        200,
    );
    consecutive(
        &[
            MSG_READ_OPEN,
            MSG_READ,
            MSG_READ_DONE,
            MSG_WRITE_OPEN,
            MSG_WRITE,
            MSG_WRITE_READY,
            MSG_WRITE_CLOSE,
            MSG_READ_CANCEL,
        ],
        300,
    );
}

#[test]
fn the_enumerated_constants_keep_their_upstream_orderings() {
    consecutive(
        &[
            PANE_LINES_SINGLE,
            PANE_LINES_DOUBLE,
            PANE_LINES_HEAVY,
            PANE_LINES_SIMPLE,
            PANE_LINES_NUMBER,
            PANE_LINES_SPACES,
        ],
        0,
    );
    consecutive(
        &[
            PROGRESS_BAR_HIDDEN,
            PROGRESS_BAR_NORMAL,
            PROGRESS_BAR_ERROR,
            PROGRESS_BAR_INDETERMINATE,
            PROGRESS_BAR_PAUSED,
        ],
        0,
    );
    consecutive(
        &[
            SCREEN_CURSOR_DEFAULT,
            SCREEN_CURSOR_BLOCK,
            SCREEN_CURSOR_UNDERLINE,
            SCREEN_CURSOR_BAR,
        ],
        0,
    );
    consecutive(
        &[
            STYLE_ALIGN_DEFAULT,
            STYLE_ALIGN_LEFT,
            STYLE_ALIGN_CENTRE,
            STYLE_ALIGN_RIGHT,
            STYLE_ALIGN_ABSOLUTE_CENTRE,
        ],
        0,
    );
    consecutive(
        &[
            STYLE_LIST_OFF,
            STYLE_LIST_ON,
            STYLE_LIST_FOCUS,
            STYLE_LIST_LEFT_MARKER,
            STYLE_LIST_RIGHT_MARKER,
        ],
        0,
    );
    consecutive(
        &[
            STYLE_RANGE_NONE,
            STYLE_RANGE_LEFT,
            STYLE_RANGE_RIGHT,
            STYLE_RANGE_PANE,
            STYLE_RANGE_WINDOW,
            STYLE_RANGE_SESSION,
            STYLE_RANGE_USER,
            STYLE_RANGE_CONTROL,
        ],
        0,
    );
    consecutive(
        &[
            STYLE_DEFAULT_BASE,
            STYLE_DEFAULT_PUSH,
            STYLE_DEFAULT_POP,
            STYLE_DEFAULT_SET,
        ],
        0,
    );
    consecutive(&[THEME_UNKNOWN, THEME_LIGHT, THEME_DARK], 0);
    consecutive(&[LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE], 0);
    consecutive(
        &[
            PROMPT_TYPE_COMMAND,
            PROMPT_TYPE_SEARCH,
            PROMPT_TYPE_TARGET,
            PROMPT_TYPE_WINDOW_TARGET,
        ],
        0,
    );
    assert_eq!(PROMPT_TYPE_INVALID, 255);
    consecutive(&[PROMPT_ENTRY, PROMPT_COMMAND], 0);
    consecutive(
        &[CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CLIENT_EXIT_DETACH],
        0,
    );
}

#[test]
fn the_shell_search_path_and_system_constants_are_the_upstream_ones() {
    let shell: Vec<u8> = _PATH_BSHELL.iter().map(|&b| b as u8).collect();
    let defpath: Vec<u8> = _PATH_DEFPATH.iter().map(|&b| b as u8).collect();
    assert_eq!(shell.len(), b"/bin/sh\0".len());
    assert_eq!(shell, b"/bin/sh\0");
    assert_eq!(defpath.len(), b"/usr/bin:/bin\0".len());
    assert_eq!(defpath, b"/usr/bin:/bin\0");

    assert_eq!(SIGCHLD, 17);
    assert_eq!(SIG_BLOCK, 0);
    assert_eq!(SIG_SETMASK, 2);
    assert_eq!(STDIN_FILENO, 0);
    assert_eq!(STDERR_FILENO, 2);
    assert_eq!(VERASE, 2);
    assert_eq!(IUTF8, 0o40000);
    assert_eq!(TCSANOW, 0);
}

#[test]
fn respawning_a_window_with_an_attached_pane_refuses_without_touching_it() {
    let _guard = globals();
    let mut rig = Rig::new(0);
    let mut item = Item::new();
    let mut sc = context(&mut item, &rig, rig.p, SPAWN_RESPAWN, -1);
    unsafe {
        let mut cause = None;
        let out = spawn_window(&mut sc, &mut cause);
        assert!(out.is_null(), "the respawn was refused");
        assert_eq!(cause.unwrap().to_str().unwrap(), "window 0:0 still active");

        assert_eq!(winlink_count(&raw mut (*rig.s).windows), 1);
        assert_eq!(winlink_find_by_index(&raw mut (*rig.s).windows, 0), rig.wl);
        assert_eq!((*rig.wl).window(), rig.w);
        assert_eq!((*rig.wl).flags, 0, "the alert flags were left alone");
        assert_eq!(window_panes_first(rig.w), rig.p);
        assert_eq!((*rig.p).fd, FAKE_FD, "the pane was never closed");
        assert_eq!(seen(cstr_ptr(&(*rig.w).name)), "keep", "no rename happened");
        assert_eq!(
            session_get_curw(rig.s),
            rig.wl,
            "the selection was left alone"
        );
    }
}

#[test]
fn an_explicit_index_already_in_use_refuses_the_window_spawn() {
    let _guard = globals();
    let mut rig = Rig::new(0);
    let mut item = Item::new();
    let mut sc = context(&mut item, &rig, null_mut(), SPAWN_DETACHED, 0);
    unsafe {
        let mut cause = None;
        let out = spawn_window(&mut sc, &mut cause);
        assert!(out.is_null(), "the spawn was refused");
        assert_eq!(cause.unwrap().to_str().unwrap(), "index 0 in use");

        assert_eq!(winlink_count(&raw mut (*rig.s).windows), 1);
        assert_eq!(winlink_find_by_index(&raw mut (*rig.s).windows, 0), rig.wl);
        assert_eq!((*rig.wl).window(), rig.w);
        assert_eq!((*rig.wl).flags, 0, "the alert flags were not cleared");
        assert_eq!(window_panes_first(rig.w), rig.p);
        assert_eq!((*rig.p).fd, FAKE_FD);
        assert_eq!(session_get_curw(rig.s), rig.wl, "nothing was selected");
        assert!(
            (*rig.s).lastw.is_empty(),
            "nothing was pushed onto the stack"
        );
    }
}

#[test]
fn respawning_a_pane_that_is_still_attached_refuses_before_any_descriptor_work() {
    let _guard = globals();
    let mut rig = Rig::new(0);
    let mut item = Item::new();
    let mut sc = context(&mut item, &rig, rig.p, SPAWN_RESPAWN, -1);
    unsafe {
        let mut cause = None;
        let out = spawn_pane(&mut sc, &mut cause);
        assert!(out.is_null(), "the respawn was refused");
        assert_eq!(cause.unwrap().to_str().unwrap(), "pane 0:0.0 still active");

        assert_eq!((*rig.p).fd, FAKE_FD, "the pane's descriptor stayed open");
        assert_eq!(window_panes_first(rig.w), rig.p, "the pane was kept");
        assert_eq!(winlink_count(&raw mut (*rig.s).windows), 1);
        assert_eq!(
            session_get_curw(rig.s),
            rig.wl,
            "the selection was left alone"
        );
        assert_eq!(seen(cstr_ptr(&(*rig.w).name)), "keep");
    }
}
