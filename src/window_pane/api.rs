//! Operations on retained panes. Logical destruction remains explicit.
use super::*;
use crate::src::shared::pane::{PANE_NEWSTATUS,PANE_DROP};
use crate::src::grid::{grid_get_cell, grid_set_cell};
use crate::src::hyperlinks::{hyperlinks_get, hyperlinks_put};
use crate::src::reactor::Interests;
use crate::src::server_client::Client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::colour::colour_palette;
use crate::src::shared::command::{cmd, cmd_retval};
use crate::src::shared::pane::{
    PANE_ACTIVITY, PANE_CAPTUREALLKEYS, PANE_CLOSEONCANCEL, PANE_CLOSEONCLICK, PANE_MINIMUM,
};
use crate::src::shared::screen::MODE_SYNC;
use crate::src::shared::window::{WindowRef, WindowWeak};
use std::time::Duration;

/// Access a pane without lending its model storage.
///
/// Methods retain the legacy event-loop exclusivity and logical-lifetime
/// preconditions. Component closures must not reenter model code, destroy an
/// owner, alter component parent links, or let component references escape.
pub trait WindowPane {
    unsafe fn compare_for_sort(&self, other: &Self, criteria: &crate::src::shared::sort::sort_criteria) -> std::cmp::Ordering;
    unsafe fn update_history_limit(&self, limit: u32);
    unsafe fn mark_theme_changed(&self);
    unsafe fn update_scrollbar_hover(&self, x: i32, y: i32);
    unsafe fn mouse_location(&self, x: i32, y: i32, slider: &mut u32) -> key_code_mouse_location;
    unsafe fn in_scrollbar_area(&self, x: i32, y: i32) -> bool;
    unsafe fn screen_progress(&self) -> crate::src::shared::display::progress_bar;
    unsafe fn screen_path(&self) -> CString;
    unsafe fn prompt_position(&self, top: bool) -> Option<(i32, i32)>;
    unsafe fn screen_mode(&self, displayed: bool) -> crate::src::shared::screen::ScreenMode;
    unsafe fn mark_unseen_changes(&self);
    unsafe fn needs_redraw(&self, scrollbar: bool) -> bool;
    unsafe fn notify_style_changed(&self);
    unsafe fn has_modes(&self) -> bool;
    unsafe fn captures_keys(&self) -> bool;
    unsafe fn closes_on_cancel(&self) -> bool;
    unsafe fn closes_on_click(&self) -> bool;
    unsafe fn has_exited(&self) -> bool;
    unsafe fn close_after_key(&self, key: key_code) -> bool;
    unsafe fn matches_terminal_name(&self, name: &CStr) -> bool;
    unsafe fn next_in_window(&self) -> Option<Self> where Self: Sized;
    unsafe fn send_theme_update(&self);
    /// Retain registry membership before operations that may dispatch callbacks.
    unsafe fn all_panes() -> Vec<Self> where Self: Sized;
    /// Copied appearance and slider geometry; no pane storage is lent.
    unsafe fn scrollbar(&self) -> super::render::PaneScrollbar;
    unsafe fn redraw_scrollbar(&self);
    unsafe fn hide_scrollbar(&self);
    unsafe fn refresh_scrollbar_style(&self);
    unsafe fn reset_default_cursor(&self);
    unsafe fn reset_border_cache(&self);
    unsafe fn border_cell(&self, kind:i32, cell:&mut grid_cell);
    unsafe fn border_style(&self, client:&ClientRef) -> grid_cell;
    unsafe fn make_status(&self, client:&ClientRef, width:u32, spans:&crate::src::shared::redraw::redraw_spans, first:usize) -> bool;
    unsafe fn has_new_status(&self) -> bool;
    unsafe fn set_new_status(&self, redraw: bool);
    unsafe fn default_colours(&self) -> (grid_cell, u32);
    /// Start a bounded screen write. A requested component must stay allocated
    /// until stop. For a pane screen (including a null request for its displayed
    /// screen), stop before replacing that screen, changing modes, or logically
    /// destroying the pane. The context observes the pane without owning it.
    unsafe fn prepare_write(&self, context: &mut crate::src::shared::screen_write::screen_write_ctx, requested: *mut screen);
    unsafe fn is_obscured(&self) -> bool;
    unsafe fn visible_ranges(base: Option<&Self>, x:i32, y:i32, width:u32, ranges: &mut Vec<visible_range>) where Self: Sized;
    unsafe fn draws_inactive_screen(&self) -> bool;
    unsafe fn window_size(&self) -> (u32,u32);
    unsafe fn schedule_offset_update(&self);
    unsafe fn output_suppressed(&self) -> bool;
    unsafe fn alternate_screen_allowed(&self) -> bool;
    unsafe fn alternate_screen_changed(&self, entered: bool);
    /// Draw with screen ownership temporarily removed from pane storage.
    unsafe fn draw_line(&self, client: &ClientRef, source: (u32,u32), width: u32, destination: (u32,u32), status: bool);
    unsafe fn draw_scrollbar(&self, client: &ClientRef, span: &crate::src::shared::redraw::redraw_span, x: u32, y: u32, width: u32);
    unsafe fn draw_prompt(&self, context: &mut crate::src::screen_redraw::redraw_draw_ctx<'_>);
    unsafe fn mark_style_changed(&self, theme: bool);

    /// Visible geometry including the reserved scrollbar area, copied for hit testing.
    unsafe fn outer_geometry(&self) -> (i32, i32, u32, u32);
    unsafe fn is_floating(&self) -> bool;
    unsafe fn border_status(&self) -> i32;
    /// Minimum width for splitting this pane into two, using the caller's
    /// already-read scrollbar mode and this pane's own scrollbar dimensions.
    unsafe fn split_minimum_width(&self, reserve_scrollbar: bool) -> u32;
    unsafe fn unzoomed_width(&self) -> Option<u32>;
    unsafe fn unzoomed_height(&self) -> Option<u32>;
    unsafe fn has_pending_change(&self) -> bool;
    unsafe fn acknowledge_change(&self);
    /// Derive an owned default Window name from this pane's command or shell.
    unsafe fn default_window_name(&self) -> CString;
    /// A borrow can become Ref::map on one RefCell containing the whole pane.
    type Palette<'a>: std::ops::Deref<Target = colour_palette>
    where
        Self: 'a;
    /// Do not reenter the pane, dispatch callbacks or let component pointers
    /// escape while the returned guard is alive.
    unsafe fn borrow_palette(&self) -> Self::Palette<'_>;

    unsafe fn id(&self) -> u32;
    unsafe fn window_observer(&self) -> WindowWeak;
    /// Move the parent identity and option inheritance together, before the
    /// caller publishes membership or dispatches move/layout notifications.
    unsafe fn reparent(&self, window: &WindowRef);
    unsafe fn geometry(&self) -> (u32, u32, i32, i32);
    /// Stable, nonowning identity resolved only under the Window layout guard.
    unsafe fn layout_identity(&self, saved: bool) -> Option<LayoutCellId>;
    unsafe fn place_in_layout(&self, cell: LayoutCellId);
    /// Clear placement only if this is still the cell that owns the pane.
    unsafe fn detach_layout(&self, cell: LayoutCellId);
    unsafe fn save_layout_for_zoom(&self);
    unsafe fn restore_layout_after_zoom(&self);
    unsafe fn mark_zoomed(&self);
    unsafe fn is_zoomed(&self) -> bool;
    unsafe fn floats_over_zoom(&self) -> bool;
    unsafe fn minimum_layout_width(&self, reserve_scrollbar: bool) -> u32;
    /// Apply copied Window geometry, including pane border and scrollbar policy.
    /// Returns whether the visible geometry changed; resize callbacks run after
    /// the pane's placement fields have been released.
    unsafe fn apply_layout(
        &self,
        geometry: crate::src::window::PaneLayoutGeometry,
        scrollbars: crate::src::window::WindowScrollbars,
    ) -> bool;
    /// Mark redraw when the cached active and inactive appearance differs.
    unsafe fn redraw_selection_change(&self);
    /// Move the visible origin before resizing; dispatches no callbacks.
    unsafe fn set_layout_offset(&self, x: i32, y: i32);
    unsafe fn refresh_palette(&self);
    unsafe fn mark_changed(&self);
    unsafe fn invalidate_style(&self);
    unsafe fn set_input_enabled(&self, enabled: bool);
    unsafe fn set_title(&self, title: &CStr) -> bool;
    /// Trim history below the base cursor when no mode owns the displayed screen.
    unsafe fn trim_history(&self);
    /// Enable modal input behavior before publishing the new pane.
    unsafe fn configure_modal(&self, capture_keys: bool, close_click: bool, close_cancel: bool);
    unsafe fn wait_until_close(&self, item: &Rc<UnsafeCell<cmdq_item>>);
    /// Start stdin delivery to an empty pane, retaining command waits in the file callback.
    unsafe fn start_input(&self, item: &Rc<UnsafeCell<cmdq_item>>) -> Result<i32, CString>;
    unsafe fn is_visible(&self) -> bool;
    unsafe fn contains(&self, x: u32, y: u32) -> bool;
    unsafe fn pane_lines(&self) -> pane_lines;

    /// Translate a mouse report after copying pane geometry; no pane borrow
    /// survives the coordinate calculation or the caller's following work.
    unsafe fn mouse_position(&self, mouse: &mouse_event, last: bool) -> Option<(u32, u32)>;
    /// Visible cursor in window coordinates, for terminal viewport following.
    unsafe fn visible_cursor_in_window(&self) -> Option<(u32, u32)>;
    /// Whether the pane still has a PTY, including an exited process being drained.
    unsafe fn has_tty(&self) -> bool;
    unsafe fn is_synchronized(&self) -> bool;
    unsafe fn start_sync(&self);
    /// Cancel the timer and leave sync mode before drawing the accumulated rows.
    /// No pane reference is retained during terminal callbacks.
    unsafe fn stop_sync(&self);
    unsafe fn clear_sync_dirty(&self);
    /// Decide whether a screen write draws immediately; synchronized writes
    /// mark rows in the pane-owned bitmap, including resize invalidation.
    unsafe fn should_draw_rows(&self, synchronized: bool, y: u32, count: u32, height: u32) -> bool;

    unsafe fn resize(&self, sx: u32, sy: u32);
    unsafe fn update_focus(&self, focused: bool);
    /// Return 0 for an unplaced pane, -1 for deferred drawing, or 1 after
    /// installing pane offsets. Client applies terminal/status offsets afterward.
    /// With `window_redraw` false, the pane is never changed; a disposable context
    /// can therefore probe placement even when dirty drawing would be deferred.
    unsafe fn prepare_render(
        &self,
        context: &mut crate::src::shared::tty::tty_ctx,
        window_redraw: bool,
    ) -> i32;
    unsafe fn request_redraw(&self, scrollbar: bool);
    /// Normal selection records activity; fallback selection only marks change.
    unsafe fn on_selected(&self, record_activity: bool);
    unsafe fn key(
        &self,
        client: Option<&ClientRef>,
        link: refbox::Weak<winlink>,
        key: key_code,
        mouse: Option<&mut mouse_event>,
    ) -> i32;
    unsafe fn paste(&self, key: key_code, bytes: &[u8]);
    unsafe fn set_mode(
        &self,
        source: Option<&Rc<UnsafeCell<window_pane>>>,
        mode: &'static window_mode,
        item: Option<&Rc<UnsafeCell<cmdq_item>>>,
        find: Option<&mut cmd_find_state>,
        arguments: Option<&mut args>,
    ) -> i32;
    unsafe fn reset_mode(&self);
    unsafe fn reset_all_modes(&self);
    /// Replace destination with an independent rectangular copy. Coordinates
    /// are relative to the visible screen, excluding history. `displayed`
    /// selects the active mode screen; false selects the process's base screen.
    /// The destination must not alias any pane screen; caller must screen_free it.
    unsafe fn copy_screen(
        &self,
        destination: &mut screen,
        x: u32,
        y: u32,
        sx: u32,
        sy: u32,
        displayed: bool,
    );
    unsafe fn copy_output(&self, offset: &window_pane_offset, destination: &mut [u8]) -> usize;
    /// Start a new output consumer at the parser's current position.
    unsafe fn output_offset(&self) -> window_pane_offset;
    unsafe fn advance_output(&self, offset: &mut window_pane_offset, count: usize);
    /// Run once after all clients have consumed the current event-loop cycle.
    unsafe fn finish_cycle(&self);
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R;
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    /// Execute pipe-pane while process descriptors and callbacks stay owned here.
    unsafe fn pipe(
        &self,
        command: refbox::Weak<cmd>,
        item: &Rc<UnsafeCell<cmdq_item>>,
    ) -> cmd_retval;
    /// Parse available process output and advance the pane's output cursor.
    unsafe fn parse_input(&self);
    /// Parse caller-owned bytes without exposing pane screens or parser storage.
    unsafe fn parse_output(&self, bytes: &[u8]);
    unsafe fn process_id(&self) -> pid_t;
    /// Record an exit before finishing waits/editors; false means a different process.
    unsafe fn process_exited(&self, pid: pid_t, status: i32) -> bool;
    unsafe fn kill_process(&self);
    /// Close descriptors before exit notifications and remain-on-exit rendering.
    unsafe fn finish_process(&self, notify: i32);
    unsafe fn spawn_process(
        context: *mut crate::src::shared::spawn::spawn_context,
        cause: *mut Option<CString>,
    ) -> Option<Self>
    where
        Self: Sized;
    unsafe fn install_editor(
        &self,
        editor: Box<crate::src::shared::spawn::spawn_editor_state>,
    ) -> crate::src::shared::spawn::EditorHandle;
    unsafe fn finish_editing(&self);
    unsafe fn editor_identity(&self) -> Option<crate::src::shared::spawn::EditorId>;
    unsafe fn cancel_editor(&self, identity: crate::src::shared::spawn::EditorId);
    unsafe fn editor_process_id(&self, identity: crate::src::shared::spawn::EditorId) -> pid_t;
    /// Encode a key or mouse report for this pane's process stream.
    unsafe fn write_key(&self, key: key_code, mouse: Option<&mouse_event>) -> i32;
    unsafe fn mode_entry(&self) -> refbox::Weak<window_mode_entry>;

    unsafe fn is_mode(&self, mode: &'static window_mode) -> bool;

    unsafe fn add_mode_formats(&self, context: &mut crate::src::shared::format::format_tree);

    unsafe fn reset_input(&self);
    unsafe fn destroy_ready(&self) -> bool;
    unsafe fn destroy(&self);
}

impl WindowPane for Rc<UnsafeCell<window_pane>> {
    unsafe fn compare_for_sort(&self, other: &Self, criteria: &crate::src::shared::sort::sort_criteria) -> std::cmp::Ordering {
        sort::compare(self, other, criteria)
    }
    unsafe fn update_history_limit(&self, limit: u32) {
        let pane = &mut *self.get();
        let id = pane.id;
        let grid = pane.base.grid_mut();
        let old_size = grid.hsize;
        grid.hlimit = limit;
        crate::src::grid::grid_collect_history(grid, 1);
        if grid.hsize != old_size {
            log_debug(format_args!("session_update_history: %{} {} -> {}", id, old_size, grid.hsize));
        }
    }
    unsafe fn mark_theme_changed(&self) { (*self.get()).flags |= PANE_THEMECHANGED; }
    unsafe fn update_scrollbar_hover(&self, x: i32, y: i32) {
        if self.is_visible() {
            if self.in_scrollbar_area(x, y) { window_pane_scrollbar_show(self); }
            else { window_pane_scrollbar_start_timer(self); }
        }
    }
    unsafe fn mouse_location(&self, x: i32, y: i32, slider: &mut u32) -> key_code_mouse_location {
        super::mouse::mouse_location(self, x, y, slider)
    }
    unsafe fn in_scrollbar_area(&self, x: i32, y: i32) -> bool {
        super::mouse::in_scrollbar_area(self, x, y) != 0
    }
    unsafe fn screen_progress(&self) -> crate::src::shared::display::progress_bar {
        (*self.get()).base.progress_bar
    }
    unsafe fn screen_path(&self) -> CString {
        (*self.get()).base.path.as_deref().unwrap_or(c"").to_owned()
    }
    unsafe fn prompt_position(&self, top: bool) -> Option<(i32, i32)> {
        let pane = &*self.get();
        if window_pane_has_prompt(pane) == 0 { return None; }
        let y = if top { pane.yoff } else {
            (pane.yoff as u32).wrapping_add(pane.sy).wrapping_sub(1) as i32
        };
        Some(((pane.xoff as u32).wrapping_add(pane.prompt_cx) as i32,y))
    }
    unsafe fn screen_mode(&self, displayed: bool) -> crate::src::shared::screen::ScreenMode {
        let pane = &*self.get();
        let screen = if displayed { &*pane.screen_ptr() } else { &pane.base };
        crate::src::shared::screen::ScreenMode::from(screen)
    }
    unsafe fn mark_unseen_changes(&self) { (*self.get()).flags |= PANE_UNSEENCHANGES; }
    unsafe fn needs_redraw(&self, scrollbar: bool) -> bool {
        (*self.get()).flags & if scrollbar { PANE_REDRAWSCROLLBAR } else { PANE_REDRAW } != 0
    }
    unsafe fn notify_style_changed(&self) {
        if (*self.get()).flags & PANE_STYLECHANGED == 0 { return; }
        let entry = self.mode_entry();
        if !entry.is_alive() { return; }
        let callback = entry.try_borrow_mut().expect("style mode not borrowed").mode.style_changed;
        if let Some(callback) = callback { callback(entry); }
    }
    unsafe fn has_modes(&self) -> bool { !(*self.get()).modes.is_empty() }
    unsafe fn captures_keys(&self) -> bool {
        let pane = &*self.get();
        pane.flags & crate::src::shared::pane::PANE_CAPTUREALLKEYS != 0 && pane.modes.is_empty()
    }
    unsafe fn closes_on_cancel(&self) -> bool { (*self.get()).flags & crate::src::shared::pane::PANE_CLOSEONCANCEL != 0 }
    unsafe fn closes_on_click(&self) -> bool { (*self.get()).flags & crate::src::shared::pane::PANE_CLOSEONCLICK != 0 }
    unsafe fn has_exited(&self) -> bool { (*self.get()).flags & PANE_EXITED != 0 }
    unsafe fn close_after_key(&self, key: key_code) -> bool {
        if !self.has_exited() || key & KEYC_MASK_KEY == KEYC_MOUSE
            || (KEYC_TYPE_MOUSEMOVE as key_code) << 32 <= key & KEYC_MASK_TYPE
                && key & KEYC_MASK_TYPE <= (KEYC_TYPE_TRIPLECLICK as key_code) << 32
            || key & KEYC_MASK_TYPE == (KEYC_TYPE_FUNCTION as key_code) << 32
                && (key & KEYC_MASK_KEY == KEYC_PASTE_START || key & KEYC_MASK_KEY == KEYC_PASTE_END)
        { return false; }
        let close = self.with_options_mut(|options| {
            let remain = options_get_number(options, c"remain-on-exit".as_ptr());
            if remain == 3 || remain == 4 {
                crate::src::options::options_set_number(options,c"remain-on-exit".as_ptr(),0);
                true
            } else { false }
        });
        if close { self.finish_process(0); }
        close
    }
    unsafe fn matches_terminal_name(&self, name: &CStr) -> bool {
        CStr::from_ptr((*self.get()).tty.as_ptr()) == name
    }
    unsafe fn next_in_window(&self) -> Option<Self> {
        let window = self.window_observer().upgrade()?;
        let next = window.next_pane(Some(self));
        window.release(c"pane successor parent");
        next
    }
    unsafe fn send_theme_update(&self) { window_pane_send_theme_update(self); }
    unsafe fn all_panes() -> Vec<Self> {
        let Some(index) = all_window_panes.storage.as_ref() else { return Vec::new() };
        let panes=index.try_borrow_mut().expect("pane registry unborrowed");
        panes.values().cloned().collect()
    }
    unsafe fn scrollbar(&self) -> super::render::PaneScrollbar { super::render::scrollbar(self) }
    unsafe fn redraw_scrollbar(&self) { window_pane_scrollbar_redraw(self); }
    unsafe fn hide_scrollbar(&self) { window_pane_scrollbar_hide(self); }
    unsafe fn refresh_scrollbar_style(&self) { super::render::refresh_scrollbar_style(self); }
    unsafe fn reset_default_cursor(&self) { window_pane_default_cursor(self); }
    unsafe fn reset_border_cache(&self) { let pane = &mut *self.get(); pane.border_gc_set=0; pane.active_border_gc_set=0; }
    unsafe fn border_cell(&self, kind:i32, cell:&mut grid_cell) { super::border::cell(self,kind,cell); }
    unsafe fn border_style(&self, client:&ClientRef) -> grid_cell { super::border::border_style(self,client) }
    unsafe fn make_status(&self, client:&ClientRef, width:u32, spans:&crate::src::shared::redraw::redraw_spans, first:usize) -> bool { super::border::make_status(self,client,width,spans,first) }
    unsafe fn has_new_status(&self) -> bool { (*self.get()).flags & PANE_NEWSTATUS != 0 }
    unsafe fn set_new_status(&self, redraw: bool) { if redraw { (*self.get()).flags |= PANE_NEWSTATUS; } else { (*self.get()).flags &= !PANE_NEWSTATUS; } }
    unsafe fn default_colours(&self) -> (grid_cell,u32) { super::render::default_colours(self) }
    unsafe fn prepare_write(&self, context: &mut crate::src::shared::screen_write::screen_write_ctx, requested: *mut screen) { super::render::prepare_write(self, context, requested); }
    unsafe fn is_obscured(&self) -> bool { super::render::is_obscured(self) }
    unsafe fn visible_ranges(base: Option<&Self>, x:i32, y:i32, width:u32, ranges: &mut Vec<visible_range>) { super::render::visible_ranges(base,x,y,width,ranges); }
    unsafe fn draws_inactive_screen(&self) -> bool {
        let window=self.window_observer().upgrade().expect("live pane parent");
        let inactive=!window.active_pane().is_some_and(|owner| Rc::ptr_eq(&owner,self));
        window.release(c"pane write screen");
        inactive || !matches!((*self.get()).screen_source,PaneScreenSource::Base)
    }
    unsafe fn window_size(&self) -> (u32,u32) { let window=self.window_observer().upgrade().expect("live pane parent"); let size=window.size(); window.release(c"pane window size"); size }
    unsafe fn schedule_offset_update(&self) { let window=self.window_observer().upgrade().expect("live pane parent"); window.schedule_offset_update(); window.release(c"pane cursor update"); }
    unsafe fn output_suppressed(&self) -> bool { (*self.get()).flags & (PANE_REDRAW | PANE_DROP) != 0 }
    unsafe fn alternate_screen_allowed(&self) -> bool { self.with_options_mut(|options| options_get_number(options, c"alternate-screen".as_ptr())) != 0 }
    unsafe fn alternate_screen_changed(&self, entered: bool) { super::render::alternate_screen_changed(self, entered); }
    unsafe fn draw_line(&self, client: &ClientRef, source: (u32,u32), width: u32, destination: (u32,u32), status: bool) { super::render::draw_line(self,client,source,width,destination,status); }
    unsafe fn draw_scrollbar(&self, client: &ClientRef, span: &crate::src::shared::redraw::redraw_span, x: u32, y: u32, width: u32) { super::render::draw_scrollbar(self,client,span,x,y,width); }
    unsafe fn draw_prompt(&self, context: &mut crate::src::screen_redraw::redraw_draw_ctx<'_>) { super::render::draw_prompt(self,context); }
    unsafe fn mark_style_changed(&self, theme: bool) { (*self.get()).flags |= PANE_STYLECHANGED; if theme { (*self.get()).flags |= PANE_THEMECHANGED; } }

    unsafe fn set_layout_offset(&self, x: i32, y: i32) {
        let pane = &mut *self.get();
        pane.xoff = x;
        pane.yoff = y;
    }
    unsafe fn refresh_palette(&self) {
        let pane = &mut *self.get();
        crate::src::style::colour::colour_palette_from_option(
            Some(&mut pane.palette),
            crate::src::options::options_owner_ptr(&mut pane.options).expect("live pane options"),
        );
    }
    unsafe fn mark_changed(&self) {
        (*self.get()).flags |= PANE_CHANGED;
    }
    unsafe fn invalidate_style(&self) {
        (*self.get()).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
    }
    unsafe fn set_input_enabled(&self, enabled: bool) {
        if enabled {
            (*self.get()).flags &= !PANE_INPUTOFF;
        } else {
            (*self.get()).flags |= PANE_INPUTOFF;
        }
    }
    unsafe fn set_title(&self, title: &CStr) -> bool {
        crate::src::screen::screen_set_title(&mut (*self.get()).base, title, 0) != 0
    }
    unsafe fn trim_history(&self) {
        let pane = &mut *self.get();
        if !pane.modes.is_empty() {
            return;
        }
        let adjust = pane
            .base
            .grid()
            .sy
            .wrapping_sub(1)
            .wrapping_sub(pane.base.cy)
            .min(pane.base.grid().hsize);
        crate::src::grid::grid_remove_history(pane.base.grid_mut(), adjust);
        pane.base.cy = pane.base.cy.wrapping_add(adjust);
        pane.flags |= PANE_REDRAW;
    }
    unsafe fn configure_modal(&self, capture_keys: bool, close_click: bool, close_cancel: bool) {
        let pane = &mut *self.get();
        if capture_keys {
            pane.flags |= PANE_CAPTUREALLKEYS;
        }
        if close_click {
            pane.flags |= PANE_CLOSEONCLICK;
        }
        if close_cancel {
            pane.flags |= PANE_CLOSEONCANCEL;
        }
    }
    unsafe fn wait_until_close(&self, item: &Rc<UnsafeCell<cmdq_item>>) {
        (*self.get()).wait_item = Rc::downgrade(item);
    }
    unsafe fn start_input(&self, item: &Rc<UnsafeCell<cmdq_item>>) -> Result<i32, CString> {
        window_pane_start_input(self, item)
    }
    unsafe fn layout_identity(&self, saved: bool) -> Option<LayoutCellId> {
        if saved {
            (*self.get()).saved_layout_cell
        } else {
            (*self.get()).layout_cell
        }
    }
    unsafe fn place_in_layout(&self, cell: LayoutCellId) {
        (*self.get()).layout_cell = Some(cell);
    }
    unsafe fn detach_layout(&self, cell: LayoutCellId) {
        if (*self.get()).layout_cell == Some(cell) {
            (*self.get()).layout_cell = None;
        }
    }
    unsafe fn save_layout_for_zoom(&self) {
        let pane = &mut *self.get();
        pane.saved_layout_cell = pane.layout_cell.take();
    }
    unsafe fn restore_layout_after_zoom(&self) {
        let pane = &mut *self.get();
        pane.layout_cell = pane.saved_layout_cell.take();
        pane.flags &= !PANE_ZOOMED;
    }
    unsafe fn mark_zoomed(&self) {
        (*self.get()).flags |= PANE_ZOOMED;
    }
    unsafe fn is_zoomed(&self) -> bool {
        (*self.get()).flags & PANE_ZOOMED != 0
    }
    unsafe fn floats_over_zoom(&self) -> bool {
        (*self.get()).flags & PANE_FLOATOVERZOOM != 0
    }
    unsafe fn minimum_layout_width(&self, reserve_scrollbar: bool) -> u32 {
        if reserve_scrollbar {
            let style = &(*self.get()).scrollbar_style;
            (PANE_MINIMUM + style.width + style.pad) as u32
        } else {
            PANE_MINIMUM as u32
        }
    }
    unsafe fn apply_layout(
        &self,
        geometry: crate::src::window::PaneLayoutGeometry,
        scrollbars: crate::src::window::WindowScrollbars,
    ) -> bool {
        let old_geometry = self.geometry();
        let (mut sx, mut sy) = geometry.size;
        let (mut xoff, mut yoff) = geometry.offset;
        let status = self.border_status();
        let has_border = match status {
            PANE_STATUS_TOP => geometry.top_border,
            PANE_STATUS_BOTTOM => geometry.bottom_border,
            _ => false,
        };
        if !geometry.floating && has_border {
            if status == PANE_STATUS_TOP {
                yoff += 1;
            }
            if sy > 1 {
                sy -= 1;
            }
        }
        let reserve = window_pane_scrollbar_reserve(&*self.get()) != 0;
        if reserve {
            let (width, pad) = {
                let pane = &*self.get();
                (
                    pane.scrollbar_style.width.max(1),
                    pane.scrollbar_style.pad.max(0),
                )
            };
            if scrollbars.position == PANE_SCROLLBARS_LEFT {
                if sx as i32 - width - pad < PANE_MINIMUM {
                    xoff += sx as i32 - PANE_MINIMUM;
                    sx = PANE_MINIMUM as u32;
                } else {
                    sx = sx.wrapping_sub(width as u32).wrapping_sub(pad as u32);
                    xoff += width + pad;
                }
            } else if sx as i32 - width - pad < PANE_MINIMUM {
                sx = PANE_MINIMUM as u32;
            } else {
                sx = sx.wrapping_sub(width as u32).wrapping_sub(pad as u32);
            }
        }
        {
            let pane = &mut *self.get();
            pane.xoff = xoff;
            pane.yoff = yoff;
            if reserve {
                pane.flags |= PANE_REDRAWSCROLLBAR;
            }
        }
        self.resize(sx, sy);
        self.geometry() != old_geometry
    }
    unsafe fn redraw_selection_change(&self) {
        let pane = &mut *self.get();
        if !grid_cells_look_equal(&pane.cached_gc, &pane.cached_active_gc)
            || pane.cached_dim != pane.cached_active_dim
            || colour_palette_get(Some(&pane.palette), pane.cached_gc.fg)
                != colour_palette_get(Some(&pane.palette), pane.cached_active_gc.fg)
            || colour_palette_get(Some(&pane.palette), pane.cached_gc.bg)
                != colour_palette_get(Some(&pane.palette), pane.cached_active_gc.bg)
        {
            pane.flags |= PANE_REDRAW;
        }
    }
    unsafe fn is_visible(&self) -> bool {
        window_pane_is_visible(self) != 0
    }
    unsafe fn contains(&self, x: u32, y: u32) -> bool {
        window_pane_contains(self, x, y) != 0
    }
    unsafe fn pane_lines(&self) -> pane_lines {
        window_pane_get_pane_lines(&*self.get())
    }

    unsafe fn outer_geometry(&self) -> (i32, i32, u32, u32) {
        window_pane_full_size_offset(self)
    }
    unsafe fn is_floating(&self) -> bool {
        window_pane_is_floating(&*self.get()) != 0
    }
    unsafe fn border_status(&self) -> i32 {
        window_pane_get_pane_status(&*self.get())
    }
    unsafe fn split_minimum_width(&self, reserve_scrollbar: bool) -> u32 {
        if reserve_scrollbar {
            let style = &(*self.get()).scrollbar_style;
            (PANE_MINIMUM * 2 + style.width + style.pad) as u32
        } else {
            (PANE_MINIMUM * 2 + 1) as u32
        }
    }
    unsafe fn unzoomed_width(&self) -> Option<u32> {
        let window = self.window_observer().upgrade().expect("pane window");
        let geometry = window.pane_layout_geometry(
            &Rc::downgrade(self),
            crate::src::window::LayoutView::Unzoomed,
        );
        let Some(geometry) = geometry else {
            window.release(c"unzoomed pane width");
            return None;
        };
        let reserve = if geometry.saved {
            let main_screen = (*self.get()).base.saved_grid.is_none();
            main_screen && window.scrollbars().mode == PANE_SCROLLBARS_ALWAYS
        } else {
            window_pane_scrollbar_reserve(&*self.get()) != 0
        };
        let mut width = geometry.size.0;
        if reserve {
            let (bar, pad) = {
                let state = &*self.get();
                (
                    state.scrollbar_style.width.max(1),
                    state.scrollbar_style.pad.max(0),
                )
            };
            width = if width as i32 - bar - pad < PANE_MINIMUM {
                PANE_MINIMUM as u32
            } else {
                width.wrapping_sub((bar + pad) as u32)
            };
        }
        window.release(c"unzoomed pane width");
        Some(width)
    }
    unsafe fn unzoomed_height(&self) -> Option<u32> {
        let window = self.window_observer().upgrade().expect("pane window");
        let geometry = window.pane_layout_geometry(
            &Rc::downgrade(self),
            crate::src::window::LayoutView::Unzoomed,
        );
        let Some(geometry) = geometry else {
            window.release(c"unzoomed pane height");
            return None;
        };
        let status = if geometry.saved && !geometry.floating {
            window.pane_border_status()
        } else {
            self.border_status()
        };
        let border = match status {
            PANE_STATUS_TOP => geometry.top_border,
            PANE_STATUS_BOTTOM => geometry.bottom_border,
            _ => false,
        };
        let height = geometry.size.1;
        let height = if !geometry.floating && border && height > 1 {
            height - 1
        } else {
            height
        };
        window.release(c"unzoomed pane height");
        Some(height)
    }
    unsafe fn has_pending_change(&self) -> bool {
        (*self.get()).flags & PANE_CHANGED != 0
    }
    unsafe fn acknowledge_change(&self) {
        (*self.get()).flags &= !PANE_CHANGED;
    }
    unsafe fn default_window_name(&self) -> CString {
        let pane = &*self.get();
        let command = crate::src::cmd::cmd_stringify_argv_cstring(&pane.argv);
        let source = command
            .as_deref()
            .filter(|text| !text.to_bytes().is_empty())
            .unwrap_or_else(|| pane.shell.as_deref().expect("pane shell"));
        crate::src::names::parse_window_name_cstring(source)
    }
    type Palette<'a> = &'a colour_palette;
    unsafe fn borrow_palette(&self) -> Self::Palette<'_> {
        &(*self.get()).palette
    }

    unsafe fn is_synchronized(&self) -> bool {
        (*self.get()).base.mode & MODE_SYNC != 0
    }
    unsafe fn start_sync(&self) {
        super::pane_sync::start(self);
    }
    unsafe fn stop_sync(&self) {
        super::pane_sync::stop(self);
    }
    unsafe fn clear_sync_dirty(&self) {
        super::pane_sync::clear_dirty(&mut *self.get());
    }
    unsafe fn should_draw_rows(&self, synchronized: bool, y: u32, count: u32, height: u32) -> bool {
        super::pane_sync::should_draw_rows(&mut *self.get(), synchronized, y, count, height)
    }

    unsafe fn id(&self) -> u32 {
        (*self.get()).id
    }

    unsafe fn window_observer(&self) -> WindowWeak {
        (*self.get()).window.clone()
    }
    unsafe fn reparent(&self, window: &WindowRef) {
        let state = &mut *self.get();
        state.window = Rc::downgrade(window);
        state
            .options
            .as_deref_mut()
            .expect("live pane options")
            .parent = Some(crate::src::options::OptionsScope::Window(Rc::downgrade(
            window,
        )));
        state.flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
    }

    unsafe fn geometry(&self) -> (u32, u32, i32, i32) {
        let pane = &*self.get();
        (pane.sx, pane.sy, pane.xoff, pane.yoff)
    }

    unsafe fn mouse_position(&self, mouse: &mouse_event, last: bool) -> Option<(u32, u32)> {
        let (width, height, xoff, yoff) = self.geometry();
        let (mx, my) = if last {
            (mouse.lx, mouse.ly)
        } else {
            (mouse.x, mouse.y)
        };
        let x = mx.wrapping_add(mouse.ox);
        let mut y = my.wrapping_add(mouse.oy);
        if mouse.statusat == 0 && y >= mouse.statuslines {
            y = y.wrapping_sub(mouse.statuslines);
        }
        if (x as i32) < xoff
            || (x as i32) >= xoff + width as i32
            || (y as i32) < yoff
            || (y as i32) >= yoff + height as i32
        {
            return None;
        }
        Some((x.wrapping_sub(xoff as u32), y.wrapping_sub(yoff as u32)))
    }

    unsafe fn visible_cursor_in_window(&self) -> Option<(u32, u32)> {
        let pane = &*self.get();
        let screen = &*pane.screen_ptr();
        (screen.mode & crate::src::shared::screen::MODE_CURSOR != 0).then_some((
            (pane.xoff as u32).wrapping_add(screen.cx),
            (pane.yoff as u32).wrapping_add(screen.cy),
        ))
    }

    unsafe fn has_tty(&self) -> bool {
        (*self.get()).fd != -1
    }

    unsafe fn resize(&self, sx: u32, sy: u32) {
        window_pane_resize(self, sx, sy);
    }

    unsafe fn update_focus(&self, focused: bool) {
        let pane = self.get();
        if (*pane).flags & PANE_EXITED != 0 {
            return;
        }
        let was_focused = (*pane).flags & PANE_FOCUSED != 0;
        if focused == was_focused {
            log_debug(format_args!(
                "window_pane_update_focus: %{} focus unchanged",
                self.id()
            ));
            return;
        }
        log_debug(format_args!(
            "window_pane_update_focus: %{} focus {}",
            self.id(),
            if focused { "in" } else { "out" }
        ));
        if (*pane).base.mode & MODE_FOCUSON != 0 {
            let _ = (*pane)
                .event
                .write(if focused { b"\x1b[I" } else { b"\x1b[O" });
        }
        // Callbacks observe the old focus flag, as in the original operation.
        events_fire_pane(
            if focused {
                c"pane-focus-in"
            } else {
                c"pane-focus-out"
            }
            .as_ptr(),
            Rc::clone(self),
        );
        if focused {
            (*pane).flags |= PANE_FOCUSED;
        } else {
            (*pane).flags &= !PANE_FOCUSED;
        }
    }

    unsafe fn on_selected(&self, record_activity: bool) {
        if record_activity {
            (*self.get()).active_point = next_active_point;
            next_active_point = next_active_point.wrapping_add(1);
        }
        (*self.get()).flags |= PANE_CHANGED;
    }

    unsafe fn prepare_render(
        &self,
        context: &mut crate::src::shared::tty::tty_ctx,
        window_redraw: bool,
    ) -> i32 {
        let pane = self.get();
        if (*pane).layout_cell.is_none() {
            return 0;
        }
        if (*pane).flags & (PANE_REDRAW | crate::src::shared::pane::PANE_DROP) != 0 {
            return -1;
        }
        if window_redraw {
            log_debug(format_args!(
                "screen_write_set_client_cb: adding %{} to deferred redraw",
                self.id()
            ));
            self.request_redraw(true);
            return -1;
        }
        context.rxoff = (*pane).xoff;
        context.xoff = context.rxoff;
        context.ryoff = (*pane).yoff;
        context.yoff = context.ryoff;
        1
    }

    unsafe fn request_redraw(&self, scrollbar: bool) {
        (*self.get()).flags |= PANE_REDRAW;
        if scrollbar {
            (*self.get()).flags |= PANE_REDRAWSCROLLBAR;
        }
    }

    unsafe fn key(
        &self,
        client: Option<&ClientRef>,
        link: refbox::Weak<winlink>,
        key: key_code,
        mouse: Option<&mut mouse_event>,
    ) -> i32 {
        window_pane_key(
            self,
            client,
            link,
            key,
            mouse.map_or(std::ptr::null_mut(), |mouse| mouse),
        )
    }

    unsafe fn paste(&self, key: key_code, bytes: &[u8]) {
        window_pane_paste(self, key, bytes);
    }

    unsafe fn set_mode(
        &self,
        source: Option<&Rc<UnsafeCell<window_pane>>>,
        mode: &'static window_mode,
        item: Option<&Rc<UnsafeCell<cmdq_item>>>,
        find: Option<&mut cmd_find_state>,
        arguments: Option<&mut args>,
    ) -> i32 {
        window_pane_set_mode(
            self,
            source,
            mode,
            item,
            find.map_or(std::ptr::null_mut(), |find| find),
            arguments.map_or(std::ptr::null_mut(), |arguments| arguments),
        )
    }

    unsafe fn reset_mode(&self) {
        window_pane_reset_mode(self);
    }
    unsafe fn reset_all_modes(&self) {
        window_pane_reset_mode_all(self);
    }

    unsafe fn copy_screen(
        &self,
        destination: &mut screen,
        x: u32,
        y: u32,
        sx: u32,
        sy: u32,
        displayed: bool,
    ) {
        let pane = self.get();
        let source = if displayed {
            (*pane).screen_ptr()
        } else {
            &raw mut (*pane).base
        };
        assert!(!source.is_null(), "live pane screen");
        assert!(
            !std::ptr::eq(source, destination),
            "screen copy cannot alias its source"
        );
        let source = &*source;
        assert!(x <= source.grid().sx && sx <= source.grid().sx - x);
        assert!(y <= source.grid().sy && sy <= source.grid().sy - y);
        // Inserting into a hyperlink table may evict an older global-history
        // entry, including one in the source. Snapshot link text before adding
        // any destination links, and release each table borrow before insertion.
        let mut cells = Vec::with_capacity(sx as usize * sy as usize);
        let mut links = std::collections::HashMap::new();
        for row in 0..sy {
            for column in 0..sx {
                let mut cell = grid_cell::default();
                grid_get_cell(
                    source.grid(),
                    x + column,
                    source.grid().hsize + y + row,
                    &mut cell,
                );
                if crate::src::screen::screen_check_selection(source, x + column, y + row) != 0 {
                    if let Some(selected) = crate::src::screen::screen_select_cell(source, &cell) {
                        cell = selected;
                    }
                }
                if cell.link != 0 {
                    links.entry(cell.link).or_insert_with(|| {
                        source.hyperlinks.as_ref().and_then(|table| {
                            hyperlinks_get(table, cell.link)
                                .map(|link| (link.uri.clone(), link.internal_id.clone()))
                        })
                    });
                }
                cells.push(cell);
            }
        }
        screen_free(destination);
        screen_init(destination, sx, sy, 0);
        destination.title = source.title.clone();
        destination.path = source.path.clone();
        destination.mode = source.mode;
        destination.cstyle = source.cstyle;
        destination.default_cstyle = source.default_cstyle;
        destination.ccolour = source.ccolour;
        destination.default_ccolour = source.default_ccolour;
        destination.cx = source.cx.saturating_sub(x).min(sx.saturating_sub(1));
        destination.cy = source.cy.saturating_sub(y).min(sy.saturating_sub(1));
        if source.cx < x || source.cy < y || source.cx >= x + sx || source.cy >= y + sy {
            destination.mode &= !crate::src::shared::screen::MODE_CURSOR;
        }
        let links: std::collections::HashMap<_, _> = links
            .into_iter()
            .map(|(id, text)| {
                let copied = text.map_or(0, |(uri, internal_id)| {
                    hyperlinks_put(
                        destination
                            .hyperlinks
                            .as_ref()
                            .expect("copy hyperlink table"),
                        &uri,
                        Some(&internal_id),
                    )
                });
                (id, copied)
            })
            .collect();
        for (index, mut cell) in cells.into_iter().enumerate() {
            if cell.link != 0 {
                cell.link = links[&cell.link];
            }
            grid_set_cell(
                destination.grid_mut(),
                index as u32 % sx,
                index as u32 / sx,
                &cell,
            );
        }
    }

    unsafe fn copy_output(&self, offset: &window_pane_offset, destination: &mut [u8]) -> usize {
        let pane = &*self.get();
        let used = offset.used.wrapping_sub(pane.base_offset);
        match pane.event.copy_input(used, destination) {
            Ok(copied) => copied,
            Err(crate::src::reactor::StreamError::Freed) => 0,
            Err(crate::src::reactor::StreamError::InvalidRange) => {
                panic!("pane offset is within input buffer")
            }
        }
    }

    unsafe fn output_offset(&self) -> window_pane_offset {
        (*self.get()).offset
    }

    unsafe fn advance_output(&self, offset: &mut window_pane_offset, count: usize) {
        let pane = &*self.get();
        if let Ok(available) = pane.event.input_len() {
            let used = offset.used.wrapping_sub(pane.base_offset);
            offset.used = offset
                .used
                .wrapping_add(count.min(available.saturating_sub(used)));
        }
    }

    unsafe fn finish_cycle(&self) {
        if (*self.get()).fd != -1 {
            finish_resize(self);
            finish_buffer(self);
        }
        (*self.get()).flags &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_ACTIVITY);
    }

    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R {
        edit(
            (*self.get())
                .options
                .as_deref_mut()
                .expect("live pane options"),
        )
    }

    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue> {
        super::format::format_value(self, key, context)
    }

    unsafe fn pipe(
        &self,
        command: refbox::Weak<cmd>,
        item: &Rc<UnsafeCell<cmdq_item>>,
    ) -> cmd_retval {
        process::pipe_pane(self, command, item)
    }
    unsafe fn parse_input(&self) {
        super::input::input_parse_pane(self)
    }
    unsafe fn parse_output(&self, bytes: &[u8]) {
        super::input::input_parse_buffer(self, bytes.as_ptr(), bytes.len())
    }
    unsafe fn process_id(&self) -> pid_t {
        (*self.get()).pid
    }
    unsafe fn process_exited(&self, pid: pid_t, status: i32) -> bool {
        lifecycle::process_exited(self, pid, status)
    }
    unsafe fn kill_process(&self) {
        lifecycle::kill_process(self)
    }
    unsafe fn finish_process(&self, notify: i32) {
        lifecycle::finish_process(self, notify)
    }
    unsafe fn spawn_process(
        context: *mut crate::src::shared::spawn::spawn_context,
        cause: *mut Option<CString>,
    ) -> Option<Self> {
        spawning::spawn_pane(context, cause)
    }
    unsafe fn install_editor(
        &self,
        editor: Box<crate::src::shared::spawn::spawn_editor_state>,
    ) -> crate::src::shared::spawn::EditorHandle {
        spawning::install_editor(self, editor)
    }
    unsafe fn finish_editing(&self) {
        spawning::finish_editing(self)
    }
    unsafe fn editor_identity(&self) -> Option<crate::src::shared::spawn::EditorId> {
        (*self.get()).editor.as_ref().map(|editor| editor.id)
    }
    unsafe fn cancel_editor(&self, identity: crate::src::shared::spawn::EditorId) {
        let pane = &mut *self.get();
        if let Some(editor) = pane.editor.as_mut().filter(|editor| editor.id == identity) {
            editor.cb = None;
        }
    }
    unsafe fn editor_process_id(&self, identity: crate::src::shared::spawn::EditorId) -> pid_t {
        (*self.get())
            .editor
            .as_ref()
            .filter(|editor| editor.id == identity)
            .map_or(-1, |editor| editor.pid)
    }
    unsafe fn write_key(&self, key: key_code, mouse: Option<&mouse_event>) -> i32 {
        super::keys::write_key(self, key, mouse)
    }
    unsafe fn mode_entry(&self) -> refbox::Weak<window_mode_entry> {
        (*self.get()).active_mode_entry()
    }
    unsafe fn is_mode(&self, mode: &'static window_mode) -> bool {
        let entry = self.mode_entry();
        entry.is_alive() && std::ptr::eq(entry.get_unchecked().mode, mode)
    }
    unsafe fn add_mode_formats(&self, context: &mut crate::src::shared::format::format_tree) {
        let entry = self.mode_entry();
        let callback = entry.is_alive().then(|| entry.get_unchecked().mode.formats).flatten();
        if let Some(callback) = callback {
            callback(entry, context);
        }
    }
    unsafe fn reset_input(&self) {
        let context = {
            let pane = &mut *self.get();
            crate::src::style::colour::colour_palette_clear(Some(&mut pane.palette));
            pane.ictx.as_deref_mut().expect("pane input context") as *mut crate::src::shared::input::input_ctx
        };
        super::input::input_reset(context, 1);
        (*self.get()).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED | PANE_REDRAW;
    }
    unsafe fn destroy_ready(&self) -> bool {
        window_pane_destroy_ready(self) != 0
    }
    unsafe fn destroy(&self) {
        window_pane_destroy(self);
    }
}

unsafe fn finish_resize(owner: &Rc<UnsafeCell<window_pane>>) {
    let pane = owner.get();
    if (*pane).resize_queue.is_empty() {
        return;
    }
    if !(*pane).resize_timer.is_initialized() {
        let observer = Rc::downgrade(owner);
        (*pane).resize_timer.set(move || {
            if let Some(owner) = observer.upgrade() {
                log_debug(format_args!(
                    "server_client_resize_timer: %{} resize timer expired",
                    owner.id()
                ));
                (*owner.get()).resize_timer.cancel();
            }
        });
    }
    if (*pane).resize_timer.is_pending() {
        return;
    }
    log_debug(format_args!(
        "server_client_check_pane_resize: %{} needs to be resized",
        owner.id()
    ));
    let (sx, sy, keep) = {
        let queue = &(*pane).resize_queue;
        for resize in queue {
            log_debug(format_args!(
                "queued resize: {}x{} -> {}x{}",
                resize.osx, resize.osy, resize.sx, resize.sy
            ));
        }
        let first = queue.front().expect("nonempty resize queue");
        let last = queue.back().expect("nonempty resize queue");
        if queue.len() == 1 || last.sx != first.osx || last.sy != first.osy {
            (last.sx, last.sy, std::ptr::null_mut())
        } else {
            let previous = &queue[queue.len() - 2];
            (
                previous.sx,
                previous.sy,
                (&**last as *const window_pane_resize).cast_mut(),
            )
        }
    };
    window_pane_send_resize(&*pane, sx, sy);
    (*pane).clear_resizes_except(keep);
    let delay = Duration::from_micros((if keep.is_null() { 250000 } else { 10000 }) as u64);
    (*pane).resize_timer.arm(delay).expect("arm timer");
}

unsafe fn finish_buffer(owner: &Rc<UnsafeCell<window_pane>>) {
    let pane = owner.get();
    let mut minimum = (*pane).offset.used;
    if (*pane).pipe_fd != -1 {
        minimum = minimum.min((*pane).pipe_offset.used);
    }
    let mut off = true;
    let mut attached = false;
    let mut cursor = clients.first();
    while let Some(client) = cursor {
        if client.attached_session().upgrade().is_some() {
            attached = true;
            if !client.is_control() {
                off = false;
            } else {
                let (offset, blocked) = client
                    .with_output_offset(owner.id(), |offset, blocked| (offset.copied(), blocked));
                if !blocked {
                    off = false;
                }
                if let Some(offset) = offset {
                    minimum = minimum.min(offset.used);
                }
            }
        }
        cursor = clients.next(&client);
    }
    if !attached {
        off = false;
    }
    let count = minimum.wrapping_sub((*pane).base_offset);
    if count != 0 {
        log_debug(format_args!(
            "server_client_check_pane_buffer: %{} has {} minimum (of {}) bytes used",
            owner.id(),
            count,
            (*pane).event.input_len().unwrap_or(0)
        ));
        let _ = (*pane).event.drain_input(count);
        if (*pane).base_offset > usize::MAX.wrapping_sub(count) {
            let base = (*pane).base_offset;
            (*pane).offset.used = (*pane).offset.used.wrapping_sub(base);
            if (*pane).pipe_fd != -1 {
                (*pane).pipe_offset.used = (*pane).pipe_offset.used.wrapping_sub(base);
            }
            let mut cursor = clients.first();
            while let Some(client) = cursor {
                if client.attached_session().upgrade().is_some() && client.is_control() {
                    client.with_output_offset(owner.id(), |offset, blocked| {
                        if !blocked {
                            if let Some(offset) = offset {
                                offset.used = offset.used.wrapping_sub(base);
                            }
                        }
                    });
                }
                cursor = clients.next(&client);
            }
            (*pane).base_offset = count;
        } else {
            (*pane).base_offset = (*pane).base_offset.wrapping_add(count);
        }
    }
    log_debug(format_args!(
        "server_client_check_pane_buffer: pane %{} is {}",
        owner.id(),
        if off { "off" } else { "on" }
    ));
    if off {
        let _ = (*pane).event.disable(Interests::READ);
    } else {
        let _ = (*pane).event.enable(Interests::READ);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reparent_changes_inheritance_without_retaining_either_window() {
        use crate::src::options::{
            options_create, options_get_string, options_set_string, OptionsScope,
        };
        use crate::src::window::Window;
        unsafe {
            let first = window::with_options_for_test();
            let second = window::with_options_for_test();
            for (owner, value) in [(&first, c"first"), (&second, c"second")] {
                owner.with_options_mut(|table| {
                    options_set_string(table, c"@parent".as_ptr(), 0, |out| {
                        out.write_all(value.to_bytes())
                    });
                });
            }
            let pane = window_pane::new();
            (*pane.get()).options = Some(options_create(None));
            pane.reparent(&first);
            let scope = OptionsScope::Pane(Rc::downgrade(&pane));
            let original = scope.resolve(c"@parent", false).expect("inherited option");
            (*pane.get()).flags &= !(PANE_STYLECHANGED | PANE_THEMECHANGED);
            pane.reparent(&second);
            assert!(pane.window_observer().ptr_eq(&Rc::downgrade(&second)));
            assert_eq!(
                (*pane.get()).flags & (PANE_STYLECHANGED | PANE_THEMECHANGED),
                PANE_STYLECHANGED | PANE_THEMECHANGED
            );
            assert_eq!(
                pane.with_options_mut(|table| options_get_string(table, c"@parent".as_ptr()))
                    .as_c_str(),
                c"second"
            );
            // Work already in flight on the old inherited value keeps that
            // table's identity, while the next lookup follows the new parent.
            assert_eq!(
                original
                    .with_entry(c"@parent", |entry| entry
                        .value
                        .string_ptr()
                        .unwrap()
                        .to_owned())
                    .unwrap()
                    .as_c_str(),
                c"first"
            );
            assert_eq!(Rc::strong_count(&first), 1);
            assert_eq!(Rc::strong_count(&second), 1);
            first.release(c"option inheritance test");
            second.release(c"option inheritance test");
            // The pane has no screen/event resources in this fixture; tear down
            // its option table explicitly, matching the normal pane destructor.
            drop((*pane.get()).options.take());
            window_pane_remove_ref(pane, c"option inheritance test".as_ptr());
        }
    }
    use crate::src::grid::grid_create;
    use crate::src::reactor::{evbuffer_add, shutdown_runtime, StreamHandle};

    unsafe fn buffered_pane(bytes: &[u8]) -> Rc<UnsafeCell<window_pane>> {
        let pane = window_pane::new();
        (*pane.get()).fd = -1;
        (*pane.get()).pipe_fd = -1;
        let stream = bufferevent_new(-1, None, None, None);
        (*pane.get()).event = StreamHandle::from_ptr(stream);
        evbuffer_add(&mut (*stream).input, bytes.as_ptr().cast(), bytes.len());
        pane
    }

    #[test]
    fn visibility_probe_preserves_pending_redraws_and_layout() {
        unsafe {
            let window = crate::src::window::test_support::zoomed_window();
            let pane = window.active_pane().unwrap();
            let layout = (*pane.get()).layout_cell;
            for flags in [0, PANE_REDRAW, crate::src::shared::pane::PANE_DROP] {
                (*pane.get()).flags = flags;
                assert_eq!(window_pane_is_visible(&pane), 1);
                assert_eq!((*pane.get()).flags, flags);
                assert_eq!((*pane.get()).layout_cell, layout);
            }
            (*pane.get()).layout_cell = None;
            assert_eq!(window_pane_is_visible(&pane), 0);
            crate::src::window::test_support::set_zoomed(&window, false);
            assert_eq!(window_pane_is_visible(&pane), 1);
            assert_eq!((*pane.get()).flags, crate::src::shared::pane::PANE_DROP);
            crate::src::window::test_support::set_zoomed(&window, true);
            (*pane.get()).layout_cell = layout;
            window.release(c"visibility probe test");
        }
    }

    #[test]
    fn fallback_selection_preserves_the_last_activity_point() {
        unsafe {
            let pane = window_pane::new();
            (*pane.get()).fd = -1;
            (*pane.get()).pipe_fd = -1;
            let previous = next_active_point;
            pane.on_selected(true);
            assert_eq!((*pane.get()).active_point, previous);
            assert_eq!(next_active_point, previous.wrapping_add(1));
            (*pane.get()).flags &= !PANE_CHANGED;
            pane.on_selected(false);
            assert_eq!((*pane.get()).active_point, previous);
            assert_eq!(next_active_point, previous.wrapping_add(1));
            assert_ne!((*pane.get()).flags & PANE_CHANGED, 0);
            next_active_point = previous;
            drop(pane);
        }
    }

    #[test]
    fn output_cursors_preserve_unread_bytes_and_clamp_skips() {
        unsafe {
            let pane = buffered_pane(b"abcdef");
            (*pane.get()).base_offset = usize::MAX - 2;
            let mut first = window_pane_offset {
                used: usize::MAX - 2,
            };
            let second = first;
            let mut bytes = [0; 3];
            assert_eq!(pane.copy_output(&first, &mut bytes), 3);
            assert_eq!(&bytes, b"abc");
            pane.advance_output(&mut first, 3);
            assert_eq!(first.used, 0);
            assert_eq!(pane.copy_output(&first, &mut bytes), 3);
            assert_eq!(&bytes, b"def");
            assert_eq!(pane.copy_output(&second, &mut bytes), 3);
            assert_eq!(&bytes, b"abc");
            pane.advance_output(&mut first, usize::MAX);
            assert_eq!(first.used, 3);
            assert_eq!(pane.copy_output(&first, &mut bytes), 0);
            std::mem::take(&mut (*pane.get()).event).free();
            assert_eq!(pane.copy_output(&first, &mut bytes), 0);
            pane.advance_output(&mut first, usize::MAX);
            assert_eq!(first.used, 3);
            drop(pane);
            shutdown_runtime();
        }
    }

    #[test]
    fn copied_screen_and_hyperlinks_survive_source_changes() {
        unsafe {
            let mut options = crate::src::options::options_create_owned(None);
            let definition = crate::src::options_table::options_table
                .iter()
                .find(|entry| CStr::from_ptr(entry.name_ptr()) == c"extended-keys")
                .unwrap();
            crate::src::options::options_default(&mut *options, definition);
            let previous_options = global_options;
            global_options = &mut *options;
            let pane = window_pane::new();
            (*pane.get()).fd = -1;
            (*pane.get()).pipe_fd = -1;
            let source = &mut (*pane.get()).base;
            source.grid = Some(grid_create(3, 2, 0));
            source.hyperlinks = Some(crate::src::hyperlinks::hyperlinks_init());
            source.mode = crate::src::shared::screen::MODE_CURSOR;
            source.cx = 2;
            source.cy = 1;
            let mut cell = grid_cell::default();
            crate::src::text::utf8::utf8_set(&mut cell.data, b'X');
            cell.link = hyperlinks_put(
                source.hyperlinks.as_ref().unwrap(),
                c"https://example.test/pane",
                Some(c"pane-link"),
            );
            grid_set_cell(source.grid_mut(), 1, 1, &cell);
            let mut copied = screen::empty();
            pane.copy_screen(&mut copied, 1, 1, 2, 1, true);
            assert_eq!(
                (copied.grid().sx, copied.grid().sy, copied.cx, copied.cy),
                (2, 1, 1, 0)
            );
            grid_set_cell((*pane.get()).base.grid_mut(), 1, 1, &grid_cell::default());
            crate::src::hyperlinks::hyperlinks_reset(
                (*pane.get()).base.hyperlinks.as_ref().unwrap(),
            );
            drop(pane);
            let mut copied_cell = grid_cell::default();
            grid_get_cell(copied.grid(), 0, 0, &mut copied_cell);
            assert_eq!(copied_cell.data.data[0], b'X');
            assert_eq!(
                hyperlinks_get(copied.hyperlinks.as_ref().unwrap(), copied_cell.link)
                    .unwrap()
                    .uri
                    .as_c_str(),
                c"https://example.test/pane"
            );
            screen_free(&mut copied);
            global_options = previous_options;
            drop(options);
        }
    }

    #[test]
    fn destroy_closes_streams_before_the_last_holder_is_dropped() {
        unsafe {
            let pane = buffered_pane(b"pending");
            let stream = (*pane.get()).event.clone();
            let observer = Rc::downgrade(&pane);
            window_pane_tree_insert(&mut all_window_panes, Rc::clone(&pane));
            pane.destroy();
            assert!(!stream.is_alive());
            assert!(window_pane_upgrade(&observer).is_none());
            assert!(observer.upgrade().is_some());
            drop(pane);
            assert!(observer.upgrade().is_none());
            shutdown_runtime();
        }
    }
}
