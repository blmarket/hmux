//! Operations on an opaque, retained window.
//!
//! The holder preserves allocation lifetime, not exclusive access. Callers must
//! preserve the server's single-threaded lifecycle and callback ordering. Scoped
//! option callbacks must not reenter the window, free/reparent the component, or
//! let references or pointers escape.

use super::*;
use crate::src::layout::custom::{layout_dump_owned, layout_parse};
use crate::src::layout::layout_resize;
use crate::src::layout::set::{
    layout_set_lookup, layout_set_next, layout_set_previous, layout_set_select,
};
use crate::src::layout::{layout_get_floating_cell, layout_get_tiled_cell, layout_spread_out};
use crate::src::resize::recalculate_sizes;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::window::{WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_RESIZE};
use crate::src::spawn::spawn_pane;

pub trait Window {
    unsafe fn id(&self) -> u32;
    /// Registry successor. The returned owner retains the explicit release duty.
    unsafe fn next_window(&self) -> Option<Rc<UnsafeCell<window>>>;
    /// Live association traversal; callbacks may change the list between calls.
    unsafe fn next_winlink(&self, after: Option<refbox::Weak<winlink>>) -> refbox::Weak<winlink>;
    /// Only alert bits, never the window's other bookkeeping flags.
    unsafe fn pending_alerts(&self) -> i32;
    unsafe fn reset_alert_timer(&self);
    /// Record flags and claim queue membership. Some(true) transfers one release duty
    /// to the alert queue. None means disabled; Some(false) means already queued.
    unsafe fn queue_alerts(&self, flags: i32) -> Option<bool>;
    /// Called after delivery, while the queue still retains this window.
    unsafe fn finish_alerts(&self);

    unsafe fn name(&self) -> CString;
    unsafe fn rename(&self, name: &CStr, untrusted: bool);
    unsafe fn active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>>;
    /// `None` starts traversal; `Some` continues after that pane, without wrapping.
    unsafe fn next_pane(
        &self,
        after: Option<&Rc<UnsafeCell<window_pane>>>,
    ) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn select_pane(&self, pane: &Rc<UnsafeCell<window_pane>>, notify: bool) -> i32;
    unsafe fn remove_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Allocate the tiled/floating layout internally, then spawn into it. The
    /// context supplies the command, source pane, session/link, and spawn flags.
    /// Its legacy `lc` field must be null on entry and is null on return.
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
    /// Pixel dimensions of a terminal cell, for the pane's PTY resize protocol.
    unsafe fn cell_size(&self) -> (u32, u32);
    unsafe fn is_zoomed(&self) -> bool;
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32);
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
    unsafe fn set_latest_client(&self, client: Option<&Rc<UnsafeCell<client>>>) -> bool;
    /// Window-owned modal/menu/selection policy used when a pane's focus changes.
    unsafe fn pane_is_focused(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool;
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
}

impl Window for Rc<UnsafeCell<window>> {
    unsafe fn next_window(&self) -> Option<Rc<UnsafeCell<window>>> {
        windows_next(&*self.get())
    }
    unsafe fn next_winlink(&self, after: Option<refbox::Weak<winlink>>) -> refbox::Weak<winlink> {
        if let Some(after) = after {
            window_winlinks_next(Some(&*self.get()), after)
        } else {
            window_winlinks_first(Some(&*self.get()))
        }
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

    unsafe fn id(&self) -> u32 {
        (*self.get()).id
    }
    unsafe fn name(&self) -> CString {
        (*self.get()).name.clone()
    }
    unsafe fn rename(&self, name: &CStr, untrusted: bool) {
        window_set_name(self, name.as_ptr(), untrusted as i32);
    }
    unsafe fn active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>> {
        (*self.get()).active_pane()
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
            context.lc.is_null(),
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
        context.lc = cell;
        let mut cause = None;
        let result = spawn_pane(context, &mut cause);
        context.lc = std::ptr::null_mut();
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
    unsafe fn cell_size(&self) -> (u32, u32) {
        ((*self.get()).xpixel, (*self.get()).ypixel)
    }
    unsafe fn is_zoomed(&self) -> bool {
        (*self.get()).flags & WINDOW_ZOOMED != 0
    }
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32) {
        resize_window(self, sx, sy, xpixel, ypixel);
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
            server_redraw_window(&*self.get());
        }
        let flags = if legacy_format {
            LAYOUT_CUSTOM_OLD_FORMAT
        } else {
            0
        };
        let new_layout = layout_dump_owned(
            (*self.get())
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root),
            flags,
        );
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
            let requested = name.or_else(|| {
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
        server_redraw_window(&*self.get());
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
    unsafe fn set_latest_client(&self, client: Option<&Rc<UnsafeCell<client>>>) -> bool {
        let latest = client.map_or_else(Weak::new, Rc::downgrade);
        if (*self.get()).latest.ptr_eq(&latest) {
            return false;
        }
        (*self.get()).latest = latest;
        true
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
}

unsafe fn resize_fire_window_resized(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut old_sx: u_int,
    mut old_sy: u_int,
) {
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
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    event_payload_set_uint(
        &mut *ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sx,
    );
    event_payload_set_uint(
        &mut *ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sy,
    );
    event_payload_set_uint(
        &mut *ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        &mut *ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"window-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe fn resize_window(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
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
        ((*w).id) as u32,
        (sx) as u32,
        (sy) as u32,
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
    server_redraw_window(&*(w));
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
    use crate::src::events::{events_add_sink, events_remove_sink};
    use crate::src::shared::events::events_callback;
    use std::cell::Cell;

    #[test]
    fn split_reports_failure_before_restoring_saved_zoom() {
        unsafe {
            let window = super::super::zoom_teardown_tests::zoomed_window();
            let pane = window.active_pane().unwrap();
            window_push_zoom(&window, 1, 1);
            assert!(!window.is_zoomed());
            // A zero-width window cannot accept default floating geometry.
            (*window.get()).sx = 0;
            let item = crate::src::cmd::queue::cmdq_get_callback_owned(
                c"split test",
                Some(Box::new(|_| crate::src::shared::command::CMD_RETURN_NORMAL)),
            );
            let link = refbox::RefBox::new(winlink {
                window_owner: Some(window.clone()),
                ..Default::default()
            });
            let mut context = spawn_context {
                item: Rc::downgrade(&item),
                s: Weak::new(),
                wl: link.downgrade(),
                tc: Weak::new(),
                wp0: Rc::downgrade(&pane),
                lc: std::ptr::null_mut(),
                name: None,
                argv: Vec::new(),
                environ: None,
                idx: -1,
                cwd: None,
                flags: SPAWN_FLOATING,
            };
            let mut arguments = args::empty();
            let mut calls = 0;
            let error = window
                .split_pane(
                    &mut context,
                    &mut arguments,
                    PANE_LINES_SINGLE,
                    true,
                    |error| {
                        calls += 1;
                        assert_eq!(error, c"invalid width");
                        assert!(!window.is_zoomed());
                        assert_ne!(
                            (*window.get()).flags & WINDOW_WASZOOMED,
                            0,
                            "the saved zoom is still pending while reporting"
                        );
                        // Reenter through another trait method during reporting.
                        assert!(window.active_pane().is_some());
                    },
                )
                .err()
                .expect("invalid floating geometry");
            assert_eq!(calls, 1);
            assert_eq!(error, c"invalid width".to_owned());
            assert_eq!((*window.get()).flags & WINDOW_WASZOOMED, 0);
            assert!(context.lc.is_null());
            crate::src::cmd::queue::cmdq_append(None, item);
            crate::src::cmd::queue::cmdq_next(None);
            drop(link);
            window.release(c"split rollback test");
        }
    }

    #[test]
    fn failed_layout_selection_restores_saved_layout_but_keeps_unzoom() {
        unsafe {
            let window = super::super::zoom_teardown_tests::zoomed_window();
            window.with_options_mut(|options| {
                let entry = crate::src::options_table::options_table
                    .iter()
                    .find(|entry| entry.name == Some(c"pane-base-index"))
                    .unwrap();
                crate::src::options::options_default(options, entry);
            });
            (*window.get()).old_layout = Some(c"previous-layout".to_owned());
            let notifications = Rc::new(Cell::new(0));
            let count = notifications.clone();
            let sink = events_add_sink(
                c"window-layout-changed",
                events_callback(move |_, _| {
                    count.set(count.get() + 1);
                }),
            );
            let error = window
                .select_layout(Some(c"not-a-layout"), false, 0, None, false)
                .unwrap_err();
            assert!(error.to_bytes().ends_with(b": not-a-layout"));
            assert_eq!(
                (*window.get()).old_layout.as_deref(),
                Some(c"previous-layout")
            );
            assert!(!window.is_zoomed(), "the command unzooms before parsing");
            assert_eq!(
                notifications.get(),
                1,
                "only unzoom notifies after failed parsing"
            );
            events_remove_sink(sink);
            window.release(c"failed layout selection test");
        }
    }

    #[test]
    fn layout_cycle_precedes_restore_and_uses_requested_serialization() {
        unsafe {
            let window = super::super::zoom_teardown_tests::zoomed_window();
            window.with_options_mut(|options| {
                let entry = crate::src::options_table::options_table
                    .iter()
                    .find(|entry| entry.name == Some(c"pane-base-index"))
                    .unwrap();
                crate::src::options::options_default(options, entry);
            });
            (*window.get()).lastlayout = -1;
            (*window.get()).old_layout = Some(c"invalid-previous-layout".to_owned());
            // A cycle ignores a name and -o, even if restoring that name would fail.
            window
                .select_layout(Some(c"invalid"), true, 1, None, true)
                .unwrap();
            assert_eq!((*window.get()).lastlayout, 0);
            let old = (*window.get()).old_layout.as_deref().unwrap();
            assert!(
                !old.to_bytes().starts_with(b"{"),
                "legacy layouts have a checksum prefix"
            );
            window
                .select_layout(Some(c"invalid"), true, -1, None, false)
                .unwrap();
            assert_eq!(
                (*window.get()).lastlayout,
                6,
                "previous wraps around presets"
            );
            assert!((*window.get())
                .old_layout
                .as_deref()
                .unwrap()
                .to_bytes()
                .starts_with(b"{"));
            window.release(c"layout cycle test");
        }
    }
}
