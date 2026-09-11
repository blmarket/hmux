//! The pane layout: the tree of cells a window's panes sit in, the layouts
//! that arrange them, and the string a layout is written as.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod cells;
mod custom;
mod set;

use crate::consts::WINDOW_ZOOMED;
use crate::window_trait::Window;
const WINDOW_WASZOOMED: core::ffi::c_int = 0x10;
use crate::fmt_args;
use crate::log::log_debug;
use crate::notify::notify_window;
use crate::pane_resize::PaneSize;
use crate::window::window_count_panes;
use crate::window_scrollbar::WindowScrollbarState;
pub use cells::LayoutCellPath;
use cells::*;
pub use set::layout_set_lookup;
use std::ffi::{CStr, CString};

/// Authorizes only the layout owner to mutate the window's tree storage.
pub(crate) struct LayoutAccess(());

#[derive(Default)]
pub(crate) struct LayoutTree {
    root: Option<Box<layout_cell>>,
    saved: Option<Box<layout_cell>>,
}

#[derive(Default)]
#[repr(C)]
struct layout_cell {
    type_0: layout_type,
    flags: core::ffi::c_int,
    parent: bool,
    sx: u_int,
    sy: u_int,
    xoff: core::ffi::c_int,
    yoff: core::ffi::c_int,
    /// The pane allocation held by this leaf, or nothing for a branch cell.
    wp: Option<RustWindowPaneWeak>,
    cells: layout_cells,
}
type layout_cells = Vec<Box<layout_cell>>;

mod drag;

impl WindowRef {
    pub(crate) fn dump_layout(&self) -> Option<CString> {
        let window = self.as_window();
        self.dump_layout_cell(window.layout().root.as_deref())
    }

    pub(crate) fn pane_layout_path(
        &self,
        pane: &RustWindowPaneWeak,
    ) -> Option<crate::layout::LayoutCellPath> {
        self.as_window()
            .layout()
            .root
            .as_deref()
            .and_then(|root| crate::layout::LayoutCellPath::for_pane(root, pane))
    }

    fn bind_layout_pane(
        &self,
        path: Option<&crate::layout::LayoutCellPath>,
        pane: &RustWindowPaneWeak,
    ) {
        crate::layout::layout_bind_pane(self, path, pane);
    }

    fn layout_cell_geometry(
        &self,
        path: &crate::layout::LayoutCellPath,
    ) -> Option<crate::pane_geometry::PaneGeometry> {
        let window = self.as_window();
        let cell = path.get(window.layout().root.as_deref()?)?;
        Some(crate::pane_geometry::PaneGeometry {
            xoff: cell.xoff,
            yoff: cell.yoff,
            sx: cell.sx,
            sy: cell.sy,
        })
    }

    fn set_layout_cell_geometry(
        &self,
        path: &crate::layout::LayoutCellPath,
        geometry: crate::pane_geometry::PaneGeometry,
    ) {
        let mut window = self.as_window_mut();
        let cell = window
            .layout_mut(LayoutAccess(()))
            .root
            .as_deref_mut()
            .and_then(|root| path.get_mut(root))
            .expect("the resized cell path is unchanged");
        crate::layout::layout_set_size(
                cell,
                geometry.sx,
                geometry.sy,
                geometry.xoff,
                geometry.yoff,
            );
    }

    pub(crate) fn layout_border_at(
        &self,
        x: u_int,
        y: u_int,
    ) -> Option<crate::layout::LayoutCellPath> {
        self.as_window()
            .layout()
            .root
            .as_deref()
            .and_then(|root| crate::layout::layout_search_by_border(root, x, y))
    }

    fn layout_parent_type(&self, path: &crate::layout::LayoutCellPath) -> Option<layout_type> {
        let window = self.as_window();
        Some(path.parent()?.get(window.layout().root.as_deref()?)?.type_0)
    }
}

impl LayoutTree {
    pub(crate) fn is_floating(&self, pane: &RustWindowPaneWeak) -> bool {
        layout_cell_for_pane(self.root.as_deref(), pane)
            .is_some_and(|(cell, _)| cell.flags & crate::consts::LAYOUT_CELL_FLOATING != 0)
    }

    pub(crate) fn contains(&self, pane: &RustWindowPaneWeak) -> bool {
        layout_cell_for_pane(self.root.as_deref(), pane).is_some()
    }

    pub(crate) fn size(&self) -> Option<crate::pane_resize::PaneSize> {
        self.root
            .as_deref()
            .map(|root| crate::pane_resize::PaneSize {
                width: root.sx,
                height: root.sy,
            })
    }
}

impl WindowRef {
    pub(crate) fn two_pane_layout(&self) -> Option<layout_type> {
        let w = self.as_window();
        let mut count = 0;
        let mut split_type = None;
        for pane in &w.panes {
            let Some((cell, parent)) =
                layout_cell_for_pane(w.layout().root.as_deref(), &pane.downgrade())
            else {
                continue;
            };
            if cell.flags & LAYOUT_CELL_FLOATING != 0 {
                continue;
            }
            count += 1;
            if count > 2 {
                return None;
            }
            split_type = Some(parent?.type_0);
        }
        if count == 2 { split_type } else { None }
    }
}

impl WindowRef {
    /// Marks the newly spawned member's existing layout association as floating.
    /// Construction must have assigned this allocation a cell in this window.
    pub(crate) fn mark_layout_pane_floating(&self, pane: &RustWindowPaneWeak) {
        let mut w = self.as_window_mut();
        assert!(
            w.panes.iter().any(|member| member.downgrade().ptr_eq(pane)),
            "the spawned pane belongs to this window"
        );
        let root = w
            .layout_mut(LayoutAccess(()))
            .root
            .as_deref_mut()
            .expect("the spawned pane has a layout");
        let path = LayoutCellPath::for_pane(root, pane).expect("the spawned pane has a cell");
        path.get_mut(root).unwrap().flags |= crate::consts::LAYOUT_CELL_FLOATING;
    }
}

impl window {
    fn restore_layout_tree(&mut self) {
        let layout = self.layout_mut(LayoutAccess(()));
        layout.root = layout.saved.take();
    }

    fn save_layout_tree(&mut self) {
        let layout = self.layout_mut(LayoutAccess(()));
        layout.saved = layout.root.take();
    }

    pub(crate) fn clear_layout_tree(&mut self) {
        let layout = self.layout_mut(LayoutAccess(()));
        layout.root = None;
        layout.saved = None;
    }
}

#[cfg(test)]
#[path = "../tests/test_coverage_auto_23.rs"]
mod coverage_tests;
#[cfg(test)]
pub(crate) mod test_support;

impl WindowRef {
    /// Exchanges two distinct panes' ownership, layout membership and geometry.
    ///
    /// Returns their pre-exchange active states for caller selection policy.
    /// Floating panes are rejected before client cleanup or mutation. Both layout
    /// paths must exist. Pane and z-order positions, option parents and appearance
    /// flags follow the destination; registration and allocation identity survive.
    /// Client cleanup and source-then-destination resizing retain their order.
    /// The caller applies selection, then uses `finish_pane_exchange` for a
    /// cross-window exchange before layout repair, redraw and notification.
    ///
    /// # Safety
    /// Panes must be distinct live members of the supplied unzoomed windows,
    /// resolved immediately before this call. Run on the server thread without
    /// conflicting pane, window, option, client or TTY payload access. Resizing
    /// invokes existing pane-mode callbacks; those must not move or remove either
    /// target. No window borrow crosses those callbacks. Do not dispatch queue
    /// hooks or unrelated callbacks before selection and exchange completion.
    pub(crate) unsafe fn exchange_pane_geometry(
        &self,
        source_pane: &RustWindowPaneWeak,
        destination: &Self,
        destination_pane: &RustWindowPaneWeak,
    ) -> Result<(bool, bool), &'static CStr> {
        let src_path = self
            .pane_layout_path(source_pane)
            .expect("the source has a cell");
        let dst_path = destination
            .pane_layout_path(destination_pane)
            .expect("the destination has a cell");
        if self.pane_is_floating(source_pane) || destination.pane_is_floating(destination_pane) {
            return Err(c"cannot swap floating panes");
        }
        let src_was_active = self.active_pane().as_ref() == Some(source_pane);
        let dst_was_active = destination.active_pane().as_ref() == Some(destination_pane);
        unsafe { crate::server::server_client_remove_pane(source_pane.as_pane()) };
        unsafe { crate::server::server_client_remove_pane(destination_pane.as_pane()) };
        let src_geometry = unsafe { source_pane.get().unwrap().geometry() };
        let dst_geometry = unsafe { destination_pane.get().unwrap().geometry() };
        unsafe {
            self.clone()
                .swap_panes(source_pane, &mut destination.clone(), destination_pane)
        };
        self.swap_pane_z_order(source_pane, destination, destination_pane);
        self.bind_layout_pane(Some(&src_path), destination_pane);
        destination.bind_layout_pane(Some(&dst_path), source_pane);

        let mut source_observation = source_pane.clone();
        let mut destination_observation = destination_pane.clone();
        {
            let pane = unsafe { source_observation.get_mut().unwrap() };
            pane.inherit_window_context(destination);
        }
        {
            let pane = unsafe { destination_observation.get_mut().unwrap() };
            pane.inherit_window_context(self);
        }
        {
            let pane = unsafe { source_observation.get_mut().unwrap() };
            pane.set_position(dst_geometry.xoff, dst_geometry.yoff);
            unsafe {
                pane.resize(PaneSize {
                    width: dst_geometry.sx,
                    height: dst_geometry.sy,
                })
            };
        }
        {
            let pane = unsafe { destination_observation.get_mut().unwrap() };
            pane.set_position(src_geometry.xoff, src_geometry.yoff);
            unsafe {
                pane.resize(PaneSize {
                    width: src_geometry.sx,
                    height: src_geometry.sy,
                })
            };
        }
        Ok((src_was_active, dst_was_active))
    }
    /// Rotates pane membership through the existing layout slots and geometries.
    /// Returns the rotated identities for immediate caller selection policy.
    /// Empty windows return an empty list without mutation. Z-order is unchanged.
    ///
    /// # Safety
    /// The window must be unzoomed, with stable live pane membership. Exclude
    /// conflicting pane/window/layout/TTY access on the server thread. Existing
    /// resize mode callbacks must not move or remove any pane during rotation.
    /// No window borrow crosses those callbacks; hooks and selection are deferred
    /// to the caller. The geometry snapshot is required across list mutation.
    pub(crate) unsafe fn rotate_pane_geometry(&self, down: bool) -> Vec<RustWindowPaneWeak> {
        unsafe {
            let slots: Vec<_> = self
                .panes()
                .iter()
                .map(|pane| (self.pane_layout_path(pane), pane.as_pane().geometry()))
                .collect();
            let count = slots.len();
            if count == 0 {
                return Vec::new();
            }
            self.rotate_pane_order(down);
            let panes = self.panes();
            for offset in 0..count {
                let index = if down { offset } else { count - offset - 1 };
                let (path, geometry) = &slots[index];
                let mut pane = panes[index].clone();
                self.bind_layout_pane(path.as_ref(), &pane);
                let payload = pane.as_pane_mut();
                payload.set_position(geometry.xoff, geometry.yoff);
                payload.resize(PaneSize {
                    width: geometry.sx,
                    height: geometry.sy,
                });
            }
            panes
        }
    }
}

unsafe fn window_restore_layout(w: &mut window) -> bool {
    unsafe {
        if w.flags & WINDOW_ZOOMED == 0 {
            return false;
        }
        w.flags &= !WINDOW_ZOOMED;
        w.restore_layout_tree();
        for pane in &mut w.panes {
            let pane = pane.as_pane_mut();
            pane.set_window_zoomed(false);
        }
        true
    }
}
impl WindowRef {
    pub unsafe fn zoom(&self, pane: &RustWindowPaneWeak) -> core::ffi::c_int {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            if w.flags & WINDOW_ZOOMED != 0
                || !w
                    .panes
                    .iter()
                    .any(|candidate| candidate.downgrade().ptr_eq(pane))
            {
                return -1;
            }
            if window_count_panes(w, 1) == 1 {
                return -1;
            }
            let activate = w.active.as_ref() != Some(pane);
            drop(payload);
            if activate {
                owner.set_active_pane(pane, 1);
            }
            if let Some(payload) = pane.clone().get_mut() {
                payload.set_window_zoomed(true);
            }
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            w.save_layout_tree();
            drop(payload);
            owner.init_layout(
                &crate::window::window_pane_find_by_id(pane.id()).expect("the layout pane exists"),
            );
            owner.as_window_mut().flags |= WINDOW_ZOOMED;
            notify_window(c"window-layout-changed", Some(owner));
            0
        }
    }
    pub unsafe fn unzoom(&self, notify: core::ffi::c_int) -> core::ffi::c_int {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            if !window_restore_layout(&mut payload) {
                return -1;
            }
            drop(payload);
            owner.fix_layout_panes(None);
            if notify != 0 {
                notify_window(c"window-layout-changed", Some(owner));
            }
            0
        }
    }
    pub unsafe fn push_zoom(
        &self,
        always: core::ffi::c_int,
        flag: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            log_debug(
                c"%s: @%u %d",
                fmt_args![
                    c"window_push_zoom".as_ptr(),
                    w.window_id(),
                    (flag != 0 && w.flags & WINDOW_ZOOMED != 0) as core::ffi::c_int
                ],
            );
            if flag != 0 && (always != 0 || w.flags & WINDOW_ZOOMED != 0) {
                w.flags |= WINDOW_WASZOOMED;
            } else {
                w.flags &= !WINDOW_WASZOOMED;
            }
            drop(payload);
            (owner.unzoom(1 as core::ffi::c_int) == 0 as core::ffi::c_int) as core::ffi::c_int
        }
    }
    pub unsafe fn pop_zoom(&self) -> core::ffi::c_int {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            log_debug(
                c"%s: @%u %d",
                fmt_args![
                    c"window_pop_zoom".as_ptr(),
                    w.window_id(),
                    (w.flags & WINDOW_WASZOOMED != 0) as core::ffi::c_int
                ],
            );
            if w.flags & WINDOW_WASZOOMED != 0 {
                let Some(active_id) = w.active_pane_id() else {
                    return 0 as core::ffi::c_int;
                };
                drop(payload);
                return (owner.zoom(
                    &crate::window::window_pane_find_by_id(active_id)
                        .expect("the selected pane exists"),
                ) == 0 as core::ffi::c_int) as core::ffi::c_int;
            }
            0 as core::ffi::c_int
        }
    }
}

impl window {
    pub(crate) unsafe fn restore_layout_on_drop(&mut self) {
        unsafe {
            if window_restore_layout(self) {
                layout_fix_panes_on_drop(
                    self.layout().root.as_deref(),
                    self.scrollbar_settings(),
                    &self.panes,
                    self.options_ref().number(c"pane-border-status") as core::ffi::c_int,
                );
            }
        }
    }
}
