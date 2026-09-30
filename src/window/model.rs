//! Window storage is private to its implementation.
use crate::src::shared::abi::{u_int, uint64_t};
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::client::{ClientRef, ClientWeak};
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::event::Timer;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::grid_cell;
use crate::src::shared::key::key_code;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{window_pane, window_pane_history, window_panes, PANE_MINIMUM};
use crate::src::shared::screen::screen;
use crate::src::shared::session::session;
use crate::src::shared::window::{WindowRef, WindowWeak};
#[cfg(test)]
use crate::src::window_pane::WindowPane as _;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::src::shared::window::{window_winlinks, WindowIndex};
/// Explicit cleanup is separate from the lifetime of retained Rc allocations.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) enum WindowLifecycle {
    #[default]
    Live,
    Destroying,
    Destroyed,
}

#[repr(C)]
/// Rc-owned window record; retain/release preserves pre-close notifications.
pub struct window {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(super) observer: WindowWeak,
    pub(super) lifecycle: WindowLifecycle,
    pub(super) id: u_int,
    /// Nonowning identity of the client last active in this window.
    pub(super) latest: ClientWeak,
    pub(super) name: std::ffi::CString,
    pub(super) name_event: Option<Timer>,
    pub(super) name_time: Option<Instant>,
    pub(super) alerts_timer: Option<Timer>,
    pub(super) offset_timer: Option<Timer>,
    pub(super) activity_time: SystemTime,
    pub(super) creation_time: SystemTime,
    /// Current pane identity; the pane index owns the allocation.
    pub(super) active: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Current modal pane is observed; the pane index owns its lifetime.
    pub(super) modal: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Pane to restore after modal dismissal; does not own that pane.
    pub(super) modal_last: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Saved zoom target observes its pane without extending its lifetime.
    pub(super) was_zoomed: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub(super) last_panes: window_pane_history,
    pub(super) z_index: window_panes,
    pub(super) panes: window_panes,
    pub(super) lastlayout: ::core::ffi::c_int,
    pub(super) layout_root: Option<Box<layout_cell>>,
    pub(super) saved_layout_root: Option<Box<layout_cell>>,
    pub(super) old_layout: Option<std::ffi::CString>,
    pub(super) sx: u_int,
    pub(super) sy: u_int,
    pub(super) manual_sx: u_int,
    pub(super) manual_sy: u_int,
    pub(super) xpixel: u_int,
    pub(super) ypixel: u_int,
    pub(super) new_sx: u_int,
    pub(super) new_sy: u_int,
    pub(super) new_xpixel: u_int,
    pub(super) new_ypixel: u_int,
    pub(super) redraw_scene_generation: uint64_t,
    pub(super) menu: Option<refbox::RefBox<crate::src::shared::menu::menu_data>>,
    pub(super) menu_last_px: u_int,
    pub(super) menu_last_py: u_int,
    pub(super) last_new_pane_x: u_int,
    pub(super) last_new_pane_y: u_int,
    pub(super) sb: ::core::ffi::c_int,
    pub(super) sb_pos: ::core::ffi::c_int,
    pub(super) inside_cell: grid_cell,
    pub(super) outside_cell: grid_cell,
    pub(super) flags: ::core::ffi::c_int,
    pub(super) alerts_queued: ::core::ffi::c_int,
    pub(super) options: Option<Box<options>>,
    pub(super) winlinks: window_winlinks,
    /// Weak traversal handle into the containing index.
    pub(super) owner: refbox::Weak<WindowIndex>,
}
impl Default for window {
    fn default() -> Self {
        Self {
            observer: Default::default(),
            lifecycle: Default::default(),
            id: Default::default(),
            latest: Default::default(),
            name: Default::default(),
            name_event: Default::default(),
            name_time: Default::default(),
            alerts_timer: Default::default(),
            offset_timer: Default::default(),
            activity_time: UNIX_EPOCH,
            creation_time: UNIX_EPOCH,
            active: Default::default(),
            modal: Default::default(),
            modal_last: Default::default(),
            was_zoomed: Default::default(),
            last_panes: Default::default(),
            z_index: Default::default(),
            panes: Default::default(),
            lastlayout: Default::default(),
            layout_root: Default::default(),
            saved_layout_root: Default::default(),
            old_layout: Default::default(),
            sx: Default::default(),
            sy: Default::default(),
            manual_sx: Default::default(),
            manual_sy: Default::default(),
            xpixel: Default::default(),
            ypixel: Default::default(),
            new_sx: Default::default(),
            new_sy: Default::default(),
            new_xpixel: Default::default(),
            new_ypixel: Default::default(),
            redraw_scene_generation: Default::default(),
            menu: Default::default(),
            menu_last_px: Default::default(),
            menu_last_py: Default::default(),
            last_new_pane_x: Default::default(),
            last_new_pane_y: Default::default(),
            sb: Default::default(),
            sb_pos: Default::default(),
            inside_cell: Default::default(),
            outside_cell: Default::default(),
            flags: Default::default(),
            alerts_queued: Default::default(),
            options: Default::default(),
            winlinks: Default::default(),
            owner: Default::default(),
        }
    }
}

impl window {
    pub(super) fn invalidate_scene(&mut self) {
        self.redraw_scene_generation = self.redraw_scene_generation.wrapping_add(1);
    }
    /// Retain the active pane for the current operation.
    pub(super) fn active_pane(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.active.upgrade()
    }

    /// The caller supplies a live Rc-backed pane.
    pub(super) fn set_active(
        &mut self,
        pane: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    ) {
        self.active = pane.map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    }

    pub(super) fn new() -> WindowRef {
        std::rc::Rc::new_cyclic(|observer| {
            let mut value = Self::default();
            value.observer = observer.clone();
            std::cell::UnsafeCell::new(value)
        })
    }

    /// Unregistered identity for tests of event payload ownership and release order.
    #[cfg(test)]
    pub(super) fn with_id_for_test(id: u32) -> WindowRef {
        let owner = Self::new();
        unsafe { (*owner.get()).id = id };
        owner
    }

    /// Geometry-only fixture, without a layout, panes, options or resize callbacks.
    #[cfg(test)]
    pub(super) fn with_size_for_test(sx: u32, sy: u32) -> WindowRef {
        let owner = Self::new();
        unsafe {
            (*owner.get()).sx = sx;
            (*owner.get()).sy = sy;
        }
        owner
    }

    /// An unregistered owner for testing option inheritance without global state.
    #[cfg(test)]
    pub(super) fn with_options_for_test() -> WindowRef {
        let owner = Self::new();
        unsafe {
            (*owner.get()).options = Some(crate::src::options::options_create(None));
        }
        owner
    }

    pub(super) fn layout_root_ptr(&mut self) -> Option<&mut layout_cell> {
        self.layout_root.as_deref_mut()
    }

    pub(super) fn saved_layout_root_ptr(&mut self) -> Option<&mut layout_cell> {
        self.saved_layout_root.as_deref_mut()
    }
}

#[cfg(test)]
mod saved_zoom_tests {
    use super::*;
    use crate::src::window::{window_pop_zoom, Window, WINDOW_WASZOOMED};
    use std::rc::Rc;

    #[test]
    fn saved_zoom_weak_target_expires_and_pop_clears_the_pending_state() {
        unsafe {
            let window = window::new();
            let pane = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let observer = Rc::downgrade(&pane);
            (*window.get()).was_zoomed = observer.clone();
            (*window.get()).flags |= WINDOW_WASZOOMED;
            assert_eq!(Rc::strong_count(&pane), 1);
            drop(pane);
            assert!(observer.upgrade().is_none());
            assert!((*window.get()).was_zoomed.upgrade().is_none());
            assert_eq!(window_pop_zoom(&window), 0);
            assert_eq!((*window.get()).flags & WINDOW_WASZOOMED, 0);
            assert!((*window.get()).was_zoomed.ptr_eq(&std::rc::Weak::new()));
            assert_eq!(window_pop_zoom(&window), 0);
            window.release(c"expired saved zoom test");
        }
    }
}
