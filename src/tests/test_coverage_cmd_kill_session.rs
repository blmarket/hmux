//! Unit tests for [`crate::cmd::cmd_kill_session`], the exec hook behind the
//! `kill-session` command.
//!
//! The hook is reached exactly as the command queue reaches it, through the
//! entry's `exec` pointer with an item whose target find state has already
//! been resolved. Around it the tests pin the entry's metadata, the constants
//! this generated file declares and the parser hands back, and each of the
//! four ways the command picks its work: `-C`'s sweep of the alert flags over
//! every window of the target session, `-a`'s walk of every other session,
//! `-g`'s walk of the target's group, and the plain kill that falls out when
//! none of those apply.
//!
//! Every destruction here stops short of freeing anything the fixtures own:
//! `server_destroy_session` runs far enough to detach the clients it finds
//! — which is what the assertions read — and then `session_destroy` takes the
//! early exit a session with no current winlink gets, leaving the tree
//! membership, the option set and the environment exactly where they were. No
//! notification is queued, no reference dropped, no process signalled.

use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_kill_session::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_PANE, CMD_FIND_SESSION,
    CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT,
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL,
    MSG_EXEC, MSG_EXIT, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_TERMINFO, MSG_SHUTDOWN, MSG_VERSION, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH,
    PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, RB_NEGINF, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL,
    WINDOW_SILENCE, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
    cmd_kill_session_entry,
};
use crate::server::CLIENT_ALLREDRAWFLAGS;
use crate::server::CLIENT_EXIT;
use crate::server::client_get_last_session;
use crate::session::{
    session_alive, session_group_add, session_group_contains, session_group_new, session_groups,
};
use crate::session::{session_environ, session_options};
use crate::tests::test_fixtures::{
    Args, Clients, Item, Registry, Session, Window, globals, link, seen, unlink_all,
};
use crate::types::*;
use crate::window::{WINLINK_VISITED, winlinks_after, winlinks_first};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe { (cmd_kill_session_entry.exec)(&*item.cmd(), item.ptr()) }
}

/// Points the item's target find state at `s`, as a resolved `-t` would have
/// left it for the hook to pick up through
/// [`cmdq_get_target`](crate::cmd::cmdq_get_target).
unsafe fn aimed(item: &mut Item, s: *mut session) {
    unsafe {
        (*item.ptr()).target.set_session(s);
    }
}

/// A session group holding no sessions yet, sitting in the server's global
/// group tree for the length of a test so that [`session_group_contains`]
/// finds it. Its name stays null because nothing on the covered path reads
/// it; members join through the real [`session_group_add`].
struct Group(*mut session_group);

impl Group {
    fn new() -> Group {
        assert!(
            session_groups.map().is_empty(),
            "the group tree is not empty"
        );
        Group(unsafe { session_group_new(c"kill-session-coverage-group".as_ptr()) })
    }

    fn ptr(&mut self) -> *mut session_group {
        self.0
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        session_groups.map().remove(c"kill-session-coverage-group");
    }
}

#[test]
fn the_entry_advertises_kill_session_and_the_switches_its_hook_reads() {
    unsafe {
        let e = &raw const cmd_kill_session_entry;
        assert_eq!((*e).name.to_string_lossy(), "kill-session");
        assert!((*e).alias.is_none());
        assert_eq!((*e).args.template.to_string_lossy(), "aCgt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());
        assert_eq!((*e).usage.to_string_lossy(), "[-aCg] [-t target-session]");

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, 0);
    }
}

#[test]
fn the_declared_constants_pin_the_values_the_command_table_and_hook_read() {
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

    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_CLIENTPID, 107);
    assert_eq!(MSG_IDENTIFY_ENVIRON, 105);
    assert_eq!(MSG_IDENTIFY_TERMINFO, 112);
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_DETACHKILL, 202);
    assert_eq!(MSG_EXIT, 203);
    assert_eq!(MSG_SHUTDOWN, 210);
    assert_eq!(MSG_EXEC, 217);

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

    assert_eq!(WINDOW_BELL, 0x1);
    assert_eq!(WINDOW_ACTIVITY, 0x2);
    assert_eq!(WINDOW_SILENCE, 0x4);
    assert_eq!(
        WINDOW_ALERTFLAGS,
        WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE
    );

    assert_eq!(WINLINK_BELL, 0x1);
    assert_eq!(WINLINK_ACTIVITY, 0x2);
    assert_eq!(WINLINK_SILENCE, 0x4);
    assert_eq!(
        WINLINK_ALERTFLAGS,
        WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE
    );

    assert_eq!(RB_NEGINF, -1);
}

#[test]
fn parsing_resolves_the_name_and_hands_over_every_declared_switch() {
    let _guard = globals();
    unsafe {
        let plain = Args::parse(c"kill-session");
        assert!(
            ::core::ptr::eq((*plain.cmd()).entry, &cmd_kill_session_entry),
            "the bare name did not resolve"
        );
        assert_eq!(args_has(&*plain.ptr(), b'C'), 0);
        assert_eq!(args_has(&*plain.ptr(), b'a'), 0);
        assert_eq!(args_has(&*plain.ptr(), b'g'), 0);
        assert_eq!(args_has(&*plain.ptr(), b't'), 0);

        let all = Args::parse(c"kill-session -Ca -g -t 0");
        assert!(::core::ptr::eq((*all.cmd()).entry, &cmd_kill_session_entry));
        assert_eq!(args_has(&*all.ptr(), b'C'), 1);
        assert_eq!(args_has(&*all.ptr(), b'a'), 1);
        assert_eq!(args_has(&*all.ptr(), b'g'), 1);
        assert_eq!(args_has(&*all.ptr(), b't'), 1);
        assert_eq!(seen(args_get(&*all.ptr(), b't')), "0");
    }
}

#[test]
fn clearing_alerts_sweeps_every_window_of_the_target_and_redraws_its_clients() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(30, "alerted");
    let mut first = Window::new(31, "first", 80, 24);
    let mut second = Window::new(32, "second", 80, 24);
    let watcher = clients.add("watcher", 80, 24);
    let drifter = clients.add("drifter", 80, 24);
    unsafe {
        let wl0 = link(&mut s, &mut first, 0);
        let wl1 = link(&mut s, &mut second, 1);
        (*first.ptr()).flags |= WINDOW_ALERTFLAGS;
        (*second.ptr()).flags |= WINDOW_ALERTFLAGS;
        (*wl0).flags |= WINLINK_ALERTFLAGS | WINLINK_VISITED;
        (*wl1).flags |= WINLINK_SILENCE;
        (*watcher).session = s.ptr();
        assert_eq!((*drifter).session, ::core::ptr::null_mut::<session>());

        let mut item = Item::new().with_args(c"kill-session -C");
        aimed(&mut item, s.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        for wl in [wl0, wl1] {
            assert_eq!((*(*wl).window()).flags & WINDOW_ALERTFLAGS, 0);
            assert_eq!((*wl).flags & WINLINK_ALERTFLAGS, 0);
        }
        assert_eq!((*wl0).flags & WINLINK_VISITED, WINLINK_VISITED);
        assert_eq!((*wl1).flags & WINLINK_SILENCE, 0);
        assert_eq!(
            (*watcher).flags & CLIENT_ALLREDRAWFLAGS,
            CLIENT_ALLREDRAWFLAGS
        );
        assert_eq!((*drifter).flags, 0);

        let walked = winlinks_first(&raw mut (*s.ptr()).windows);
        assert_eq!(walked, wl0);
        assert_eq!(winlinks_after(walked), wl1);
        assert_eq!(
            winlinks_after(winlinks_after(walked)),
            null_mut::<winlink>()
        );
    }
    unlink_all(&mut s);
}

#[test]
fn the_plain_run_detaches_only_the_target_sessions_clients_and_leaves_it_standing() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut registry = Registry::new();
    let mut home = Session::new(33, "home");
    registry.add_session(&mut home);
    let going = clients.add("going", 80, 24);
    unsafe {
        (*going).session = home.ptr();

        let mut item = Item::new().with_args(c"kill-session");
        aimed(&mut item, home.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert!((*going).session.is_null());
        assert!(client_get_last_session(going).is_null());
        assert_ne!((*going).flags & CLIENT_EXIT as uint64_t, 0);
        assert_eq!(session_options(home.ptr()), home.options());
        assert_eq!(session_environ(home.ptr()), home.environ());
        assert_eq!(session_alive(home.ptr()), 1);
    }
}

#[test]
fn killing_all_other_sessions_spares_exactly_the_target() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut registry = Registry::new();
    let mut away = Session::new(34, "away");
    let mut distant = Session::new(35, "distant");
    let mut home = Session::new(36, "home");
    registry.add_session(&mut away);
    registry.add_session(&mut distant);
    registry.add_session(&mut home);
    let kept = clients.add("kept", 80, 24);
    let evicted = clients.add("evicted", 80, 24);
    unsafe {
        (*kept).session = home.ptr();
        (*evicted).session = away.ptr();

        let mut item = Item::new().with_args(c"kill-session -a");
        aimed(&mut item, home.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert!((*evicted).session.is_null());
        assert_ne!((*evicted).flags & CLIENT_EXIT as uint64_t, 0);
        assert_eq!((*kept).session, home.ptr());
        assert_eq!((*kept).flags & CLIENT_EXIT as uint64_t, 0);
        assert_eq!(session_alive(home.ptr()), 1);
        assert_eq!(session_alive(distant.ptr()), 1);
    }
}

#[test]
fn the_group_flag_takes_every_member_of_the_targets_group_and_no_other() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut registry = Registry::new();
    let mut one = Session::new(37, "one");
    let mut two = Session::new(38, "two");
    let mut stranger = Session::new(39, "stranger");
    registry.add_session(&mut one);
    registry.add_session(&mut two);
    registry.add_session(&mut stranger);
    let mut group = Group::new();
    let first = clients.add("first", 80, 24);
    let second = clients.add("second", 80, 24);
    let bystander = clients.add("bystander", 80, 24);
    unsafe {
        session_group_add(group.ptr(), one.ptr());
        session_group_add(group.ptr(), two.ptr());
        (*first).session = one.ptr();
        (*second).session = two.ptr();
        (*bystander).session = stranger.ptr();
        assert_eq!(session_group_contains(one.ptr()), group.ptr());
        assert!(session_group_contains(stranger.ptr()).is_null());

        let mut item = Item::new().with_args(c"kill-session -g");
        aimed(&mut item, one.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert!((*first).session.is_null());
        assert_ne!((*first).flags & CLIENT_EXIT as uint64_t, 0);
        assert!((*second).session.is_null());
        assert_ne!((*second).flags & CLIENT_EXIT as uint64_t, 0);
        assert_eq!((*bystander).session, stranger.ptr());
        assert_eq!((*bystander).flags & CLIENT_EXIT as uint64_t, 0);
        assert_eq!(session_group_contains(one.ptr()), group.ptr());
        assert_eq!(session_alive(one.ptr()), 1);
        assert_eq!(session_alive(stranger.ptr()), 1);
    }
}

#[test]
fn the_group_flag_without_a_group_falls_through_to_killing_the_target_alone() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut registry = Registry::new();
    let mut solo = Session::new(40, "solo");
    registry.add_session(&mut solo);
    let going = clients.add("going", 80, 24);
    unsafe {
        (*going).session = solo.ptr();
        assert!(session_group_contains(solo.ptr()).is_null());

        let mut item = Item::new().with_args(c"kill-session -g");
        aimed(&mut item, solo.ptr());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert!((*going).session.is_null());
        assert_ne!((*going).flags & CLIENT_EXIT as uint64_t, 0);
        assert_eq!(session_alive(solo.ptr()), 1);
    }
}
