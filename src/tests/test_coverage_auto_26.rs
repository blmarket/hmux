//! Coverage for [`crate::cmd`] – cmd_find helpers with [`Target`] fixture.

use crate::cmd::{
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, cmd_find_best_client, cmd_find_clear_state,
    cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_nothing, cmd_find_from_pane,
    cmd_find_from_session, cmd_find_from_session_window, cmd_find_from_window,
    cmd_find_from_winlink, cmd_find_from_winlink_pane, cmd_find_target, cmd_find_valid_state,
};
use crate::tests::test_fixtures::{Item, Target, globals};

// ---------------------------------------------------------------------------
// cmd_find_clear_state / empty_state / valid_state
// ---------------------------------------------------------------------------

#[test]
fn cmd_find_clear_state_resets_and_sets_flags() {
    let _g = globals();
    let mut fs: crate::types::cmd_find_state = *Box::new(crate::types::cmd_find_state::default());
    {
        cmd_find_clear_state(&mut fs, 0x2);
        assert_eq!(fs.flags, 0x2);
        assert_eq!(fs.idx, -1);
        assert!(fs.session().is_none());
        assert!(fs.winlink_ref().is_none());
        assert!(fs.window().is_none());
        assert!(fs.pane_list_ref().is_none());
        assert_eq!(cmd_find_empty_state(&fs), 1);

        cmd_find_clear_state(&mut fs, 0);
        assert_eq!(fs.flags, 0);
        assert_eq!(fs.idx, -1);
    }
}

#[test]
fn cmd_find_empty_and_valid_state_with_target() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let mut empty: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        cmd_find_clear_state(&mut empty, 0);
        assert_eq!(cmd_find_empty_state(&empty), 1);
        assert_eq!(cmd_find_valid_state(&empty), 0);

        let fs = target.state();
        // state() returns a valid find state pointing at session curw
        let mut valid = fs;
        assert_eq!(cmd_find_empty_state(&valid), 0);
        assert_eq!(cmd_find_valid_state(&valid), 1);

        // clearing makes it invalid again
        cmd_find_clear_state(&mut valid, 0);
        assert_eq!(cmd_find_valid_state(&valid), 0);
        assert_eq!(cmd_find_empty_state(&valid), 1);
    }
}

#[test]
fn cmd_find_copy_state_duplicates_all_fields() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let mut src = target.state();
        let mut dst: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        cmd_find_clear_state(&mut dst, 0x99);
        cmd_find_copy_state(&mut dst, &src);
        assert_eq!(
            dst.session().as_ref().map(|s| s.as_ptr()),
            src.session().as_ref().map(|s| s.as_ptr())
        );
        assert!(core::ptr::eq(
            dst.winlink_ref().unwrap().get().unwrap(),
            src.winlink_ref().unwrap().get().unwrap()
        ));
        assert_eq!(
            dst.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            src.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr())
        );
        assert!(core::ptr::addr_eq(
            dst.pane_list_ref().unwrap().get().unwrap(),
            src.pane_list_ref().unwrap().get().unwrap()
        ));
        assert_eq!(dst.idx, src.idx);
        // copy does not touch flags or current
        assert_eq!(dst.flags, 0x99);
        // valid copy is still valid
        assert_eq!(cmd_find_valid_state(&dst), 1);
        assert_eq!(cmd_find_empty_state(&dst), 0);
    }
}

// ---------------------------------------------------------------------------
// cmd_find_from_* helpers – build find states from Target pieces
// ---------------------------------------------------------------------------

#[test]
fn cmd_find_from_session_builds_valid_state() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let s = target.session();
        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        cmd_find_from_session(&mut fs, &*s, 0);
        assert_eq!(
            fs.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            s
        );
        assert!(fs.winlink_ref().is_some());
        assert!(!fs.window().is_none());
        assert!(fs.pane_list_ref().is_some());
        assert_eq!(cmd_find_valid_state(&fs), 1);
        // curw's window matches w
        assert!(
            fs.window().unwrap().ptr_eq(
                fs.winlink_ref()
                    .unwrap()
                    .get()
                    .unwrap()
                    .window_handle()
                    .unwrap()
            )
        );
    }
}

#[test]
fn cmd_find_from_winlink_builds_state_pointing_at_winlink() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    target.add_window(1, 80, 24);
    unsafe {
        let wl0 = target.winlink(0);
        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        cmd_find_from_winlink(&mut fs, &*wl0, 0);
        assert!(core::ptr::eq(fs.winlink_ref().unwrap().get().unwrap(), wl0));
        assert!(fs.window().unwrap().ptr_eq((*wl0).window_handle().unwrap()));
        assert_eq!(
            fs.session().as_ref().map(|s| s.as_ptr()),
            Some((*wl0).session().expect("a link has a session").as_ptr())
        );
        assert!(
            fs.pane_list_ref()
                .unwrap()
                .ptr_eq(&(*wl0).window_handle().unwrap().active_pane().unwrap())
        );
        assert_eq!(cmd_find_valid_state(&fs), 1);

        let wl1 = target.winlink(1);
        cmd_find_from_winlink(&mut fs, &*wl1, 0);
        assert!(core::ptr::eq(fs.winlink_ref().unwrap().get().unwrap(), wl1));
        assert!(fs.window().unwrap().ptr_eq((*wl1).window_handle().unwrap()));
        assert_eq!(cmd_find_valid_state(&fs), 1);
    }
}

#[test]
fn cmd_find_from_session_window_and_from_window() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    target.add_window(1, 80, 24);
    unsafe {
        let s = target.session();
        let w0 = target.window(0);
        let w1 = target.window(1);

        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(
            cmd_find_from_session_window(
                &mut fs,
                &*s,
                &crate::window::window_ref_of(&*w0).unwrap(),
                0
            ),
            0
        );
        assert_eq!(
            fs.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            s
        );
        assert_eq!(
            fs.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            w0
        );
        assert_eq!(cmd_find_valid_state(&fs), 1);

        assert_eq!(
            cmd_find_from_session_window(
                &mut fs,
                &*s,
                &crate::window::window_ref_of(&*w1).unwrap(),
                0
            ),
            0
        );
        assert_eq!(
            fs.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            w1
        );
        assert_eq!(cmd_find_valid_state(&fs), 1);

        // from_window finds the session that owns the window
        let mut fs2: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(
            cmd_find_from_window(&mut fs2, &crate::window::window_ref_of(&*w0).unwrap(), 0),
            0
        );
        assert_eq!(
            fs2.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            w0
        );
        assert_eq!(
            fs2.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            s
        );
        assert_eq!(cmd_find_valid_state(&fs2), 1);

        // unknown window (not in any session) fails
        let mut orphan = crate::tests::test_fixtures::Window::new(999, "orphan", 80, 24);
        let mut fs3: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(cmd_find_from_window(&mut fs3, &orphan.reference(), 0), -1);
        assert_eq!(cmd_find_valid_state(&fs3), 0);
    }
}

#[test]
fn cmd_find_from_pane_and_from_winlink_pane() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let wp = target.pane(0);
        let wl = target.winlink(0);

        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(cmd_find_from_pane(&mut fs, &*wp, 0), 0);
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            wp
        ));
        assert!(!fs.window().is_none());
        assert!(!fs.session().is_none());
        assert_eq!(cmd_find_valid_state(&fs), 1);

        let mut fs2: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        cmd_find_from_winlink_pane(&mut fs2, &*wl, &*wp, 0);
        assert!(core::ptr::addr_eq(
            fs2.pane_list_ref().unwrap().get().unwrap(),
            wp
        ));
        assert!(core::ptr::eq(fs2.winlink_ref().unwrap().get().unwrap(), wl));
        assert!(fs2.window().unwrap().ptr_eq((*wl).window_handle().unwrap()));
        assert_eq!(
            fs2.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            (*wl)
                .session()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr())
        );
        assert_eq!(cmd_find_valid_state(&fs2), 1);
    }
}

#[test]
fn cmd_find_from_nothing_finds_registered_session() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(cmd_find_from_nothing(&mut fs, 0), 0);
        assert_eq!(
            fs.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            target.session()
        );
        assert!(fs.winlink_ref().is_some());
        assert!(!fs.window().is_none());
        assert!(fs.pane_list_ref().is_some());
        assert_eq!(cmd_find_valid_state(&fs), 1);

        // best_client with no clients attached to session returns null (no attached client)
        let c = cmd_find_best_client(&*target.session());
        assert!(c.is_none());
    }
}

// ---------------------------------------------------------------------------
// cmd_find_target – resolve targets via Item targeting Target
// ---------------------------------------------------------------------------

#[test]
fn cmd_find_target_null_target_copies_current() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        // item's current is target.state() via targeting()
        let mut item = Item::new().targeting(&mut target);
        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        let rc = cmd_find_target(&mut fs, &mut *item.ptr(), None, CMD_FIND_PANE, 0);
        assert_eq!(rc, 0);
        let expected = target.state();
        assert_eq!(
            fs.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            expected
                .session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr())
        );
        assert!(core::ptr::eq(
            fs.winlink_ref().unwrap().get().unwrap(),
            expected.winlink_ref().unwrap().get().unwrap()
        ));
        assert_eq!(
            fs.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            expected
                .window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr())
        );
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            expected.pane_list_ref().unwrap().get().unwrap()
        ));
    }
}

#[test]
fn cmd_find_target_explicit_ids_and_window_index() {
    let _g = globals();
    let mut target = Target::new(80, 24);
    target.add_window(1, 80, 24);
    unsafe {
        let mut item = Item::new().targeting(&mut target);
        let mut fs: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());

        // session by id "$0" with SESSION type
        let t_session = std::ffi::CString::new("$0").unwrap();
        assert_eq!(
            cmd_find_target(
                &mut fs,
                &mut *item.ptr(),
                Some(&t_session),
                CMD_FIND_SESSION,
                0
            ),
            0
        );
        assert_eq!(
            fs.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            target.session()
        );

        // window by id "@0"
        let t_window = std::ffi::CString::new("@0").unwrap();
        assert_eq!(
            cmd_find_target(
                &mut fs,
                &mut *item.ptr(),
                Some(&t_window),
                CMD_FIND_WINDOW,
                0
            ),
            0
        );
        assert_eq!(
            fs.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            target.window(0)
        );

        // pane by id "%0"
        let t_pane = std::ffi::CString::new("%0").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, &mut *item.ptr(), Some(&t_pane), CMD_FIND_PANE, 0),
            0
        );
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            target.pane(0)
        ));

        // window by index "0" and "1" via session (session is current "0")
        let t_idx0 = std::ffi::CString::new("0").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, &mut *item.ptr(), Some(&t_idx0), CMD_FIND_WINDOW, 0),
            0
        );
        assert_eq!(
            fs.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            target.window(0)
        );

        let t_idx1 = std::ffi::CString::new("1").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, &mut *item.ptr(), Some(&t_idx1), CMD_FIND_WINDOW, 0),
            0
        );
        assert_eq!(
            fs.window()
                .as_ref()
                .map_or(core::ptr::null_mut(), |w| w.as_ptr()),
            target.window(1)
        );

        // nonsense target with QUIET should fail
        let t_bad = std::ffi::CString::new("no-such-session-xyz").unwrap();
        let mut fs2: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(
            cmd_find_target(
                &mut fs2,
                &mut *item.ptr(),
                Some(&t_bad),
                CMD_FIND_SESSION,
                2 // CMD_FIND_QUIET
            ),
            -1
        );
    }
}
