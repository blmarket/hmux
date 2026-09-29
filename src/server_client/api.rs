//! Operations on retained clients. Legacy dispatch remains explicitly unsafe:
//! an Rc keeps storage alive, but does not make reentrant model borrows exclusive.

use super::*;
use crate::src::control::{control_get_window_size, control_write_output};
use crate::src::session::Session;
use crate::src::shared::environment::environ;
use crate::src::shared::prompt::{prompt_free_cb, prompt_type};
use crate::src::shared::status::status_prompt_input_cb;
use crate::src::window::Window;
use std::any::Any;
use std::cell::UnsafeCell;
use std::rc::{Rc, Weak};

/// The caller serializes access on the server thread and releases model and
/// component borrows before callbacks. Logical client loss remains explicit;
/// existing deferred-owner release sites must keep their deferred release duty.
pub trait Client {
    unsafe fn attached_session(&self) -> Weak<UnsafeCell<session>>;
    unsafe fn set_session(&self, session: Option<&Rc<UnsafeCell<session>>>);
    unsafe fn reattach_after_session_destroy(&self, target: Option<&Rc<UnsafeCell<session>>>);
    unsafe fn is_dead(&self) -> bool;
    unsafe fn is_control(&self) -> bool;
    /// Deliver the audible/visual part after Session has deduplicated the alert.
    unsafe fn alert(&self, kind: &CStr, visual: i32, current: bool, index: i32);
    unsafe fn uses_legacy_layout_format(&self) -> bool;
    unsafe fn focuses_window(&self, window: &Rc<UnsafeCell<window>>) -> bool;
    unsafe fn participates_in_window_sizing(&self) -> bool;
    unsafe fn window_size(&self, window: Option<&Rc<UnsafeCell<window>>>) -> (u32, u32, u32, u32);
    unsafe fn constrain_window_size(
        &self,
        window: &Rc<UnsafeCell<window>>,
        sx: &mut u32,
        sy: &mut u32,
    );
    /// `flags` contains only CLIENT_*REDRAW* bits, including status-force.
    unsafe fn request_redraw(&self, flags: u64);
    /// Ordered injection used by send-keys -K. The queue insertion point must
    /// survive until dispatch finishes, under the existing command lifetime rules.
    unsafe fn handle_key_after(
        &self,
        event: Box<key_event>,
        after: Option<&Rc<UnsafeCell<cmdq_item>>>,
        next: Option<&mut Weak<UnsafeCell<cmdq_item>>>,
    ) -> i32;
    unsafe fn print(&self, parse: bool, buffer: &mut SegmentedBuf);
    unsafe fn control_write_output(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Scoped access to this control client's consumer offset. Call only for an
    /// attached control client; the bool is the existing output-disabled flag.
    /// Do not reenter models, release owners, or let the reference escape.
    unsafe fn with_output_offset<R>(
        &self,
        pane: u32,
        read: impl FnOnce(Option<&mut window_pane_offset>, bool) -> R,
    ) -> R;
    unsafe fn set_return_value(&self, value: i32);
    unsafe fn request_exit(&self, value: i32);
    /// The closure cannot reenter models, destroy owners, or leak component
    /// references. Clone the environment before invoking another entity.
    unsafe fn with_environment<R>(&self, read: impl FnOnce(Option<&environ>) -> R) -> R;
    unsafe fn cwd(&self, fallback: Option<&Rc<UnsafeCell<session>>>) -> Option<CString>;
    unsafe fn set_key_table(&self, name: Option<&CStr>);
    unsafe fn set_prompt(
        &self,
        find: Option<&cmd_find_state>,
        message: &CStr,
        input: Option<&CStr>,
        inputcb: status_prompt_input_cb,
        freecb: prompt_free_cb,
        flags: i32,
        kind: prompt_type,
    );
    /// Callbacks receive the retained holder, never a live client field borrow.
    /// The legacy overlay implementation continues to own callback retirement.
    unsafe fn set_overlay(
        &self,
        check: overlay_check_cb,
        mode: overlay_mode_cb,
        draw: overlay_draw_cb,
        key: overlay_key_cb,
        free: overlay_free_cb,
        resize: overlay_resize_cb,
        data: Box<dyn Any>,
    );
    unsafe fn clear_overlay(&self);
    /// Read the installed caller-owned payload without leaking references or
    /// calling back into models. Returned owned handles may be used afterwards.
    unsafe fn with_overlay_data<R>(&self, read: impl FnOnce(Option<&dyn Any>) -> R) -> R;
    /// Temporarily remove clipping for an overlay's own output. Restore only if
    /// callbacks did not retire/replace the overlay during `draw`; no model
    /// borrow spans that closure. The supplied callback replaces the old one.
    unsafe fn with_overlay_check_disabled<R>(
        &self,
        restore: overlay_check_cb,
        draw: impl FnOnce() -> R,
    ) -> R;
    unsafe fn terminal_size(&self) -> (u32, u32);
    unsafe fn refresh_terminal_size(&self);
    unsafe fn draw_overlay_screen(
        &self,
        screen: &screen,
        x: u32,
        y: u32,
        sx: u32,
        sy: u32,
        style: &tty_style_ctx,
    );
    /// Prepare direct overlay output, deferring it when a full overlay redraw
    /// is already pending. Coordinates refer to the complete terminal.
    unsafe fn prepare_overlay_render(&self, context: &mut tty_ctx, x: u32, y: u32) -> bool;
    /// Select this client's view of a pane for a terminal command: 0 skips it,
    /// -1 defers to full redraw, and 1 permits immediate output.
    unsafe fn prepare_pane_render(
        &self,
        context: &mut tty_ctx,
        pane: &Rc<UnsafeCell<window_pane>>,
    ) -> i32;
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    unsafe fn detach(&self, message: msgtype);
    unsafe fn suspend(&self);
    unsafe fn lost(&self);
}

impl Client for Rc<UnsafeCell<client>> {
    unsafe fn alert(&self, kind: &CStr, visual: i32, current: bool, index: i32) {
        use crate::src::shared::alerts::{VISUAL_BOTH, VISUAL_OFF};
        if visual == VISUAL_OFF || visual == VISUAL_BOTH {
            crate::src::tty::tty_putcode(
                &raw mut (*self.get()).tty,
                crate::src::shared::tty::TTYC_BEL,
            );
        }
        if visual != VISUAL_OFF {
            // No client field reference survives status callbacks.
            crate::src::status::status_message_set(Some(self), -1, 1, 0, 0, |out| {
                out.write_all(kind.to_bytes())?;
                if current {
                    out.write_all(b" in current window")
                } else {
                    write!(out, " in window {}", index)
                }
            });
        }
    }

    unsafe fn attached_session(&self) -> Weak<UnsafeCell<session>> {
        (*self.get()).session.clone()
    }

    unsafe fn set_session(&self, session: Option<&Rc<UnsafeCell<session>>>) {
        server_client_set_session(self, session);
    }

    unsafe fn reattach_after_session_destroy(&self, target: Option<&Rc<UnsafeCell<session>>>) {
        // A normal set_session(None) has observable focus/socket effects here.
        (*self.get()).session = Weak::new();
        (*self.get()).last_session = Weak::new();
        self.set_session(target);
        if target.is_none() {
            (*self.get()).flags |= CLIENT_EXIT as u64;
        }
    }

    unsafe fn is_dead(&self) -> bool {
        (*self.get()).flags & CLIENT_DEAD as u64 != 0
    }

    unsafe fn is_control(&self) -> bool {
        (*self.get()).flags & CLIENT_CONTROL as u64 != 0
    }

    unsafe fn uses_legacy_layout_format(&self) -> bool {
        self.is_control() && (*self.get()).flags & CLIENT_CONTROL_NEWLAYOUTS == 0
    }

    unsafe fn focuses_window(&self, window: &Rc<UnsafeCell<window>>) -> bool {
        let flags = (*self.get()).flags;
        if flags & CLIENT_FOCUSED as u64 == 0 || (*self.get()).overlay_draw.is_some() {
            return false;
        }
        let Some(session) = self.attached_session().upgrade() else {
            return false;
        };
        if !session.is_attached() {
            return false;
        }
        let link = session.current_winlink();
        link.get_unchecked()
            .window_handle()
            .is_some_and(|owner| Rc::ptr_eq(&owner, window))
    }

    unsafe fn participates_in_window_sizing(&self) -> bool {
        let client = &*self.get();
        if client.session.upgrade().is_none() || client.flags & CLIENT_NOSIZEFLAGS as u64 != 0 {
            return false;
        }
        if client.flags & CLIENT_IGNORESIZE as u64 != 0 {
            let mut next = clients.first();
            while let Some(owner) = next {
                next = clients.next(&owner);
                let other = &*owner.get();
                if other.session.upgrade().is_some()
                    && other.flags & (CLIENT_NOSIZEFLAGS | CLIENT_IGNORESIZE) as u64 == 0
                {
                    return false;
                }
            }
        }
        client.flags & CLIENT_CONTROL as u64 == 0
            || client.flags & (CLIENT_SIZECHANGED as u64 | CLIENT_WINDOWSIZECHANGED) != 0
    }

    unsafe fn window_size(&self, window: Option<&Rc<UnsafeCell<window>>>) -> (u32, u32, u32, u32) {
        let (mut sx, mut sy) = (0, 0);
        let overridden = window.is_some_and(|window| {
            control_get_window_size(&*self.get(), window.id(), &mut sx, &mut sy) != 0
                && sx != 0
                && sy != 0
        });
        if !overridden {
            let status_lines =
                if (*self.get()).flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as u64 != 0 {
                    0
                } else if let Some(session) = self.attached_session().upgrade() {
                    session.status_layout().1
                } else {
                    options_get_number(global_s_options, c"status".as_ptr()) as u32
                };
            sx = (*self.get()).tty.sx;
            sy = (*self.get()).tty.sy.wrapping_sub(status_lines);
        }
        (sx, sy, (*self.get()).tty.xpixel, (*self.get()).tty.ypixel)
    }

    unsafe fn constrain_window_size(
        &self,
        window: &Rc<UnsafeCell<window>>,
        sx: &mut u32,
        sy: &mut u32,
    ) {
        if (*self.get()).flags & CLIENT_WINDOWSIZECHANGED == 0 {
            return;
        }
        let (mut cx, mut cy) = (0, 0);
        if control_get_window_size(&*self.get(), window.id(), &mut cx, &mut cy) != 0 {
            log_debug(format_args!(
                "clients_calculate_size: {} size for @{} is {}x{}",
                log_cstr(
                    (*self.get())
                        .name
                        .as_ref()
                        .map_or(std::ptr::null(), |name| name.as_ptr())
                ),
                window.id(),
                cx,
                cy,
            ));
            if cx != 0 && *sx > cx {
                *sx = cx;
            }
            if cy != 0 && *sy > cy {
                *sy = cy;
            }
        }
    }

    unsafe fn request_redraw(&self, flags: u64) {
        assert_eq!(
            flags
                & !(CLIENT_ALLREDRAWFLAGS as u64
                    | CLIENT_REDRAWSCROLLBARS
                    | CLIENT_STATUSFORCE as u64),
            0,
            "only redraw flags may be requested"
        );
        (*self.get()).flags |= flags;
    }

    unsafe fn handle_key_after(
        &self,
        event: Box<key_event>,
        after: Option<&Rc<UnsafeCell<cmdq_item>>>,
        next: Option<&mut Weak<UnsafeCell<cmdq_item>>>,
    ) -> i32 {
        server_client_handle_key_after(self, event, after, next)
    }

    unsafe fn print(&self, parse: bool, buffer: &mut SegmentedBuf) {
        server_client_print(Some(self), i32::from(parse), buffer);
    }

    unsafe fn control_write_output(&self, pane: &Rc<UnsafeCell<window_pane>>) {
        if self.attached_session().upgrade().is_some() && self.is_control() {
            control_write_output(self, pane);
        }
    }

    unsafe fn with_output_offset<R>(
        &self,
        pane: u32,
        read: impl FnOnce(Option<&mut window_pane_offset>, bool) -> R,
    ) -> R {
        let client = &mut *self.get();
        let mut off = 0;
        let offset = control_pane_offset(
            client
                .control_state
                .as_deref_mut()
                .expect("control client state"),
            client.flags,
            pane,
            &mut off,
        );
        read(offset, off != 0)
    }

    unsafe fn set_return_value(&self, value: i32) {
        (*self.get()).retval = value;
    }

    unsafe fn request_exit(&self, value: i32) {
        self.set_return_value(value);
        (*self.get()).flags |= CLIENT_EXIT as u64;
    }

    unsafe fn with_environment<R>(&self, read: impl FnOnce(Option<&environ>) -> R) -> R {
        read((*self.get()).environ.as_deref())
    }

    unsafe fn cwd(&self, fallback: Option<&Rc<UnsafeCell<session>>>) -> Option<CString> {
        if cfg_finished == 0 {
            if let Some(startup) = (&cfg_client).upgrade() {
                return (*startup.get()).cwd.clone();
            }
        }
        if self.attached_session().upgrade().is_none() && (*self.get()).cwd.is_some() {
            return (*self.get()).cwd.clone();
        }
        if let Some(path) = fallback.and_then(|session| session.cwd()) {
            return Some(path);
        }
        if let Some(path) = self
            .attached_session()
            .upgrade()
            .and_then(|session| session.cwd())
        {
            return Some(path);
        }
        Some(find_home_cstr().unwrap_or(c"/").to_owned())
    }

    unsafe fn set_key_table(&self, name: Option<&CStr>) {
        server_client_set_key_table(self, name.map_or(std::ptr::null(), CStr::as_ptr));
    }

    unsafe fn set_prompt(
        &self,
        find: Option<&cmd_find_state>,
        message: &CStr,
        input: Option<&CStr>,
        inputcb: status_prompt_input_cb,
        freecb: prompt_free_cb,
        flags: i32,
        kind: prompt_type,
    ) {
        crate::src::status::status_prompt_set(
            self,
            find.map_or(std::ptr::null_mut(), |find| {
                (find as *const cmd_find_state).cast_mut()
            }),
            message.as_ptr(),
            input.map_or(std::ptr::null(), CStr::as_ptr),
            inputcb,
            freecb,
            flags,
            kind,
        );
    }

    unsafe fn set_overlay(
        &self,
        check: overlay_check_cb,
        mode: overlay_mode_cb,
        draw: overlay_draw_cb,
        key: overlay_key_cb,
        free: overlay_free_cb,
        resize: overlay_resize_cb,
        data: Box<dyn Any>,
    ) {
        server_client_set_overlay(self, check, mode, draw, key, free, resize, data);
    }

    unsafe fn clear_overlay(&self) {
        server_client_clear_overlay(self);
    }

    unsafe fn with_overlay_data<R>(&self, read: impl FnOnce(Option<&dyn Any>) -> R) -> R {
        read((*self.get()).overlay_data.as_deref())
    }

    unsafe fn with_overlay_check_disabled<R>(
        &self,
        restore: overlay_check_cb,
        draw: impl FnOnce() -> R,
    ) -> R {
        let generation = (*self.get()).overlay_generation;
        // Retire the previous callback before running output, as the popup
        // implementation did. Its captured values may themselves reenter.
        drop((*self.get()).overlay_check.take());
        let result = draw();
        if (*self.get()).overlay_generation == generation && (*self.get()).overlay_data.is_some() {
            let displaced = std::mem::replace(&mut (*self.get()).overlay_check, restore);
            drop(displaced);
        }
        result
    }

    unsafe fn terminal_size(&self) -> (u32, u32) {
        ((*self.get()).tty.sx, (*self.get()).tty.sy)
    }

    unsafe fn refresh_terminal_size(&self) {
        tty_resize(&raw mut (*self.get()).tty);
    }

    unsafe fn draw_overlay_screen(
        &self,
        source: &screen,
        x: u32,
        y: u32,
        sx: u32,
        sy: u32,
        style: &tty_style_ctx,
    ) {
        for row in 0..sy {
            crate::src::tty_draw::tty_draw_line(
                &raw mut (*self.get()).tty,
                source,
                0,
                row,
                sx,
                x,
                y.wrapping_add(row),
                Some(style),
            );
        }
    }

    unsafe fn prepare_overlay_render(&self, context: &mut tty_ctx, x: u32, y: u32) -> bool {
        if (*self.get()).flags & CLIENT_REDRAWOVERLAY as u64 != 0 {
            return false;
        }
        context.wox = 0;
        context.woy = 0;
        (context.wsx, context.wsy) = self.terminal_size();
        context.rxoff = x as i32;
        context.xoff = context.rxoff;
        context.ryoff = y as i32;
        context.yoff = context.ryoff;
        true
    }

    unsafe fn prepare_pane_render(
        &self,
        context: &mut tty_ctx,
        pane: &Rc<UnsafeCell<window_pane>>,
    ) -> i32 {
        use crate::src::window::WindowPane;
        let session = self
            .attached_session()
            .upgrade()
            .expect("terminal client session");
        let window = pane.window_observer().upgrade().expect("live pane parent");
        let invisible = context.flags & TTY_CTX_INVISIBLE_PANES != 0;
        let matches = if invisible {
            session.with_winlinks(|links| {
                let mut link =
                    crate::src::window::winlinks_minmax(links, crate::src::shared::tree::RB_NEGINF);
                while link.is_alive() {
                    let entry = link.get_unchecked();
                    if entry
                        .window_handle()
                        .is_some_and(|owner| Rc::ptr_eq(owner, &window))
                    {
                        return true;
                    }
                    link = crate::src::window::winlinks_next(entry);
                }
                false
            })
        } else {
            let link = session.current_winlink();
            link.get_unchecked()
                .window_handle()
                .is_some_and(|owner| Rc::ptr_eq(owner, &window))
        };
        window.release(c"client pane render parent");
        if invisible || !matches {
            return i32::from(matches);
        }
        let result = pane.prepare_render(
            context,
            (*self.get()).flags & CLIENT_REDRAWWINDOW as u64 != 0,
        );
        if result != 1 {
            return result;
        }
        let view = tty_window_offset(&(*self.get()).tty);
        context.wox = view.ox;
        context.woy = view.oy;
        context.wsx = view.sx;
        context.wsy = view.sy;
        if view.bigger {
            context.flags |= TTY_CTX_WINDOW_BIGGER;
        } else {
            context.flags &= !TTY_CTX_WINDOW_BIGGER;
        }
        if (*self.get()).flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as u64 == 0 {
            let (position, lines) = session.status_layout();
            if position == 0 {
                context.yoff = (context.yoff as u32).wrapping_add(lines) as i32;
            }
        }
        1
    }

    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut format_tree,
    ) -> Option<crate::src::format::FormatValue> {
        crate::src::format::client_format_value(self, key, context)
    }

    unsafe fn detach(&self, message: msgtype) {
        server_client_detach(self, message);
    }

    unsafe fn suspend(&self) {
        server_client_suspend(self);
    }

    unsafe fn lost(&self) {
        server_client_lost(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::shared::control::control_state;
    use std::cell::Cell;

    #[test]
    fn temporarily_disabled_clipping_does_not_replace_a_reentrant_overlay() {
        unsafe {
            let client = client::new();
            let stale_calls = Rc::new(Cell::new(0));
            let replacement_calls = Rc::new(Cell::new(0));
            client.set_overlay(
                Some(Box::new(|_, _, _, _| visible_ranges::default())),
                None,
                None,
                None,
                None,
                None,
                Box::new(1_u32),
            );
            let stale = stale_calls.clone();
            let restored: overlay_check_cb = Some(Box::new(move |_, _, _, _| {
                stale.set(stale.get() + 1);
                visible_ranges::default()
            }));
            let replacement = replacement_calls.clone();
            let result = client.with_overlay_check_disabled(restored, || {
                assert!(server_client_overlay_check(&client, 0, 0, 1).is_none());
                client.set_overlay(
                    Some(Box::new(move |_, _, _, _| {
                        replacement.set(replacement.get() + 1);
                        visible_ranges::default()
                    })),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Box::new(2_u32),
                );
                7
            });
            assert_eq!(result, 7);
            assert_eq!(
                client.with_overlay_data(|data| data.unwrap().downcast_ref::<u32>().copied()),
                Some(2)
            );
            assert!(server_client_overlay_check(&client, 0, 0, 1).is_some());
            assert_eq!(stale_calls.get(), 0);
            assert_eq!(replacement_calls.get(), 1);
            client.clear_overlay();
        }
    }

    #[test]
    fn control_dimensions_and_partial_caps_remain_separate() {
        unsafe {
            let client = client::new();
            (*client.get()).flags = CLIENT_CONTROL as u64;
            (*client.get()).tty.sx = 120;
            (*client.get()).tty.sy = 40;
            (*client.get()).tty.xpixel = 8;
            (*client.get()).tty.ypixel = 16;
            (*client.get()).control_state = Some(Box::new(control_state::empty()));
            let window = window::new();

            crate::src::control::control_set_window_size(&mut *client.get(), window.id(), 80, 0);
            // A partial override falls back to terminal dimensions, but its
            // nonzero width still constrains a later/manual sizing result.
            assert_eq!(client.window_size(Some(&window)), (120, 40, 8, 16));
            let (mut sx, mut sy) = (150, 50);
            client.constrain_window_size(&window, &mut sx, &mut sy);
            assert_eq!((sx, sy), (150, 50));
            (*client.get()).flags |= CLIENT_WINDOWSIZECHANGED;
            client.constrain_window_size(&window, &mut sx, &mut sy);
            assert_eq!((sx, sy), (80, 50));

            crate::src::control::control_set_window_size(&mut *client.get(), window.id(), 90, 30);
            assert_eq!(client.window_size(Some(&window)), (90, 30, 8, 16));
            assert_eq!(client.window_size(None), (120, 40, 8, 16));
            crate::src::window::window_remove_ref(window, c"client API sizing test".as_ptr());
        }
    }

    #[test]
    fn sizing_eligibility_does_not_filter_explicit_dimensions() {
        unsafe {
            let client = client::new();
            let session = session::new();
            (*client.get()).session = Rc::downgrade(&session);
            (*client.get()).tty.sx = 100;
            (*client.get()).tty.sy = 35;
            (*client.get()).flags = CLIENT_CONTROL as u64;
            assert!(!client.participates_in_window_sizing());
            assert_eq!(client.window_size(None), (100, 35, 0, 0));
            (*client.get()).flags |= CLIENT_SIZECHANGED as u64;
            assert!(client.participates_in_window_sizing());
            (*client.get()).flags |= CLIENT_SUSPENDED as u64;
            assert!(!client.participates_in_window_sizing());
            assert_eq!(client.window_size(None), (100, 35, 0, 0));
            assert_eq!(Rc::strong_count(&session), 1);
            (*client.get()).session = Weak::new();
        }
    }

    #[test]
    fn holder_overlay_callback_can_close_itself_during_dispatch() {
        unsafe {
            let session = session::new();
            let link = crate::src::window::winlink_add(&raw mut (*session.get()).windows, 0);
            (*session.get()).set_curw(link.clone());
            let client = client::new();
            (*client.get()).session = Rc::downgrade(&session);
            (*client.get()).tty.client = Rc::downgrade(&client);
            let freed = Rc::new(Cell::new(0));
            let called = Rc::new(Cell::new(0));
            let free_count = freed.clone();
            let draw_count = called.clone();
            client.set_overlay(
                None,
                None,
                Some(Box::new(move |owner| {
                    draw_count.set(draw_count.get() + 1);
                    owner.clear_overlay();
                    assert!(owner.attached_session().upgrade().is_some());
                })),
                None,
                Some(Box::new(move |owner| {
                    assert!(!owner.is_dead());
                    free_count.set(free_count.get() + 1);
                })),
                None,
                Box::new(()),
            );
            server_client_overlay_draw(&client);
            assert_eq!(called.get(), 1);
            assert_eq!(freed.get(), 1);
            assert!((*client.get()).overlay_data.is_none());
            assert_eq!((*client.get()).tty.flags & (TTY_FREEZE | TTY_NOCURSOR), 0);
            server_client_overlay_draw(&client);
            assert_eq!(called.get(), 1);
            (*client.get()).session = Weak::new();
            (*session.get()).set_curw(refbox::Weak::new());
            crate::src::window::winlink_remove(&raw mut (*session.get()).windows, link);
        }
    }
}
