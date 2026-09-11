use super::*;
use crate::WindowPane;
use crate::cmd::CmdListRef;
use crate::environ::new_environment_box;
use crate::options::OptionsRef;
use crate::pane_geometry::PaneGeometryState;
use crate::pane_output::PaneOutputOffset;
use crate::pane_resize::{PaneResizeQueue, PaneSize};
use crate::tests::test_fixtures::{Clients, Target, globals, zeroed_term};
use crate::window_dimensions::WindowDimensionsState;

#[test]
fn client_ids_are_stable_and_are_not_reused_after_drop() {
    let _guard = globals();
    std::thread::spawn(|| {
        let first = ClientRef::new(client::default());
        let id = first.id();
        assert_eq!(first.clone().id(), id);
        drop(first);
        let second = ClientRef::new(client::default());
        assert!(second.id() > id);
    })
    .join()
    .unwrap();
}

#[test]
fn moved_client_payload_cannot_recover_its_previous_owner() {
    let _guard = globals();
    let mut original = ClientRef::new(client::default());
    let moved = std::mem::take(unsafe { original.as_client_mut() });
    assert!(client_ref_of(&moved).is_none());
    let replacement = ClientRef::new(moved);
    assert!(
        client_ref_of(unsafe { replacement.as_client() })
            .unwrap()
            .ptr_eq(&replacement)
    );
}

#[test]
fn client_observer_expires_when_its_last_owner_drops() {
    let _guard = globals();
    std::thread::spawn(|| {
        let reference = ClientRef::new(client::default());
        let observer = reference.downgrade();
        assert!(client_ref_of(unsafe { reference.as_client() }).is_some());
        drop(reference);
        assert!(observer.upgrade().is_none());
    })
    .join()
    .unwrap();
}

#[test]
fn client_cleanup_works_during_thread_local_teardown() {
    let _guard = globals();
    thread_local! {
        static LAST_CLIENT: std::cell::RefCell<Option<ClientRef>> = const {
            std::cell::RefCell::new(None)
        };
    }
    std::thread::spawn(|| {
        LAST_CLIENT.with_borrow_mut(|last| {
            *last = Some(ClientRef::new(client::default()));
        });
    })
    .join()
    .unwrap();
}

/// A client attached to `t`'s session that keeps a pane of its own, which is
/// what makes [`server_client_get_pane`] consult the per-window entry rather
/// than answer the window's active pane.
unsafe fn client_with_its_own_pane(attached: &mut Clients, t: &mut Target) -> *mut client {
    unsafe {
        let c = attached.add("client", 80, 24);
        (*c).set_attached_session(Some(t.session_handle()));
        (*c).flags |= CLIENT_ACTIVEPANE as uint64_t;
        c
    }
}

/// Drops the per-window sizes a test left on `c`, since nothing here takes
/// the client down through `server_destroy`.
unsafe fn forget_client_windows(c: *mut client) {
    unsafe { drop(core::mem::take(&mut (*c).windows)) };
}

#[test]
fn a_client_finds_the_pane_it_made_active() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut t);
        let wp = t.pane(0);

        server_client_set_pane(&mut *c, &*wp);

        assert_eq!(
            server_client_get_pane(&*c).map(|pane| pane.id()),
            Some((*wp).pane_id())
        );
        forget_client_windows(c);
    }
}

#[test]
fn a_client_that_has_made_no_pane_active_finds_none() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut t);
        server_client_add_client_window(&mut *c, (*t.window(0)).window_id());

        assert!(server_client_get_pane(&*c).is_none());
        forget_client_windows(c);
    }
}

#[test]
fn taking_a_pane_away_forgets_the_client_that_had_it_active() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut t);
        let wp = t.pane(0);
        server_client_set_pane(&mut *c, &*wp);

        server_client_remove_pane(&*wp);

        assert!((*c).windows.is_empty(), "the entry naming the pane is gone");
    }
}

fn ranges(values: &[(u_int, u_int)]) -> visible_ranges {
    visible_ranges {
        ranges: values
            .iter()
            .map(|&(px, nx)| visible_range { px, nx })
            .collect(),
        used: values.len() as u_int,
    }
}

#[test]
fn visible_range_helpers_cover_each_overlap_shape() {
    let mut r = ranges(&[]);
    {
        assert!(r.is_empty());
        r.ensure_capacity(2);
        assert_eq!(r.ranges.len(), 2);
        r.ensure_capacity(1);
        assert_eq!(r.ranges.len(), 2);

        r.set_overlay_range(10, 5, 10, 3, 3, 4, 30);
        assert_eq!((r.used, r.ranges[0].px, r.ranges[0].nx), (1, 3, 30));

        r.set_overlay_range(10, 5, 10, 3, 3, 5, 30);
        assert_eq!((r.used, r.ranges[0].px, r.ranges[0].nx), (2, 3, 7));
        assert_eq!((r.ranges[1].px, r.ranges[1].nx), (20, 13));
        assert!(!r.is_empty());

        r.set_overlay_range(10, 5, 10, 3, 12, 6, 3);
        assert_eq!((r.ranges[0].px, r.ranges[0].nx), (0, 0));
        assert_eq!((r.ranges[1].px, r.ranges[1].nx), (0, 0));
        assert!(r.is_empty());

        r.set_overlay_range(10, 5, 10, 3, 0, 7, 5);
        assert_eq!((r.ranges[0].px, r.ranges[0].nx), (0, 5));
    }
}

#[test]
fn overlay_capabilities_match_each_overlay_kind() {
    assert_eq!(Overlay::None.check(), OverlayCheck::None);
    assert_eq!(Overlay::Menu.check(), OverlayCheck::Menu);
    assert_eq!(Overlay::Popup.check(), OverlayCheck::Popup);
    assert_eq!(
        Overlay::DisplayPanes { keys: true }.check(),
        OverlayCheck::None
    );
    assert!(!Overlay::None.has_mode());
    assert!(Overlay::Menu.has_mode());
    assert!(Overlay::Popup.has_mode());
    assert!(!Overlay::DisplayPanes { keys: false }.has_mode());
    assert!(!Overlay::None.has_key());
    assert!(Overlay::Menu.has_key());
    assert!(Overlay::Popup.has_key());
    assert!(Overlay::DisplayPanes { keys: true }.has_key());
    assert!(!Overlay::DisplayPanes { keys: false }.has_key());
    assert!(!Overlay::None.has_resize());
    assert!(Overlay::Menu.has_resize());
    assert!(Overlay::Popup.has_resize());
    assert!(!Overlay::DisplayPanes { keys: true }.has_resize());
    assert!(OverlayData::None.is_none());
    let menu = MenuDataRef::new(menu_data {
        item: None,
        flags: 0,
        style: None,
        border_style: None,
        selected_style: None,
        style_gc: crate::grid::grid_default_cell,
        border_style_gc: crate::grid::grid_default_cell,
        selected_style_gc: crate::grid::grid_default_cell,
        border_lines: crate::overlay::BOX_LINES_DEFAULT,
        fs: cmd_find_state::default(),
        s: ScreenRef::default(),
        r: VisibleRangesRef::default(),
        px: 0,
        py: 0,
        menu: crate::overlay::menu_create(c"menu"),
        choice: -1,
        cb: None,
    });
    let popup = crate::overlay::PopupDataRef::new(popup_data::default());
    let panes = DisplayPanesRef::default();
    assert_eq!(OverlayData::Menu(menu.clone()).menu(), menu);
    assert_eq!(OverlayData::Popup(popup.clone()).popup(), popup);
    assert_eq!(
        OverlayData::DisplayPanes(panes.clone()).display_panes(),
        panes
    );
}

#[test]
fn control_flag_parser_handles_all_spellings() {
    let _guard = globals();
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("flags", 80, 24);
        assert_eq!(
            server_client_control_flags(c, c"pause-after"),
            CLIENT_CONTROL_PAUSEAFTER
        );
        assert_eq!(c.pause_age, 0);
        assert_eq!(
            server_client_control_flags(c, c"pause-after=7"),
            CLIENT_CONTROL_PAUSEAFTER
        );
        assert_eq!(c.pause_age, 7000);
        assert_eq!(
            server_client_control_flags(c, c"no-output"),
            CLIENT_CONTROL_NOOUTPUT as u64
        );
        assert_eq!(
            server_client_control_flags(c, c"wait-exit"),
            CLIENT_CONTROL_WAITEXIT
        );
        assert_eq!(server_client_control_flags(c, c"unknown"), 0);
    }
}

#[test]
fn get_flags_formats_every_reported_flag_in_order() {
    let _guard = globals();
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("flags", 80, 24);
        c.flags = CLIENT_ATTACHED as u64
            | CLIENT_FOCUSED as u64
            | CLIENT_CONTROL as u64
            | CLIENT_IGNORESIZE as u64
            | CLIENT_NO_DETACH_ON_DESTROY
            | CLIENT_CONTROL_NOOUTPUT as u64
            | CLIENT_CONTROL_WAITEXIT
            | CLIENT_CONTROL_PAUSEAFTER
            | CLIENT_READONLY as u64
            | CLIENT_ACTIVEPANE
            | CLIENT_SUSPENDED as u64
            | CLIENT_UTF8 as u64;
        c.pause_age = 12000;
        assert_eq!(server_client_get_flags(c).to_bytes(), b"attached,focused,control-mode,ignore-size,no-detach-on-destroy,no-output,wait-exit,pause-after=12,read-only,active-pane,suspended,UTF-8");
        c.flags = 0;
        assert!(server_client_get_flags(c).to_bytes().is_empty());
    }
}

unsafe fn identify(c: &mut client, ty: msgtype, bytes: &[u8]) -> core::ffi::c_int {
    let mut message =
        imsg::from_imsg_message(ty, (IMSG_HEADER_SIZE + bytes.len()) as u32, 0, 0, bytes);
    unsafe { server_client_dispatch_identify(c, &mut message) }
}

#[test]
fn identify_dispatch_accepts_each_piece_and_rejects_bad_shapes() {
    let _guard = globals();
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("identify", 80, 24);
        c.environ = Some(new_environment_box());
        let feature = 3_i32.to_ne_bytes();
        let flags = (CLIENT_UTF8 as i32).to_ne_bytes();
        let longflags = CLIENT_ACTIVEPANE.to_ne_bytes();
        let pid = 1234_i32.to_ne_bytes();
        assert_eq!(identify(c, MSG_IDENTIFY_FEATURES, &feature), 0);
        assert_eq!(c.term_features, 3);
        assert_eq!(identify(c, MSG_IDENTIFY_FLAGS, &flags), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_LONGFLAGS, &longflags), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_TERM, b"xterm\0"), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_TERMINFO, b"RGB\0"), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_TTYNAME, b"/dev/pts/test\0"), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_CWD, b"/\0"), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_ENVIRON, b"A=B\0"), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_ENVIRON, b"IGNORED\0"), 0);
        assert_eq!(identify(c, MSG_IDENTIFY_CLIENTPID, &pid), 0);
        assert_eq!(c.pid, 1234);
        assert_eq!(identify(c, 999, b"anything"), 0);

        for ty in [
            MSG_IDENTIFY_TERM,
            MSG_IDENTIFY_TERMINFO,
            MSG_IDENTIFY_TTYNAME,
            MSG_IDENTIFY_CWD,
            MSG_IDENTIFY_ENVIRON,
        ] {
            assert_eq!(identify(c, ty, b"not-terminated"), -1);
            assert_eq!(identify(c, ty, b""), -1);
        }
        for ty in [
            MSG_IDENTIFY_FEATURES,
            MSG_IDENTIFY_FLAGS,
            MSG_IDENTIFY_LONGFLAGS,
            MSG_IDENTIFY_CLIENTPID,
        ] {
            assert_eq!(identify(c, ty, b"x"), -1);
        }
        assert_eq!(identify(c, MSG_IDENTIFY_STDIN, b"x"), -1);
        assert_eq!(identify(c, MSG_IDENTIFY_STDOUT, b"x"), -1);

        c.fd = -1;
        c.flags |= CLIENT_EXIT as u64;
        assert_eq!(identify(c, MSG_IDENTIFY_DONE, b""), 0);
        assert!(c.flags & CLIENT_IDENTIFIED as u64 != 0);
        assert_eq!(c.name.as_deref().unwrap(), c"/dev/pts/test");
        assert_eq!(identify(c, MSG_IDENTIFY_TERM, b"other\0"), -1);
    }
}

#[test]
fn identify_done_supplies_default_names() {
    let _guard = globals();
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("identify", 80, 24);
        c.environ = Some(new_environment_box());
        c.fd = -1;
        c.pid = 42;
        c.flags = CLIENT_EXIT as u64;
        assert_eq!(identify(c, MSG_IDENTIFY_DONE, b""), 0);
        assert_eq!(c.term_name.as_deref().unwrap(), c"unknown");
        assert_eq!(c.name.as_deref().unwrap(), c"client-42");
    }
}

#[test]
fn client_weak_session_and_pan_handles_follow_their_owners() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(80, 24);
    unsafe {
        let c = &mut *attached.add("handles", 80, 24);
        assert!(client_get_last_session(c).is_none());
        assert!(client_get_pan_window(c).is_none());
        client_set_last_session(c, Some(&*target.session()));
        client_set_pan_window(c, crate::window::window_ref_of(&*target.window(0)).as_ref());
        assert_eq!(
            client_get_last_session(c).unwrap().as_ptr(),
            target.session()
        );
        assert_eq!(client_get_pan_window(c).unwrap().as_ptr(), target.window(0));
        client_set_last_session(c, None);
        client_set_pan_window(c, None);
        assert!(client_get_last_session(c).is_none());
        assert!(client_get_pan_window(c).is_none());
    }
}

#[test]
fn pane_selection_falls_back_to_active_pane() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(80, 24);
    unsafe {
        let c = &mut *attached.add("pane", 80, 24);
        assert!(server_client_get_pane(c).is_none());
        server_client_set_pane(c, &*target.pane(0));
        assert!(c.windows.is_empty());
        c.set_attached_session(Some(target.session_handle()));
        let mut session = target.session_handle().clone();
        let current = session.as_session_mut().curw.take();
        server_client_set_pane(c, &*target.pane(0));
        assert!(c.windows.is_empty());
        session.as_session_mut().curw = current;
        assert_eq!(
            server_client_get_pane(c).map(|pane| pane.id()),
            Some((*target.pane(0)).pane_id())
        );
        c.flags |= CLIENT_ACTIVEPANE;
        assert_eq!(
            server_client_get_pane(c).map(|pane| pane.id()),
            Some((*target.pane(0)).pane_id())
        );
        let cw = server_client_add_client_window(c, (*target.window(0)).window_id())
            as *const client_window;
        let same = server_client_add_client_window(c, (*target.window(0)).window_id());
        assert!(core::ptr::eq(cw, same));
        assert!(server_client_get_client_window(c, u_int::MAX).is_none());
        forget_client_windows(c);
    }
}

#[test]
fn client_pane_owners_preserve_saved_selection_and_handle_missing_targets() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(80, 24);
    let other = target.add_window(1, 80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut target);
        let mut session = target.session_handle().clone();
        let window_id = (*target.window(0)).window_id();
        let active = server_client_get_pane(&*c).unwrap();
        assert!(active.ptr_eq(&window_pane_find_by_id((*target.pane(0)).pane_id()).unwrap()));
        server_client_add_client_window(&mut *c, window_id).pane = None;
        assert!(server_client_get_pane(&*c).is_none());
        let other_pane = window_pane_find_by_id((*target.pane(other)).pane_id()).unwrap();
        server_client_add_client_window(&mut *c, window_id).pane = Some(other_pane.clone());
        assert!(server_client_get_pane(&*c).unwrap().ptr_eq(&other_pane));
        (*c).flags &= !CLIENT_ACTIVEPANE;
        assert!(server_client_get_pane(&*c).unwrap().ptr_eq(&active));
        let current = session.as_session_mut().curw.take();
        assert!(server_client_get_pane(&*c).is_none());
        session.as_session_mut().curw = current;
        forget_client_windows(c);
    }
}

#[test]
fn client_handle_lookup_and_counts_track_live_clients() {
    let _guard = globals();
    let mut attached = Clients::new();
    let target = Target::new(80, 24);
    unsafe {
        let first = attached.add("first", 80, 24);
        let second = attached.add("second", 80, 24);
        (*first).set_attached_session(Some(target.session_handle()));
        (*second).set_attached_session(None);
        assert_eq!(server_client_how_many(), 1);
        let first_ref = client_ref_of(&*first).unwrap();
        assert!(core::ptr::eq(first_ref.as_ptr(), first));
        assert!(first_ref.downgrade().upgrade().is_some());
    }
}

#[test]
fn bracket_paste_tracks_start_content_end_and_masked_keys() {
    let _guard = globals();
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("paste", 80, 24);
        assert_eq!(server_client_is_bracket_paste(c, b'x' as key_code), 0);
        assert_eq!(server_client_is_bracket_paste(c, KEYC_PASTE_START), 0);
        assert_ne!(c.flags & CLIENT_BRACKETPASTING as u64, 0);
        assert_eq!(server_client_is_bracket_paste(c, b'x' as key_code), 1);
        assert_eq!(
            server_client_is_bracket_paste(c, KEYC_PASTE_START | KEYC_META),
            0
        );
        assert_eq!(server_client_is_bracket_paste(c, KEYC_PASTE_END), 0);
        assert_eq!(c.flags & CLIENT_BRACKETPASTING as u64, 0);
        assert_eq!(server_client_is_bracket_paste(c, b'x' as key_code), 0);
    }
}

#[test]
fn assume_paste_covers_disabled_bracketed_fast_repeat_and_expiry_paths() {
    let _guard = globals();
    let mut attached = Clients::new();
    let target = Target::new(80, 24);
    unsafe {
        let c = &mut *attached.add("assume", 80, 24);
        c.set_attached_session(Some(target.session_handle()));
        (target.session_handle().options()).set_number(c"assume-paste-time", 0);
        assert_eq!(server_client_is_assume_paste(c), 0);
        c.flags |= CLIENT_BRACKETPASTING as u64;
        (target.session_handle().options()).set_number(c"assume-paste-time", 50);
        assert_eq!(server_client_is_assume_paste(c), 0);
        c.flags &= !(CLIENT_BRACKETPASTING as u64);
        c.tty.term = Some(zeroed_term());
        c.activity_time = timeval {
            tv_sec: 10,
            tv_usec: 20_000,
        };
        c.last_activity_time = timeval {
            tv_sec: 10,
            tv_usec: 0,
        };
        assert_eq!(server_client_is_assume_paste(c), 0);
        assert_ne!(c.flags & CLIENT_ASSUMEPASTING as u64, 0);
        assert_eq!(server_client_is_assume_paste(c), 1);
        c.activity_time = timeval {
            tv_sec: 11,
            tv_usec: 0,
        };
        assert_eq!(server_client_is_assume_paste(c), 0);
        assert_eq!(c.flags & CLIENT_ASSUMEPASTING as u64, 0);
    }
}

#[test]
fn key_table_selection_and_activity_cover_detached_attached_default_and_named_tables() {
    let _guard = globals();
    let mut attached = Clients::new();
    let target = Target::new(80, 24);
    unsafe {
        let c = &mut *attached.add("keys", 80, 24);
        assert_eq!(server_client_get_key_table(c).as_ref(), c"root");
        c.set_attached_session(Some(target.session_handle()));
        assert_eq!(server_client_get_key_table(c).as_ref(), c"root");
        (target.session_handle().options()).set_string(
            c"key-table",
            0,
            c"%s",
            fmt_args![c"focused-server-client".as_ptr()],
        );
        assert_eq!(
            server_client_get_key_table(c).as_ref(),
            c"focused-server-client"
        );
        server_client_set_key_table(c, None);
        assert_eq!(
            c.keytable().unwrap().name().as_c_str(),
            c"focused-server-client"
        );
        c.activity_time = timeval {
            tv_sec: 5,
            tv_usec: 100_000,
        };
        c.keytable().unwrap().set_activity_time(timeval {
            tv_sec: 3,
            tv_usec: 900_000,
        });
        assert_eq!(server_client_key_table_activity_diff(c), 1200);
        assert_eq!(
            server_client_is_default_key_table(c, &c.keytable().unwrap().borrow()),
            1
        );
        server_client_set_key_table(c, Some(c"root"));
        assert_eq!(
            server_client_is_default_key_table(c, &c.keytable().unwrap().borrow()),
            0
        );
        c.keytable = None;
    }
}

#[test]
fn mouse_location_in_pane_covers_inside_outside_and_each_plain_border() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let mut pane = target.state().pane_ref().unwrap();
        pane.as_pane_mut()
            .configure_test(crate::window_pane::PaneTestSetup::Geometry(crate::pane_geometry::PaneGeometry {
                xoff: 2,
                yoff: 2,
                sx: 20,
                sy: 6,
            }));
        let mut slider = 0;
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 4, 4, &mut slider),
            KEYC_MOUSE_LOCATION_PANE
        );
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 22, 4, &mut slider),
            KEYC_MOUSE_LOCATION_BORDER
        );
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 4, 8, &mut slider),
            KEYC_MOUSE_LOCATION_BORDER
        );
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 4, 1, &mut slider),
            KEYC_MOUSE_LOCATION_BORDER
        );
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 39, 11, &mut slider),
            KEYC_MOUSE_LOCATION_NOWHERE
        );
    }
}

#[test]
fn pane_terminal_names_borrow_only_the_initialized_string() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let pane = unsafe { &mut *target.pane(0) };
    let mut name = [b'x'; 32];
    name[..5].copy_from_slice(b"/\xfe/1\0");
    unsafe { pane.configure_test(crate::window_pane::PaneTestSetup::Terminal(name)) };
    assert_eq!(pane.terminal_name().to_bytes(), b"/\xfe/1");
    name[0] = 0;
    unsafe { pane.configure_test(crate::window_pane::PaneTestSetup::Terminal(name)) };
    assert_eq!(pane.terminal_name(), c"");
}

#[test]
fn nested_detection_covers_missing_empty_nonmatching_and_matching_tty() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(40, 12);
    unsafe {
        let c = &mut *attached.add("nested", 40, 12);
        c.environ = Some(new_environment_box());
        assert_eq!(server_client_check_nested(c), 0);
        c.environ_mut().set(c"TMUX", 0, c"");
        assert_eq!(server_client_check_nested(c), 0);
        c.environ_mut().set(c"TMUX", 0, c"/tmp/tmux,1,0");
        c.ttyname = Some(c"/dev/pts/focused".to_owned());
        assert_eq!(server_client_check_nested(c), 0);
        let wp = &mut *target.pane(0);
        let bytes = b"/dev/pts/focused\0";
        let mut name = [0; 32];
        name[..bytes.len()].copy_from_slice(bytes);
        wp.configure_test(crate::window_pane::PaneTestSetup::Terminal(name));
        assert_eq!(server_client_check_nested(c), 1);
    }
}

#[test]
fn latest_modes_repeat_click_and_exit_checks_cover_safe_early_and_state_paths() {
    let _guard = globals();
    let mut attached = Clients::new();
    let target = Target::new(40, 12);
    unsafe {
        let c = &mut *attached.add("state", 40, 12);
        server_client_update_latest(c);
        c.set_attached_session(Some(target.session_handle()));
        server_client_update_latest(c);
        server_client_update_latest(c);
        c.flags |= CLIENT_CONTROL as u64;
        server_client_check_modes(c);
        server_client_reset_state(c);
        c.flags &= !(CLIENT_CONTROL as u64);
        c.flags |= CLIENT_SUSPENDED as u64;
        server_client_check_modes(c);
        server_client_reset_state(c);
        c.flags &= !(CLIENT_SUSPENDED as u64);
        server_client_check_modes(c);
        c.flags |= CLIENT_REPEAT as u64;
        server_client_repeat_timer(c);
        assert_eq!(c.flags & CLIENT_REPEAT as u64, 0);
        c.flags |= CLIENT_DOUBLECLICK as u64;
        server_client_click_timer(c);
        assert_eq!(
            c.flags & (CLIENT_DOUBLECLICK | CLIENT_TRIPLECLICK) as u64,
            0
        );
        server_client_check_exit(c);
        c.flags |= CLIENT_DEAD as u64;
        server_client_check_exit(c);
        c.flags &= !(CLIENT_DEAD as u64);
        c.flags |= CLIENT_EXITED as u64;
        server_client_check_exit(c);
    }
}

#[test]
fn cwd_and_print_helpers_cover_null_detached_session_and_visibility_paths() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(40, 12);
    unsafe {
        let c = &mut *attached.add("cwd", 40, 12);
        c.cwd = Some(c"/tmp/focused-cwd".to_owned());
        assert_eq!(
            server_client_get_cwd(Some(c), None).as_c_str(),
            c"/tmp/focused-cwd"
        );
        c.set_attached_session(Some(target.session_handle()));
        assert!(!server_client_get_cwd(Some(c), None).is_empty());
        assert!(!server_client_get_cwd(None, Some(&*target.session())).is_empty());
        let mut escaped = ByteBuffer::from(b"one\n\\two\x01".to_vec());
        server_client_print(None, 0, &mut escaped);
        let mut parsed = ByteBuffer::from(b"one\ntwo\nrest".to_vec());
        server_client_print(None, 1, &mut parsed);
    }
}

#[test]
fn attached_client_print_preserves_parsed_lines_and_escaped_bytes() {
    let _guard = globals();
    let mut attached = Clients::new();
    let target = Target::new(40, 12);
    unsafe {
        let c = &mut *attached.add("print", 40, 12);
        c.set_attached_session(Some(target.session_handle()));
        let pane = server_client_get_pane(c).unwrap();
        let mut parsed = ByteBuffer::from(b"first\r\nsecond\0ignored\nrest".to_vec());
        server_client_print(Some(c), 1, &mut parsed);
        assert_eq!(parsed.as_slice(), b"rest");
        let mut literal = ByteBuffer::from(b"one\x01".to_vec());
        server_client_print(Some(c), 0, &mut literal);
        assert_eq!(literal.as_slice(), b"one\x01");
        let mode = pane.get().unwrap().active_mode().unwrap();
        assert_eq!(mode.mode(), WindowMode::View);
        let data = mode.state.copy_mode_data_ref().unwrap();
        let grid = data.backing.as_ref().unwrap().grid();
        for (row, expected) in [c"first", c"second", c"rest", c"one\\001"]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                crate::grid::grid_string_cells(
                    grid,
                    0,
                    row as u_int,
                    40,
                    None,
                    crate::grid::GRID_STRING_TRIM_SPACES,
                    None
                )
                .as_c_str(),
                expected
            );
        }
    }
}

#[test]
fn overlay_lifecycle_sets_timer_tty_state_replaces_and_clears_without_payload_callbacks() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(40, 12);
    unsafe {
        let c = &mut *attached.add("overlay", 40, 12);
        c.set_attached_session(Some(target.session_handle()));
        c.flags |= CLIENT_FOCUSED as uint64_t;
        crate::session::session_ref_of(&mut *target.session())
            .expect("session owner")
            .add_attached();
        server_client_update_focus(c);
        assert_ne!(*(*target.pane(0)).flags() & crate::window::PANE_FOCUSED, 0);
        server_client_clear_overlay(c);
        server_client_set_overlay(
            c,
            25,
            Overlay::DisplayPanes { keys: false },
            OverlayState::None,
        );
        assert!(c.overlay_timer.is_armed());
        assert_eq!(*(*target.pane(0)).flags() & crate::window::PANE_FOCUSED, 0);
        assert_ne!(c.tty.flags & TTY_FREEZE, 0);
        assert_ne!(c.tty.flags & TTY_NOCURSOR, 0);
        server_client_set_overlay(
            c,
            0,
            Overlay::DisplayPanes { keys: true },
            OverlayState::None,
        );
        assert!(!c.overlay_timer.is_armed());
        assert_eq!(c.tty.flags & TTY_FREEZE, TTY_FREEZE);
        server_client_clear_overlay(c);
        assert!(c.overlay().is_none());
        assert_ne!(*(*target.pane(0)).flags() & crate::window::PANE_FOCUSED, 0);
        assert_eq!(c.tty.flags & (TTY_FREEZE | TTY_NOCURSOR), 0);
        server_client_clear_overlay(c);
    }
}

#[test]
fn resize_checks_cover_unmarked_window_empty_queue_and_each_coalescing_shape() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let mut pane = target.state().pane_ref().unwrap();
        let window = pane.window().unwrap();
        server_client_check_window_resize(&window);
        let mut w = window.as_window_mut();
        w.flags |= WINDOW_RESIZE;
        w.set_pending_size(crate::pane_resize::PaneSize {
            width: 60,
            height: 20,
        });
        drop(w);
        server_client_check_window_resize(&window);
        let w = window.as_window();
        assert_eq!(
            (w.dimensions().size.width, w.dimensions().size.height),
            (40, 12)
        );
        drop(w);

        pane.deliver_pending_resize();
        pane.as_pane_mut().resize(PaneSize {
                width: 41,
                height: 13,
            });
        pane.deliver_pending_resize();
        assert!(crate::window_pane::resize_timer_for_test(pane.as_pane()).is_armed());
        crate::window_pane::resize_timer_for_test(pane.as_pane()).disarm();
        pane.as_pane_mut().resize(PaneSize {
                width: 42,
                height: 14,
            });
        pane.as_pane_mut().resize(PaneSize {
                width: 45,
                height: 16,
            });
        pane.deliver_pending_resize();
        crate::window_pane::resize_timer_for_test(pane.as_pane()).disarm();
    }
}

#[test]
fn pane_buffer_and_attached_lost_cover_empty_consumers_and_latest_client_reselection() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut target = Target::new(40, 12);
    unsafe {
        let first = attached.add("first-latest", 40, 12);
        let second = attached.add("second-latest", 40, 12);
        (*first).set_attached_session(Some(target.session_handle()));
        (*second).set_attached_session(Some(target.session_handle()));
        (*first).activity_time = timeval {
            tv_sec: 1,
            tv_usec: 0,
        };
        (*second).activity_time = timeval {
            tv_sec: 2,
            tv_usec: 0,
        };
        let empty = crate::tests::test_fixtures::Session::new(99, "no-window");
        let missing = attached.add("missing-current-window", 40, 12);
        (*missing).set_attached_session(Some(empty.handle()));
        (*missing).activity_time.tv_sec = 99;
        let detached = attached.add("detached", 40, 12);
        (*detached).activity_time.tv_sec = 100;
        window_set_latest(&mut *target.window(0), Some(&*first));
        server_client_attached_lost(&mut *first);
        assert_eq!(
            window_get_latest(&*target.window(0)).map(|latest| latest.as_ptr()),
            Some(second)
        );

        (*missing).set_attached_session(None);
        let wp = &mut *target.pane(0);
        wp.maintain_output();
        (*first).set_attached_session(None);
        (*second).set_attached_session(None);
        wp.maintain_output();
    }
}

#[test]
fn maintenance_refreshes_mode_styles_and_clears_each_windows_pane_redraw_flags() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    target.add_window(1, 20, 6);
    unsafe {
        let state = target.state();
        let window = state.window().unwrap();
        let mut pane = state.pane_ref().unwrap();
        window.options().set_number(c"automatic-rename", 0);
        let other_window = state
            .session()
            .unwrap()
            .as_session()
            .windows
            .get(&1)
            .unwrap()
            .window_handle()
            .unwrap()
            .clone();
        other_window.options().set_number(c"automatic-rename", 0);
        let mut other = other_window.as_window().panes[0].downgrade();
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode( None, WindowMode::View, None, None),
            0
        );
        *pane.get_mut().unwrap().flags_mut() =
            PANE_STYLECHANGED | PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        *other.get_mut().unwrap().flags_mut() = PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        server_client_loop();
        assert_eq!(
            *pane.get().unwrap().flags() & (PANE_REDRAW | PANE_REDRAWSCROLLBAR),
            0
        );
        assert_eq!(
            *other.get().unwrap().flags() & (PANE_REDRAW | PANE_REDRAWSCROLLBAR),
            0
        );
        assert_eq!(
            pane.get().unwrap().active_mode().unwrap().mode(),
            WindowMode::View
        );
    }
}

#[test]
fn terminal_metadata_uses_the_session_pane_even_with_a_client_selection() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let other = target.add_window(1, 20, 6);
    let mut attached = Clients::new();
    unsafe {
        let client = &mut *client_with_its_own_pane(&mut attached, &mut target);
        client.tty.term = Some(zeroed_term());
        let active = &mut *target.pane(0);
        active.base_mut().set_path(c"/session/path", 0);
        active.base_mut().set_progress_bar(PROGRESS_BAR_NORMAL, 42);
        let other_pane = &*target.pane(other);
        let selection = server_client_add_client_window(client, (*target.window(0)).window_id());
        selection.pane = (other_pane).observation();
        assert_eq!(
            server_client_get_pane(client).unwrap().id(),
            other_pane.pane_id()
        );
        server_client_set_path(client);
        server_client_set_progress_bar(client);
        assert_eq!(client.path.as_deref(), Some(c"/session/path"));
        assert_eq!(client.progress_bar.state, PROGRESS_BAR_NORMAL);
        assert_eq!(client.progress_bar.progress, 42);
        (*target.pane(0)).base_mut().clear_path();
        (*target.pane(0))
            .base_mut()
            .set_progress_bar(PROGRESS_BAR_HIDDEN, 0);
        server_client_set_path(client);
        server_client_set_progress_bar(client);
        assert_eq!(client.path.as_deref(), Some(c""));
        assert_eq!(client.progress_bar.state, PROGRESS_BAR_HIDDEN);
        assert_eq!(client.progress_bar.progress, 0);
        forget_client_windows(client);
    }
}

#[test]
fn terminal_metadata_leaves_cached_values_when_the_session_has_no_active_pane() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let mut attached = Clients::new();
    unsafe {
        let client = &mut *attached.add("client", 20, 6);
        client.set_attached_session(Some(target.session_handle()));
        client.path = Some(c"cached".to_owned());
        client.progress_bar = progress_bar {
            state: PROGRESS_BAR_PAUSED,
            progress: 19,
        };
        (*target.window(0)).active = None;
        server_client_set_path(client);
        server_client_set_progress_bar(client);
        assert_eq!(client.path.as_deref(), Some(c"cached"));
        assert_eq!(client.progress_bar.state, PROGRESS_BAR_PAUSED);
        assert_eq!(client.progress_bar.progress, 19);
        client.set_attached_session(None);
        server_client_set_title(client);
        server_client_set_path(client);
        server_client_set_progress_bar(client);
        assert_eq!(client.path.as_deref(), Some(c"cached"));
        assert_eq!(client.progress_bar.progress, 19);
    }
}

#[test]
fn focus_update_accepts_clients_without_a_session_or_current_window() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let mut attached = Clients::new();
    unsafe {
        let client = &mut *attached.add("unattached", 20, 6);
        server_client_update_focus(client);
        client.set_attached_session(Some(target.session_handle()));
        (*target.session()).curw = None;
        server_client_update_focus(client);
        assert_eq!(*(*target.pane(0)).flags() & crate::window::PANE_FOCUSED, 0);
    }
}

#[test]
fn reset_state_reads_mouse_tracking_from_every_shown_pane_screen() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("mouse", 40, 12);
        c.set_attached_session(Some(target.session_handle()));
        c.tty.client = Some(client_ref_of(c).unwrap().downgrade());
        c.tty.term = Some(zeroed_term());
        c.tty.out = Some(Box::new(crate::reactor::ByteBuffer::new()));
        (&*target.session())
            .options_ref()
            .clone()
            .set_number(c"mouse", 1);
        (&*target.session())
            .options_ref()
            .clone()
            .set_number(c"focus-follows-mouse", 0);
        let id = crate::window::window_add_pane(&mut *target.window(0), None, 0, 0).id();
        let mut pane = window_pane_find_by_id(id).unwrap();
        {
            let pane = pane.get_mut().unwrap();
            let mode = pane.base().mode() | MODE_MOUSE_ALL;
            let pane_screen_mode = mode;
            pane.base_mut().set_mode(pane_screen_mode);
        }
        server_client_reset_state(c);
        assert_ne!(c.tty.mode & MODE_MOUSE_ALL, 0);
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode(
                None,
                WindowMode::View,
                None,
                None
            ),
            0
        );
        let shown = pane.get().unwrap().active_mode().unwrap()
            .state
            .copy_mode_data_ref()
            .unwrap()
            .screen
            .clone();
        {
            let mut screen = shown.borrow_mut();
            let mode = screen.mode() & !ALL_MOUSE_MODES;
            screen.set_mode(mode);
        }
        server_client_reset_state(c);
        assert_eq!(c.tty.mode & MODE_MOUSE_ALL, 0);
        assert_ne!(c.tty.mode & MODE_MOUSE_BUTTON, 0);
        (&*target.session())
            .options_ref()
            .clone()
            .set_number(c"focus-follows-mouse", 1);
        server_client_reset_state(c);
        assert_ne!(c.tty.mode & MODE_MOUSE_ALL, 0);
        c.set_attached_session(None);
        server_client_reset_state(c);
    }
}

#[test]
fn deferred_redraw_keeps_pane_positions_until_the_output_queue_drains() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("deferred", 20, 6);
        c.set_attached_session(Some(target.session_handle()));
        c.tty.client = Some(client_ref_of(c).unwrap().downgrade());
        c.tty.term = Some(zeroed_term());
        c.tty.out = Some(Box::new(crate::reactor::ByteBuffer::new()));
        c.tty.out.as_mut().unwrap().append(b"pending");
        let second = crate::window::window_add_pane(&mut *target.window(0), None, 0, 0).id();
        let mut other = window_pane_find_by_id(second).unwrap();
        *(*target.pane(0)).flags_mut() |= PANE_REDRAW;
        *other.get_mut().unwrap().flags_mut() = PANE_REDRAWSCROLLBAR;
        server_client_check_redraw(c);
        assert_eq!(c.redraw_panes, 1);
        assert_eq!(c.redraw_scrollbars, 2);
        assert_ne!(c.flags & CLIENT_REDRAWPANES as uint64_t, 0);
        assert_ne!(c.flags & CLIENT_REDRAWSCROLLBARS, 0);
        assert_eq!(c.tty.out.as_mut().unwrap().as_slice(), b"pending");
        c.tty.out.as_mut().unwrap().clear();
        *(*target.pane(0)).flags_mut() &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR);
        *other.get_mut().unwrap().flags_mut() &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR);
        server_client_check_redraw(c);
        assert_eq!(c.redraw_panes, 0);
        assert_eq!(c.redraw_scrollbars, 0);
        assert_eq!(
            c.flags & (CLIENT_REDRAWPANES as uint64_t | CLIENT_REDRAWSCROLLBARS),
            0
        );
    }
}

#[test]
fn status_redraw_refreshes_owned_list_modes_only_in_the_current_window() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let foreign = target.add_window(1, 40, 12);
    let state = target.state();
    let window = state.window().unwrap();
    let mut panes = vec![state.pane_ref().unwrap()];
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("mode-updates", 40, 12);
        c.set_attached_session(Some(target.session_handle()));
        for _ in 0..2 {
            let id = crate::window::window_add_pane(&mut window.as_window_mut(), None, 0, 0).id();
            panes.push(window_pane_find_by_id(id).unwrap());
        }
        for (pane, mode) in
            panes
                .iter_mut()
                .zip([WindowMode::Buffer, WindowMode::Client, WindowMode::Tree])
        {
            assert_eq!(
                (pane.get_mut().unwrap()).set_mode( None, mode, Some(&state), None),
                0
            );
            *pane.get_mut().unwrap().flags_mut() &= !PANE_REDRAW;
        }
        let mut foreign_pane = window_pane_find_by_id((*target.pane(foreign)).pane_id()).unwrap();
        assert_eq!(
            (foreign_pane.get_mut().unwrap()).set_mode(
                None,
                WindowMode::Buffer,
                Some(&state),
                None,
            ),
            0
        );
        *foreign_pane.get_mut().unwrap().flags_mut() &= !PANE_REDRAW;
        for flags in [
            0,
            (CLIENT_REDRAWSTATUS | CLIENT_CONTROL) as uint64_t,
            (CLIENT_REDRAWSTATUS | CLIENT_SUSPENDED) as uint64_t,
        ] {
            c.flags = flags;
            server_client_check_modes(c);
            assert!(
                panes
                    .iter()
                    .all(|pane| *pane.get().unwrap().flags() & PANE_REDRAW == 0)
            );
        }
        c.flags = CLIENT_REDRAWSTATUS as uint64_t;
        server_client_check_modes(c);
        assert!(
            panes
                .iter()
                .all(|pane| *pane.get().unwrap().flags() & PANE_REDRAW != 0)
        );
        assert_eq!(*foreign_pane.get().unwrap().flags() & PANE_REDRAW, 0);
        c.set_attached_session(None);
        server_client_check_modes(c);
    }
}

#[test]
fn default_key_table_names_share_options_and_survive_replacement() {
    let _guard = globals();
    let target = Target::new(20, 6);
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("table-owner", 20, 6);
        c.set_attached_session(Some(target.session_handle()));
        let options = target.session_handle().options();
        options.set_string(c"key-table", 0, c"retained-table", fmt_args![]);
        let name = server_client_get_key_table(c);
        assert!(std::rc::Rc::ptr_eq(
            &name,
            &options.string_ref(c"key-table")
        ));
        options.set_string(c"key-table", 0, c"replacement-table", fmt_args![]);
        assert_eq!(name.as_ref(), c"retained-table");
        assert_eq!(
            server_client_get_key_table(c).as_ref(),
            c"replacement-table"
        );
        options.set_string(c"key-table", 0, c"", fmt_args![]);
        assert_eq!(server_client_get_key_table(c).as_ref(), c"root");
        c.set_attached_session(None);
        assert_eq!(server_client_get_key_table(c).as_ref(), c"root");
    }
}

#[test]
fn queued_keys_and_pastes_use_live_panes_and_skip_missing_targets() {
    use crate::tests::test_fixtures::{Item, StreamBuffer};
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let mut attached = Clients::new();
    let output = StreamBuffer::new();
    let state = target.state();
    let mut pane = state.pane_ref().unwrap();
    unsafe {
        let c = &mut *attached.add("queued-input", 20, 6);
        c.set_attached_session(Some(target.session_handle()));
        c.tty.client = Some(client_ref_of(c).unwrap().downgrade());
        c.tty.term = Some(zeroed_term());
        c.tty.out = Some(Box::new(ByteBuffer::new()));
        target.session_handle().options().set_string(
            c"key-table",
            0,
            c"queued-input-empty",
            fmt_args![],
        );
        server_client_set_key_table(c, None);
        let mut item = Item::with_client();
        item.set_client(c);
        let item = item.handle();
        pane.get_mut().unwrap().configure_test(crate::window_pane::PaneTestSetup::Descriptor(0));
        pane.get_mut().unwrap().configure_test(crate::window_pane::PaneTestSetup::Stream(output.ptr()));
        let send = |key, bytes: &[u8]| {
            server_client_key_callback(
                &item,
                Box::new(key_event {
                    key,
                    m: mouse_event::default(),
                    buf: bytes.to_vec(),
                }),
            )
        };
        assert_eq!(send(b'x' as key_code, b""), CMD_RETURN_NORMAL);
        assert_eq!(output.written(), b"x");
        c.flags |= CLIENT_BRACKETPASTING;
        assert_eq!(send(b'y' as key_code, b"paste payload"), CMD_RETURN_NORMAL);
        assert_eq!(output.written(), b"paste payload");
        c.flags |= CLIENT_READONLY as uint64_t;
        assert_eq!(send(b'z' as key_code, b"ignored"), CMD_RETURN_NORMAL);
        assert!(output.written().is_empty());
        c.flags &= !(CLIENT_READONLY as uint64_t);
        pane.get_mut().unwrap().configure_test(crate::window_pane::PaneTestSetup::Descriptor(-1));
        let window = pane.window().unwrap();
        let pane_options = pane.get().unwrap().options_ref().clone();
        window.remove_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        );
        assert!(pane.get().is_none());
        assert_eq!(send(b'z' as key_code, b"missing pane"), CMD_RETURN_NORMAL);
        assert!(output.written().is_empty());
        let mut payload = crate::tests::test_fixtures::PaneAllocation::default();
        payload.set_pane_id(99);
        *payload.base_mut() = RustScreen::new_with_server_options(20, 6, 0);
        unsafe { payload.configure_test(crate::window_pane::PaneTestSetup::Options(Some(pane_options))) };
        payload.configure_test(crate::window_pane::PaneTestSetup::Descriptor(0));
        payload.configure_test(crate::window_pane::PaneTestSetup::Stream(output.ptr()));
        let unregistered = (payload).into_owner();
        let observed = unregistered.downgrade();
        crate::window::window_panes_insert_tail(&mut window.as_window_mut(), unregistered);
        let mut unregistered = observed;
        {
            let mut payload = window.as_window_mut();
            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == unregistered.id())
                .map(|pane| pane.downgrade());
        }
        assert!(window_pane_find_by_id(unregistered.id()).is_none());
        c.flags &= !CLIENT_BRACKETPASTING;
        assert_eq!(send(b'x' as key_code, b""), CMD_RETURN_NORMAL);
        assert!(output.written().is_empty());
        c.flags |= CLIENT_BRACKETPASTING;
        assert_eq!(
            send(b'y' as key_code, b"listed pane paste"),
            CMD_RETURN_NORMAL
        );
        assert_eq!(output.written(), b"listed pane paste");
        unregistered.get_mut().unwrap().configure_test(crate::window_pane::PaneTestSetup::Descriptor(-1));
        c.set_attached_session(None);
        assert_eq!(send(b'z' as key_code, b"detached"), CMD_RETURN_NORMAL);
        assert!(output.written().is_empty());
        drop(target);
    }
}

#[test]
fn repeat_timing_reads_the_attached_sessions_current_options() {
    let _guard = globals();
    let target = Target::new(20, 6);
    let mut attached = Clients::new();
    unsafe {
        crate::key_bindings::key_bindings_add(
            c"repeat-owner",
            b'x' as key_code,
            None,
            1,
            Some(CmdListRef::empty()),
        );
        let table = key_bindings_get_table_ref(c"repeat-owner", 0).unwrap();
        let binding = table.borrow().binding(b'x' as key_code).unwrap().clone();
        let c = &mut *attached.add("repeat-owner", 20, 6);
        assert_eq!(server_client_repeat_time(c, &binding), 0);
        c.set_attached_session(Some(target.session_handle()));
        let options = target.session_handle().options();
        options.set_number(c"repeat-time", 50);
        options.set_number(c"initial-repeat-time", 120);
        assert_eq!(server_client_repeat_time(c, &binding), 120);
        c.flags |= CLIENT_REPEAT as uint64_t;
        c.last_key = b'x' as key_code;
        assert_eq!(server_client_repeat_time(c, &binding), 50);
        options.set_number(c"repeat-time", 75);
        assert_eq!(server_client_repeat_time(c, &binding), 75);
        c.last_key = b'y' as key_code;
        assert_eq!(server_client_repeat_time(c, &binding), 120);
        options.set_number(c"initial-repeat-time", 0);
        assert_eq!(server_client_repeat_time(c, &binding), 75);
        options.set_number(c"repeat-time", 0);
        assert_eq!(server_client_repeat_time(c, &binding), 0);
        c.set_attached_session(None);
        assert_eq!(server_client_is_assume_paste(c), 0);
    }
}

#[test]
fn latest_client_updates_follow_the_current_window_and_tolerate_missing_links() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    target.add_window(1, 20, 6);
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("latest-owner", 20, 6);
        c.set_attached_session(Some(target.session_handle()));
        let client = client_ref_of(c).unwrap();
        server_client_update_latest(c);
        assert!(
            window_get_latest(&*target.window(0))
                .unwrap()
                .ptr_eq(&client)
        );
        assert!(window_get_latest(&*target.window(1)).is_none());
        let session = target.session_handle().clone();
        session.set_curw(target.winlink(1).as_ref());
        server_client_update_latest(c);
        assert!(
            window_get_latest(&*target.window(1))
                .unwrap()
                .ptr_eq(&client)
        );
        server_client_update_latest(c);
        session.set_curw(None);
        server_client_update_latest(c);
        c.set_attached_session(None);
        server_client_update_latest(c);
    }
}

#[test]
fn mouse_hit_testing_reads_scrollbars_and_listed_borders_then_skips_retired_panes() {
    use crate::pane_scrollbar::PaneScrollbar;
    use crate::pane_scrollbar_style::PaneScrollbarStyleState;

    let _guard = globals();
    let mut target = Target::new(40, 12);
    let mut pane = target.state().pane_ref().unwrap();
    let window = pane.window().unwrap();
    unsafe {
        pane.as_pane_mut()
            .configure_test(crate::window_pane::PaneTestSetup::Geometry(crate::pane_geometry::PaneGeometry {
                xoff: 4,
                yoff: 2,
                sx: 20,
                sy: 6,
            }));
        pane.as_pane_mut()
            .set_slider(crate::pane_scrollbar::PaneScrollbarSlider { sb_slider_y: 2, sb_slider_h: 2 });
        pane.as_pane_mut()
            .set_scrollbar_style(crate::pane_scrollbar_style::PaneScrollbarStyle {
                width: 2,
                padding: 1,
                ..Default::default()
            });
        let mut offset = 99;
        for (position, x) in [(PANE_SCROLLBARS_RIGHT, 25), (PANE_SCROLLBARS_LEFT, 1)] {
            window.set_scrollbar_settings(crate::window_scrollbar::WindowScrollbarSettings {
                sb: crate::window::PANE_SCROLLBARS_ALWAYS,
                sb_pos: position,
            });
            assert_eq!(
                server_client_check_mouse_in_pane(&pane, x, 3, &mut offset),
                KEYC_MOUSE_LOCATION_SCROLLBAR_UP
            );
            assert_eq!(
                server_client_check_mouse_in_pane(&pane, x, 5, &mut offset),
                KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER
            );
            assert_eq!(offset, 1);
            assert_eq!(
                server_client_check_mouse_in_pane(&pane, x, 6, &mut offset),
                KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN
            );
        }
        window.set_scrollbar_settings(crate::window_scrollbar::WindowScrollbarSettings {
            sb: 0,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        let mut payload = crate::tests::test_fixtures::PaneAllocation::default();
        payload.set_pane_id(99);
        *payload.base_mut() = RustScreen::new_with_server_options(4, 2, 0);
        unsafe { payload.configure_test(crate::window_pane::PaneTestSetup::Options(Some(pane.as_pane().options_ref().clone()))) };
        payload.configure_test(crate::window_pane::PaneTestSetup::Geometry(crate::pane_geometry::PaneGeometry {
            xoff: 30,
            yoff: 8,
            sx: 4,
            sy: 2,
        }));
        let listed = (payload).into_owner();
        let observed = listed.downgrade();
        crate::window::window_panes_insert_tail(&mut window.as_window_mut(), listed);
        let mut listed = observed;
        assert!(window_pane_find_by_id(listed.id()).is_none());
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 34, 9, &mut offset),
            KEYC_MOUSE_LOCATION_BORDER
        );
        window.as_window_mut().flags |= WINDOW_ZOOMED;
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 34, 9, &mut offset),
            KEYC_MOUSE_LOCATION_NOWHERE
        );
        *listed.as_pane_mut().flags_mut() |= PANE_ZOOMED;
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 34, 9, &mut offset),
            KEYC_MOUSE_LOCATION_BORDER
        );
        window.as_window_mut().flags &= !WINDOW_ZOOMED;
        window.remove_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        );
        assert_eq!(
            server_client_check_mouse_in_pane(&pane, 5, 3, &mut offset),
            KEYC_MOUSE_LOCATION_NOWHERE
        );
        drop(target);
    }
}

#[test]
fn mouse_events_follow_focus_and_keep_drag_targets_across_current_window_changes() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    target.add_window(1, 40, 12);
    let mut attached = Clients::new();
    let mut first = target.state().pane_ref().unwrap();
    let window = first.window().unwrap();
    unsafe {
        let second_id =
            crate::window::window_add_pane(&mut window.as_window_mut(), None, 100, 0).id();
        let mut second = window_pane_find_by_id(second_id).unwrap();
        first
            .as_pane_mut()
            .configure_test(crate::window_pane::PaneTestSetup::Geometry(crate::pane_geometry::PaneGeometry {
                xoff: 0,
                yoff: 0,
                sx: 19,
                sy: 12,
            }));
        second
            .as_pane_mut()
            .configure_test(crate::window_pane::PaneTestSetup::Geometry(crate::pane_geometry::PaneGeometry {
                xoff: 20,
                yoff: 0,
                sx: 20,
                sy: 12,
            }));
        {
            let mut payload = window.as_window_mut();
            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == first.id())
                .map(|pane| pane.downgrade());
        }
        let c = &mut *attached.add("mouse-owners", 40, 12);
        c.set_attached_session(Some(target.session_handle()));
        c.flags |= crate::status::CLIENT_STATUSOFF as uint64_t;
        c.tty.client = Some(client_ref_of(c).unwrap().downgrade());
        c.tty.osx = 40;
        c.tty.osy = 12;
        c.tty.mouse_last_pane = -1;
        let mut event = key_event {
            m: mouse_event {
                x: 25,
                y: 3,
                sgr_type: b'M' as u_int,
                sgr_b: (MOUSE_MASK_DRAG | 3) as u_int,
                ..Default::default()
            },
            ..Default::default()
        };
        target
            .session_handle()
            .options()
            .set_number(c"focus-follows-mouse", 0);
        assert_eq!(
            server_client_check_mouse(c, &mut event),
            KEYC_MOUSEMOVE_PANE
        );
        assert_eq!(event.m.wp, second_id as core::ffi::c_int);
        assert_eq!(window.active_pane_id(), Some(first.id()));
        target
            .session_handle()
            .options()
            .set_number(c"focus-follows-mouse", 1);
        assert_eq!(
            server_client_check_mouse(c, &mut event),
            KEYC_MOUSEMOVE_PANE
        );
        assert_eq!(window.active_pane_id(), Some(second_id));
        event.m = mouse_event {
            x: 26,
            y: 3,
            lx: 25,
            ly: 3,
            b: MOUSE_MASK_DRAG as u_int,
            sgr_type: b'M' as u_int,
            sgr_b: MOUSE_MASK_DRAG as u_int,
            ..Default::default()
        };
        assert_eq!(
            server_client_check_mouse(c, &mut event),
            KEYC_MOUSEDRAG1_PANE
        );
        assert_eq!(c.tty.mouse_last_pane, second_id as core::ffi::c_int);
        let session = target.session_handle().clone();
        session.set_curw(target.winlink(1).as_ref());
        event.m.x = 27;
        assert_eq!(
            server_client_check_mouse(c, &mut event),
            KEYC_MOUSEDRAG1_PANE
        );
        assert_eq!(event.m.wp, second_id as core::ffi::c_int);
        assert_eq!(event.m.w, window.window_id() as core::ffi::c_int);
        c.tty.mouse_scrolling_flag = 1;
        assert_eq!(
            server_client_check_mouse(c, &mut event),
            KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER
        );
        assert_eq!(event.m.wp, second_id as core::ffi::c_int);
        let released = std::rc::Rc::new(std::cell::Cell::new(false));
        let observed = released.clone();
        let held = window.clone();
        c.tty.mouse_drag_release = Some(std::rc::Rc::new(move |_, m| {
            assert_eq!(m.wp, second_id as core::ffi::c_int);
            let window = held.clone();
            window.remove_pane(
                &crate::window::window_pane_find_by_id(second_id).expect("the pane exists"),
            );
            observed.set(true);
        }));
        event.m.b = 3;
        event.m.sgr_b = 0;
        event.m.sgr_type = b'm' as u_int;
        assert_ne!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
        assert!(released.get());
        assert!(second.get().is_none());
        assert_eq!(c.tty.mouse_last_pane, -1);
        assert!(c.tty.mouse_drag_release.is_none());
        assert_eq!(c.tty.mouse_scrolling_flag, 0);
    }
}

#[test]
fn mouse_status_ranges_resolve_owned_links_and_skip_missing_context() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    target.add_window(7, 40, 12);
    let mut attached = Clients::new();
    unsafe {
        let c = &mut *attached.add("mouse-ranges", 40, 12);
        c.set_attached_session(Some(target.session_handle()));
        let mut session = target.session_handle().clone();
        session.as_session_mut().statusat = 0;
        session.as_session_mut().statuslines = 1;
        c.status.entries[0].ranges.push(style_range {
            type_0: STYLE_RANGE_WINDOW,
            argument: 7,
            string: [0; 16],
            start: 0,
            end: 10,
        });
        let mut event = key_event {
            m: mouse_event {
                x: 2,
                y: 0,
                b: MOUSE_WHEEL_UP as u_int,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_ne!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
        assert_eq!(event.m.w, 1);
        assert_eq!(event.m.wp, -1);
        c.status.entries[0].ranges[0].argument = 8;
        assert_eq!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
        c.status.entries[0].ranges[0].type_0 = STYLE_RANGE_PANE;
        c.status.entries[0].ranges[0].argument = 0;
        assert_ne!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
        assert_eq!(event.m.wp, 0);
        c.status.entries[0].ranges[0].argument = 99;
        assert_eq!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
        session.set_curw(None);
        assert_eq!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
        c.set_attached_session(None);
        assert_eq!(server_client_check_mouse(c, &mut event), KEYC_UNKNOWN);
    }
}

#[test]
fn attached_session_lookup_retains_a_live_session_and_rejects_a_dropped_one() {
    let _guard = globals();
    let session = SessionRef::new(session::default());
    let weak = session.downgrade();
    let mut client = client::default();
    client.set_attached_session(Some(&session));
    let held = client
        .attached_session()
        .expect("the session is registered");
    assert!(held.ptr_eq(&session));
    drop(session);
    assert!(weak.upgrade().is_some());
    assert!(client.attached_session().is_some());
    drop(held);
    assert!(weak.upgrade().is_none());
    assert!(client.attached_session().is_none());
    assert!(client::default().attached_session().is_none());
}
