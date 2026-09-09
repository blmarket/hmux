use super::*;
use crate::cmdq::cmd_find_from_winlink;
use crate::reactor::Reactor;
use crate::session::sessions_empty;

use crate::tests::test_fixtures::{Item, Registry, Session, Window, ensure_reactor, globals, link};
use crate::window::{WinlinkRef, winlink_count};

/// Runs the item's parsed command through the `kill-window` entry's exec
/// hook, the way the command queue would.
fn run_kill(item: &mut Item) -> cmd_retval {
    unsafe { item.with_command(|command, item| (cmd_kill_window_entry.exec)(command, item)) }
}

/// The find state of `wl`, as resolution would leave it.
unsafe fn fs_of(wl: &WinlinkRef) -> cmd_find_state {
    let mut fs = cmd_find_state::default();
    unsafe { cmd_find_from_winlink(&mut fs, wl.get().unwrap(), 0) };
    fs
}

/// `-a` on a session that holds the target window at two indices and
/// nothing else: the "kill all the others" walk finds nothing to take, the
/// count of winlinks carrying the target's own window comes to two, and
/// that is the one shape in which the command kills the target window as
/// well.
///
/// Killing it empties the session, so the last `session_detach` hands over
/// to `server_destroy_session_group` and the session is destroyed — which
/// is why this is the only branch of the hook whose session does not
/// survive its own test. Everything it frees or unlinks is the command's
/// own.
#[test]
fn with_a_a_window_the_session_holds_twice_is_killed_as_well() {
    let _guard = globals();
    ensure_reactor();
    let mut registry = Registry::new();
    let mut s = Session::new(800_000, "twice");
    let mut w = Window::new(800_001, "twice", 80, 24);
    registry.add_session(&mut s);
    registry.add_window(&mut w);
    link(&mut s, &mut w, 0);
    let wl0 = WinlinkRef::new(s.reference(), 0).unwrap();
    link(&mut s, &mut w, 1);
    let wl1 = WinlinkRef::new(s.reference(), 1).unwrap();

    let mut item = Item::new().with_args(c"kill-window -a");
    unsafe {
        s.reference().set_cwd(c"/".to_owned());
        assert!(core::ptr::eq(
            s.handle()
                .curw()
                .unwrap()
                .get()
                .expect("the current link is live"),
            wl0.get().unwrap()
        ));
        assert!(
            wl0.get()
                .unwrap()
                .window_handle()
                .unwrap()
                .ptr_eq(w.handle())
        );
        assert!(
            wl1.get()
                .unwrap()
                .window_handle()
                .unwrap()
                .ptr_eq(w.handle())
        );
        {
            let mut item = item.item_mut();
            item.target = fs_of(&wl0);
            item.source = item.target.clone();
        }

        assert_eq!(run_kill(&mut item), CMD_RETURN_NORMAL);
        crate::reactor::current().run_once();

        assert_eq!(
            winlink_count(&s.handle().as_session().windows),
            0,
            "both of the window's winlinks went with it"
        );
        assert!(
            w.handle().as_window().winlinks.is_empty(),
            "and the window is linked nowhere"
        );
        assert!(sessions_empty(), "the emptied session was destroyed");
    }
}

/// `-a` on a session whose target window is held twice *beside* another
/// window: the walk takes the other window first, one winlink at a time,
/// and only then does the double-linked target qualify.
#[test]
fn with_a_the_other_windows_go_first_and_the_doubled_target_follows() {
    let _guard = globals();
    ensure_reactor();
    let mut registry = Registry::new();
    let mut s = Session::new(800_002, "twice-plus");
    let mut w = Window::new(800_003, "target", 80, 24);
    let mut other = Window::new(800_004, "other", 80, 24);
    registry.add_session(&mut s);
    registry.add_window(&mut w);
    registry.add_window(&mut other);
    link(&mut s, &mut w, 0);
    let wl0 = WinlinkRef::new(s.reference(), 0).unwrap();
    link(&mut s, &mut other, 1);
    link(&mut s, &mut w, 2);
    let wl2 = WinlinkRef::new(s.reference(), 2).unwrap();

    let mut item = Item::new().with_args(c"kill-window -a");
    unsafe {
        s.reference().set_cwd(c"/".to_owned());
        assert!(
            wl2.get()
                .unwrap()
                .window_handle()
                .unwrap()
                .ptr_eq(w.handle())
        );
        {
            let mut item = item.item_mut();
            item.target = fs_of(&wl0);
            item.source = item.target.clone();
        }

        assert_eq!(run_kill(&mut item), CMD_RETURN_NORMAL);
        crate::reactor::current().run_once();

        assert!(
            other.handle().as_window().winlinks.is_empty(),
            "the other window was let go first"
        );
        assert!(
            w.handle().as_window().winlinks.is_empty(),
            "and the doubled target followed it"
        );
        assert!(sessions_empty());
    }
}
