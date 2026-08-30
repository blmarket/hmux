//! Unit tests for [`crate::cmd::cmd_switch_client`] — the `switch-client`
//! entry's metadata, every constant the file declares for its own
//! compilation, and every deterministic branch of [`cmd_switch_client_exec`]
//! the fixtures can reach without a live server.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entry the parser resolved, over an item whose queue state carries the
//! current find state. Around it the tests pin the entry (name, alias,
//! template, usage, flag words, the empty source and target flags) and every
//! constant, then walk its branches: `-t` naming a missing session fails the
//! find; `-T` refuses a table that does not exist and adopts one that does
//! without switching at all; `-O` with a name no order answers refuses;
//! `-r` grants, revokes, or — against a read-only client whose peer runs as
//! somebody else — refuses through the item's client; `-n`/`-p` walk to the
//! neighbouring registered session and refuse without one; `-l` prefers a
//! live last session and refuses an unregistered ghost; the plain form makes
//! a targeted pane active or moves nothing when nobody is behind the item,
//! and answers normal at once without a client. The tail is pinned too: the
//! update-environment pass copies the client's DISPLAY into the target
//! session unless `-E` skips it, and the repeat bit on the queue state leaves
//! the client's key table alone where an ordinary switch re-chooses it from
//! the new session's `key-table` option.
//!
//! Safety notes. Fixture clients carry CLIENT_ATTACHED, so any complaint
//! `cmdq_error` raises lands in the server's message log — read back below —
//! while `file_error` declines to print (`file_can_print` answers 0) before
//! a descriptor would exist; their peers are marked bad for the same reason,
//! which is also why every refusing test leaves its client session-less.
//! Switches raise notifications that sit on the global command queue nothing
//! ever drains, like the other suites. Key tables are held by strong handles
//! while clients use them, and are taken down by [`TakenTables`] only
//! after the test's clients have left the server list.

use crate::arguments::{args_count, args_get, args_has};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_switch_client::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CLIENT_IGNORESIZE,
    CLIENT_READONLY, CMD_CLIENT_CFLAG, CMD_FIND_PANE, CMD_FIND_PREFER_UNATTACHED, CMD_FIND_SESSION,
    CMD_FIND_WINDOW, CMD_READONLY, CMD_RETURN_ERROR as SUBJ_ERROR,
    CMD_RETURN_NORMAL as SUBJ_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT, CMDQ_STATE_REPEAT,
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL,
    MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN,
    MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK,
    MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK,
    MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY,
    PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE,
    PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE,
    PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND,
    PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_INDEX, SORT_MODIFIER, SORT_NAME, SORT_ORDER,
    SORT_SIZE, SORT_Z, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_switch_client_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::cmd::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmdq_get_state};
use crate::ffi::getuid;
use crate::key_bindings::{key_bindings_get_table, key_bindings_remove_table, key_table_name};
use crate::proc::PEER_BAD;
use crate::proc::peer_ptr;
use crate::server::message_log;
use crate::session::session_get_curw;
use crate::sort::CLIENT_ATTACHED;
use crate::tests::test_fixtures::{
    Args, Environ, Item, Pane, Registry, Session, Window, globals, link, seen, unlink, zeroed,
    zeroed_client,
};
use crate::types::*;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it. Its uid backs the
/// read-only check.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` an environment, an attached mark (so refusals land in the
/// message log instead of a stream), the bad peer, and an owned `root` key
/// table. Undone when the guard goes away.
unsafe fn wire(c: *mut client) -> Wired {
    unsafe {
        (*c).environ = Some(Environ::new().owned());
        (*c).flags |= CLIENT_ATTACHED as uint64_t;
        (*c).peer = Some(bad_peer());
        let table_ref =
            crate::key_bindings::key_bindings_get_table_ref(c"root".as_ptr(), 1).unwrap();
        (*c).keytable_ref = Some(table_ref);
        Wired { c }
    }
}

/// What [`wire`] handed a client, taken back down again: the handle goes
/// first so nothing outlives the tables' own teardown.
struct Wired {
    c: *mut client,
}

impl Drop for Wired {
    fn drop(&mut self) {
        unsafe {
            (*self.c).keytable_ref = None;
        }
    }
}

/// The key tables a test created, taken back down again when the guard goes
/// away — but only those that did not exist before, and only after the test's
/// clients have left the server list, since removing a table re-homes every
/// client still using it.
struct TakenTables {
    names: Vec<CString>,
}

impl TakenTables {
    fn new() -> TakenTables {
        TakenTables { names: Vec::new() }
    }

    /// Records `name` for removal unless a table by that name already exists.
    fn claim(&mut self, name: &str) {
        let cs = CString::new(name).expect("a table name has no NUL");
        unsafe {
            if key_bindings_get_table(cs.as_ptr(), 0).is_null() {
                self.names.push(cs);
            }
        }
    }
}

impl Drop for TakenTables {
    fn drop(&mut self) {
        for name in &self.names {
            unsafe { key_bindings_remove_table(name.as_ptr()) };
        }
    }
}

/// Everything the server has recorded so far, oldest first. Entries
/// accumulate across the whole test binary, so assertions look for their own
/// wording rather than exact contents.
unsafe fn logged_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Two registered sessions, `$one` and `$two` in name order, each carrying
/// one linked window with one pane. Nothing else about them is special.
struct SwitchFixture {
    registry: Registry,
    one: Session,
    two: Session,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    wl_one: *mut winlink,
    wl_two: *mut winlink,
}

impl SwitchFixture {
    fn new() -> SwitchFixture {
        let mut f = SwitchFixture {
            registry: Registry::new(),
            one: Session::new(1, "one"),
            two: Session::new(2, "two"),
            windows: Vec::new(),
            panes: Vec::new(),
            wl_one: null_mut(),
            wl_two: null_mut(),
        };
        f.registry.add_session(&mut f.one);
        f.registry.add_session(&mut f.two);
        let mut w_one = Window::new(10, "one-win", 80, 24);
        let mut p_one = Pane::new(10, 80, 24, 100);
        w_one.add_pane(&mut p_one);
        f.registry.add_window(&mut w_one);
        f.registry.add_pane(&mut p_one);
        f.wl_one = link(&mut f.one, &mut w_one, 0);
        f.windows.push(w_one);
        f.panes.push(p_one);
        let mut w_two = Window::new(11, "two-win", 80, 24);
        let mut p_two = Pane::new(11, 80, 24, 100);
        w_two.add_pane(&mut p_two);
        f.registry.add_window(&mut w_two);
        f.registry.add_pane(&mut p_two);
        f.wl_two = link(&mut f.two, &mut w_two, 0);
        f.windows.push(w_two);
        f.panes.push(p_two);
        f
    }

    fn one(&mut self) -> *mut session {
        self.one.ptr()
    }

    fn two(&mut self) -> *mut session {
        self.two.ptr()
    }
}

impl Drop for SwitchFixture {
    fn drop(&mut self) {
        unlink(&mut self.one, self.wl_one);
        unlink(&mut self.two, self.wl_two);
    }
}

/// Runs the parsed command an item carries through the switch-client entry's
/// exec hook, the way the command queue would call it.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        assert!(::core::ptr::eq(e, &cmd_switch_client_entry));
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// Builds a parsed `switch-client` line whose command runs as `c` — both the
/// item's own client and its target client, which is what a key binding
/// hands the hook.
unsafe fn item_as(c: *mut client, line: &CStr) -> Item {
    unsafe {
        let mut item = Item::new().with_args(line);
        item.set_client(c);
        cmdq_set_target_client(item.ptr(), c);
        item
    }
}

/// Fills the queue's current state from `wl`, as a resolved target upstream
/// of the hook would have left it.
unsafe fn aim(item: &mut Item, wl: *mut winlink) {
    unsafe {
        let st = cmdq_get_state(&*item.ptr());
        cmd_find_from_winlink(&mut (*st).current, wl, 0);
    }
}

#[test]
fn the_entry_advertises_switch_client_and_the_file_declares_its_constants() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_switch_client_entry;
        assert_eq!((*e).name.to_string_lossy(), "switch-client");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "switchc"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "c:EFlnO:pt:rT:Z");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]"
        );
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 0);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, CMD_READONLY | CMD_CLIENT_CFLAG);

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
        assert_eq!(SUBJ_NORMAL, CMD_RETURN_NORMAL);
        assert_eq!(CMD_RETURN_WAIT, 1);
        assert_eq!(CMD_RETURN_STOP, 2);
        assert_eq!(SUBJ_ERROR, CMD_RETURN_ERROR);
        assert_eq!(SORT_ACTIVITY, 0);
        assert_eq!(SORT_CREATION, 1);
        assert_eq!(SORT_INDEX, 2);
        assert_eq!(SORT_MODIFIER, 3);
        assert_eq!(SORT_NAME, 4);
        assert_eq!(SORT_ORDER, 5);
        assert_eq!(SORT_SIZE, 6);
        assert_eq!(SORT_Z, 7);
        assert_eq!(SORT_END, 8);

        assert_eq!(CMD_FIND_PREFER_UNATTACHED, 0x1);
        assert_eq!(CMDQ_STATE_REPEAT, 0x1);
        assert_eq!(CMD_READONLY, 0x2);
        assert_eq!(CMD_CLIENT_CFLAG, 0x8);
        assert_eq!(CLIENT_READONLY, 0x800);
        assert_eq!(CLIENT_IGNORESIZE, 0x20000);
    }
}

#[test]
fn parsing_resolves_the_name_the_alias_and_their_letters() {
    let _guard = globals();
    unsafe {
        let full = Args::parse(c"switch-client -c /tmp -EFlnprZ -O index -t two -T tbl");
        assert!(::core::ptr::eq(
            (*full.cmd()).entry,
            &cmd_switch_client_entry
        ));
        let a = full.ptr();
        for flag in *b"EFlnprZ" {
            assert_eq!(args_has(&*a, flag), 1, "-{flag} missing");
        }
        assert_eq!(args_count(&*a), 0);
        assert_eq!(seen(args_get(&*a, b'c')), "/tmp");
        assert_eq!(seen(args_get(&*a, b'O')), "index");
        assert_eq!(seen(args_get(&*a, b't')), "two");
        assert_eq!(seen(args_get(&*a, b'T')), "tbl");

        let aliased = Args::parse(c"switchc -Z");
        assert!(::core::ptr::eq(
            (*aliased.cmd()).entry,
            &cmd_switch_client_entry
        ));
        assert_eq!(args_has(&*aliased.ptr(), b'Z'), 1);

        let mut unknown = cmd_parse_from_string(c"switch-client -q".as_ptr(), null_mut());
        assert_eq!(unknown.status, CMD_PARSE_ERROR);
        let err = unknown.take_error();
        assert!(err.contains("unknown flag"), "{err}");
        assert!(err.contains("-q"), "{err}");

        let mut hungry = cmd_parse_from_string(c"switch-client -t".as_ptr(), null_mut());
        assert_eq!(hungry.status, CMD_PARSE_ERROR);
        let err = hungry.take_error();
        assert!(err.contains("expects an argument"), "{err}");
        assert!(err.contains("-t"), "{err}");
    }
}

/// Naming a session that does not exist fails the find and reports it by
/// name through the item's attached client.
/// `-T` naming a table nobody created refuses after the find has succeeded,
/// leaving the client, its session and its key table alone.
/// `-T` naming an existing table installs it onto the client, replaces the
/// old table, and stops there: no session change follows.
/// `-r` on a plain client skips the uid check entirely, grants the read-only
/// flags, and carries on into the switch itself.
/// `-r` against a client already marked read-only consults the peer's uid:
/// a peer running as somebody else refuses the demotion, and nothing moves.
/// `-r` on a read-only client whose peer runs as this user clears the flags
/// and proceeds.
/// `-O` with a name no sort order answers refuses once the find has
/// succeeded; a named order rides along quietly.
/// `-n` and `-p` walk to the neighbouring registered session in sorted
/// order; with no session behind the target client they refuse instead of
/// guessing.
/// `-l` switches to the last session while it is still alive, and refuses
/// through the client once it is not.
/// The plain form makes the targeted pane active — stacking the old active
/// pane — through the zoom push/pop dance, and records pane, winlink and
/// session in the queue's current state.
/// With no client behind the item the plain form answers normal at once,
/// before anything would be redrawn or re-homed.
#[test]
fn plain_switch_without_a_client_answers_normal_at_once() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(3, "quiet");
    registry.add_session(&mut s);
    let mut w = Window::new(3, "win", 80, 24);
    let mut p = Pane::new(3, 80, 24, 100);
    w.add_pane(&mut p);
    registry.add_window(&mut w);
    registry.add_pane(&mut p);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item = Item::new().with_args(c"switch-client");
        aim(&mut item, wl);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(session_get_curw(s.ptr()), wl);
        assert!(crate::cmd::cmdq_get_client(&*item.ptr()).is_null());
    }
    unlink(&mut s, wl);
}

#[test]
fn switch_client_missing_session_returns_error() {
    let _guard = globals();
    let mut _f = SwitchFixture::new();
    let mut c = zeroed_client();
    let _wired = unsafe { wire(&raw mut *c) };
    unsafe {
        let mut item = item_as(&raw mut *c, c"switch-client -t no-such-session");
        assert_eq!(run(&mut item), CMD_RETURN_ERROR);
    }
}

#[test]
fn switch_client_key_table_branch() {
    let _guard = globals();
    let mut _f = SwitchFixture::new();
    let mut c = zeroed_client();
    let _wired = unsafe { wire(&raw mut *c) };
    unsafe {
        let mut item_bad = item_as(&raw mut *c, c"switch-client -T non-existent-table");
        assert_eq!(run(&mut item_bad), CMD_RETURN_ERROR);

        let mut item_good = item_as(&raw mut *c, c"switch-client -T root");
        assert_eq!(run(&mut item_good), CMD_RETURN_NORMAL);
        assert_eq!(seen(key_table_name((*c).keytable())), "root");
    }
}

#[test]
fn switch_client_read_only_checks() {
    let _guard = globals();
    let mut f = SwitchFixture::new();
    let mut c = zeroed_client();
    let _wired = unsafe { wire(&raw mut *c) };
    unsafe {
        // Runs as this user, which the read-only check allows.
        (*peer_ptr(&c.peer)).uid = getuid();
        c.session = f.one();

        let mut item_ro = item_as(&raw mut *c, c"switch-client -r");
        assert_eq!(run(&mut item_ro), CMD_RETURN_NORMAL);
        assert_ne!(c.flags & CLIENT_READONLY as u64, 0);

        let mut item_unro = item_as(&raw mut *c, c"switch-client -r");
        assert_eq!(run(&mut item_unro), CMD_RETURN_NORMAL);
        assert_eq!(c.flags & CLIENT_READONLY as u64, 0);

        // Make read-only again, then pretend peer is somebody else
        c.flags |= CLIENT_READONLY as u64;
        // Runs as somebody else, which the read-only check refuses.
        (*peer_ptr(&c.peer)).uid = getuid().wrapping_add(1);
        let mut item_denied = item_as(&raw mut *c, c"switch-client -r");
        assert_eq!(run(&mut item_denied), CMD_RETURN_ERROR);
    }
}

#[test]
fn switch_client_sort_order_and_navigation() {
    let _guard = globals();
    let mut f = SwitchFixture::new();
    let mut c = zeroed_client();
    let _wired = unsafe { wire(&raw mut *c) };
    unsafe {
        c.session = f.one();

        let mut item_bad_order = item_as(&raw mut *c, c"switch-client -O badorder");
        assert_eq!(run(&mut item_bad_order), CMD_RETURN_ERROR);
    }
}
