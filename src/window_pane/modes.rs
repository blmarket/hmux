use super::*;
use crate::args::RustArguments;
use crate::notify::notify_pane;
use crate::window::{PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_CHANGED, PANE_UNSEENCHANGES};

/// Starts a pane mode or brings an existing mode to the front.
pub(super) unsafe fn set_mode(
    wp: &mut window_pane,
    source_pane: Option<RustWindowPaneWeak>,
    mode: WindowMode,
    fs: Option<&cmd_find_state>,
    args: Option<&RustArguments>,
) -> core::ffi::c_int {
    unsafe {
        if wp
            .modes
            .first()
            .is_some_and(|current| current.mode() == mode)
        {
            return 1 as core::ffi::c_int;
        }
        let window = wp
            .window_context()
            .expect("a pane mode has a window context");
        let already = wp.modes.iter().position(|open| open.mode() == mode);
        if let Some(at) = already {
            let open = wp.modes.remove(at);
            wp.modes.insert(0, open);
        } else {
            let pane = (wp).observation().expect("a mode opens on an owned pane");
            let entry = Box::new(window_mode_entry::pending(pane.clone(), source_pane));
            wp.modes.insert(0, entry);
            let entry = wp.modes.first_mut().expect("mode was just inserted");
            entry.initialize(mode, pane, fs, args);
        }
        wp.screen = PaneScreen::Mode;
        *wp.flags_mut() |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
        window.fix_layout_panes(None);
        window.redraw_borders();
        window.redraw_status();
        notify_pane(c"pane-mode-changed", Some(wp));
        0 as core::ffi::c_int
    }
}
pub(super) unsafe fn reset_mode(wp: &mut window_pane) {
    unsafe {
        if wp.modes.is_empty() {
            return;
        }
        let window = wp.window_context();
        let mut open = wp.modes.remove(0);
        open.release();
        drop(open);
        let geometry = wp.geometry();
        wp.screen = if wp.modes.is_empty() {
            PaneScreen::Base
        } else {
            PaneScreen::Mode
        };
        if let Some(next) = wp.modes.first_mut() {
            log_debug(
                c"%s: next mode is %s",
                fmt_args![c"window_pane_reset_mode".as_ptr(), next.mode().name()],
            );
            next.resize(geometry.sx, geometry.sy);
        } else {
            *wp.flags_mut() &= !PANE_UNSEENCHANGES;
            log_debug(
                c"%s: no next mode",
                fmt_args![c"window_pane_reset_mode".as_ptr()],
            );
        }
        *wp.flags_mut() |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
        if let Some(window) = window {
            window.fix_layout_panes(None);
            window.redraw_borders();
            window.redraw_status();
        }
        notify_pane(c"pane-mode-changed", Some(wp));
    }
}
pub(super) unsafe fn reset_modes(wp: &mut window_pane) {
    unsafe {
        while !wp.modes.is_empty() {
            wp.reset_mode();
        }
    }
}
pub(super) fn update_default_cursor(wp: &mut window_pane) {
    {
        let options = wp.options_ref().clone();
        if wp.showing_base() {
            wp.base_mut().set_default_cursor(&options);
            return;
        }
        let mode = wp
            .modes
            .first_mut()
            .expect("the shown mode is present");
        mode.update_default_cursor(&options);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
#[test]
fn shown_screen_borrows_handle_pending_modes_and_base_fallback() {
    let mut pane = window_pane::default();
    assert!(core::ptr::eq(&*pane.try_screen_ref().unwrap(), pane.base()));
    pane.screen = PaneScreen::Mode;
    assert!(core::ptr::eq(&*pane.try_screen_ref().unwrap(), pane.base()));
    pane.modes.push(Box::new(window_mode_entry {
        wp: None,
        swp: None,
        state: WindowModeState::None,
        screen: None,
        prefix: 1,
    }));
    assert!(pane.try_screen_ref().is_none());
    pane.modes[0].state =
        WindowModeState::Clock(Box::new(crate::modes::window_clock_mode_data {
            screen: RustScreen::default(),
            tim: 0,
            timer: TimerHandle::ZERO,
        }));
    assert!(pane.try_screen_ref().is_none());
    pane.modes[0].screen = Some(ModeScreen::Clock);
    let WindowModeState::Clock(data) = &pane.modes[0].state else {
        unreachable!()
    };
    assert!(core::ptr::eq(&*pane.screen_ref(), &data.screen));
    pane.screen = PaneScreen::Base;
    assert!(core::ptr::eq(&*pane.screen_ref(), pane.base()));
    pane.screen = PaneScreen::Mode;
    pane.modes.clear();
    assert!(core::ptr::eq(&*pane.screen_ref(), pane.base()));
}

}
