use super::*;

use crate::proc::PEER_BAD;
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, unlink, zeroed,
};
use ::core::ffi::{CStr, c_int};

/// Runs the item's parsed command through the entry's exec hook, the way
/// the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &cmd_new_window_entry;
        item.with_command(|command, item| (e.exec)(command, item))
    }
}

/// A session holding one window of one pane at index 0, with the server's
/// trees carrying both, taken back down again on the way out.
struct One {
    _registry: Registry,
    session: Session,
    _window: Window,
    _pane: Pane,
    wl: *mut winlink,
}

impl One {
    fn new() -> One {
        let mut registry = Registry::new();
        let mut session = Session::new(0, "0");
        let mut window = Window::new(900_100, "keep", 80, 24);
        let mut pane = Pane::new(950_100, 80, 24, 100);
        window.add_pane(&mut pane);
        registry.add_session(&mut session);
        registry.add_window(&mut window);
        let wl = link(&mut session, &mut window, 0);
        One {
            _registry: registry,
            session,
            _window: window,
            _pane: pane,
            wl,
        }
    }
}

impl Drop for One {
    fn drop(&mut self) {
        let wl = self.wl;
        unlink(&mut self.session, wl);
    }
}

/// Empties the server's message log, so the line a refusal records is the
/// only one in it: letting go of whatever was recorded before is the same
/// reset the shared fixtures give the server's trees.
fn reset_message_log() {
    crate::tests::test_fixtures::reset_message_log();
}

/// The lines the server has recorded since the log was reset.
fn recorded_messages() -> Vec<String> {
    crate::tests::test_fixtures::logged_messages()
}

/// Drives the hook over a session of one window at index 0, with the
/// target find state resolved to no winlink at all and to `idx` — which is
/// what `cmd_find` leaves behind for an offset target such as `-t 0:+1`
/// under `CMD_FIND_WINDOW_INDEX`. Answers the return value and the lines
/// the run recorded.
fn drive(line: &'static CStr, idx: c_int) -> (cmd_retval, Vec<String>) {
    unsafe {
        let mut clients = Clients::new();
        let mut peer = zeroed::<tmuxpeer>();
        peer.flags |= PEER_BAD;
        let caller = clients.add("caller", 80, 24);
        (*caller).peer = Some(crate::proc::PeerRef::new(*peer));

        let mut one = One::new();
        let mut item = Item::with_client().with_args(line);
        let p = item.ptr();
        item.set_client(caller);
        (*p).set_target_client(
            caller
                .as_ref()
                .and_then(crate::server::client_ref_of)
                .as_ref(),
        );
        let mut target = *Box::new(cmd_find_state::default());
        target.set_session(one.session.ptr().as_ref());
        target.idx = idx;
        (*p).target = target.clone();
        (*p).source = target.clone();
        *(item.read()).current() = target.clone();

        reset_message_log();
        let retval = run(&mut item);
        let messages = recorded_messages();

        assert_eq!((*caller).retval, 1, "the caller was told it failed");
        let s = one.session.ptr();
        assert!(
            (*s).windows
                .get(&0)
                .map(Box::as_ref)
                .is_some_and(|link| core::ptr::eq(link, one.wl)),
            "the window stayed where it was linked"
        );
        assert!(
            (*s).windows.get(&1).is_none(),
            "nothing was shuffled up out of the way"
        );
        (retval, messages)
    }
}

#[test]
fn a_shuffle_with_no_winlink_to_shuffle_falls_back_to_the_targets_index() {
    let _guard = globals();
    ensure_reactor();
    {
        for line in [c"new-window -a -d", c"new-window -b -d"] {
            let (retval, messages) = drive(line, 0);
            assert_eq!(retval, CMD_RETURN_ERROR, "{line:?}");
            assert_eq!(messages.len(), 1, "{messages:?}");
            let message = messages.into_iter().next().unwrap_or_default();
            assert!(
                message.contains("create window failed: index 0 in use"),
                "the target's own index was spawned at, {line:?}: {message}"
            );
        }
    }
}
