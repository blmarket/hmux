use super::*;
use crate::types::client;
use crate::proc::tmuxpeer;
use crate::cfg::{cfg_print_causes, replace_configuration_finished};

use crate::environ::new_environment_box;
use crate::key_bindings::{key_bindings_get_table_ref, key_bindings_remove_table};
use crate::server::client_get_last_session;

use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Target, Window, ensure_reactor, globals, link,
    unlink_all, zeroed,
};
use crate::window::CLIENT_EXIT;
use crate::window::window_active_pane;
use ::std::ffi::CString;

/// The globals turn every test here takes, with the server's message log
/// emptied first: the refusal paths append to it, and what earlier tests
/// logged is of no interest here. Only ever reached under the [`globals`]
/// lock.
fn attached_globals() -> crate::tests::test_fixtures::GlobalsGuard {
    let guard = globals();
    crate::tests::test_fixtures::reset_message_log();
    guard
}

/// The `root` key table, which every attach here creates behind the test's
/// back — [`wire_up`] takes a reference to it and `server_client_set_key_table`
/// looks it up by the session's `key-table` option — taken back down when
/// the guard goes away. It goes out of scope after the test's clients have
/// left the server list, since removing a table re-homes every client still
/// using it.
struct RootTable;

impl Drop for RootTable {
    fn drop(&mut self) {
        unsafe { key_bindings_remove_table(c"root") };
    }
}

/// Configuration completion for the length of a test, back to what it was afterwards
/// even if the test panics.
struct ConfigFinished(bool);

impl ConfigFinished {
    fn new() -> ConfigFinished {
        ConfigFinished(replace_configuration_finished(true))
    }
}

impl Drop for ConfigFinished {
    fn drop(&mut self) {
        replace_configuration_finished(self.0);
    }
}

/// The private per-client state the attach paths read behind the scenes: an
/// environment for the nested check and `update-environment`, a tty name
/// for the nested check and the terminal open, a peer for the read-only uid
/// check, and an owned key table so `set_key_table` has something to
/// replace. The wiring outlives the call under test and undoes itself.
struct Wired {
    c: *mut client,
    #[allow(dead_code)]
    ttyname: CString,
}

unsafe fn wire_up(c: *mut client) -> Wired {
    unsafe {
        (*c).environ = Some(new_environment_box());
        let ttyname = CString::new("/dev/pts/attach").expect("no NUL");
        (*c).ttyname = Some(ttyname.clone());
        (*c).peer = Some(crate::proc::PeerRef::new(*zeroed::<tmuxpeer>()));
        let table_ref = key_bindings_get_table_ref(c"root", 1).unwrap();
        (*c).keytable_ref = Some(table_ref);
        Wired { c, ttyname }
    }
}

impl Wired {
    /// Mutably borrows the peer the client owns.
    ///
    /// # Safety
    /// The client must remain alive and other access to its peer must not overlap.
    unsafe fn peer_mut(&mut self) -> std::cell::RefMut<'_, tmuxpeer> {
        unsafe { (*self.c).peer_handle().borrow_mut() }
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
/// whole test binary, so assertions look for their own wording.
fn logged_messages() -> Vec<String> {
    crate::tests::test_fixtures::logged_messages()
}

/// Hands cfg.rs's cause list to `cfg_print_causes`, which frees every
/// entry. With no client behind the item each cause only reaches
/// `log_debug`.
fn drain_config_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(&*item.ptr());
    }
}

/// A client that already has a session takes the first arm of the command,
/// where `-d` still detaches every *other* client of the session being
/// attached to, with `MSG_DETACH` as their exit message.
#[test]
fn switching_with_d_detaches_the_other_clients_of_the_target() {
    let _guard = attached_globals();
    ensure_reactor();
    let _root = RootTable;
    let mut t = Target::new(80, 24);
    let mut old = Session::new(9, "old");
    let mut list = Clients::new();
    let c = list.add("d-switcher", 80, 24);
    unsafe {
        let _wired = wire_up(c);
        (*c).set_attached_session(Some(old.handle()));
        let other = list.add("d-displaced", 80, 24);
        (*other).set_attached_session(Some(t.session_handle()));

        let mut item = Item::new();
        item.set_client(c);
        let item = item.targeting(&mut t);
        let rv = cmd_attach_session(&item.read(), None, 1, 0, 0, None, 0, None);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert!(
            (*c).attached_session()
                .is_some_and(|attached| attached.ptr_eq(t.session_handle()))
        );
        assert!(
            client_get_last_session(&*c)
                .is_some_and(|s| core::ptr::eq(&*s.as_session(), &*old.ptr()))
        );
        assert_eq!((*other).exit_type, CLIENT_EXIT_DETACH);
        assert_eq!((*other).exit_msgtype, MSG_DETACH);
        assert_eq!(
            (*other)
                .exit_session
                .as_deref()
                .expect("client text")
                .to_string_lossy()
                .into_owned(),
            "0"
        );
        assert_ne!((*other).flags & CLIENT_EXIT as u64, 0);
        assert_eq!((*c).flags & CLIENT_EXIT as u64, 0, "the attacher stays");
        assert_eq!(
            (*c).flags & CLIENT_ATTACHED as u64,
            0,
            "a switch does not re-attach"
        );
    }
}

/// `-x` differs from `-d` on that arm too only in the message the displaced
/// clients are given.
#[test]
fn switching_with_x_kills_the_other_clients_instead() {
    let _guard = attached_globals();
    ensure_reactor();
    let _root = RootTable;
    let mut t = Target::new(80, 24);
    let old = Session::new(9, "old");
    let mut list = Clients::new();
    let c = list.add("x-switcher", 80, 24);
    unsafe {
        let _wired = wire_up(c);
        (*c).set_attached_session(Some(old.handle()));
        let other = list.add("x-displaced", 80, 24);
        (*other).set_attached_session(Some(t.session_handle()));

        let mut item = Item::new();
        item.set_client(c);
        let item = item.targeting(&mut t);
        let rv = cmd_attach_session(&item.read(), None, 0, 1, 0, None, 0, None);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert!(
            (*c).attached_session()
                .is_some_and(|attached| attached.ptr_eq(t.session_handle()))
        );
        assert_eq!((*other).exit_type, CLIENT_EXIT_DETACH);
        assert_eq!((*other).exit_msgtype, MSG_DETACHKILL);
    }
}

/// A session whose current window has lost its last pane still resolves as
/// a target, and the command then takes the winlink-only arm: no pane is
/// made active and the queue's current state is filled in from the winlink
/// alone, which leaves it without a pane. Such a window is one "on the way
/// to destruction", which is why `recalculate_sizes` skips it too.
#[test]
fn a_target_window_without_panes_leaves_the_current_state_without_one() {
    let _guard = attached_globals();
    ensure_reactor();
    let _root = RootTable;
    let mut registry = Registry::new();
    let mut full = Session::new(11, "full");
    let mut solo = Session::new(12, "solo");
    registry.add_session(&mut full);
    registry.add_session(&mut solo);
    let mut w_full = Window::new(40, "full-win", 80, 24);
    let mut p_full = Pane::new(40, 80, 24, 100);
    w_full.add_pane(&mut p_full);
    registry.add_window(&mut w_full);
    let _wl_full = link(&mut full, &mut w_full, 0);
    let mut w_solo = Window::new(41, "solo-win", 80, 24);
    registry.add_window(&mut w_solo);
    let wl_solo = link(&mut solo, &mut w_solo, 0);
    let mut list = Clients::new();
    let c = list.add("solofan", 80, 24);
    unsafe {
        let _wired = wire_up(c);
        (*c).set_attached_session(Some(full.handle()));
        assert!(window_active_pane(&*w_solo.ptr()).is_none());

        let mut item = Item::new();
        item.set_client(c);
        let rv = cmd_attach_session(&item.read(), Some(c"solo"), 0, 0, 0, None, 1, None);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert!(
            (*c).attached_session()
                .is_some_and(|attached| attached.ptr_eq(solo.handle()))
        );
        assert!(core::ptr::eq((&*solo.ptr()).curw().unwrap(), wl_solo));

        let current = (*item.ptr()).current();
        assert_eq!(
            (*current).session().as_ref().map(|s| s.as_ptr()),
            Some(solo.ptr())
        );
        assert!(core::ptr::eq(
            (*current).winlink_ref().unwrap().get().unwrap(),
            wl_solo
        ));
        assert_eq!(
            (*current)
                .window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            w_solo.ptr()
        );
        assert!((*current).pane_list_ref().is_none());
    }
    unlink_all(&mut full);
    unlink_all(&mut solo);
}

/// A client with no session and no terminal behind it is refused by
/// `server_client_open`, whose complaint the command passes on with its own
/// wording. Nothing is attached.
#[test]
fn a_client_that_is_not_a_terminal_is_refused_the_attach() {
    let _guard = attached_globals();
    let _root = RootTable;
    let mut t = Target::new(80, 24);
    let mut list = Clients::new();
    let c = list.add("no-terminal", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let _wired = wire_up(c);

        let mut item = Item::new();
        item.set_client(c);
        let item = item.targeting(&mut t);
        let rv = cmd_attach_session(&item.read(), None, 0, 0, 0, None, 0, None);
        assert_eq!(rv, CMD_RETURN_ERROR);

        assert_eq!(
            logged_messages(),
            ["no-terminal message: open terminal failed: not a terminal"]
        );
        assert!((*c).attached_session().is_none());
        assert_eq!(
            crate::SessionAttachmentState::session_attached(&*t.session()),
            0
        );
    }
}

/// Once the configuration files have been read every attach ends by handing
/// the session it attached to `cfg_show_causes`, which with nothing pending
/// has nothing to say. The attach itself goes through either way.
#[test]
fn a_finished_config_hands_the_session_to_cfg_show_causes() {
    let _guard = attached_globals();
    ensure_reactor();
    let _root = RootTable;
    let mut t = Target::new(80, 24);
    let mut list = Clients::new();
    let c = list.add("cfgfan", 80, 24);
    unsafe {
        drain_config_causes();
        let _finished = ConfigFinished::new();
        (*c).flags |= CLIENT_CONTROL as u64;
        let _wired = wire_up(c);

        let mut item = Item::new();
        item.set_client(c);
        let item = item.targeting(&mut t);
        let rv = cmd_attach_session(&item.read(), None, 0, 0, 0, None, 0, None);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert!(
            (*c).attached_session()
                .is_some_and(|attached| attached.ptr_eq(t.session_handle()))
        );
        assert_eq!(
            crate::SessionAttachmentState::session_attached(&*t.session()),
            1
        );
    }
}
/// `-r` against a client that is already read-only and whose peer runs as
/// this user passes the uid check and goes on to attach, with the read-only
/// and ignore-size flags granted a second time.
#[test]
fn a_read_only_client_whose_peer_is_this_user_still_attaches() {
    let _guard = attached_globals();
    ensure_reactor();
    let _root = RootTable;
    let mut t = Target::new(80, 24);
    let mut list = Clients::new();
    let c = list.add("ro-peer", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_CONTROL as u64 | CLIENT_READONLY as u64;
        let mut wired = wire_up(c);
        wired.peer_mut().uid = getuid();

        let mut item = Item::new();
        item.set_client(c);
        let item = item.targeting(&mut t);
        let rv = cmd_attach_session(&item.read(), None, 0, 0, 1, None, 0, None);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert!(
            (*c).attached_session()
                .is_some_and(|attached| attached.ptr_eq(t.session_handle()))
        );
        assert_eq!(
            (*c).flags & (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64,
            (CLIENT_READONLY | CLIENT_IGNORESIZE) as u64
        );
    }
}
