use super::*;
use crate::cmd::{CMDQ_STATE_NOHOOKS as STATE_NOHOOKS, cmdq_new};
use crate::log::log_add_level;
use crate::options::options_get_ptr;
use crate::options::{options_array_set, options_set_string};
use crate::tests::test_fixtures::{
    Args, Clients, Item, Pane, Registry, Session, Window, globals, link, seen, unlink,
    zeroed_client,
};
use ::core::ffi::CStr;
use ::core::ptr::null_mut;
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
    wl: *mut winlink,
    item: Item,
    queue: Box<cmdq_list>,
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
        let mut w = World {
            registry: Registry::new(),
            session: Session::new(1, "hooks"),
            window: Window::new(1, "hooks", 80, 24),
            pane: Pane::new(1, 80, 24, 100),
            wl: null_mut::<winlink>(),
            item: Item::new(),
            queue: cmdq_new(),
            _guard: guard,
        };
        if with_pane {
            let pane = &raw mut w.pane;
            unsafe { w.window.add_pane(&mut *pane) };
        }
        w.wl = link(&mut w.session, &mut w.window, 0);
        w.registry.add_session(&mut w.session);
        let queue = &raw mut *w.queue;
        unsafe { w.item.queue_onto(&mut *queue) };
        w
    }

    fn item(&mut self) -> *mut cmdq_item {
        self.item.ptr()
    }

    /// The commands of the items queued after the test's own item, in the
    /// order the queue would run them. An item's name is
    /// `[<command>/<pointer>]`, and the pointer is not the same twice.
    fn inserted(&mut self) -> Vec<String> {
        let anchor = self.item();
        let Some(at) = self
            .queue
            .list
            .iter()
            .position(|item| item.as_ptr() == anchor)
        else {
            return Vec::new();
        };
        self.queue
            .list
            .iter()
            .skip(at + 1)
            .map(|item| {
                let name = unsafe { seen(cstr_ptr(&item.item().name)) };
                let command = name.trim_start_matches('[').split('/').next();
                command.expect("a name is never empty").to_string()
            })
            .collect()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unlink(&mut self.session, self.wl);
    }
}

/// The queue a notification is appended to, reached the only way there is
/// from outside `cmd_queue`: an item put on it says which queue it is on.
/// The probe item stays there, the way every notification a unit test
/// raises does, since nothing runs the queue.
unsafe fn global_queue(state: &CmdqStateRef) -> *mut cmdq_list {
    unsafe {
        let probe = Args::parse(c"display-message probe");
        (*cmdq_append(null_mut(), cmdq_get_command(probe.list_ref(), Some(state)))).queue
    }
}

/// A notification entry the way `notify_add` builds one, with a name and a
/// buffer name the callback frees again.
fn entry(name: &CStr) -> Box<notify_entry> {
    unsafe {
        Box::new(notify_entry {
            name: Some(name.to_owned()),
            fs: cmd_find_state::default(),
            formats: format_create(null_mut(), null_mut(), 0, FORMAT_NOJOBS),
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
        notify_hook(world.item(), c"window-linked");
    }
    assert!(world.inserted().is_empty());
}

#[test]
fn a_name_that_is_not_a_hook_at_all_inserts_nothing() {
    let mut world = World::new();
    unsafe {
        notify_hook(world.item(), c"not-a-hook");
    }
    assert!(world.inserted().is_empty());
}

#[test]
fn a_session_hook_inserts_one_item_per_command_in_it() {
    let mut world = World::new();
    unsafe {
        let o = options_get_ptr(world.session.options(), c"window-linked".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_set(o, 0, c"display-message first".as_ptr(), 0, &mut cause),
            0
        );
        assert_eq!(
            options_array_set(o, 1, c"display-message second".as_ptr(), 0, &mut cause),
            0
        );
        notify_hook(world.item(), c"window-linked");
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
        let o = options_get_ptr(world.pane.options(), c"pane-mode-changed".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_set(o, 0, c"display-message pane".as_ptr(), 0, &mut cause),
            0
        );
        notify_hook(world.item(), c"pane-mode-changed");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_target_with_no_pane_looks_for_the_hook_on_the_window() {
    let mut world = World::without_a_pane();
    unsafe {
        let o = options_get_ptr(world.window.options(), c"pane-mode-changed".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_set(o, 0, c"display-message window".as_ptr(), 0, &mut cause),
            0
        );
        notify_hook(world.item(), c"pane-mode-changed");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_user_option_is_parsed_as_a_command_line_of_its_own() {
    let mut world = World::new();
    unsafe {
        options_set_string(
            world.session.options(),
            c"@hook".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"display-message user".as_ptr()],
        );
        notify_hook(world.item(), c"@hook");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_user_option_that_is_not_a_command_line_inserts_nothing() {
    let mut world = World::new();
    unsafe {
        options_set_string(
            world.session.options(),
            c"@hook".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"no-such-command".as_ptr()],
        );
        notify_hook(world.item(), c"@hook");
    }
    assert_eq!(world.inserted(), Vec::<String>::new(), "invalid");
}

#[test]
fn an_empty_user_option_parses_to_a_command_list_with_nothing_in_it() {
    let mut world = World::new();
    unsafe {
        options_set_string(
            world.session.options(),
            c"@hook".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"".as_ptr()],
        );
        notify_hook(world.item(), c"@hook");
    }
    assert_eq!(world.inserted(), vec!["cmdq_empty_command".to_string()]);
}

#[test]
fn a_hook_is_named_in_the_log_when_the_log_is_on() {
    let mut world = World::new();
    unsafe {
        log_add_level();
        assert_ne!(log_get_level(), 0);
        let o = options_get_ptr(world.session.options(), c"window-linked".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_set(o, 0, c"display-message logged".as_ptr(), 0, &mut cause),
            0
        );
        notify_hook(world.item(), c"window-linked");
    }
    assert_eq!(world.inserted(), vec!["display-message".to_string()]);
}

#[test]
fn a_target_that_is_not_valid_any_more_is_found_from_nothing() {
    let mut world = World::new();
    unsafe {
        let target = cmdq_get_target(world.item());
        (*target).set_session(world.session.ptr());
        (*target).set_winlink(world.wl);
        (*target).set_window(world.window.ptr());
        (*target).set_pane(world.pane.ptr());
        let o = options_get_ptr(world.session.options(), c"window-linked".as_ptr());
        let mut cause: Option<CString> = None;
        options_array_set(o, 0, c"display-message valid".as_ptr(), 0, &mut cause);
        notify_hook(world.item(), c"window-linked");
        assert_eq!(world.inserted(), vec!["display-message".to_string()]);

        (*target).set_pane(null_mut::<window_pane>());
        notify_hook(world.item(), c"window-linked");
        assert_eq!(world.inserted().len(), 2);
    }
}

#[test]
fn a_hook_with_no_command_list_behind_it_leaves_the_item_where_it_was() {
    let mut world = World::new();
    let mut ne = notify_entry {
        name: Some(c"window-linked".to_owned()),
        fs: cmd_find_state::default(),
        formats: unsafe { format_create(null_mut(), null_mut(), 0, FORMAT_NOJOBS) },
        client_ref: None,
        session_ref: None,
        window_ref: None,
        pane: -1,
        pbname: None,
    };
    unsafe {
        let state = cmdq_new_state(&raw mut ne.fs, null_mut(), STATE_NOHOOKS);
        assert_eq!(
            notify_insert_one_hook(world.item(), &ne, None, &state),
            world.item()
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
            ne.client_ref = client_ref_from_ptr(c);
            ne.session_ref = Some(world.session.reference());
            ne.window_ref = Some(world.window.reference());
            ne.fs.set_session(world.session.ptr());
            assert_eq!(
                notify_callback(world.item(), CmdqCallbackData::NotifyEntry(ne)),
                CMD_RETURN_NORMAL
            );
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
        ne.fs.set_session(world.session.ptr());
        notify_callback(world.item(), CmdqCallbackData::NotifyEntry(ne));
        assert!(weak.upgrade().is_some());
    }
}

#[test]
fn every_notification_keeps_its_named_targets_until_callback() {
    let mut world = World::new();
    let mut list = Clients::new();
    let c = list.add("c", 80, 24);
    unsafe {
        (*c).session = world.session.ptr();
        let client_weak = client_ref_from_ptr(c).unwrap().downgrade();
        let session_weak = world.session.weak();

        notify_client(c"client-detached".as_ptr(), c);

        notify_session(c"session-created".as_ptr(), world.session.ptr());
        notify_winlink(c"window-linked".as_ptr(), world.wl);
        notify_session_window(
            c"window-unlinked".as_ptr(),
            world.session.ptr(),
            world.window.ptr(),
        );
        notify_window(c"window-renamed".as_ptr(), world.window.ptr());
        notify_pane(c"pane-mode-changed".as_ptr(), world.pane.ptr());
        notify_paste_buffer(c"buffer0".as_ptr(), 0);
        notify_paste_buffer(c"buffer0".as_ptr(), 1);

        assert!(session_weak.upgrade().is_some());
        assert!(client_weak.upgrade().is_some());
    }
}

#[test]
fn a_hook_with_no_session_anywhere_is_looked_for_in_the_global_options() {
    let _guard = globals();
    let mut item = Item::new();
    unsafe {
        notify_hook(item.ptr(), c"window-linked");
        // Nothing was queued: an item off a queue is one cmdq_insert_after
        // could not have put anything after.
        assert!((*item.ptr()).queue.is_null());
    }
}

#[test]
fn a_notification_of_a_session_that_is_gone_is_found_from_nothing() {
    let _guard = globals();
    let mut s = Session::new(2, "gone");
    unsafe {
        notify_session(c"session-closed".as_ptr(), s.ptr());
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
        unsafe {
            (*(*running.ptr()).state()).flags = STATE_NOHOOKS;
        }
        queue
    };
    unsafe {
        running.queue_onto(&mut *queue);
        (*queue).running = true;
        notify_window(c"window-renamed".as_ptr(), world.window.ptr());
    }
    let after = world.inserted();
    unsafe {
        (*queue).running = false;
        (*queue).list.clear();
    }
    assert_eq!(after, before);
}
