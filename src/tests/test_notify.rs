use super::*;
use crate::cmdq::CMDQ_STATE_NOHOOKS as STATE_NOHOOKS;
use crate::cmdq::CmdqListOps;
use crate::log::log_add_level;
use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};
use crate::tests::test_fixtures::{
    Args, Clients, Item, Pane, Registry, Session, Window, globals, link, seen_str, zeroed_client,
};
use ::core::ffi::CStr;
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// A session in the server's tree with one window linked into it and one
/// pane in that window, which is the least a hook needs to find a target,
/// plus a command-queue item to hang inserted hooks off.
///
/// The guard is declared last so that it is dropped after the registry has
/// taken the session back out of the server's tree: while it is held no
/// other test can see the tree half-emptied.
struct World {
    registry: Registry,
    session: Session,
    window: Window,
    pane: Pane,
    wl: crate::window::WinlinkRef,
    item: Item,
    queue: CmdqListRef,
    _guard: MutexGuard<'static, ()>,
}

impl World {
    /// A world whose window has a pane, so that a target found from
    /// nothing is a complete one.
    fn new() -> World {
        World::build(true)
    }

    /// A world whose window has no pane at all, so that a target found
    /// from nothing has no pane in it.
    fn without_a_pane() -> World {
        World::build(false)
    }

    fn build(with_pane: bool) -> World {
        let guard = globals();
        let registry = Registry::new();
        let mut session = Session::new(1, "hooks");
        let mut window = Window::new(1, "hooks", 80, 24);
        let mut pane = Pane::new(1, 80, 24, 100);
        if with_pane {
            window.add_pane(&mut pane);
        }
        link(&mut session, &mut window, 0);
        let wl = crate::window::WinlinkRef::new(session.handle().clone(), 0).unwrap();
        let mut w = World {
            registry,
            session,
            window,
            pane,
            wl,
            item: Item::new(),
            queue: CmdqListRef::empty(),
            _guard: guard,
        };
        w.registry.add_session(&mut w.session);
        w.item.queue_onto(&w.queue);
        w
    }

    fn item_mut(&self) -> std::cell::RefMut<'_, cmdq_item> {
        self.item.item_mut()
    }

    fn read(&self) -> std::cell::Ref<'_, cmdq_item> {
        self.item.read()
    }

    /// The commands of the items queued after the test's own item, in the
    /// order the queue would run them. An item's name is
    /// `[<command>/<pointer>]`, and the pointer is not the same twice.
    fn inserted(&self) -> Vec<String> {
        let anchor = self.item.read();
        let Some(at) = self.queue.items().position(|item| item.points_to(&anchor)) else {
            return Vec::new();
        };
        self.queue
            .items()
            .skip(at + 1)
            .map(|item| {
                let name = seen_str(item.read().name.as_deref());
                let command = name.trim_start_matches('[').split('/').next();
                command.expect("a name is never empty").to_string()
            })
            .collect()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let mut owner = self.session.handle().clone();
        unsafe {
            let session = owner.as_session_mut();
            if session.curw_idx == Some(self.wl.index()) {
                session.curw_idx = None;
            }
            crate::window::winlink_remove(&mut session.windows, self.wl.index());
        }
    }
}

/// The queue a notification is appended to, reached the only way there is
/// from outside `cmd_queue`: an item put on it says which queue it is on.
/// The probe item stays there, the way every notification a unit test
/// raises does, since nothing runs the queue.
unsafe fn global_queue(state: &CmdqStateRef) -> CmdqListRef {
    unsafe {
        let probe = Args::parse(c"display-message probe");
        cmdq_append(None, (probe.list_ref()).queue_items(Some(state)))
            .expect("the probe appends a queue item")
            .item()
            .queue
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap()
    }
}

/// A notification entry the way `notify_add` builds one, with a name and a
/// buffer name the callback frees again.
fn entry(name: &CStr) -> Box<notify_entry> {
    {
        Box::new(notify_entry {
            name: Some(name.to_owned()),
            fs: cmd_find_state::default(),
            formats: format_create(None, None, 0, FORMAT_NOJOBS),
            client_ref: None,
            session_ref: None,
            window_ref: None,
            pane: -1,
            pbname: Some(c"buffer0".to_owned()),
        })
    }
}

#[test]
fn a_hook_with_no_commands_behind_it_inserts_nothing() {
    let mut world = World::new();
    unsafe {
        notify_hook(&world.read(), c"window-linked");
    }
    assert!(world.inserted().is_empty());
}

#[test]
fn a_name_that_is_not_a_hook_at_all_inserts_nothing() {
    let mut world = World::new();
    unsafe {
        notify_hook(&world.read(), c"not-a-hook");
    }
    assert!(world.inserted().is_empty());
}

#[test]
fn a_session_hook_inserts_one_item_per_command_in_it() {
    let mut world = World::new();
    unsafe {
        world
            .session
            .options()
            .with_entry_mut(c"window-linked", false, |entry| {
                let entry = entry.unwrap();
                let mut cause: Option<CString> = None;
                assert_eq!(
                    RustOptionsEngine.array_set(
                        entry,
                        0,
                        Some(c"display-message first"),
                        0,
                        &mut cause
                    ),
                    0
                );
                assert_eq!(
                    RustOptionsEngine.array_set(
                        entry,
                        1,
                        Some(c"display-message second"),
                        0,
                        &mut cause
                    ),
                    0
                );
            });
        notify_hook(&world.read(), c"window-linked");
    }
    assert_eq!(
        world.inserted(),
        vec!["display-message".to_string(), "display-message".to_string()]
    );
}

#[test]
fn a_hook_the_session_does_not_carry_is_looked_for_on_the_pane() {
    let mut world = World::new();
    unsafe {
        world
            .pane
            .options()
            .with_entry_mut(c"pane-mode-changed", false, |entry| {
                let entry = entry.unwrap();
                let mut cause: Option<CString> = None;
                assert_eq!(
                    RustOptionsEngine.array_set(
                        entry,
                        0,
                        Some(c"display-message pane"),
                        0,
                        &mut cause
                    ),
                    0
                );
            });
        notify_hook(&world.read(), c"pane-mode-changed");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_target_with_no_pane_looks_for_the_hook_on_the_window() {
    let mut world = World::without_a_pane();
    unsafe {
        world
            .window
            .options()
            .with_entry_mut(c"pane-mode-changed", false, |entry| {
                let entry = entry.unwrap();
                let mut cause: Option<CString> = None;
                assert_eq!(
                    RustOptionsEngine.array_set(
                        entry,
                        0,
                        Some(c"display-message window"),
                        0,
                        &mut cause
                    ),
                    0
                );
            });
        notify_hook(&world.read(), c"pane-mode-changed");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_user_option_is_parsed_as_a_command_line_of_its_own() {
    let mut world = World::new();
    unsafe {
        world.session.options().set_string(
            c"@hook",
            0,
            c"%s",
            fmt_args![c"display-message user".as_ptr()],
        );
        notify_hook(&world.read(), c"@hook");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_user_option_that_is_not_a_command_line_inserts_nothing() {
    let mut world = World::new();
    unsafe {
        world.session.options().set_string(
            c"@hook",
            0,
            c"%s",
            fmt_args![c"no-such-command".as_ptr()],
        );
        notify_hook(&world.read(), c"@hook");
    }
    assert_eq!(world.inserted(), Vec::<String>::new(), "invalid");
}

#[test]
fn an_empty_user_option_parses_to_a_command_list_with_nothing_in_it() {
    let mut world = World::new();
    unsafe {
        world
            .session
            .options()
            .set_string(c"@hook", 0, c"%s", fmt_args![c"".as_ptr()]);
        notify_hook(&world.read(), c"@hook");
    }
    assert_eq!(world.inserted(), vec!["cmdq_empty_command".to_string()]);
}

#[test]
fn a_hook_is_named_in_the_log_when_the_log_is_on() {
    let mut world = World::new();
    unsafe {
        log_add_level();
        assert_ne!(log_get_level(), 0);
        world
            .session
            .options()
            .with_entry_mut(c"window-linked", false, |entry| {
                let entry = entry.unwrap();
                let mut cause: Option<CString> = None;
                assert_eq!(
                    RustOptionsEngine.array_set(
                        entry,
                        0,
                        Some(c"display-message logged"),
                        0,
                        &mut cause
                    ),
                    0
                );
            });
        notify_hook(&world.read(), c"window-linked");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_target_that_is_not_valid_any_more_is_found_from_nothing() {
    let mut world = World::new();
    unsafe {
        {
            let pane = world.pane.ptr();
            let mut item = world.item_mut();
            let target = &mut item.target;
            target.set_session_ref(Some(world.session.handle()));
            target.set_winlink(world.wl.get());
            target.set_window_ref(Some(world.window.handle()));
            target.set_pane(pane.as_ref());
        }
        world
            .session
            .options()
            .with_entry_mut(c"window-linked", false, |entry| {
                let entry = entry.unwrap();
                let mut cause: Option<CString> = None;
                RustOptionsEngine.array_set(
                    entry,
                    0,
                    Some(c"display-message valid"),
                    0,
                    &mut cause,
                );
            });
        notify_hook(&world.read(), c"window-linked");
        assert_eq!(world.inserted(), vec!["display-message".to_string()]);

        world
            .item_mut()
            .target
            .set_pane(None::<&crate::types::window_pane>);
        notify_hook(&world.read(), c"window-linked");
        assert_eq!(world.inserted().len(), 2);
    }
}

#[test]
fn a_hook_with_no_command_list_behind_it_leaves_the_item_where_it_was() {
    let mut world = World::new();
    let mut ne = notify_entry {
        name: Some(c"window-linked".to_owned()),
        fs: cmd_find_state::default(),
        formats: { format_create(None, None, 0, FORMAT_NOJOBS) },
        client_ref: None,
        session_ref: None,
        window_ref: None,
        pane: -1,
        pbname: None,
    };
    unsafe {
        let state = CmdqStateRef::create(Some(&ne.fs), None, STATE_NOHOOKS);
        let item = world.item.handle();
        assert_eq!(
            notify_insert_one_hook(&item, &ne, None, &state).as_ptr(),
            item.as_ptr()
        );
    }
    assert!(world.inserted().is_empty());
}

#[test]
fn the_callback_dispatches_every_control_notification_and_frees_its_entry() {
    let mut world = World::new();
    let mut list = Clients::new();
    let c = list.add("control", 80, 24);
    for name in [
        c"pane-mode-changed",
        c"window-layout-changed",
        c"window-pane-changed",
        c"window-unlinked",
        c"window-linked",
        c"window-renamed",
        c"client-session-changed",
        c"client-detached",
        c"session-renamed",
        c"session-created",
        c"session-closed",
        c"session-window-changed",
        c"paste-buffer-changed",
        c"paste-buffer-deleted",
        c"nothing-in-particular",
    ] {
        unsafe {
            let mut ne = entry(name);
            ne.client_ref = crate::server::client_ref_of(&*c);
            ne.session_ref = Some(world.session.reference());
            ne.window_ref = Some(world.window.reference());
            ne.fs.set_session(world.session.ptr().as_ref());
            assert_eq!(notify_callback(&world.item.handle(), ne), CMD_RETURN_NORMAL);
        }
    }
    assert!(world.inserted().is_empty());
}

#[test]
fn the_callback_releases_the_client_handle_the_entry_held() {
    let mut world = World::new();
    let mut client = zeroed_client();
    unsafe {
        let weak = world.session.weak();
        let mut ne = entry(c"window-renamed");
        ne.client_ref = Some(client.clone());
        ne.session_ref = Some(world.session.reference());
        ne.window_ref = Some(world.window.reference());
        ne.fs.set_session(world.session.ptr().as_ref());
        notify_callback(&world.item.handle(), ne);
        assert!(weak.upgrade().is_some());
    }
}

#[test]
fn every_notification_keeps_its_named_targets_until_callback() {
    let mut world = World::new();
    let mut list = Clients::new();
    let c = list.add("c", 80, 24);
    unsafe {
        (*c).set_attached_session(Some(world.session.handle()));
        let client_weak = crate::server::client_ref_of(&*c).unwrap().downgrade();
        let session_weak = world.session.weak();

        notify_client(c"client-detached", c.as_mut());

        notify_session(c"session-created", world.session.ptr().as_ref());
        notify_winlink(c"window-linked", world.wl.get().unwrap());
        notify_session_window(
            c"window-unlinked",
            &*world.session.ptr(),
            world.window.handle(),
        );
        notify_window(c"window-renamed", Some(world.window.handle()));
        notify_pane(c"pane-mode-changed", world.pane.ptr().as_ref());
        notify_paste_buffer(c"buffer0", 0);
        notify_paste_buffer(c"buffer0", 1);

        assert!(session_weak.upgrade().is_some());
        assert!(client_weak.upgrade().is_some());
    }
}

#[test]
fn a_hook_with_no_session_anywhere_is_looked_for_in_the_global_options() {
    let _guard = globals();
    let mut item = Item::new();
    unsafe {
        notify_hook(&item.read(), c"window-linked");
        // Nothing was queued: an item off a queue is one cmdq_insert_after
        // could not have put anything after.
        assert!((*item.ptr()).queue.is_none());
    }
}

#[test]
fn a_notification_of_a_session_that_is_gone_is_found_from_nothing() {
    let _guard = globals();
    let mut s = Session::new(2, "gone");
    unsafe {
        notify_session(c"session-closed", s.ptr().as_ref());
        assert!(s.weak().upgrade().is_some());
    }
}

#[test]
fn a_command_that_asked_for_no_hooks_notifies_nothing() {
    let mut world = World::new();
    let mut running = Item::new();
    let before = world.inserted();
    let queue = unsafe {
        let queue = global_queue(&running.state_ref());
        running.state_ref().state().flags = STATE_NOHOOKS;
        queue
    };
    unsafe {
        running.queue_onto(&queue);
        queue.set_running(true);
        notify_window(c"window-renamed", Some(world.window.handle()));
    }
    let after = world.inserted();
    {
        queue.set_running(false);
        queue.clear();
    }
    assert_eq!(after, before);
}
