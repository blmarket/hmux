//! Coverage for [`crate::cmd`] – cmd_find helpers with [`Target`] fixture.

use crate::cmd::{
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, cmd_find_best_client, cmd_find_clear_state,
    cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_nothing, cmd_find_from_pane,
    cmd_find_from_session, cmd_find_from_session_window, cmd_find_from_window,
    cmd_find_from_winlink, cmd_find_from_winlink_pane, cmd_find_target, cmd_find_valid_state,
};
use crate::tests::test_fixtures::{Item, Target, globals};
use crate::window::window_get_active;
use ::core::ptr::null_mut;

// ---------------------------------------------------------------------------
// cmd_find_clear_state / empty_state / valid_state
// ---------------------------------------------------------------------------

#[test]
fn cmd_find_clear_state_resets_and_sets_flags() {
    let _g = globals();
    let mut fs: crate::types::cmd_find_state = *Box::new(crate::types::cmd_find_state::default());
    unsafe {
        cmd_find_clear_state(&mut fs, 0x2);
        assert_eq!(fs.flags, 0x2);
        assert_eq!(fs.idx, -1);
        assert!(fs.session().is_null());
        assert!(fs.winlink().is_null());
        assert!(fs.window().is_null());
        assert!(fs.pane().is_null());
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
        assert_eq!(dst.session(), src.session());
        assert_eq!(dst.winlink(), src.winlink());
        assert_eq!(dst.window(), src.window());
        assert_eq!(dst.pane(), src.pane());
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
        cmd_find_from_session(&mut fs, s, 0);
        assert_eq!(fs.session(), s);
        assert!(!fs.winlink().is_null());
        assert!(!fs.window().is_null());
        assert!(!fs.pane().is_null());
        assert_eq!(cmd_find_valid_state(&fs), 1);
        // curw's window matches w
        assert_eq!(fs.window(), (*fs.winlink()).window());
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
        cmd_find_from_winlink(&mut fs, wl0, 0);
        assert_eq!(fs.winlink(), wl0);
        assert_eq!(fs.window(), (*wl0).window());
        assert_eq!(fs.session(), (*wl0).session());
        assert_eq!(fs.pane(), window_get_active((*wl0).window()));
        assert_eq!(cmd_find_valid_state(&fs), 1);

        let wl1 = target.winlink(1);
        cmd_find_from_winlink(&mut fs, wl1, 0);
        assert_eq!(fs.winlink(), wl1);
        assert_eq!(fs.window(), (*wl1).window());
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
        assert_eq!(cmd_find_from_session_window(&mut fs, s, w0, 0), 0);
        assert_eq!(fs.session(), s);
        assert_eq!(fs.window(), w0);
        assert_eq!(cmd_find_valid_state(&fs), 1);

        assert_eq!(cmd_find_from_session_window(&mut fs, s, w1, 0), 0);
        assert_eq!(fs.window(), w1);
        assert_eq!(cmd_find_valid_state(&fs), 1);

        // from_window finds the session that owns the window
        let mut fs2: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(cmd_find_from_window(&mut fs2, w0, 0), 0);
        assert_eq!(fs2.window(), w0);
        assert_eq!(fs2.session(), s);
        assert_eq!(cmd_find_valid_state(&fs2), 1);

        // unknown window (not in any session) fails
        let mut orphan = crate::tests::test_fixtures::Window::new(999, "orphan", 80, 24);
        let mut fs3: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(cmd_find_from_window(&mut fs3, orphan.ptr(), 0), -1);
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
        assert_eq!(cmd_find_from_pane(&mut fs, wp, 0), 0);
        assert_eq!(fs.pane(), wp);
        assert!(!fs.window().is_null());
        assert!(!fs.session().is_null());
        assert_eq!(cmd_find_valid_state(&fs), 1);

        let mut fs2: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        cmd_find_from_winlink_pane(&mut fs2, wl, wp, 0);
        assert_eq!(fs2.pane(), wp);
        assert_eq!(fs2.winlink(), wl);
        assert_eq!(fs2.window(), (*wl).window());
        assert_eq!(fs2.session(), (*wl).session());
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
        assert_eq!(fs.session(), target.session());
        assert!(!fs.winlink().is_null());
        assert!(!fs.window().is_null());
        assert!(!fs.pane().is_null());
        assert_eq!(cmd_find_valid_state(&fs), 1);

        // best_client with no clients attached to session returns null (no attached client)
        let c = cmd_find_best_client(target.session());
        assert!(c.is_null());
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
        let rc = cmd_find_target(
            &mut fs,
            item.ptr(),
            null_mut::<::core::ffi::c_char>(),
            CMD_FIND_PANE,
            0,
        );
        assert_eq!(rc, 0);
        let expected = target.state();
        assert_eq!(fs.session(), expected.session());
        assert_eq!(fs.winlink(), expected.winlink());
        assert_eq!(fs.window(), expected.window());
        assert_eq!(fs.pane(), expected.pane());
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
        let t_session = ::std::ffi::CString::new("$0").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, item.ptr(), t_session.as_ptr(), CMD_FIND_SESSION, 0),
            0
        );
        assert_eq!(fs.session(), target.session());

        // window by id "@0"
        let t_window = ::std::ffi::CString::new("@0").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, item.ptr(), t_window.as_ptr(), CMD_FIND_WINDOW, 0),
            0
        );
        assert_eq!(fs.window(), target.window(0));

        // pane by id "%0"
        let t_pane = ::std::ffi::CString::new("%0").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, item.ptr(), t_pane.as_ptr(), CMD_FIND_PANE, 0),
            0
        );
        assert_eq!(fs.pane(), target.pane(0));

        // window by index "0" and "1" via session (session is current "0")
        let t_idx0 = ::std::ffi::CString::new("0").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, item.ptr(), t_idx0.as_ptr(), CMD_FIND_WINDOW, 0),
            0
        );
        assert_eq!(fs.window(), target.window(0));

        let t_idx1 = ::std::ffi::CString::new("1").unwrap();
        assert_eq!(
            cmd_find_target(&mut fs, item.ptr(), t_idx1.as_ptr(), CMD_FIND_WINDOW, 0),
            0
        );
        assert_eq!(fs.window(), target.window(1));

        // nonsense target with QUIET should fail
        let t_bad = ::std::ffi::CString::new("no-such-session-xyz").unwrap();
        let mut fs2: crate::types::cmd_find_state =
            *Box::new(crate::types::cmd_find_state::default());
        assert_eq!(
            cmd_find_target(
                &mut fs2,
                item.ptr(),
                t_bad.as_ptr(),
                CMD_FIND_SESSION,
                2 // CMD_FIND_QUIET
            ),
            -1
        );
    }
}
