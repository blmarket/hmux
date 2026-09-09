//! Coverage for [`crate::modes`] and [`crate::modes`] —
//! constants and mode metadata reachable without a live server.
//!
//! Both modules start with wire-protocol constants and the callbacks a
//! [`WindowMode`] variant dispatches to, followed by tree builders that touch
//! the server trees. The tests below stay on the deterministic surface: ladder
//! checks for the `WINDOW_TREE_*` / `WINDOW_CUSTOMIZE_*` constants, substring
//! checks for the default format strings, [`WindowMode::Tree`] /
//! [`WindowMode::Customize`] metadata sanity, and a few fixture-driven checks
//! that a [`Target`]/[`Session`]/[`Window`]/[`Pane`] chain can be built
//! under `globals()`. Nothing here hits `fatal`.

use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::window_dimensions::WindowDimensionsState;
use crate::window_name::WindowNameState;
use crate::window_trait::Window as _;

use crate::pane_geometry::PaneGeometryState;

use crate::modes::{
    WINDOW_CUSTOMIZE_DEFAULT_FORMAT, WINDOW_CUSTOMIZE_GLOBAL_SESSION,
    WINDOW_CUSTOMIZE_GLOBAL_WINDOW, WINDOW_CUSTOMIZE_KEY, WINDOW_CUSTOMIZE_NONE,
    WINDOW_CUSTOMIZE_PANE, WINDOW_CUSTOMIZE_RESET, WINDOW_CUSTOMIZE_SERVER,
    WINDOW_CUSTOMIZE_SESSION, WINDOW_CUSTOMIZE_UNSET, WINDOW_CUSTOMIZE_WINDOW,
};
use crate::modes::{
    WINDOW_TREE_DEFAULT_COMMAND, WINDOW_TREE_DEFAULT_FORMAT, WINDOW_TREE_DEFAULT_KEY_FORMAT,
    WINDOW_TREE_NONE, WINDOW_TREE_PANE, WINDOW_TREE_SESSION, WINDOW_TREE_WINDOW,
};

use crate::tests::test_fixtures::{Pane, Session, Target, Window, globals, seen};
use crate::types::WindowMode;
use crate::window::window_active_pane;
use ::core::ffi::CStr;

// ---------------------------------------------------------------------------
// window_tree constants — ladders and default strings
// ---------------------------------------------------------------------------

#[test]
fn window_tree_type_ladder_is_consecutive() {
    assert_eq!(WINDOW_TREE_NONE, 0);
    assert_eq!(WINDOW_TREE_SESSION, 1);
    assert_eq!(WINDOW_TREE_WINDOW, 2);
    assert_eq!(WINDOW_TREE_PANE, 3);
}

#[test]
fn window_tree_default_strings_contain_expected_fragments() {
    unsafe {
        let cmd = CStr::from_ptr(WINDOW_TREE_DEFAULT_COMMAND.as_ptr());
        let fmt = WINDOW_TREE_DEFAULT_FORMAT;
        let kfmt = CStr::from_ptr(WINDOW_TREE_DEFAULT_KEY_FORMAT.as_ptr());
        assert_eq!(cmd.to_str().unwrap(), "switch-client -Zt '%%'");
        let fmt_s = fmt.to_str().unwrap();
        assert!(
            fmt_s.contains("pane_format"),
            "fmt missing pane_format: {fmt_s:?}"
        );
        assert!(
            fmt_s.contains("window_format"),
            "fmt missing window_format: {fmt_s:?}"
        );
        assert!(
            fmt_s.contains("session_windows"),
            "fmt missing session_windows"
        );
        let kfmt_s = kfmt.to_str().unwrap();
        assert!(kfmt_s.contains("line"), "kfmt missing line: {kfmt_s:?}");
        assert!(kfmt_s.contains("M-"), "kfmt missing M-: {kfmt_s:?}");
    }
}

#[test]
fn window_tree_mode_carries_tree_mode_name_and_format() {
    unsafe {
        let name = seen(WindowMode::Tree.name().as_ptr());
        assert_eq!(name, "tree-mode");
        assert!(WindowMode::Tree.default_format().is_some());
        let fmt = seen(WindowMode::Tree.default_format().unwrap().as_ptr());
        assert!(fmt.contains("pane_format") || fmt.contains("window_format"));
        assert!(WindowMode::Tree.has_key());
        assert!(!WindowMode::Tree.has_command());
    }
}

// ---------------------------------------------------------------------------
// window_customize constants and mode
// ---------------------------------------------------------------------------

#[test]
fn window_customize_scope_ladder_and_change_constants() {
    assert_eq!(WINDOW_CUSTOMIZE_NONE, 0);
    assert_eq!(WINDOW_CUSTOMIZE_KEY, 1);
    assert_eq!(WINDOW_CUSTOMIZE_SERVER, 2);
    assert_eq!(WINDOW_CUSTOMIZE_GLOBAL_SESSION, 3);
    assert_eq!(WINDOW_CUSTOMIZE_SESSION, 4);
    assert_eq!(WINDOW_CUSTOMIZE_GLOBAL_WINDOW, 5);
    assert_eq!(WINDOW_CUSTOMIZE_WINDOW, 6);
    assert_eq!(WINDOW_CUSTOMIZE_PANE, 7);

    assert_eq!(WINDOW_CUSTOMIZE_UNSET, 0);
    assert_eq!(WINDOW_CUSTOMIZE_RESET, 1);
    assert_ne!(WINDOW_CUSTOMIZE_UNSET, WINDOW_CUSTOMIZE_RESET);
}

#[test]
fn window_customize_default_format_mentions_scope() {
    let fmt = WINDOW_CUSTOMIZE_DEFAULT_FORMAT;
    let s = fmt.to_str().unwrap();
    assert!(s.contains("is_option"), "fmt was {s:?}");
    assert!(s.contains("option_value"), "fmt was {s:?}");
    assert!(!s.is_empty());
    assert!(s.len() < 200);
}

#[test]
fn window_customize_mode_carries_customize_name_and_callbacks() {
    unsafe {
        let name = seen(WindowMode::Customize.name().as_ptr());
        assert_eq!(name, "options-mode");
        assert!(WindowMode::Customize.default_format().is_some());
        let fmt = seen(WindowMode::Customize.default_format().unwrap().as_ptr());
        assert!(fmt.contains("is_option"));
        assert!(WindowMode::Customize.has_key());
        assert!(!WindowMode::Customize.has_command());
    }
}

// ---------------------------------------------------------------------------
// Fixture-driven checks — Window/Pane/Session + globals()
// ---------------------------------------------------------------------------

#[test]
fn window_pane_and_session_fixtures_hold_expected_invariants() {
    let _guard = globals();
    let mut sess = Session::new(42, "auto09-sess");
    let mut win = Window::new(7, "auto09-win", 80, 24);
    let mut pane = Pane::new(99, 80, 24, 100);
    win.add_pane(&mut pane);
    unsafe {
        assert_eq!(crate::SessionIdentity::session_id(&*sess.ptr()), 42);
        assert_eq!(
            crate::SessionNameState::session_name(&*sess.ptr()),
            Some(c"auto09-sess")
        );
        assert_eq!(
            seen(
                (*win.ptr())
                    .window_name()
                    .expect("a window has a name")
                    .as_ptr()
            ),
            "auto09-win"
        );
        assert_eq!((*win.ptr()).dimensions().size.width, 80);
        assert_eq!((*win.ptr()).dimensions().size.height, 24);
        assert!(window_active_pane(&*win.ptr()).is_some_and(|active_owner| {
            active_owner
                .get()
                .is_some_and(|active| core::ptr::addr_eq(active, pane.ptr()))
        }));
        assert_eq!((*pane.ptr()).pane_id(), 99);
        assert_eq!((*pane.ptr()).geometry().width, 80);
        assert_eq!((*pane.ptr()).geometry().height, 24);
        assert_eq!(*(*pane.ptr()).fd(), -1);
        // options are present
        let _ = (*win.ptr()).options_ref();
        let _ = (*pane.ptr()).options_ref();
        let _ = (&mut *sess.ptr()).options_ref().clone();
    }
}

#[test]
fn target_registers_session_window_and_pane_under_globals() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(1, 80, 24);
    unsafe {
        // session 0 named "0" is registered
        let s = target.session();
        assert_eq!(crate::SessionNameState::session_name(&*s), Some(c"0"));
        assert_eq!(crate::SessionIdentity::session_id(&*s), 0);
        // first window is the one Target built
        let w0 = target.window(0);
        let w1 = target.window(1);
        assert!(!w0.is_null());
        assert!(!w1.is_null());
        assert_ne!(w0, w1);
        assert_eq!((*w0).window_id(), 0);
        assert_eq!((*w1).window_id(), 1);
        // panes are distinct
        let p0 = target.pane(0);
        let p1 = target.pane(1);
        assert_ne!(p0, p1);
        // find state points at session's curw
        let fs = target.state();
        assert_eq!(
            fs.session()
                .as_ref()
                .map_or(core::ptr::null_mut(), |s| s.as_ptr()),
            s
        );
        assert!(core::ptr::eq(
            (&*s).curw().unwrap(),
            fs.winlink_ref().unwrap().get().unwrap()
        ));
        // zeroed helper produces a zeroed struct
        let z: crate::types::window_pane = *Box::new(crate::types::window_pane::default());
        assert_eq!(z.pane_id(), 0);
        assert_eq!(*z.fd(), 0);
    }
}
