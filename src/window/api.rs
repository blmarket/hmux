//! Operations on an opaque, retained window.
//!
//! The holder preserves allocation lifetime, not exclusive access. Callers must
//! preserve the server's single-threaded lifecycle and callback ordering. Scoped
//! option callbacks must not reenter the window, free/reparent the component, or
//! let references or pointers escape.

use super::*;
use crate::src::layout::custom::{layout_dump, layout_parse};
use crate::src::layout::layout_resize;
use crate::src::layout::set::{
    layout_set_lookup, layout_set_next, layout_set_previous, layout_set_select,
};
use crate::src::layout::{layout_get_floating_cell, layout_get_tiled_cell, layout_spread_out};
use crate::src::resize::recalculate_sizes;
use crate::src::server_client::Client as _;
use crate::src::shared::client::ClientRef;
use crate::src::shared::menu::menu_data;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::{WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_RESIZE};
use crate::src::spawn::spawn_pane;
use crate::src::window_pane::WindowPane as _;
use std::time::{Duration, Instant, SystemTime};

#[derive(Clone, Copy)]
pub enum LayoutView {
    Visible,
    /// Use the saved, unzoomed tree when one exists, otherwise the current tree.
    Unzoomed,
}

#[derive(Clone, Copy)]
pub enum PaneOrder {
    Index,
    Stacking,
}

fn find_layout_pane<'a>(
    root: &'a layout_cell,
    pane: &Weak<UnsafeCell<window_pane>>,
) -> Option<&'a layout_cell> {
    if root.wp.ptr_eq(pane) {
        return Some(root);
    }
    root.cells
        .iter()
        .find_map(|child| find_layout_pane(child, pane))
}

/// Operations on independently movable registry heads. Entries remain weak;
/// successful lookup and traversal return one explicit Window release duty.
pub trait WindowIndex {
    unsafe fn insert(&mut self, window: &WindowRef) -> Option<WindowRef>;
    unsafe fn remove(&mut self, window: &WindowRef) -> bool;
    unsafe fn first(&self) -> Option<WindowRef>;
    unsafe fn resolve(&self, id: u32) -> Option<WindowRef>;
    fn is_empty(&self) -> bool;
}

impl WindowIndex for windows {
    unsafe fn insert(&mut self, window: &WindowRef) -> Option<WindowRef> {
        windows_insert(self, window)
    }
    unsafe fn remove(&mut self, window: &WindowRef) -> bool {
        windows_remove(self, window)
    }
    unsafe fn first(&self) -> Option<WindowRef> {
        windows_minmax(self)
    }
    unsafe fn resolve(&self, id: u32) -> Option<WindowRef> {
        let map = self
            .storage
            .as_ref()?
            .try_borrow_mut()
            .expect("window index already borrowed");
        map.get(&id)?.upgrade()
    }
    fn is_empty(&self) -> bool {
        self.storage.as_ref().is_none_or(|owner| {
            owner
                .try_borrow_mut()
                .expect("window index already borrowed")
                .is_empty()
        })
    }
}

pub trait Window {
    /// Create and register an initialized Window; linking transfers the caller's
    /// explicit release duty to its winlink.
    unsafe fn create(sx: u32, sy: u32, xpixel: u32, ypixel: u32) -> Self
    where
        Self: Sized;
    unsafe fn find_by_id(id: u32) -> Option<Self>
    where
        Self: Sized;
    unsafe fn find_by_id_str(id: &CStr) -> Option<Self>
    where
        Self: Sized;
    unsafe fn retain(&self, from: &CStr) -> Self
    where
        Self: Sized;
    unsafe fn update_focus(&self);
    unsafe fn update_focus_for(window: Option<&Self>)
    where
        Self: Sized;
    unsafe fn zoom_pane(pane: &Rc<UnsafeCell<window_pane>>) -> i32
    where
        Self: Sized;
    unsafe fn redraw_active_switch(&self, pane: Option<&Rc<UnsafeCell<window_pane>>>);
    unsafe fn pane_at(&self, x: u32, y: u32) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn find_pane(&self, name: &CStr) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn active_pane_over_zoom(&self) -> i32;
    /// Save the previous zoom identity before temporarily displaying all panes.
    unsafe fn push_zoom(&self, always: bool, flag: bool) -> i32;
    unsafe fn pop_zoom(&self) -> i32;
    unsafe fn add_pane(
        &self,
        other: Option<&Rc<UnsafeCell<window_pane>>>,
        hlimit: u32,
        flags: i32,
    ) -> Rc<UnsafeCell<window_pane>>;
    /// Retire selection and history before the caller removes or moves membership.
    unsafe fn forget_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    unsafe fn destroy_panes(&self);
    unsafe fn winlink_flags(link: refbox::Weak<winlink>, escape: bool) -> CString
    where
        Self: Sized;

    /// Immediate edits to weak pane membership only. End this guard before
    /// querying panes, resizing, calling Window operations or delivering events.
    /// A future whole-Window RefCell maps one borrow to this component.
    type PaneOrderMut<'a>: std::ops::DerefMut<Target = window_panes>
    where
        Self: 'a;
    unsafe fn borrow_pane_order_mut(&self, order: PaneOrder) -> Self::PaneOrderMut<'_>;
    /// Selection history is separate from index and stacking order. Only pure
    /// history edits may run while this component is borrowed.
    type PaneHistoryMut<'a>: std::ops::DerefMut<Target = window_pane_history>
    where
        Self: 'a;
    unsafe fn borrow_pane_history_mut(&self) -> Self::PaneHistoryMut<'_>;
    /// Same-window swaps borrow once; cross-window swaps borrow distinct owners.
    /// Membership remains weak and neither selection nor notifications change.
    unsafe fn swap_pane_order(
        &self,
        order: PaneOrder,
        first: &Weak<UnsafeCell<window_pane>>,
        other: &WindowRef,
        second: &Weak<UnsafeCell<window_pane>>,
    );
    /// A bounded view of the owned layout tree. End the guard before calling
    /// Window operations, pane resizing, format expansion or event callbacks.
    /// Cell pointers used by layout algorithms must not escape this guard.
    type LayoutRootMut<'a>: std::ops::DerefMut<Target = Option<Box<layout_cell>>>
    where
        Self: 'a;
    unsafe fn borrow_layout_root_mut(&self) -> Self::LayoutRootMut<'_>;
    /// Read a visible or saved tree only within a pure layout calculation.
    /// Release the guard before any Window/Pane query, rendering or callback;
    /// pointers used by legacy tree walkers must not escape the borrow.
    type LayoutRoot<'a>: std::ops::Deref<Target = layout_cell>
    where
        Self: 'a;
    unsafe fn borrow_layout_root(&self, view: LayoutView) -> Option<Self::LayoutRoot<'_>>;
    /// Resolve an already captured identity across visible/saved tree transfers.
    /// End the guard before model queries, resizing, formatting or callbacks;
    /// references and cell/parent pointers must not escape it.
    type LayoutCell<'a>: std::ops::Deref<Target = layout_cell>
    where
        Self: 'a;
    type LayoutCellMut<'a>: std::ops::DerefMut<Target = layout_cell>
    where
        Self: 'a;
    unsafe fn borrow_layout_cell(&self, id: *mut layout_cell) -> Option<Self::LayoutCell<'_>>;
    unsafe fn borrow_layout_cell_mut(
        &self,
        id: *mut layout_cell,
    ) -> Option<Self::LayoutCellMut<'_>>;
    unsafe fn last_layout_preset(&self) -> i32;
    unsafe fn layout_string(&self, view: LayoutView, legacy: bool) -> Option<CString>;
    unsafe fn pane_layout_cell(
        &self,
        pane: &Weak<UnsafeCell<window_pane>>,
        view: LayoutView,
    ) -> Option<&layout_cell>;
    /// Preset selection records its result after arrangement notifications.
    unsafe fn remember_layout_preset(&self, preset: i32);

    unsafe fn id(&self) -> u32;
    /// Registry successor. The returned owner retains the explicit release duty.
    unsafe fn next_window(&self) -> Option<WindowRef>;
    /// Live association traversal; callbacks may change the list between calls.
    unsafe fn next_winlink(&self, after: Option<refbox::Weak<winlink>>) -> refbox::Weak<winlink>;
    /// Immediate association edits; the Session index retains the link itself.
    unsafe fn add_winlink(&self, link: refbox::Weak<winlink>);
    unsafe fn remove_winlink(&self, link: refbox::Weak<winlink>);
    /// Preserve unlink's ownership-based check. Borrow the published winlink
    /// owner directly: cloning it before this query changes the decision.
    unsafe fn is_linked_outside_group(&self, members: usize) -> bool;
    unsafe fn has_layout(&self) -> bool;
    unsafe fn contains_pane(&self, pane: &Weak<UnsafeCell<window_pane>>) -> bool;
    /// Only alert bits, never the window's other bookkeeping flags.
    unsafe fn pending_alerts(&self) -> i32;
    unsafe fn reset_alert_timer(&self);
    /// Record flags and claim queue membership. Some(true) transfers one release duty
    /// to the alert queue. None means disabled; Some(false) means already queued.
    unsafe fn queue_alerts(&self, flags: i32) -> Option<bool>;
    /// Called after delivery, while the queue still retains this window.
    unsafe fn finish_alerts(&self);
    /// Clear pending flag bits without changing alert-queue membership.
    unsafe fn clear_alert_flags(&self);

    /// Publish a newly created window's first pane and client before layout,
    /// naming and creation notifications. No selection callbacks run here.
    unsafe fn initialize_pane(
        &self,
        pane: &Rc<UnsafeCell<window_pane>>,
        client: Option<&ClientRef>,
    );
    /// Respawn must select its surviving pane even when it was already active.
    /// Clear the old identity before running the existing selection procedure.
    unsafe fn select_respawned_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Name a new window without a rename event; an explicit name disables
    /// automatic naming. Creation notification belongs to the caller.
    unsafe fn initialize_name(&self, name: CString, explicit: bool);
    unsafe fn name(&self) -> CString;
    unsafe fn rename(&self, name: &CStr, untrusted: bool);
    /// Commit a due automatic-name check, or queue its remaining delay. No
    /// formatting or pane callbacks execute while the timer state is borrowed.
    unsafe fn begin_name_check(&self, now: Instant) -> bool;
    /// Coalesce cursor-driven offset updates at the existing 10 ms deadline.
    /// The timer observes this window weakly; dispatch borrows only on demand.
    unsafe fn schedule_offset_update(&self);
    unsafe fn active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>>;
    /// Copy identity without upgrading it or changing pane ownership.
    unsafe fn active_pane_observer(&self) -> Weak<UnsafeCell<window_pane>>;
    /// Most recently active pane from selection history, independent of z-order.
    unsafe fn last_active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn modal_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>>;
    /// Remember the current active identity and publish a newly spawned modal.
    /// Selection, redraw and notifications remain with the caller, after this
    /// bounded state edit. Pane membership and lifetime are caller guarantees.
    unsafe fn begin_modal_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Retain the current pane order for sorting without retaining a Window borrow.
    unsafe fn pane_snapshot(&self) -> Vec<Rc<UnsafeCell<window_pane>>>;
    /// Copy membership before querying Pane policy, which may consult this Window.
    unsafe fn pane_count(&self, with_floating: bool) -> u32;
    /// Retain stacking order before rendering, which may reenter and reorder it.
    unsafe fn stacking_snapshot(&self) -> Vec<Rc<UnsafeCell<window_pane>>>;
    unsafe fn step_pane(
        &self,
        order: PaneOrder,
        after: Option<&Weak<UnsafeCell<window_pane>>>,
        reverse: bool,
    ) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn scrollbar_mode(&self) -> i32;
    unsafe fn scrollbar_position(&self) -> i32;
    /// Refresh cached scrollbar policy after options change. Pane resizing is
    /// performed by the caller after this borrow ends.
    unsafe fn refresh_scrollbars(&self);
    unsafe fn pane_border_status(&self) -> i32;
    unsafe fn pane_border_lines(&self) -> pane_lines;
    unsafe fn pane_at_index(&self, index: u32) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn pane_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32>;
    unsafe fn pane_history_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32>;
    unsafe fn pane_stacking_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32>;
    /// Advance in pane order with wrapping, retaining the original zero-step result.
    unsafe fn pane_by_number(
        &self,
        pane: Option<&Rc<UnsafeCell<window_pane>>>,
        count: u32,
        reverse: bool,
    ) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn creation_time(&self) -> SystemTime;
    unsafe fn activity_time(&self) -> SystemTime;
    /// `None` starts traversal; `Some` continues after that pane, without wrapping.
    unsafe fn next_pane(
        &self,
        after: Option<&Rc<UnsafeCell<window_pane>>>,
    ) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn select_pane(&self, pane: &Rc<UnsafeCell<window_pane>>, notify: bool) -> i32;
    unsafe fn remove_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Allocate the tiled/floating layout internally, then spawn into it. The
    /// context supplies the command, source pane, session/link, and spawn flags.
    /// Its nonowning layout reservation must be empty on entry and is cleared on return.
    /// Report errors before restoring zoom, preserving command/control event
    /// ordering. The callback may reenter; no component borrow spans the call.
    unsafe fn split_pane(
        &self,
        context: &mut spawn_context,
        arguments: &mut args,
        lines: pane_lines,
        restore_zoom: bool,
        report_error: impl FnOnce(&CStr),
    ) -> Result<Rc<UnsafeCell<window_pane>>, CString>;
    unsafe fn size(&self) -> (u32, u32);
    unsafe fn manual_size(&self) -> (u32, u32);
    /// Select manual sizing and record both requested dimensions together.
    unsafe fn set_manual_size(&self, sx: u32, sy: u32);
    unsafe fn pending_resize(&self) -> Option<(u32, u32)>;
    unsafe fn apply_pending_resize(&self);
    unsafe fn defer_resize(&self, sx: u32, sy: u32, xpixel: u32, ypixel: u32);
    /// Pixel dimensions of a terminal cell, for the pane's PTY resize protocol.
    unsafe fn cell_size(&self) -> (u32, u32);
    unsafe fn is_zoomed(&self) -> bool;
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32);
    /// Adopt a parsed layout's dimensions without resizing the previous tree
    /// or firing resize events. Preserve the terminal cell's pixel dimensions.
    unsafe fn set_layout_size(&self, sx: u32, sy: u32);
    /// Preserve command precedence: cycle, spread, then named/saved layout.
    /// `cycle` is -1 (previous), 0, or 1 (next). `legacy_format` preserves the
    /// attached control client's old custom-layout serialization format.
    unsafe fn select_layout(
        &self,
        name: Option<&CStr>,
        restore_previous: bool,
        cycle: i32,
        spread: Option<&Rc<UnsafeCell<window_pane>>>,
        legacy_format: bool,
    ) -> Result<(), CString>;
    unsafe fn zoom(&self, pane: &Rc<UnsafeCell<window_pane>>) -> i32;
    unsafe fn unzoom(&self, notify: bool) -> i32;
    unsafe fn update_activity(&self);
    /// Return whether the identity changed. Attachment and input dispatch have
    /// different notifications and keep that orchestration in their callers.
    unsafe fn set_latest_client(&self, client: Option<&ClientRef>) -> bool;
    unsafe fn is_latest_client(&self, client: &ClientRef) -> bool;
    /// Window-owned modal/menu/selection policy used when a pane's focus changes.
    unsafe fn pane_is_focused(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool;
    /// Observe the independently owned menu; no Window/component pointer escapes.
    unsafe fn menu_observer(&self) -> Option<refbox::Weak<menu_data>>;
    /// Detach only the expected menu (or any menu for None). The caller performs
    /// explicit cancellation after this operation, outside the Window borrow.
    unsafe fn take_menu(
        &self,
        expected: Option<&refbox::Weak<menu_data>>,
    ) -> Option<refbox::RefBox<menu_data>>;
    /// Publish first, then let the caller cancel a displaced menu outside the borrow.
    unsafe fn replace_menu(
        &self,
        menu: refbox::RefBox<menu_data>,
    ) -> Option<refbox::RefBox<menu_data>>;
    /// Clamp to the current dimensions and remember the resulting menu position.
    unsafe fn place_menu(&self, position: (u32, u32), size: (u32, u32)) -> (u32, u32);
    unsafe fn last_menu_position(&self) -> (u32, u32);
    /// Advance remembered placement only for omitted axes. The previous offset
    /// determines wrapping; explicit coordinates leave the cascade unchanged.
    unsafe fn resolve_floating_position(&self, x: Option<i32>, y: Option<i32>) -> (i32, i32);
    unsafe fn invalidate_scene(&self);
    unsafe fn scene_generation(&self) -> u64;
    /// Render each fill cell outside the Window borrow, publishing its fallback
    /// first, in inside/outside order as observed by format callbacks.
    unsafe fn refresh_fill_cells(&self);
    unsafe fn fill_cell(&self, inside: bool) -> grid_cell;
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R;
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    /// Required for owners that may be final: close callbacks execute while an
    /// owner is still live, and may retain it to postpone destruction.
    unsafe fn release(self, from: &CStr)
    where
        Self: Sized;
    /// Run the final-close decision while a containing winlink still publishes
    /// its Rc. The caller then detaches and drops exactly that reference; this
    /// operation must not manufacture an extra owner before testing the count.
    unsafe fn prepare_release(&self, from: &CStr);
}

impl Window for WindowRef {
    unsafe fn create(sx: u32, sy: u32, xpixel: u32, ypixel: u32) -> Self {
        window_create(sx, sy, xpixel, ypixel)
    }
    unsafe fn find_by_id(id: u32) -> Option<Self> {
        window_find_by_id(id)
    }
    unsafe fn find_by_id_str(id: &CStr) -> Option<Self> {
        window_find_by_id_str(id.as_ptr())
    }
    unsafe fn retain(&self, from: &CStr) -> Self {
        window_add_ref(self, from.as_ptr())
    }
    unsafe fn update_focus(&self) {
        window_update_focus(Some(self));
    }
    unsafe fn update_focus_for(window: Option<&Self>) {
        window_update_focus(window);
    }
    unsafe fn zoom_pane(pane: &Rc<UnsafeCell<window_pane>>) -> i32 {
        window_zoom(pane)
    }
    unsafe fn redraw_active_switch(&self, pane: Option<&Rc<UnsafeCell<window_pane>>>) {
        window_redraw_active_switch(self, pane);
    }
    unsafe fn pane_at(&self, x: u32, y: u32) -> Option<Rc<UnsafeCell<window_pane>>> {
        window_get_active_at(self, x, y)
    }
    unsafe fn find_pane(&self, name: &CStr) -> Option<Rc<UnsafeCell<window_pane>>> {
        window_find_string(self, name)
    }
    unsafe fn active_pane_over_zoom(&self) -> i32 {
        window_active_pane_is_over_zoom(self)
    }
    unsafe fn push_zoom(&self, always: bool, flag: bool) -> i32 {
        window_push_zoom(self, always as i32, flag as i32)
    }
    unsafe fn pop_zoom(&self) -> i32 {
        window_pop_zoom(self)
    }
    unsafe fn add_pane(
        &self,
        other: Option<&Rc<UnsafeCell<window_pane>>>,
        hlimit: u32,
        flags: i32,
    ) -> Rc<UnsafeCell<window_pane>> {
        window_add_pane(self, other, hlimit, flags)
    }
    unsafe fn forget_pane(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        window_lost_pane(self, pane);
    }
    unsafe fn destroy_panes(&self) {
        window_destroy_panes(self);
    }
    unsafe fn winlink_flags(link: refbox::Weak<winlink>, escape: bool) -> CString {
        window_printable_flags(link, escape as i32)
    }

    type LayoutCell<'a> = &'a layout_cell;
    type LayoutCellMut<'a> = &'a mut layout_cell;

    unsafe fn borrow_layout_cell(&self, id: *mut layout_cell) -> Option<Self::LayoutCell<'_>> {
        let state = &*self.get();
        state
            .layout_root
            .as_deref()
            .and_then(|root| root.find(id))
            .or_else(|| {
                state
                    .saved_layout_root
                    .as_deref()
                    .and_then(|root| root.find(id))
            })
    }

    unsafe fn borrow_layout_cell_mut(
        &self,
        id: *mut layout_cell,
    ) -> Option<Self::LayoutCellMut<'_>> {
        let state = &mut *self.get();
        state
            .layout_root
            .as_deref_mut()
            .and_then(|root| root.find_mut(id))
            .or_else(|| {
                state
                    .saved_layout_root
                    .as_deref_mut()
                    .and_then(|root| root.find_mut(id))
            })
    }

    type PaneOrderMut<'a> = &'a mut window_panes;
    unsafe fn borrow_pane_order_mut(&self, order: PaneOrder) -> Self::PaneOrderMut<'_> {
        let state = &mut *self.get();
        match order {
            PaneOrder::Index => &mut state.panes,
            PaneOrder::Stacking => &mut state.z_index,
        }
    }
    type PaneHistoryMut<'a> = &'a mut window_pane_history;
    unsafe fn borrow_pane_history_mut(&self) -> Self::PaneHistoryMut<'_> {
        &mut (*self.get()).last_panes
    }
    unsafe fn swap_pane_order(
        &self,
        order: PaneOrder,
        first: &Weak<UnsafeCell<window_pane>>,
        other: &WindowRef,
        second: &Weak<UnsafeCell<window_pane>>,
    ) {
        if Rc::ptr_eq(self, other) {
            self.borrow_pane_order_mut(order).swap(first, second);
            return;
        }
        let mut left = self.borrow_pane_order_mut(order);
        let mut right = other.borrow_pane_order_mut(order);
        let left_position = left.position(first).expect("first pane is not in order");
        let right_position = right.position(second).expect("second pane is not in order");
        let left_pane = left.remove_at(first);
        let right_pane = right.remove_at(second);
        left.insert_at(left_position, right_pane);
        right.insert_at(right_position, left_pane);
    }
    type LayoutRootMut<'a> = &'a mut Option<Box<layout_cell>>;
    unsafe fn borrow_layout_root_mut(&self) -> Self::LayoutRootMut<'_> {
        &mut (*self.get()).layout_root
    }
    type LayoutRoot<'a> = &'a layout_cell;
    unsafe fn borrow_layout_root(&self, view: LayoutView) -> Option<Self::LayoutRoot<'_>> {
        let state = &*self.get();
        match view {
            LayoutView::Visible => state.layout_root.as_deref(),
            LayoutView::Unzoomed => state
                .saved_layout_root
                .as_deref()
                .or(state.layout_root.as_deref()),
        }
    }
    unsafe fn layout_string(&self, view: LayoutView, legacy: bool) -> Option<CString> {
        let root = self.borrow_layout_root(view)?;
        layout_dump(root, legacy)
    }
    unsafe fn pane_layout_cell(
        &self,
        pane: &Weak<UnsafeCell<window_pane>>,
        view: LayoutView,
    ) -> Option<&layout_cell> {
        let state = &*self.get();
        let saved = match view {
            LayoutView::Unzoomed => state.saved_layout_root.as_deref(),
            LayoutView::Visible => None,
        };
        saved
            .and_then(|root| find_layout_pane(root, pane))
            .or_else(|| {
                state
                    .layout_root
                    .as_deref()
                    .and_then(|root| find_layout_pane(root, pane))
            })
    }

    unsafe fn last_layout_preset(&self) -> i32 {
        (*self.get()).lastlayout
    }
    unsafe fn remember_layout_preset(&self, preset: i32) {
        (*self.get()).lastlayout = preset;
    }

    unsafe fn refresh_fill_cells(&self) {
        for inside in [true, false] {
            let mut fallback = crate::src::grid::grid_default_cell;
            fallback.attr |= crate::src::shared::grid::GRID_ATTR_CHARSET as u16;
            crate::src::text::utf8::utf8_set(
                &mut fallback.data,
                crate::src::shared::borders::CELL_BORDERS
                    [crate::src::shared::borders::CELL_NONE as usize] as u8,
            );
            {
                let state = &mut *self.get();
                if inside {
                    state.inside_cell = fallback;
                } else {
                    state.outside_cell = fallback;
                }
            }
            if let Some(rendered) = crate::src::window_border::window_render_fill_cell(self, inside)
            {
                let state = &mut *self.get();
                if inside {
                    state.inside_cell = rendered;
                } else {
                    state.outside_cell = rendered;
                }
            }
        }
    }
    unsafe fn fill_cell(&self, inside: bool) -> grid_cell {
        let state = &*self.get();
        if inside {
            state.inside_cell
        } else {
            state.outside_cell
        }
    }

    unsafe fn menu_observer(&self) -> Option<refbox::Weak<menu_data>> {
        (*self.get()).menu.as_ref().map(refbox::RefBox::downgrade)
    }
    unsafe fn take_menu(
        &self,
        expected: Option<&refbox::Weak<menu_data>>,
    ) -> Option<refbox::RefBox<menu_data>> {
        let state = &mut *self.get();
        if expected.is_some_and(|expected| {
            !state
                .menu
                .as_ref()
                .is_some_and(|current| expected.is(current))
        }) {
            return None;
        }
        state.menu.take()
    }
    unsafe fn replace_menu(
        &self,
        menu: refbox::RefBox<menu_data>,
    ) -> Option<refbox::RefBox<menu_data>> {
        (*self.get()).menu.replace(menu)
    }
    unsafe fn place_menu(&self, (mut px, mut py): (u32, u32), (sx, sy): (u32, u32)) -> (u32, u32) {
        let state = &mut *self.get();
        if sx >= state.sx {
            px = 0;
        } else if px.wrapping_add(sx) > state.sx {
            px = state.sx.wrapping_sub(sx);
        }
        if sy >= state.sy {
            py = 0;
        } else if py.wrapping_add(sy) > state.sy {
            py = state.sy.wrapping_sub(sy);
        }
        state.menu_last_px = px;
        state.menu_last_py = py;
        (px, py)
    }
    unsafe fn last_menu_position(&self) -> (u32, u32) {
        let state = &*self.get();
        (state.menu_last_px, state.menu_last_py)
    }
    unsafe fn resolve_floating_position(&self, x: Option<i32>, y: Option<i32>) -> (i32, i32) {
        let state = &mut *self.get();
        let x = x.unwrap_or_else(|| {
            state.last_new_pane_x =
                if state.last_new_pane_x == 0 || state.last_new_pane_x > state.sx {
                    4
                } else {
                    state.last_new_pane_x.wrapping_add(4)
                };
            state.last_new_pane_x as i32
        });
        let y = y.unwrap_or_else(|| {
            state.last_new_pane_y =
                if state.last_new_pane_y == 0 || state.last_new_pane_y > state.sy {
                    2
                } else {
                    state.last_new_pane_y.wrapping_add(2)
                };
            state.last_new_pane_y as i32
        });
        (x, y)
    }
    unsafe fn invalidate_scene(&self) {
        (*self.get()).invalidate_scene();
    }
    unsafe fn scene_generation(&self) -> u64 {
        (*self.get()).redraw_scene_generation
    }
    unsafe fn add_winlink(&self, link: refbox::Weak<winlink>) {
        window_winlinks_append(&mut *self.get(), link);
    }
    unsafe fn remove_winlink(&self, link: refbox::Weak<winlink>) {
        window_winlinks_remove(&mut *self.get(), link);
    }
    unsafe fn next_window(&self) -> Option<WindowRef> {
        windows_next(&*self.get())
    }
    unsafe fn next_winlink(&self, after: Option<refbox::Weak<winlink>>) -> refbox::Weak<winlink> {
        if let Some(after) = after {
            window_winlinks_next(Some(&*self.get()), after)
        } else {
            window_winlinks_first(Some(&*self.get()))
        }
    }
    unsafe fn contains_pane(&self, pane: &Weak<UnsafeCell<window_pane>>) -> bool {
        window_has_pane(&*self.get(), pane)
    }
    unsafe fn is_linked_outside_group(&self, members: usize) -> bool {
        Rc::strong_count(self) != members
    }
    unsafe fn has_layout(&self) -> bool {
        (*self.get()).layout_root.is_some()
    }
    unsafe fn pending_alerts(&self) -> i32 {
        (*self.get()).flags & crate::src::shared::window::WINDOW_ALERTFLAGS
    }
    unsafe fn reset_alert_timer(&self) {
        super::alerts::reset_timer(self);
    }
    unsafe fn queue_alerts(&self, flags: i32) -> Option<bool> {
        super::alerts::queue(self, flags)
    }
    unsafe fn finish_alerts(&self) {
        (*self.get()).alerts_queued = 0;
        (*self.get()).flags &= !crate::src::shared::window::WINDOW_ALERTFLAGS;
    }
    unsafe fn clear_alert_flags(&self) {
        (*self.get()).flags &= !crate::src::shared::window::WINDOW_ALERTFLAGS;
    }

    unsafe fn id(&self) -> u32 {
        (*self.get()).id
    }
    unsafe fn initialize_pane(
        &self,
        pane: &Rc<UnsafeCell<window_pane>>,
        client: Option<&ClientRef>,
    ) {
        let state = &mut *self.get();
        let observer = Rc::downgrade(pane);
        state.panes.push_front(observer.clone());
        state.z_index.push_front(observer.clone());
        state.active = observer;
        state.latest = client.map_or_else(Weak::new, Rc::downgrade);
    }
    unsafe fn select_respawned_pane(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        (*self.get()).active = Weak::new();
        window_set_active_pane(self, pane, 0);
    }
    unsafe fn initialize_name(&self, name: CString, explicit: bool) {
        drop(window_replace_name(self, name));
        if explicit {
            self.with_options_mut(|options| {
                crate::src::options::options_set_number(options, c"automatic-rename".as_ptr(), 0)
            });
        }
    }
    unsafe fn name(&self) -> CString {
        (*self.get()).name.clone()
    }
    unsafe fn rename(&self, name: &CStr, untrusted: bool) {
        window_set_name(self, name.as_ptr(), untrusted as i32);
    }
    unsafe fn begin_name_check(&self, now: Instant) -> bool {
        let state = &mut *self.get();
        let left = crate::src::names::name_time_left(state.name_time, now);
        if !left.is_zero() {
            if state.name_event.is_none() {
                log_debug(format_args!(
                    "@{} name timer queued ({} left)",
                    state.id,
                    left.as_micros()
                ));
                let observer = Rc::downgrade(self);
                state.name_event = Some(
                    Timer::new(left, move || {
                        if let Some(owner) = observer.upgrade() {
                            drop((*owner.get()).name_event.take());
                            log_debug(format_args!("@{} name timer expired", owner.id()));
                        }
                    })
                    .expect("arm timer"),
                );
            } else {
                log_debug(format_args!(
                    "@{} name timer already queued ({} left)",
                    state.id,
                    left.as_micros()
                ));
            }
            return false;
        }
        state.name_time = Some(now);
        drop(state.name_event.take());
        true
    }
    unsafe fn schedule_offset_update(&self) {
        let state = &mut *self.get();
        if state.offset_timer.is_none() {
            let delay = Duration::from_micros(10_000);
            let observer = Rc::downgrade(self);
            state.offset_timer = Some(
                Timer::new(delay, move || {
                    if let Some(window) = observer.upgrade() {
                        drop((*window.get()).offset_timer.take());
                        crate::src::tty::tty_update_window_offset(&window);
                        window.release(c"offset update timer");
                    }
                })
                .expect("arm timer"),
            );
        }
    }
    unsafe fn active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>> {
        (*self.get()).active_pane()
    }
    unsafe fn active_pane_observer(&self) -> Weak<UnsafeCell<window_pane>> {
        (*self.get()).active.clone()
    }
    unsafe fn modal_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>> {
        (*self.get()).modal.upgrade()
    }
    unsafe fn begin_modal_pane(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        let state = &mut *self.get();
        state.modal_last = state.active.clone();
        state.modal = Rc::downgrade(pane);
    }
    unsafe fn last_active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>> {
        crate::src::shared::pane::pane_history_first(&(*self.get()).last_panes)
    }
    unsafe fn pane_snapshot(&self) -> Vec<Rc<UnsafeCell<window_pane>>> {
        (*self.get()).panes.snapshot()
    }
    unsafe fn pane_count(&self, with_floating: bool) -> u32 {
        self.pane_snapshot().into_iter().fold(0u32, |count, pane| {
            if with_floating || !pane.is_floating() {
                count.wrapping_add(1)
            } else {
                count
            }
        })
    }
    unsafe fn stacking_snapshot(&self) -> Vec<Rc<UnsafeCell<window_pane>>> {
        (*self.get()).z_index.snapshot()
    }
    unsafe fn step_pane(
        &self,
        order: PaneOrder,
        after: Option<&Weak<UnsafeCell<window_pane>>>,
        reverse: bool,
    ) -> Option<Rc<UnsafeCell<window_pane>>> {
        let state = &*self.get();
        let panes = match order {
            PaneOrder::Index => &state.panes,
            PaneOrder::Stacking => &state.z_index,
        };
        match (after, reverse) {
            (None, false) => panes.first(),
            (None, true) => panes.last(),
            (Some(pane), false) => panes.next(pane),
            (Some(pane), true) => panes.previous(pane),
        }
    }
    unsafe fn scrollbar_mode(&self) -> i32 {
        (*self.get()).sb
    }
    unsafe fn scrollbar_position(&self) -> i32 {
        (*self.get()).sb_pos
    }
    unsafe fn refresh_scrollbars(&self) {
        let state = &mut *self.get();
        let options = state.options.as_deref_mut().expect("live window options");
        state.sb = options_get_number(options, c"pane-scrollbars") as i32;
        state.sb_pos = options_get_number(options, c"pane-scrollbars-position") as i32;
    }
    unsafe fn pane_border_status(&self) -> i32 {
        window_get_pane_status(&*self.get())
    }
    unsafe fn pane_border_lines(&self) -> pane_lines {
        window_get_pane_lines(&*self.get())
    }
    unsafe fn pane_at_index(&self, index: u32) -> Option<Rc<UnsafeCell<window_pane>>> {
        let state = &mut *self.get();
        let base = options_get_number(
            state.options.as_deref_mut().expect("window options"),
            c"pane-base-index",
        ) as u32;
        state
            .panes
            .storage
            .get(index.wrapping_sub(base) as usize)
            .map(|pane| pane.upgrade().expect("live pane in ordering"))
    }
    unsafe fn pane_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32> {
        let state = &mut *self.get();
        let base = options_get_number(
            state.options.as_deref_mut().expect("window options"),
            c"pane-base-index",
        ) as u32;
        state
            .panes
            .position(pane)
            .map(|index| base.wrapping_add(index as u32))
    }
    unsafe fn pane_history_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32> {
        (*self.get())
            .last_panes
            .iter()
            .position(|entry| entry.ptr_eq(pane))
            .map(|index| index as u32)
    }
    unsafe fn pane_stacking_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32> {
        let order = (*self.get()).z_index.snapshot();
        let mut index = 0_u32;
        for owner in order {
            let floating = owner.is_floating();
            if Rc::downgrade(&owner).ptr_eq(pane) {
                return Some(if floating {
                    index
                } else {
                    index.wrapping_add(1)
                });
            }
            if floating {
                index = index.wrapping_add(1);
            }
        }
        None
    }
    unsafe fn pane_by_number(
        &self,
        pane: Option<&Rc<UnsafeCell<window_pane>>>,
        count: u32,
        reverse: bool,
    ) -> Option<Rc<UnsafeCell<window_pane>>> {
        let panes = &(*self.get()).panes;
        let mut current = pane.cloned();
        for _ in 0..count {
            current = current
                .as_ref()
                .and_then(|pane| {
                    let pane = Rc::downgrade(pane);
                    if reverse {
                        panes.previous(&pane)
                    } else {
                        panes.next(&pane)
                    }
                })
                .or_else(|| if reverse { panes.last() } else { panes.first() });
        }
        current
    }
    unsafe fn creation_time(&self) -> SystemTime {
        (*self.get()).creation_time
    }
    unsafe fn activity_time(&self) -> SystemTime {
        (*self.get()).activity_time
    }
    unsafe fn next_pane(
        &self,
        after: Option<&Rc<UnsafeCell<window_pane>>>,
    ) -> Option<Rc<UnsafeCell<window_pane>>> {
        let panes = &(*self.get()).panes;
        after.map_or_else(|| panes.first(), |pane| panes.next(&Rc::downgrade(pane)))
    }
    unsafe fn select_pane(&self, pane: &Rc<UnsafeCell<window_pane>>, notify: bool) -> i32 {
        assert!(
            window_has_pane(&*self.get(), &Rc::downgrade(pane)),
            "pane belongs to window"
        );
        window_set_active_pane(self, pane, notify as i32)
    }
    unsafe fn remove_pane(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        assert!(
            window_has_pane(&*self.get(), &Rc::downgrade(pane)),
            "pane belongs to window"
        );
        window_remove_pane(self, pane);
    }
    unsafe fn split_pane(
        &self,
        context: &mut spawn_context,
        arguments: &mut args,
        lines: pane_lines,
        restore_zoom: bool,
        report_error: impl FnOnce(&CStr),
    ) -> Result<Rc<UnsafeCell<window_pane>>, CString> {
        assert!(
            context.layout.is_none(),
            "layout cells must stay inside the window operation"
        );
        let item = context
            .item
            .upgrade()
            .expect("split command retained by caller");
        let pane = context
            .wp0
            .upgrade()
            .expect("split source pane retained by caller");
        assert!(
            window_has_pane(&*self.get(), &Rc::downgrade(&pane)),
            "split source belongs to window"
        );
        assert!(
            context
                .wl
                .get_unchecked()
                .window_handle()
                .is_some_and(|window| Rc::ptr_eq(window, self)),
            "split link belongs to window"
        );
        if context.flags & crate::src::shared::spawn::SPAWN_MODAL != 0 {
            if context.flags & SPAWN_FLOATING == 0 {
                let error = c"modal pane must be floating".to_owned();
                report_error(&error);
                return Err(error);
            }
            if (*self.get()).modal.upgrade().is_some() {
                let error = c"window already has a modal pane".to_owned();
                report_error(&error);
                return Err(error);
            }
        }
        let layout = if context.flags & SPAWN_FLOATING != 0 {
            layout_get_floating_cell(&item, arguments, lines, self, &pane, context.flags)
        } else {
            layout_get_tiled_cell(&item, arguments, self, &pane, context.flags)
        };
        let cell = match layout {
            Ok(cell) => cell,
            Err(error) => {
                report_error(&error);
                if restore_zoom {
                    window_pop_zoom(self);
                }
                return Err(error);
            }
        };
        context.layout = Some(cell);
        let mut cause = None;
        let result = spawn_pane(context, &mut cause);
        context.layout = None;
        result.ok_or_else(|| {
            let mut message = b"create pane failed: ".to_vec();
            message.extend_from_slice(
                cause
                    .expect("failed pane spawn provides a diagnostic")
                    .as_bytes(),
            );
            let error = CString::new(message).expect("spawn diagnostic contains no NUL");
            report_error(&error);
            if restore_zoom || context.flags & SPAWN_FLOATING == 0 {
                window_pop_zoom(self);
            }
            error
        })
    }
    unsafe fn size(&self) -> (u32, u32) {
        ((*self.get()).sx, (*self.get()).sy)
    }
    unsafe fn manual_size(&self) -> (u32, u32) {
        let state = &*self.get();
        (state.manual_sx, state.manual_sy)
    }
    unsafe fn set_manual_size(&self, sx: u32, sy: u32) {
        let state = &mut *self.get();
        crate::src::options::options_set_number(
            state.options.as_deref_mut().expect("live window options"),
            c"window-size".as_ptr(),
            crate::src::shared::window::WINDOW_SIZE_MANUAL as _,
        );
        state.manual_sx = sx;
        state.manual_sy = sy;
    }
    unsafe fn pending_resize(&self) -> Option<(u32, u32)> {
        let state = &*self.get();
        (state.flags & WINDOW_RESIZE != 0).then_some((state.new_sx, state.new_sy))
    }
    unsafe fn apply_pending_resize(&self) {
        if (*self.get()).flags & WINDOW_RESIZE != 0 {
            resize_window(
                self,
                (*self.get()).new_sx,
                (*self.get()).new_sy,
                (*self.get()).new_xpixel as i32,
                (*self.get()).new_ypixel as i32,
            );
        }
    }
    unsafe fn defer_resize(&self, sx: u32, sy: u32, xpixel: u32, ypixel: u32) {
        let state = &mut *self.get();
        state.new_sx = sx;
        state.new_sy = sy;
        state.new_xpixel = xpixel;
        state.new_ypixel = ypixel;
        state.flags |= WINDOW_RESIZE;
    }
    unsafe fn cell_size(&self) -> (u32, u32) {
        ((*self.get()).xpixel, (*self.get()).ypixel)
    }
    unsafe fn is_zoomed(&self) -> bool {
        (*self.get()).flags & WINDOW_ZOOMED != 0
    }
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32) {
        resize_window(self, sx, sy, xpixel, ypixel);
    }
    unsafe fn set_layout_size(&self, sx: u32, sy: u32) {
        window_resize(self, sx, sy, -1, -1);
    }
    unsafe fn select_layout(
        &self,
        name: Option<&CStr>,
        restore_previous: bool,
        cycle: i32,
        spread: Option<&Rc<UnsafeCell<window_pane>>>,
        legacy_format: bool,
    ) -> Result<(), CString> {
        assert!((-1..=1).contains(&cycle), "layout cycle direction");
        if self.unzoom(true) == 0 {
            server_redraw_window(self);
        }
        let new_layout = self.layout_string(LayoutView::Visible, legacy_format);
        let old_layout = window_replace_old_layout(self, new_layout);
        if cycle > 0 {
            layout_set_next(self);
        } else if cycle < 0 {
            layout_set_previous(self);
        } else if let Some(pane) = spread {
            assert!(
                window_has_pane(&*self.get(), &Rc::downgrade(pane)),
                "spread pane belongs to window"
            );
            layout_spread_out(pane);
        } else {
            let requested = name.or({
                if restore_previous {
                    old_layout.as_deref()
                } else {
                    None
                }
            });
            let preset = if restore_previous {
                -1
            } else {
                requested.map_or((*self.get()).lastlayout, |name| {
                    layout_set_lookup(name.as_ptr())
                })
            };
            if preset != -1 {
                layout_set_select(self, preset as u32);
            } else if let Some(name) = requested {
                let mut cause = None;
                if layout_parse(self, name.as_ptr(), &mut cause) == -1 {
                    let mut message = cause
                        .expect("failed layout parse provides a diagnostic")
                        .into_bytes();
                    message.extend_from_slice(b": ");
                    message.extend_from_slice(name.to_bytes());
                    drop(window_replace_old_layout(self, old_layout));
                    return Err(CString::new(message).expect("layout diagnostic contains no NUL"));
                }
            } else {
                return Ok(());
            }
        }
        recalculate_sizes();
        server_redraw_window(self);
        events_fire_window(c"window-layout-changed".as_ptr(), self.clone());
        Ok(())
    }
    unsafe fn zoom(&self, pane: &Rc<UnsafeCell<window_pane>>) -> i32 {
        assert!(
            window_has_pane(&*self.get(), &Rc::downgrade(pane)),
            "zoom pane belongs to window"
        );
        window_zoom(pane)
    }
    unsafe fn unzoom(&self, notify: bool) -> i32 {
        window_unzoom(self, notify as i32)
    }
    unsafe fn update_activity(&self) {
        window_update_activity(self);
    }
    unsafe fn set_latest_client(&self, client: Option<&ClientRef>) -> bool {
        let latest = client.map_or_else(Weak::new, Rc::downgrade);
        if (*self.get()).latest.ptr_eq(&latest) {
            return false;
        }
        (*self.get()).latest = latest;
        true
    }
    unsafe fn is_latest_client(&self, client: &ClientRef) -> bool {
        (*self.get()).latest.ptr_eq(&Rc::downgrade(client))
    }
    unsafe fn pane_is_focused(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool {
        use crate::src::server_client::Client;
        if (*self.get()).menu.is_some()
            || !self
                .active_pane()
                .is_some_and(|active| Rc::ptr_eq(&active, pane))
        {
            return false;
        }
        let mut cursor = clients.first();
        while let Some(client) = cursor {
            if client.focuses_window(self) {
                return true;
            }
            cursor = clients.next(&client);
        }
        false
    }
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R {
        edit(
            (*self.get())
                .options
                .as_deref_mut()
                .expect("live window options"),
        )
    }
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue> {
        crate::src::format::window_format_value(self, key, context)
    }
    unsafe fn release(self, from: &CStr) {
        window_remove_ref(self, from.as_ptr());
    }
    unsafe fn prepare_release(&self, from: &CStr) {
        window_prepare_release(self, from.as_ptr());
    }
}

unsafe fn resize_fire_window_resized(w_owner: &WindowRef, mut old_sx: u_int, mut old_sy: u_int) {
    let mut w = w_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w_owner, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut ep, &fs);
    event_payload_set_window(
        &mut ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    event_payload_set_uint(
        &mut ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sx,
    );
    event_payload_set_uint(
        &mut ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sy,
    );
    event_payload_set_uint(
        &mut ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        &mut ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"window-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe fn resize_window(
    w_owner: &WindowRef,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    let mut w = w_owner.get();
    let mut old_sx: u_int = (*w).sx;
    let mut old_sy: u_int = (*w).sy;
    if sx < WINDOW_MINIMUM as u_int {
        sx = WINDOW_MINIMUM as u_int;
    }
    if sx > WINDOW_MAXIMUM as u_int {
        sx = WINDOW_MAXIMUM as u_int;
    }
    if sy < WINDOW_MINIMUM as u_int {
        sy = WINDOW_MINIMUM as u_int;
    }
    if sy > WINDOW_MAXIMUM as u_int {
        sy = WINDOW_MAXIMUM as u_int;
    }
    let zoomed_owner = window_zoomed_pane(&*w);
    if zoomed_owner.is_some() {
        window_unzoom(w_owner, 1 as ::core::ffi::c_int);
    }
    layout_resize(w_owner, sx, sy);
    if sx
        < (*(*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root))
        .g
        .sx
    {
        sx = (*(*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root))
        .g
        .sx;
    }
    if sy
        < (*(*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root))
        .g
        .sy
    {
        sy = (*(*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root))
        .g
        .sy;
    }
    window_resize(w_owner, sx, sy, xpixel, ypixel);
    log_debug(format_args!(
        "{}: @{} resized to {}x{}; layout {}x{}",
        "resize_window",
        { (*w).id },
        { sx },
        { sy },
        ((*(*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root))
        .g
        .sx) as u32,
        ((*(*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root))
        .g
        .sy) as u32
    ));
    if let Some(zoomed_owner) = zoomed_owner {
        if window_has_pane(&*w, &std::rc::Rc::downgrade(&zoomed_owner)) {
            window_zoom(&zoomed_owner);
        }
    }
    tty_update_window_offset(&(*(w)).observer.upgrade().expect("live window"));
    server_redraw_window(w_owner);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    resize_fire_window_resized(w_owner, old_sx, old_sy);
    (*w).flags &= !WINDOW_RESIZE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_cell_borrows_follow_identity_through_saved_tree_transfer() {
        unsafe {
            let window = window::new();
            let first = layout_cell::new();
            let original = first.id();
            *window.borrow_layout_root_mut() = Some(first);
            window.borrow_layout_cell_mut(original).unwrap().g.xoff = 7;
            let detached = window.borrow_layout_root_mut().take();
            (*window.get()).saved_layout_root = detached;
            let second = layout_cell::new();
            let replacement = second.id();
            *window.borrow_layout_root_mut() = Some(second);
            window.borrow_layout_cell_mut(original).unwrap().g.xoff = 11;
            window.borrow_layout_cell_mut(replacement).unwrap().g.xoff = 23;
            assert_eq!(window.borrow_layout_cell(original).unwrap().g.xoff, 11);
            assert_eq!(window.borrow_layout_cell(replacement).unwrap().g.xoff, 23);
            drop((*window.get()).saved_layout_root.take());
            assert!(window.borrow_layout_cell(original).is_none());
            assert_eq!(window.borrow_layout_cell(replacement).unwrap().g.xoff, 23);
            window.release(c"layout identity borrow test");
        }
    }

    #[test]
    fn floating_cascade_records_only_default_axes_and_wraps_after_crossing_bounds() {
        unsafe {
            let owner = window::new();
            (*owner.get()).sx = 8;
            (*owner.get()).sy = 4;
            assert_eq!(owner.resolve_floating_position(None, None), (4, 2));
            assert_eq!(owner.resolve_floating_position(Some(-3), None), (-3, 4));
            assert_eq!(owner.resolve_floating_position(None, Some(12)), (8, 12));
            assert_eq!(owner.resolve_floating_position(None, None), (12, 6));
            assert_eq!(owner.resolve_floating_position(None, None), (4, 2));
            owner.release(c"floating cascade test");
        }
    }

    #[test]
    fn fill_rendering_keeps_inside_outside_formats_and_composes_border_style() {
        unsafe {
            // Rendering initializes a temporary screen, whose input-mode setup
            // reads the server's extended-keys option.
            let mut server_options = options_create(None);
            crate::src::options::options_default(
                &mut *server_options,
                crate::src::options_table::options_table
                    .iter()
                    .find(|entry| entry.name == Some(c"extended-keys"))
                    .unwrap(),
            );
            let previous_server_options =
                std::mem::replace(&mut crate::src::tmux::global_options, &mut *server_options);
            let owner = window::new();
            let mut options = options_create(None);
            let entry = crate::src::options_table::options_table
                .iter()
                .find(|entry| entry.name == Some(c"fill-character"))
                .unwrap();
            crate::src::options::options_default(&mut *options, entry);
            (*owner.get()).options = Some(options);
            owner.with_options_mut(|options| {
                crate::src::options::options_set_string(
                    options,
                    c"fill-character".as_ptr(),
                    0,
                    |out| out.write_all(b"#[fg=red]#{?is_inside,I,O}"),
                );
            });
            owner.refresh_fill_cells();
            let original = owner.fill_cell(true);
            assert_eq!(original.data.data[0], b'I');
            assert_eq!(owner.fill_cell(false).data.data[0], b'O');
            let mut border = crate::src::grid::grid_default_cell;
            border.fg = 4;
            border.bg = 3;
            border.attr = crate::src::shared::grid::GRID_ATTR_UNDERSCORE as u16;
            crate::src::window_border::window_get_fill_cell(&owner, 1, &mut border);
            assert_eq!(border.data.data[0], b'I');
            assert_eq!(border.fg, 1, "fill overrides an explicit foreground");
            assert_eq!(
                border.bg, 3,
                "default fill background preserves border style"
            );
            assert_ne!(
                border.attr & crate::src::shared::grid::GRID_ATTR_UNDERSCORE as u16,
                0
            );
            owner.with_options_mut(|options| {
                crate::src::options::options_set_string(
                    options,
                    c"fill-character".as_ptr(),
                    0,
                    |out| out.write_all(b"#{?is_outside,X,Y}"),
                );
            });
            owner.refresh_fill_cells();
            assert_eq!(owner.fill_cell(true).data.data[0], b'Y');
            assert_eq!(owner.fill_cell(false).data.data[0], b'X');
            assert_eq!(
                original.data.data[0], b'I',
                "fill snapshots survive refresh"
            );
            owner.release(c"fill rendering test");
            crate::src::tmux::global_options = previous_server_options;
            crate::src::options::options_free(server_options);
        }
    }

    #[test]
    fn offset_timer_only_observes_window_and_is_cancelled_on_close() {
        unsafe {
            let window = window::new();
            let observer = Rc::downgrade(&window);
            window.schedule_offset_update();
            window.schedule_offset_update();
            assert!((*window.get()).offset_timer.is_some());
            assert_eq!(Rc::strong_count(&window), 1);
            while (*window.get()).offset_timer.is_some() {
                crate::src::reactor::poll_runtime();
            }
            assert_eq!(
                Rc::strong_count(&window),
                1,
                "dispatch releases its temporary owner"
            );
            window.schedule_offset_update();
            assert!((*window.get()).offset_timer.is_some());
            window.release(c"offset timer test");
            assert!(observer.upgrade().is_none());
            crate::src::reactor::init_runtime();
            crate::src::reactor::poll_runtime();
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn automatic_name_timer_releases_completed_and_cancelled_waits() {
        unsafe {
            let window = window::new();
            let observer = Rc::downgrade(&window);
            let now = Instant::now();
            assert!(window.begin_name_check(now));
            assert!(!window.begin_name_check(now + Duration::from_micros(100_000)));
            assert!((*window.get()).name_event.is_some());
            assert!(!window.begin_name_check(now + Duration::from_micros(200_000)));
            assert!(window.begin_name_check(now + Duration::from_micros(500_000)));
            assert!((*window.get()).name_event.is_none());
            assert_eq!(
                (*window.get()).name_time,
                Some(now + Duration::from_micros(500_000))
            );
            for _ in 0..2 {
                assert!(!window.begin_name_check(now + Duration::from_micros(999_999)));
                assert!((*window.get()).name_event.is_some());
                while (*window.get()).name_event.is_some() {
                    crate::src::reactor::poll_runtime();
                }
            }
            assert_eq!(Rc::strong_count(&window), 1);
            window.release(c"automatic name timer test");
            assert!(observer.upgrade().is_none());
            crate::src::reactor::shutdown_runtime();
        }
    }
}
