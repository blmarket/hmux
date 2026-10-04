//! Operations on an opaque, retained window.
//!
//! The holder preserves allocation lifetime, not exclusive access. Callers must
//! preserve the server's single-threaded lifecycle and callback ordering. Scoped
//! option callbacks must not reenter the window, free/reparent the component, or
//! let references or pointers escape.

use super::*;
use crate::src::server_client::Client as _;
use crate::src::shared::client::ClientRef;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::{WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_RESIZE};
use crate::src::spawn::spawn_pane;
use crate::src::window_pane::WindowPane as _;
use std::time::{Duration, Instant, SystemTime};

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
    unsafe fn redraw_active_switch(&self, pane: Option<&Rc<UnsafeCell<window_pane>>>);
    unsafe fn pane_at(&self, x: u32, y: u32) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn find_pane(&self, name: &CStr) -> Option<Rc<UnsafeCell<window_pane>>>;
    /// Create a pane after `other` (before it with SPAWN_BEFORE), or as the
    /// first pane of an empty window, and arrange the strip. The spawn
    /// protocol notifies once the pane's process starts.
    unsafe fn add_pane(
        &self,
        other: Option<&Rc<UnsafeCell<window_pane>>>,
        hlimit: u32,
        flags: i32,
    ) -> Rc<UnsafeCell<window_pane>>;
    /// Whether one more pane of `width` keeps the strip within
    /// WINDOW_MAXIMUM columns.
    unsafe fn has_room_for_pane(&self, width: PaneWidth) -> bool;
    /// Insert a pane already reparented to this window after `target`, or
    /// before it, and arrange the strip. The caller notifies.
    unsafe fn insert_pane(
        &self,
        pane: &Rc<UnsafeCell<window_pane>>,
        target: &Rc<UnsafeCell<window_pane>>,
        before: bool,
    );
    /// Retire a pane that moves to another window: its selection, history and
    /// membership. The strip closes the gap and window-layout-changed fires.
    unsafe fn detach_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    unsafe fn winlink_flags(link: refbox::Weak<winlink>, escape: bool) -> CString
    where
        Self: Sized;

    /// Selection history is separate from pane order. Only pure history edits
    /// may run while this component is borrowed.
    type PaneHistoryMut<'a>: std::ops::DerefMut<Target = window_pane_history>
    where
        Self: 'a;
    unsafe fn borrow_pane_history_mut(&self) -> Self::PaneHistoryMut<'_>;
    /// Swap two panes' places in the strip, in one window or across two whose
    /// panes the caller has already reparented, and arrange each strip.
    /// Neither selection nor notifications change.
    unsafe fn swap_pane_order(
        &self,
        first: &Weak<UnsafeCell<window_pane>>,
        other: &WindowRef,
        second: &Weak<UnsafeCell<window_pane>>,
    );
    /// Toggle a pane between half and full width, arrange the strip and
    /// redraw; window-layout-changed fires. Widening is refused, returning
    /// false, when it would take the strip past WINDOW_MAXIMUM columns.
    unsafe fn toggle_pane_width(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool;
    /// Move the last pane to the front (`down`) or the first pane to the end,
    /// and arrange the strip. Neither selection nor notifications change.
    unsafe fn rotate_panes(&self, down: bool);
    /// The strip's rectangles in pane order, before pane border and scrollbar
    /// adjustments. Retained panes and copied geometry only.
    unsafe fn pane_cells(&self) -> Vec<(Rc<UnsafeCell<window_pane>>, layout_geometry)>;
    /// The area clients clip and pan across: the strip's extent, which is the
    /// last pane's first column plus the window width, by the window height.
    unsafe fn logical_size(&self) -> (u32, u32);
    /// The strip as a single-row layout string. `legacy` selects the
    /// checksummed format of older control clients.
    unsafe fn layout_string(&self, legacy: bool) -> Option<CString>;

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

    /// Publish a newly created window's first pane and client, and arrange it,
    /// before naming and creation notifications. No selection callbacks run here.
    unsafe fn initialize_pane(
        &self,
        pane: &Rc<UnsafeCell<window_pane>>,
        client: Option<&ClientRef>,
    );
    /// Respawn keeps only `pane`: destroy every other pane, arrange the strip,
    /// and select the survivor even when it was already active.
    unsafe fn reset_to_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
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
    /// Most recently active pane from selection history.
    unsafe fn last_active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>>;
    /// Retain the current pane order without retaining a Window borrow, for
    /// sorting, rendering and other walks that may reenter the Window.
    unsafe fn pane_snapshot(&self) -> Vec<Rc<UnsafeCell<window_pane>>>;
    unsafe fn step_pane(
        &self,
        after: Option<&Weak<UnsafeCell<window_pane>>>,
        reverse: bool,
    ) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn scrollbar_mode(&self) -> i32;
    unsafe fn scrollbar_position(&self) -> i32;
    /// Refresh cached scrollbar policy after options change. Pane resizing is
    /// performed by the caller after this borrow ends.
    unsafe fn refresh_scrollbars(&self);
    unsafe fn pane_border_lines(&self) -> pane_lines;
    unsafe fn pane_at_index(&self, index: u32) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn pane_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32>;
    unsafe fn pane_history_index(&self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<u32>;
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
    /// Retire and destroy a pane. The strip closes the gap and
    /// window-layout-changed fires.
    unsafe fn remove_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Insert a pane beside the context's source pane and spawn into it. The
    /// context supplies the source pane, session/link and spawn flags. A full
    /// strip refuses before any side effect.
    unsafe fn new_pane(
        &self,
        context: &mut spawn_context,
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
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32);
    /// Arrange the strip again after options or pane modes it reads changed.
    /// Silent; returns whether any pane moved.
    unsafe fn refit(&self) -> bool;
    unsafe fn update_activity(&self);
    /// Return whether the identity changed. Attachment and input dispatch have
    /// different notifications and keep that orchestration in their callers.
    unsafe fn set_latest_client(&self, client: Option<&ClientRef>) -> bool;
    unsafe fn is_latest_client(&self, client: &ClientRef) -> bool;
    /// Window-owned selection policy used when a pane's focus changes.
    unsafe fn pane_is_focused(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool;
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
    unsafe fn redraw_active_switch(&self, pane: Option<&Rc<UnsafeCell<window_pane>>>) {
        window_redraw_active_switch(self, pane);
    }
    unsafe fn pane_at(&self, x: u32, y: u32) -> Option<Rc<UnsafeCell<window_pane>>> {
        window_get_active_at(self, x, y)
    }
    unsafe fn find_pane(&self, name: &CStr) -> Option<Rc<UnsafeCell<window_pane>>> {
        window_find_string(self, name)
    }
    unsafe fn add_pane(
        &self,
        other: Option<&Rc<UnsafeCell<window_pane>>>,
        hlimit: u32,
        flags: i32,
    ) -> Rc<UnsafeCell<window_pane>> {
        window_add_pane(self, other, hlimit, flags)
    }
    unsafe fn has_room_for_pane(&self, width: PaneWidth) -> bool {
        let mut widths = window_pane_widths(self);
        widths.push(width);
        window_strip_fits(self, &widths)
    }
    unsafe fn insert_pane(
        &self,
        pane: &Rc<UnsafeCell<window_pane>>,
        target: &Rc<UnsafeCell<window_pane>>,
        before: bool,
    ) {
        let state = &mut *self.get();
        assert!(
            window_has_pane(state, &Rc::downgrade(target)),
            "insertion target belongs to window"
        );
        if before {
            window_pane_list_insert_before(state, target, pane);
        } else {
            window_pane_list_insert_after(state, target, pane);
        }
        state.invalidate_scene();
        window_arrange(self);
    }
    unsafe fn detach_pane(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        window_take_pane(self, pane);
    }
    unsafe fn winlink_flags(link: refbox::Weak<winlink>, escape: bool) -> CString {
        window_printable_flags(link, escape as i32)
    }

    type PaneHistoryMut<'a> = &'a mut window_pane_history;
    unsafe fn borrow_pane_history_mut(&self) -> Self::PaneHistoryMut<'_> {
        &mut (*self.get()).last_panes
    }
    unsafe fn swap_pane_order(
        &self,
        first: &Weak<UnsafeCell<window_pane>>,
        other: &WindowRef,
        second: &Weak<UnsafeCell<window_pane>>,
    ) {
        if Rc::ptr_eq(self, other) {
            (*self.get()).panes.swap(first, second);
        } else {
            let left = &mut (*self.get()).panes;
            let right = &mut (*other.get()).panes;
            let left_position = left.position(first).expect("first pane is not in order");
            let right_position = right.position(second).expect("second pane is not in order");
            let left_pane = left.remove_at(first);
            let right_pane = right.remove_at(second);
            left.insert_at(left_position, right_pane);
            right.insert_at(right_position, left_pane);
            (*other.get()).invalidate_scene();
            window_arrange(other);
        }
        (*self.get()).invalidate_scene();
        window_arrange(self);
    }
    unsafe fn toggle_pane_width(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool {
        let position = (*self.get())
            .panes
            .position(&Rc::downgrade(pane))
            .expect("pane belongs to window");
        let width = pane.width_preference().toggled();
        // Narrowing always succeeds, even in a strip a resize took past the
        // maximum.
        if width == PaneWidth::Full {
            let mut widths = window_pane_widths(self);
            widths[position] = width;
            if !window_strip_fits(self, &widths) {
                return false;
            }
        }
        pane.set_width_preference(width);
        (*self.get()).invalidate_scene();
        window_arrange(self);
        server_redraw_window(self);
        events_fire_window(c"window-layout-changed".as_ptr(), self.clone());
        true
    }
    unsafe fn rotate_panes(&self, down: bool) {
        let state = &mut *self.get();
        let moved = if down {
            state.panes.last()
        } else {
            state.panes.first()
        }
        .expect("rotation window has panes");
        let observer = Rc::downgrade(&moved);
        assert!(
            state.panes.remove(&observer),
            "pane is not in its window order"
        );
        if down {
            state.panes.push_front(observer);
        } else {
            state.panes.push_back(observer);
        }
        state.invalidate_scene();
        window_arrange(self);
    }
    unsafe fn pane_cells(&self) -> Vec<(Rc<UnsafeCell<window_pane>>, layout_geometry)> {
        window_strip_cells(self)
    }
    unsafe fn logical_size(&self) -> (u32, u32) {
        let state = &*self.get();
        // Never narrower than the window, even before a pane is arranged.
        (state.extent.max(state.sx), state.sy)
    }
    unsafe fn layout_string(&self, legacy: bool) -> Option<CString> {
        let active = self.active_pane_observer();
        let entries = self
            .pane_cells()
            .into_iter()
            .map(|(pane, geometry)| {
                let observer = Rc::downgrade(&pane);
                strip::LayoutEntry {
                    geometry,
                    id: pane.id(),
                    index: self.pane_index(&observer).expect("strip pane has an index"),
                    active: active.ptr_eq(&observer),
                    last: self.pane_history_index(&observer),
                }
            })
            .collect::<Vec<_>>();
        let state = &*self.get();
        strip::layout_string(&entries, (state.strip_width, state.sy), legacy)
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
        state.active = observer;
        state.latest = client.map_or_else(Weak::new, Rc::downgrade);
        window_arrange(self);
    }
    unsafe fn reset_to_pane(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        let observer = Rc::downgrade(pane);
        assert!(
            (*self.get()).panes.remove(&observer),
            "pane is not in its window order"
        );
        window_destroy_panes(self);
        (*self.get()).panes.push_front(observer);
        (*self.get()).invalidate_scene();
        window_arrange(self);
        (*self.get()).active = Weak::new();
        window_set_active_pane(self, pane, 0);
    }
    unsafe fn initialize_name(&self, name: CString, explicit: bool) {
        drop(window_replace_name(self, name));
        if explicit {
            self.with_options_mut(|options| {
                crate::src::options::options_set_number(options, c"automatic-rename", 0)
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
    unsafe fn last_active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>> {
        crate::src::shared::pane::pane_history_first(&(*self.get()).last_panes)
    }
    unsafe fn pane_snapshot(&self) -> Vec<Rc<UnsafeCell<window_pane>>> {
        (*self.get()).panes.snapshot()
    }
    unsafe fn step_pane(
        &self,
        after: Option<&Weak<UnsafeCell<window_pane>>>,
        reverse: bool,
    ) -> Option<Rc<UnsafeCell<window_pane>>> {
        let panes = &(*self.get()).panes;
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
    unsafe fn new_pane(
        &self,
        context: &mut spawn_context,
    ) -> Result<Rc<UnsafeCell<window_pane>>, CString> {
        let pane = context
            .wp0
            .upgrade()
            .expect("source pane retained by caller");
        assert!(
            window_has_pane(&*self.get(), &Rc::downgrade(&pane)),
            "source pane belongs to window"
        );
        assert!(
            context
                .wl
                .get_unchecked()
                .window_handle()
                .is_some_and(|window| Rc::ptr_eq(window, self)),
            "spawn link belongs to window"
        );
        if !self.has_room_for_pane(PaneWidth::Half) {
            return Err(c"no space for a new pane".to_owned());
        }
        let mut cause = None;
        spawn_pane(context, &mut cause).ok_or_else(|| {
            let mut message = b"create pane failed: ".to_vec();
            message.extend_from_slice(
                cause
                    .expect("failed pane spawn provides a diagnostic")
                    .as_bytes(),
            );
            CString::new(message).expect("spawn diagnostic contains no NUL")
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
            c"window-size",
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
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32) {
        resize_window(self, sx, sy, xpixel, ypixel);
    }
    unsafe fn refit(&self) -> bool {
        window_arrange(self)
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
        if !self
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
    event_payload_set_window(&mut ep, c"window".as_ptr(), std::rc::Rc::clone(w_owner));
    event_payload_set_uint(&mut ep, c"width".as_ptr(), (*w).sx);
    event_payload_set_uint(&mut ep, c"height".as_ptr(), (*w).sy);
    event_payload_set_uint(&mut ep, c"old_width".as_ptr(), old_sx);
    event_payload_set_uint(&mut ep, c"old_height".as_ptr(), old_sy);
    events_fire(c"window-resized".as_ptr(), ep);
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
    window_resize(w_owner, sx, sy, xpixel, ypixel);
    log_debug(format_args!(
        "{}: @{} resized to {}x{}; strip {} columns",
        "resize_window",
        { (*w).id },
        { sx },
        { sy },
        { (*w).strip_width }
    ));
    server_redraw_window(w_owner);
    events_fire_window(
        c"window-layout-changed".as_ptr(),
        std::rc::Rc::clone(w_owner),
    );
    resize_fire_window_resized(w_owner, old_sx, old_sy);
    (*w).flags &= !WINDOW_RESIZE;
}

#[cfg(test)]
mod tests {
    use super::*;

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
                crate::src::options::options_set_string(options, c"fill-character", 0, |out| {
                    out.write_all(b"#[fg=red]#{?is_inside,I,O}")
                });
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
                crate::src::options::options_set_string(options, c"fill-character", 0, |out| {
                    out.write_all(b"#{?is_outside,X,Y}")
                });
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
}
