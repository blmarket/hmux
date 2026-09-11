use crate::screen::Screen as _;
use crate::grid::Grid as _;
use super::*;
use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::tests::test_fixtures::{Target, globals, zeroed_client};

#[test]
fn stale_key_targets_leave_replacement_modes_open_and_current_targets_can_close_them() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let fs = target.state();
    let mut pane = fs.pane_ref().unwrap();
    unsafe {
        let pane_options = pane.get().unwrap().options_ref().clone();
        let window_options = fs.window().unwrap().options();
        let session_options = fs.session().unwrap().options();
        pane_options.set_parent(Some(&window_options));
        window_options.set_parent(crate::tmux::global_w_options.get().as_ref());
        session_options.set_parent(crate::tmux::global_s_options.get().as_ref());
        let mut client = zeroed_client();
        for mode in [
            WindowMode::Clock,
            WindowMode::Buffer,
            WindowMode::Client,
            WindowMode::Tree,
            WindowMode::Customize,
        ] {
            assert_eq!(
                (pane.get_mut().unwrap()).set_mode( None, mode, Some(&fs), None),
                0
            );
            let stale = (pane.get().unwrap()).active_mode()
                .unwrap()
                .key_target()
                .unwrap();
            (pane.get_mut().unwrap()).reset_mode();
            assert_eq!(
                (pane.get_mut().unwrap()).set_mode( None, mode, Some(&fs), None),
                0
            );
            let current = (pane.get().unwrap()).active_mode()
                .unwrap()
                .key_target()
                .unwrap();
            stale.dispatch(client.as_client_mut(), b'q' as key_code, None);
            assert_eq!(
                (pane.get().unwrap()).active_mode()
                    .unwrap()
                    .mode(),
                mode
            );
            current.dispatch(client.as_client_mut(), b'q' as key_code, None);
            assert!((pane.get().unwrap()).active_mode().is_none());
        }
        pane_options.set_parent(None);
        window_options.set_parent(None);
        session_options.set_parent(None);
    }
}

#[test]
fn a_key_target_outliving_its_pane_does_nothing() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let fs = target.state();
    let mut pane = fs.pane_ref().unwrap();
    unsafe {
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode(
                None,
                WindowMode::Clock,
                Some(&fs),
                None
            ),
            0
        );
        let key_target = (pane.get().unwrap()).active_mode()
            .unwrap()
            .key_target()
            .unwrap();
        drop(target);
        assert!(pane.get().is_none());
        let mut client = zeroed_client();
        key_target.dispatch(client.as_client_mut(), b'q' as key_code, None);
    }
}

#[test]
fn every_mode_initializes_resizes_and_releases_its_own_state() {
    let _guard = globals();
    for (mode, name) in [
        (WindowMode::Clock, c"clock-mode"),
        (WindowMode::Copy, c"copy-mode"),
        (WindowMode::View, c"view-mode"),
        (WindowMode::Buffer, c"buffer-mode"),
        (WindowMode::Client, c"client-mode"),
        (WindowMode::Tree, c"tree-mode"),
        (WindowMode::Customize, c"options-mode"),
    ] {
        let mut target = Target::new(80, 24);
        let state = target.state();
        unsafe {
            let pane = &mut *target.pane(0);
            let window_options = state.window().unwrap().options();
            pane.options_ref().set_parent(Some(&window_options));
            window_options.set_parent(crate::tmux::global_w_options.get().as_ref());
            state
                .session()
                .unwrap()
                .options()
                .set_parent(crate::tmux::global_s_options.get().as_ref());
            let source = (mode == WindowMode::Copy).then_some(pane.pane_id());
            assert_eq!(
                (pane).set_mode(
                    source.and_then(crate::window::window_pane_find_by_id),
                    mode,
                    Some(&state),
                    None
                ),
                0
            );
            assert_eq!(mode.name(), name);
            assert_eq!(pane.active_mode().unwrap().mode(), mode);
            assert!(pane.active_mode().unwrap().screen.is_some());
            let shown = pane.screen_ref();
            let size = RustScreen::grid(&shown);
            assert_eq!((size.width(), size.height()), (80, 24));
            drop(shown);
            pane.resize(crate::PaneSize { width: 40, height: 12 });
            let shown = pane.screen_ref();
            let size = RustScreen::grid(&shown);
            assert_eq!((size.width(), size.height()), (40, 12));
            drop(shown);
            (pane).reset_modes();
            assert!(pane.active_mode().is_none());
            assert!(core::ptr::eq(&*pane.screen_ref(), pane.base()));
        }
    }
}

#[test]
fn update_targets_refresh_current_lists_and_skip_replaced_or_retired_modes() {
    let _guard = globals();
    for mode in [WindowMode::Buffer, WindowMode::Client, WindowMode::Tree] {
        let mut target = Target::new(40, 12);
        let state = target.state();
        let mut pane = state.pane_ref().unwrap();
        unsafe {
            assert_eq!(
                (pane.get_mut().unwrap()).set_mode( None, mode, Some(&state), None),
                0
            );
            let stale = (pane.get().unwrap()).active_mode()
                .unwrap()
                .update_target()
                .unwrap();
            (pane.get_mut().unwrap()).reset_mode();
            assert_eq!(
                (pane.get_mut().unwrap()).set_mode( None, mode, Some(&state), None),
                0
            );
            *pane.get_mut().unwrap().flags_mut() &= !crate::window::PANE_REDRAW;
            stale.dispatch();
            assert_eq!(*pane.get().unwrap().flags() & crate::window::PANE_REDRAW, 0);
            let current = (pane.get().unwrap()).active_mode()
                .unwrap()
                .update_target()
                .unwrap();
            current.dispatch();
            assert_ne!(*pane.get().unwrap().flags() & crate::window::PANE_REDRAW, 0);
            let retired = (pane.get().unwrap()).active_mode()
                .unwrap()
                .update_target()
                .unwrap();
            drop(target);
            assert!(pane.get().is_none());
            retired.dispatch();
        }
    }
}
