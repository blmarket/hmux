//! Unit tests for [`crate::cmd::cmd_attach_session`] — the `attach-session`
//! command's entry metadata, its message-protocol and flag constants, and
//! every branch of [`cmd_attach_session`] the fixtures can reach without a
//! live daemon.
//!
//! The command is one of the deepest in the tree, so a note on what its tests
//! lean on. Every client that gets past the first checks is wired up by
//! [`wire_up`]: `server_client_check_nested` reads the client's environment,
//! `server_client_open` compares its tty name against the controlling
//! terminal, `server_client_set_key_table` replaces whatever key table the
//! client carries, and `recalculate_sizes` walks the server's client list for
//! every attach. The fresh-terminal half of an attach is reached with control
//! clients, which `server_client_open` waives and for whom the `MSG_READY`
//! send to the peer is skipped; the real `tty_open` path and the
//! "open terminal failed" refusal would need a working terminal, so they stay
//! uncovered here. Error refusals are reported through the server's message
//! log by clients carrying `CLIENT_ATTACHED`, because `file_error` would
//! otherwise try to open a stream to the (absent) peer. The key tables these
//! paths create behind the test's back are taken down again by
//! [`TakenTables`], after the client list has been emptied.

use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_attach_session::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_EXIT_DETACH, CLIENT_IGNORESIZE, CLIENT_READONLY,
    CMD_FIND_PANE, CMD_FIND_PREFER_UNATTACHED, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_READONLY,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_STARTSERVER, CMDQ_STATE_REPEAT, MSG_DETACH,
    MSG_DETACHKILL, MSG_READY, cmd_attach_session, cmd_attach_session_entry,
};
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_server_access::CLIENT_EXIT;
use crate::cmd::{cmdq_get_current, cmdq_get_state};
use crate::environ::{environ_entry_value, environ_find, environ_ptr, environ_set, environ_t};
use crate::ffi::getuid;
use crate::fmt_args;
use crate::key_bindings::{key_bindings_get_table, key_bindings_remove_table};
use crate::options::options_set_string;
use crate::proc::peer_ptr;
use crate::server::client_get_last_session;
use crate::server::message_log;
use crate::session::{session_attached, session_cwd, session_get_curw};
use crate::session::{session_set_cwd, winlink_of};
use crate::tests::test_fixtures::{
    Clients, Environ, Item, Pane, Registry, Session, Target, Window, ensure_reactor, globals, link,
    seen, unlink, unlink_all, zeroed,
};
use crate::types::*;
use crate::window::PaneStack;
use crate::window::window_get_active;
use crate::window::window_get_latest;
use crate::window::window_pane_stack_first;
use ::core::ffi::c_char;
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;

/// The key tables a test's attach paths created, taken back down again when
/// the guard goes away — but only those that did not exist before, and only
/// after the test's clients have left the server list, since removing a table
/// re-homes every client still using it.
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

/// The private per-client state the attach paths read behind the scenes: an
/// environment for the nested check and `update-environment`, a tty name for
/// the nested check and the terminal-open refusal, a peer for the read-only
/// uid check, and an owned key table so `set_key_table` has something to
/// replace. The wiring outlives the call under test and undoes itself.
struct Wired {
    c: *mut client,
    #[allow(dead_code)]
    ttyname: CString,
}

unsafe fn wire_up(c: *mut client) -> Wired {
    unsafe {
        (*c).environ = Some(Environ::new().owned());
        let ttyname = CString::new("/dev/pts/attach").expect("no NUL");
        (*c).ttyname = Some(ttyname.clone());
        (*c).peer = Some(zeroed::<tmuxpeer>());
        let table_ref =
            crate::key_bindings::key_bindings_get_table_ref(c"root".as_ptr(), 1).unwrap();
        (*c).keytable_ref = Some(table_ref);
        Wired { c, ttyname }
    }
}

impl Wired {
    /// The peer the client owns.
    fn peer(&self) -> *mut tmuxpeer {
        unsafe { peer_ptr(&(*self.c).peer) }
    }

    /// The environment the client carries.
    fn environ(&self) -> *mut environ_t {
        unsafe { environ_ptr(&(*self.c).environ) }
    }
}

impl Drop for Wired {
    fn drop(&mut self) {
        unsafe {
            (*self.c).keytable_ref = None;
        };
    }
}

/// Everything the server's message log holds. Entries accumulate across the
/// whole test binary, so assertions look for their own wording rather than
/// count lines.
unsafe fn logged_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Hands cfg.rs's cause list to `cfg_print_causes`, which frees every entry.
/// With no client behind the item each cause only reaches `log_debug`.
unsafe fn drain_config_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_attach_session_entry;
        assert_eq!((*e).name.to_bytes(), b"attach-session");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"attach"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-dErx] [-c working-directory] [-f flags] [-t target-session]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"c:dEf:rt:x");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        for flag in [&raw const (*e).source, &raw const (*e).target] {
            assert_eq!((*flag).flag, 0);
            assert_eq!((*flag).type_0, CMD_FIND_PANE);
            assert_eq!((*flag).flags, 0);
        }

        assert_eq!((*e).flags & CMD_STARTSERVER, CMD_STARTSERVER);
        assert_eq!((*e).flags & CMD_READONLY, CMD_READONLY);
        assert_eq!((*e).flags & !(CMD_STARTSERVER | CMD_READONLY), 0);
    }
}

#[test]
fn constants_used_by_the_attach_paths_match_upstream() {
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_DETACHKILL, 202);
    assert_eq!(MSG_READY, 207);

    assert_eq!(CMD_FIND_PREFER_UNATTACHED, 0x1);
    assert_eq!(CMDQ_STATE_REPEAT, 0x1);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);

    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(CLIENT_ATTACHED, 0x80);
    assert_eq!(CLIENT_READONLY, 0x800);
    assert_eq!(CLIENT_CONTROL, 0x2000);
    assert_eq!(CLIENT_IGNORESIZE, 0x20000);

    assert_eq!(CLIENT_EXIT_DETACH, 2);
}

/// With no session in the tree the command refuses at once. A config-file
/// command has no client behind it, so the complaint lands in cfg.rs's
/// private cause list, which nothing outside cfg.rs can read; the test pins
/// the return value and empties the list again. A client-backed item instead
/// reports through the server's message log, where the wording is checked.
#[test]
fn attach_without_sessions_reports_an_error_and_files_a_cause() {
    let _guard = globals();

    let mut item = Item::new().from_file(c"fixture.conf", 3);
    let rv = unsafe { cmd_attach_session(item.ptr(), null(), 0, 0, 0, null(), 0, null()) };
    assert_eq!(rv, CMD_RETURN_ERROR);
    unsafe { drain_config_causes() };

    let mut clients = Clients::new();
    let c = clients.add("no-sessions", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let _wired = wire_up(c);
        let mut item = Item::new();
        item.set_client(c);
        let rv = cmd_attach_session(item.ptr(), null(), 0, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(msgs.iter().any(|m| m.contains("no sessions")), "{msgs:?}");
    }
}

#[test]
fn attach_without_a_client_answers_normal_once_sessions_exist() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(0, "quiet");
    registry.add_session(&mut s);
    let mut item = Item::new();
    let rv = unsafe { cmd_attach_session(item.ptr(), null(), 0, 0, 0, null(), 0, null()) };
    assert_eq!(rv, CMD_RETURN_NORMAL);
}

#[test]
fn a_client_still_inside_tmux_is_refused() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("nested", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let wired = wire_up(c);
        environ_set(
            wired.environ(),
            c"TMUX".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"/tmp/tmux-fixture/0,default,1".as_ptr()],
        );
        let tty = wired.ttyname.as_bytes_with_nul();
        for (i, &b) in tty.iter().enumerate() {
            (*t.pane(0)).tty[i] = b as c_char;
        }

        let mut item = Item::new();
        item.set_client(c);
        let rv = cmd_attach_session(item.ptr(), null(), 0, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("sessions should be nested")),
            "{msgs:?}"
        );
        assert!((*c).session.is_null());
    }
}

/// Naming a session that does not exist fails the find and leaves both the
/// client and the target untouched.
#[test]
fn attach_to_a_missing_session_reports_it_by_name() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("seeker", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let _wired = wire_up(c);
        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), c"missing".as_ptr(), 0, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("can't find session: missing")),
            "{msgs:?}"
        );
        assert!((*c).session.is_null());
    }
}

/// The full fresh-attach path: no `-t`, so the target comes from the queue's
/// current state, and a control client, which skips the terminal open and the
/// `MSG_READY` send — what lets a fixture reach this path at all.
#[test]
fn an_unattached_client_attaches_to_the_current_target() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("fresh", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);
        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), null(), 0, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let s = t.session();
        assert_eq!((*c).session, s);
        assert!(client_get_last_session(c).is_null());
        assert_eq!((*c).flags & CLIENT_ATTACHED as u64, CLIENT_ATTACHED as u64);
        assert_eq!(session_attached(s), 1);
        assert_eq!(window_get_latest(t.window(0)), c);
        assert!(!(*c).keytable().is_null());

        let current = cmdq_get_current(item.ptr());
        assert_eq!((*current).session(), s);
        assert_eq!((*current).winlink(), t.winlink(0));
        assert_eq!((*current).pane(), t.pane(0));
    }
}

/// `-d` detaches every *other* client sitting on the target session, marking
/// each with the `MSG_DETACH` exit message and the session it lost.
#[test]
fn attaching_with_d_detaches_the_other_clients_of_the_target() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("d-main", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);
        let other = clients.add("d-other", 80, 24);
        (*other).session = t.session();

        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), null(), 1, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*other).exit_type, CLIENT_EXIT_DETACH);
        assert_eq!((*other).exit_msgtype, MSG_DETACH);
        assert_eq!(seen(cstr_ptr(&(*other).exit_session)), "0");
        assert_ne!((*other).flags & CLIENT_EXIT as u64, 0);
        assert_eq!((*c).session, t.session());
        assert_eq!((*c).flags & CLIENT_EXIT as u64, 0, "the attacher stays");
    }
}

/// `-x` differs from `-d` only in the message sent to the displaced clients:
/// `MSG_DETACHKILL` instead of `MSG_DETACH`.
#[test]
fn attaching_with_x_kills_the_other_clients_instead() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("x-main", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);
        let other = clients.add("x-other", 80, 24);
        (*other).session = t.session();

        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), null(), 0, 1, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*other).exit_type, CLIENT_EXIT_DETACH);
        assert_eq!((*other).exit_msgtype, MSG_DETACHKILL);
        assert_eq!(seen(cstr_ptr(&(*other).exit_session)), "0");
    }
}

/// `-r` against a client already marked read-only consults the peer's uid:
/// a peer running as somebody else refuses the promotion, and nothing is
/// attached or granted.
#[test]
fn read_only_clients_need_a_peer_running_as_this_user() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("readonly", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64 | CLIENT_READONLY as u64;
        let mut wired = wire_up(c);
        (*wired.peer()).uid = getuid().wrapping_add(1);
        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), null(), 0, 0, 1, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_ERROR);
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("client is read-only")),
            "{msgs:?}"
        );
        assert!((*c).session.is_null());
        assert_eq!((*c).flags & CLIENT_IGNORESIZE as u64, 0);
    }
}

/// A client not already read-only skips the uid check entirely, so `-r` both
/// attaches it and grants the read-only and ignore-size flags.
#[test]
fn read_only_marking_survives_when_the_peer_is_allowed() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("ro-ok", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let mut wired = wire_up(c);
        (*wired.peer()).uid = getuid();
        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), null(), 0, 0, 1, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*c).session, t.session());
        assert_eq!(
            (*c).flags & (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64,
            (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64
        );
    }
}

/// `-f` hands its comma-separated list to `server_client_set_flags`, which is
/// observable in the client's flags once the attach has gone through.
#[test]
fn the_f_flag_rewrites_the_clients_flag_set() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("fflagged", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);
        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(
            item.ptr(),
            null(),
            0,
            0,
            0,
            null(),
            0,
            c"read-only,ignore-size".as_ptr(),
        );
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*c).session, t.session());
        assert_eq!(
            (*c).flags & (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64,
            (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64
        );
    }
}

/// `-c` formats its argument against the target and installs the result as
/// the session's working directory, freeing the old one. The fixture's cwd
/// string is owned by the session fixture, so the test restores it afterwards.
/// The formatted cwd also merges the running command's name into the format
/// tree, which is why the item carries a command entry.
#[test]
fn the_c_flag_rewrites_the_sessions_working_directory() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("cwdfan", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);

        let sp = t.session();
        let orig = ::core::ffi::CStr::from_ptr(session_cwd(sp)).to_owned();
        session_set_cwd(sp, c"/attach-fixture".to_owned());

        let mut item = Item::new();
        item.set_client(c);
        (*item.cmd()).entry = &cmd_attach_session_entry;
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(
            item.ptr(),
            null(),
            0,
            0,
            0,
            c"/tmp/#{session_name}".as_ptr(),
            0,
            null(),
        );
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen(session_cwd(sp)), "/tmp/0");

        session_set_cwd(sp, orig);
    }
}

/// Two registered sessions with one window each, and a client attached to the
/// first, ready to be switched onto the second by naming it.
struct SwitchFixture {
    registry: Registry,
    one: Session,
    two: Session,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    wl_one: *mut winlink,
    wl_two: *mut winlink,
}

impl Drop for SwitchFixture {
    fn drop(&mut self) {
        self.wl_one = null_mut();
        self.wl_two = null_mut();
        unlink_all(&mut self.one);
        unlink_all(&mut self.two);
    }
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

/// Attaching a client that already has a session takes the other branch of
/// the command: no terminal open, `-E` skipping the update-environment pass,
/// the old session kept as `last_session`, and — without the repeat bit — the
/// key table re-chosen from the new session's `key-table` option.
#[test]
fn switching_sessions_keeps_last_session_but_honours_E() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    tables.claim("custom");
    let mut f = SwitchFixture::new();
    let mut clients = Clients::new();
    let c = clients.add("switcher", 80, 24);
    unsafe {
        let wired = wire_up(c);
        environ_set(
            wired.environ(),
            c"DISPLAY".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"unix:10".as_ptr()],
        );
        (*c).session = f.one();

        options_set_string(
            f.two.options(),
            c"key-table".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"custom".as_ptr()],
        );

        let mut item = Item::new();
        item.set_client(c);

        let st = cmdq_get_state(&*item.ptr());
        cmd_find_from_winlink(&mut (*st).current, f.wl_one, 0);

        let rv = cmd_attach_session(item.ptr(), c"two".as_ptr(), 0, 0, 0, null(), 1, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*c).session, f.two());
        assert_eq!(client_get_last_session(c), f.one());

        assert!(
            environ_find(&*f.two.environ(), c"DISPLAY".as_ptr()).is_none(),
            "the environment was updated despite -E"
        );
        assert_eq!(
            (*c).keytable(),
            key_bindings_get_table(c"custom".as_ptr(), 0)
        );

        let current = cmdq_get_current(item.ptr());
        assert_eq!((*current).session(), f.two());
        assert_eq!((*current).winlink(), f.wl_two);
    }
}

/// Without `-E` every variable named by the session's default
/// `update-environment` list is copied from the client's environment into the
/// session's; DISPLAY rides along here as the witness.
#[test]
fn switching_without_E_applies_the_update_environment_list() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut f = SwitchFixture::new();
    let mut clients = Clients::new();
    let c = clients.add("envswitch", 80, 24);
    unsafe {
        let wired = wire_up(c);
        environ_set(
            wired.environ(),
            c"DISPLAY".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"unix:10".as_ptr()],
        );
        (*c).session = f.one();

        let mut item = Item::new();
        item.set_client(c);
        let rv = cmd_attach_session(item.ptr(), c"two".as_ptr(), 0, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*c).session, f.two());
        let envent = environ_find(&*f.two.environ(), c"DISPLAY".as_ptr())
            .expect("DISPLAY was not carried over");
        assert_eq!(seen(environ_entry_value(envent)), "unix:10");
    }
}

/// A repeat (`CMDQ_STATE_REPEAT`, which lives on the queue's shared state
/// that `cmdq_get_flags` reads back) skips the key-table re-choice even though
/// the session itself still changes.
#[test]
fn repeat_state_leaves_the_key_table_alone() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    tables.claim("custom");
    let mut f = SwitchFixture::new();
    let mut clients = Clients::new();
    let c = clients.add("repeater", 80, 24);
    unsafe {
        let wired = wire_up(c);
        environ_set(
            wired.environ(),
            c"DISPLAY".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"unix:10".as_ptr()],
        );
        (*c).session = f.one();
        options_set_string(
            f.two.options(),
            c"key-table".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"custom".as_ptr()],
        );

        let mut item = Item::new();
        item.set_client(c);
        let st = cmdq_get_state(&*item.ptr());
        cmd_find_from_winlink(&mut (*st).current, f.wl_one, 0);
        (*st).flags = CMDQ_STATE_REPEAT;

        let home = (*c).keytable();
        let rv = cmd_attach_session(item.ptr(), c"two".as_ptr(), 0, 0, 0, null(), 1, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*c).session, f.two());
        assert_eq!((*c).keytable(), home);
    }
}

/// The whole flag surface at once, through the entry's own `.exec` pointer —
/// the very code the command queue would call: `-d` detaches the other client
/// of the target, `-r` grants its flags (the peer is this user), `-f` sets the
/// ignore-size flag, `-c` rewrites the cwd and `-t 0` names the session.
#[test]
fn exec_attaches_through_the_parsed_arguments() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("execfan", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let mut wired = wire_up(c);
        (*wired.peer()).uid = getuid();

        let other = clients.add("exec-other", 80, 24);
        (*other).session = t.session();

        let sp = t.session();
        let orig = ::core::ffi::CStr::from_ptr(session_cwd(sp)).to_owned();
        session_set_cwd(sp, c"/attach-fixture".to_owned());

        let mut item = Item::new().with_args(c"attach-session -d -r -f ignore-size -c /tmp -t 0");
        item.set_client(c);
        let mut item = item.targeting(&mut t);

        let cmd = item.cmd();
        let rv = (cmd_attach_session_entry.exec)(&*cmd, item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!((*c).session, t.session());
        assert_eq!(
            (*c).flags & (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64,
            (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64
        );
        assert_eq!((*other).exit_msgtype, MSG_DETACH);
        assert_eq!(seen(session_cwd(sp)), "/tmp");

        session_set_cwd(sp, orig);
    }
}

/// An argumentless `attach-session` from nowhere — no client, one session in
/// the tree — answers normal without touching anything.
#[test]
fn exec_without_a_client_or_arguments_answers_normal() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(3, "bare");
    registry.add_session(&mut s);
    let mut item = Item::new().with_args(c"attach-session");
    let cmd = item.cmd();
    let rv = unsafe { (cmd_attach_session_entry.exec)(&*cmd, item.ptr()) };
    assert_eq!(rv, CMD_RETURN_NORMAL);
}

/// Naming a pane explicitly makes that pane the window's active one, stacks
/// the previously active pane as last-visited, and leaves the current state
/// naming the pane the attach went through.
#[test]
fn a_pane_target_makes_that_pane_active() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut registry = Registry::new();
    let mut s = Session::new(4, "sess");
    registry.add_session(&mut s);
    let mut w = Window::new(20, "win", 80, 24);
    let mut p0 = Pane::new(30, 80, 24, 100);
    let mut p1 = Pane::new(31, 80, 24, 100);
    w.add_pane(&mut p0);
    w.add_pane(&mut p1);
    registry.add_window(&mut w);
    registry.add_pane(&mut p0);
    registry.add_pane(&mut p1);
    let wl0 = link(&mut s, &mut w, 0);
    let mut clients = Clients::new();
    let c = clients.add("panefan", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);

        assert_eq!(window_get_active(w.ptr()), p0.ptr());

        let mut item = Item::new();
        item.set_client(c);
        let st = cmdq_get_state(&*item.ptr());
        cmd_find_from_winlink(&mut (*st).current, wl0, 0);

        let rv = cmd_attach_session(
            item.ptr(),
            c"sess:0.%31".as_ptr(),
            0,
            0,
            0,
            null(),
            0,
            null(),
        );
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!(window_get_active(w.ptr()), p1.ptr());
        assert_eq!(
            window_pane_stack_first(w.ptr(), PaneStack::LastUsed),
            p0.ptr()
        );
        assert_eq!((*c).session, s.ptr());

        let current = cmdq_get_current(item.ptr());
        assert_eq!((*current).pane(), p1.ptr());
        assert_eq!((*current).winlink(), wl0);

        unlink(&mut s, wl0);
    }
}

/// Naming another window of the target session moves the session's current
/// winlink onto it, pushes the previous one onto the last-visited stack, and
/// records index, winlink and pane in the queue's current state.
#[test]
fn a_window_target_moves_the_sessions_current_window() {
    let _guard = globals();
    ensure_reactor();
    let mut tables = TakenTables::new();
    tables.claim("root");
    let mut t = Target::new(80, 24);
    t.add_window(1, 80, 24);
    let mut clients = Clients::new();
    let c = clients.add("winfan", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);

        let s = t.session();
        assert_eq!(session_get_curw(s), t.winlink(0));

        let mut item = Item::new();
        item.set_client(c);
        let mut item = item.targeting(&mut t);
        let rv = cmd_attach_session(item.ptr(), c"0:1.%1".as_ptr(), 0, 0, 0, null(), 0, null());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!(session_get_curw(s), t.winlink(1));
        assert_eq!(winlink_of(s, (*s).lastw.first().copied()), t.winlink(0));
        assert_eq!((*c).session, s);

        let current = cmdq_get_current(item.ptr());
        assert_eq!((*current).winlink(), t.winlink(1));
        assert_eq!((*current).pane(), t.pane(1));
        assert_eq!((*current).idx, 1);
    }
}
