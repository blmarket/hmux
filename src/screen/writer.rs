use super::Screen;
use super::handles::ScreenWriteLease;
use super::write;
use super::write::{screen_write_ctx, screen_write_state};
use crate::fmt_engine::FmtArg;
use crate::types::*;
use core::ffi::{CStr, c_int};
use std::cell::RefMut;

/// An active writer for a screen, optionally carrying pane rendering context.
///
/// Persistent output state stays with the writer. Each operation borrows its
/// target screen together with that state, retaining pane rendering context.
pub(crate) struct RustScreenWriteCtx<'a> {
    state: screen_write_state,
    active: bool,
    target: Option<&'a mut RustScreen>,
    shared_target: Option<ScreenWriteLease<'a>>,
    screen_access: Option<RefMut<'a, RustScreen>>,
}

pub(crate) trait ScreenWriteCtx {
    fn reset(&mut self);
    fn clearscreen(&mut self, bg: u_int);
    fn cursormove(&mut self, px: c_int, py: c_int, origin: c_int);
    fn fast_copy(&mut self, src: &RustScreen, px: u_int, py: u_int, nx: u_int, ny: u_int);
    fn preview(&mut self, src: &RustScreen, nx: u_int, ny: u_int);
    fn hline(
        &mut self,
        nx: u_int,
        top: c_int,
        bottom: c_int,
        lines: box_lines,
        gc: Option<&grid_cell>,
    );
    fn vline(&mut self, ny: u_int, top: c_int, bottom: c_int);
    fn clearcharacter(&mut self, nx: u_int, bg: u_int);
    fn clearendofline(&mut self, bg: u_int);
    fn box_(
        &mut self,
        nx: u_int,
        ny: u_int,
        lines: box_lines,
        gc: Option<&grid_cell>,
        title: Option<&CStr>,
    );
    fn menu(
        &mut self,
        menu: &menu,
        choice: c_int,
        lines: box_lines,
        menu_gc: &grid_cell,
        border_gc: &grid_cell,
        choice_gc: &grid_cell,
    );
    fn cell(&mut self, gc: &grid_cell);
    fn putc(&mut self, gc: &grid_cell, ch: u_char);
    fn puts(&mut self, gc: &grid_cell, fmt: &CStr, args: &[FmtArg]);
    fn nputs(&mut self, maxlen: ssize_t, gc: &grid_cell, fmt: &CStr, args: &[FmtArg]);
    fn vnputs(&mut self, maxlen: ssize_t, gc: &grid_cell, fmt: &CStr, args: &[FmtArg]);
    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        cx: u_int,
        width: u_int,
        lines: u_int,
        more: c_int,
        gc: &grid_cell,
        fmt: &CStr,
        args: &[FmtArg],
    ) -> c_int;
    fn collect_add(&mut self, gc: &grid_cell);
    fn collect_end(&mut self);
    fn cursorup(&mut self, ny: u_int);
    fn cursordown(&mut self, ny: u_int);
    fn cursorright(&mut self, nx: u_int);
    fn cursorleft(&mut self, nx: u_int);
    fn mode_set(&mut self, mode: c_int);
    fn mode_clear(&mut self, mode: c_int);
    fn backspace(&mut self);
    fn alignmenttest(&mut self);
    fn insertcharacter(&mut self, nx: u_int, bg: u_int);
    fn deletecharacter(&mut self, nx: u_int, bg: u_int);
    fn linefeed(&mut self, wrapped: c_int, bg: u_int);
    fn scrollup(&mut self, lines: u_int, bg: u_int);
    fn scrolldown(&mut self, lines: u_int, bg: u_int);
    fn carriagereturn(&mut self);
    fn clearendofscreen(&mut self, bg: u_int);
    fn clearstartofscreen(&mut self, bg: u_int);
    fn scrollregion(&mut self, top: u_int, bottom: u_int);
    fn deleteline(&mut self, ny: u_int, bg: u_int);
    fn clearline(&mut self, bg: u_int);
    fn clearstartofline(&mut self, bg: u_int);
    fn reverseindex(&mut self, bg: u_int);
    fn insertline(&mut self, ny: u_int, bg: u_int);
    fn cursor_position(&self) -> (u_int, u_int);
    fn screen_mut(&mut self) -> &mut dyn Screen<Grid = crate::grid::RustGrid>;
    fn size(&self) -> (u_int, u_int);
    fn format_draw(
        &mut self,
        base: &grid_cell,
        available: u_int,
        expanded: &[u8],
        ranges: Option<&mut style_ranges>,
        default_colours: c_int,
    );
    fn setselection(&mut self, clip: &CStr, text: &[u_char]);
    fn clearhistory(&mut self);
    fn fullredraw(&mut self);
    fn rawstring(&mut self, data: &[u_char], allow_invisible_panes: c_int);
    fn alternateon(&mut self, gc: &mut grid_cell, cursor: c_int);
    fn alternateoff(&mut self, gc: &mut grid_cell, cursor: c_int);
    fn stop(&mut self);
    fn finish(mut self)
    where
        Self: Sized,
    {
        self.stop();
    }
}

impl<'a> RustScreenWriteCtx<'a> {
    /// Creates an inactive writer context for a later start operation.
    fn new() -> Self {
        Self {
            state: screen_write_state::default(),
            active: false,
            target: None,
            shared_target: None,
            screen_access: None,
        }
    }

    fn with_screen<R>(&self, f: impl FnOnce(&RustScreen) -> R) -> R {
        assert!(self.active, "screen writer is inactive");
        if let Some(screen) = self.screen_access.as_deref() {
            f(screen)
        } else if let Some(target) = self.shared_target.as_ref() {
            f(&target.borrow())
        } else {
            f(self.target.as_deref().expect("active writer has a target"))
        }
    }

    fn context_mut(&mut self) -> screen_write_ctx<'_> {
        assert!(self.active, "screen writer is inactive");
        self.screen_access = None;
        if self.shared_target.is_some()
            && self.state.init_ctx_cb.is_none()
            && let Some(pane) = self.state.wp.as_mut()
            && let Some(pane) = unsafe { pane.get_mut() }
            && *pane.flags() & crate::window::PANE_STYLECHANGED != 0
        {
            let mut defaults = crate::grid::grid_default_cell;
            unsafe { crate::tty::tty_default_colours(&mut defaults, pane) };
        }
        if let Some(target) = self.shared_target.as_ref() {
            screen_write_ctx::on_shared(&mut self.state, target.borrow_mut())
        } else {
            screen_write_ctx::new(
                &mut self.state,
                self.target
                    .as_deref_mut()
                    .expect("active writer has a target"),
            )
        }
    }

    /// Starts writing to a standalone screen using this context.
    ///
    /// The screen remains exclusively borrowed until the writer is dropped.
    fn start(&mut self, s: &'a mut RustScreen) {
        assert!(!self.active, "screen writer is already active");
        self.state = write::screen_write_start(s);
        self.target = Some(s);
        self.active = true;
    }

    /// Starts writing to a pane base screen using this context.
    ///
    /// The pane remains exclusively borrowed until the writer is dropped.
    fn start_pane_base(&mut self, wp: &'a mut (impl crate::WindowPane + ?Sized)) {
        assert!(!self.active, "screen writer is already active");
        self.state = write::screen_write_start_pane_base(wp);
        self.target = Some(wp.base_mut().into_inner());
        self.active = true;
    }

    fn start_callback_owned(
        &mut self,
        s: &'a mut RustScreen,
        init_ctx: super::write::screen_write_init_ctx,
    ) {
        assert!(!self.active, "screen writer is already active");
        self.state = write::screen_write_start_callback(s, init_ctx);
        self.target = Some(s);
        self.active = true;
    }

    /// Starts writing to a screen with an owned terminal-context callback.
    pub(crate) fn on_callback_owned(
        s: &'a mut RustScreen,
        init_ctx: super::write::screen_write_init_ctx,
    ) -> Self {
        let mut writer = Self::new();
        writer.start_callback_owned(s, init_ctx);
        writer
    }

    /// Starts writing to a standalone screen.
    pub(crate) fn on_screen(s: &'a mut RustScreen) -> Self {
        let mut writer = Self::new();
        writer.start(s);
        writer
    }

    pub(crate) fn on_borrowed_screen(screen: &'a mut super::ScreenMut<'_>) -> Self {
        Self::on_screen(screen.as_inner())
    }

    /// Writes to a shared screen, checking exclusive access for each operation.
    pub(crate) fn on_shared_screen(target: &'a ScreenRef) -> Self {
        let mut writer = Self::new();
        writer.shared_target = Some(target.begin_write());
        {
            let mut screen = writer
                .shared_target
                .as_ref()
                .expect("writer has a screen")
                .borrow_mut();
            writer.state = write::screen_write_start(&mut screen);
        }
        writer.active = true;
        writer
    }

    /// Writes to a shared screen with an owned terminal-context callback.
    pub(crate) fn on_shared_callback(
        target: &'a ScreenRef,
        init_ctx: super::write::screen_write_init_ctx,
    ) -> Self {
        let mut writer = Self::on_shared_screen(target);
        writer.state.init_ctx_cb = init_ctx;
        writer
    }

    /// Writes to a shared screen with retained pane output context.
    pub(crate) fn on_shared_pane_screen(
        target: &'a ScreenRef,
        pane: Option<RustWindowPaneWeak>,
    ) -> Self {
        let mut writer = Self::on_shared_screen(target);
        writer.state.wp = pane;
        writer
    }

    /// Starts writing to the screen currently shown by a pane.
    pub(crate) fn on_pane(wp: &'a mut (impl crate::WindowPane + ?Sized)) -> Self {
        if wp.showing_base() {
            return Self::on_pane_base(wp);
        }
        let pane = wp.observation();
        let mode = wp.active_mode_mut().expect("pane has a mode");
        let mut writer = match mode.screen_target() {
            crate::modes::ModeScreenTarget::Owned(screen) => Self::on_screen(screen.into_inner()),
            crate::modes::ModeScreenTarget::Shared(screen) => Self::on_shared_screen(screen),
        };
        writer.state.wp = pane;
        writer
    }

    /// Starts writing to a pane's base screen.
    pub(crate) fn on_pane_base(wp: &'a mut (impl crate::WindowPane + ?Sized)) -> Self {
        let mut writer = Self::new();
        writer.start_pane_base(wp);
        writer
    }
}

impl ScreenWriteCtx for RustScreenWriteCtx<'_> {
    /// Resets the screen through the active writer.
    fn reset(&mut self) {
        unsafe { write::screen_write_reset(&mut self.context_mut()) };
    }

    /// Clears the target screen.
    fn clearscreen(&mut self, bg: u_int) {
        unsafe { write::screen_write_clearscreen(&mut self.context_mut(), bg) };
    }

    /// Moves the target cursor.
    fn cursormove(&mut self, px: c_int, py: c_int, origin: c_int) {
        unsafe { write::screen_write_cursormove(&mut self.context_mut(), px, py, origin) };
    }

    /// Copies a rectangle from another screen.
    fn fast_copy(&mut self, src: &RustScreen, px: u_int, py: u_int, nx: u_int, ny: u_int) {
        unsafe { write::screen_write_fast_copy(&mut self.context_mut(), src, px, py, nx, ny) };
    }

    /// Draws a preview of another screen on the target screen.
    fn preview(&mut self, src: &RustScreen, nx: u_int, ny: u_int) {
        unsafe { write::screen_write_preview(&mut self.context_mut(), src, nx, ny) };
    }

    /// Draws a horizontal line on the target screen.
    fn hline(
        &mut self,
        nx: u_int,
        top: c_int,
        bottom: c_int,
        lines: box_lines,
        gc: Option<&grid_cell>,
    ) {
        unsafe { write::screen_write_hline(&mut self.context_mut(), nx, top, bottom, lines, gc) };
    }

    /// Draws a vertical line on the target screen.
    fn vline(&mut self, ny: u_int, top: c_int, bottom: c_int) {
        unsafe { write::screen_write_vline(&mut self.context_mut(), ny, top, bottom) };
    }

    /// Clears characters on the target screen.
    fn clearcharacter(&mut self, nx: u_int, bg: u_int) {
        unsafe { write::screen_write_clearcharacter(&mut self.context_mut(), nx, bg) };
    }

    /// Clears to the end of the target line.
    fn clearendofline(&mut self, bg: u_int) {
        write::screen_write_clearendofline(&mut self.context_mut(), bg);
    }

    /// Draws a box on the target screen.
    fn box_(
        &mut self,
        nx: u_int,
        ny: u_int,
        lines: box_lines,
        gc: Option<&grid_cell>,
        title: Option<&CStr>,
    ) {
        unsafe { write::screen_write_box(&mut self.context_mut(), nx, ny, lines, gc, title) };
    }

    /// Draws a menu through the active writer.
    fn menu(
        &mut self,
        menu: &menu,
        choice: c_int,
        lines: box_lines,
        menu_gc: &grid_cell,
        border_gc: &grid_cell,
        choice_gc: &grid_cell,
    ) {
        unsafe {
            write::screen_write_menu(
                &mut self.context_mut(),
                menu,
                choice,
                lines,
                menu_gc,
                border_gc,
                choice_gc,
            )
        };
    }

    /// Writes one cell through the active writer.
    fn cell(&mut self, gc: &grid_cell) {
        unsafe { write::screen_write_cell(&mut self.context_mut(), gc) };
    }

    /// Writes one character through the active writer.
    fn putc(&mut self, gc: &grid_cell, ch: u_char) {
        unsafe { write::screen_write_putc(&mut self.context_mut(), gc, ch) };
    }

    /// Writes a formatted string through the active writer.
    fn puts(&mut self, gc: &grid_cell, fmt: &CStr, args: &[FmtArg]) {
        unsafe { write::screen_write_puts(&mut self.context_mut(), gc, fmt, args) };
    }

    /// Writes a bounded formatted string through the active writer.
    fn nputs(&mut self, maxlen: ssize_t, gc: &grid_cell, fmt: &CStr, args: &[FmtArg]) {
        unsafe { write::screen_write_nputs(&mut self.context_mut(), maxlen, gc, fmt, args) };
    }

    /// Writes formatted text with a column limit through the active writer.
    fn vnputs(&mut self, maxlen: ssize_t, gc: &grid_cell, fmt: &CStr, args: &[FmtArg]) {
        unsafe { write::screen_write_vnputs(&mut self.context_mut(), maxlen, gc, fmt, args) };
    }

    /// Writes formatted text with line wrapping through the active writer.
    fn text(
        &mut self,
        cx: u_int,
        width: u_int,
        lines: u_int,
        more: c_int,
        gc: &grid_cell,
        fmt: &CStr,
        args: &[FmtArg],
    ) -> c_int {
        unsafe {
            write::screen_write_text(
                &mut self.context_mut(),
                cx,
                width,
                lines,
                more,
                gc,
                fmt,
                args,
            )
        }
    }

    /// Adds a cell to the collected write stream.
    fn collect_add(&mut self, gc: &grid_cell) {
        unsafe { write::screen_write_collect_add(&mut self.context_mut(), gc) };
    }

    /// Ends the collected write stream.
    fn collect_end(&mut self) {
        unsafe { write::screen_write_collect_end(&mut self.context_mut()) };
    }

    /// Moves the cursor by rows.
    fn cursorup(&mut self, ny: u_int) {
        unsafe { write::screen_write_cursorup(&mut self.context_mut(), ny) };
    }

    /// Moves the cursor down by rows.
    fn cursordown(&mut self, ny: u_int) {
        unsafe { write::screen_write_cursordown(&mut self.context_mut(), ny) };
    }

    /// Moves the cursor right by columns.
    fn cursorright(&mut self, nx: u_int) {
        unsafe { write::screen_write_cursorright(&mut self.context_mut(), nx) };
    }

    /// Moves the cursor left by columns.
    fn cursorleft(&mut self, nx: u_int) {
        unsafe { write::screen_write_cursorleft(&mut self.context_mut(), nx) };
    }

    /// Enables screen modes on the target screen.
    fn mode_set(&mut self, mode: c_int) {
        write::screen_write_mode_set(&mut self.context_mut(), mode);
    }

    /// Disables screen modes on the target screen.
    fn mode_clear(&mut self, mode: c_int) {
        write::screen_write_mode_clear(&mut self.context_mut(), mode);
    }

    /// Moves the cursor back over a character, respecting wrapped lines.
    fn backspace(&mut self) {
        unsafe { write::screen_write_backspace(&mut self.context_mut()) };
    }

    /// Fills the target screen with the terminal alignment-test character.
    fn alignmenttest(&mut self) {
        unsafe { write::screen_write_alignmenttest(&mut self.context_mut()) };
    }

    /// Inserts characters at the cursor.
    fn insertcharacter(&mut self, nx: u_int, bg: u_int) {
        unsafe { write::screen_write_insertcharacter(&mut self.context_mut(), nx, bg) };
    }

    /// Deletes characters at the cursor.
    fn deletecharacter(&mut self, nx: u_int, bg: u_int) {
        unsafe { write::screen_write_deletecharacter(&mut self.context_mut(), nx, bg) };
    }

    /// Advances the screen by one line.
    fn linefeed(&mut self, wrapped: c_int, bg: u_int) {
        unsafe { write::screen_write_linefeed(&mut self.context_mut(), wrapped, bg) };
    }

    /// Scrolls the target region up.
    fn scrollup(&mut self, lines: u_int, bg: u_int) {
        unsafe { write::screen_write_scrollup(&mut self.context_mut(), lines, bg) };
    }

    /// Scrolls the target region down.
    fn scrolldown(&mut self, lines: u_int, bg: u_int) {
        unsafe { write::screen_write_scrolldown(&mut self.context_mut(), lines, bg) };
    }

    /// Returns the cursor to the start of its line.
    fn carriagereturn(&mut self) {
        unsafe { write::screen_write_carriagereturn(&mut self.context_mut()) };
    }

    /// Clears from the cursor to the end of the screen.
    fn clearendofscreen(&mut self, bg: u_int) {
        unsafe { write::screen_write_clearendofscreen(&mut self.context_mut(), bg) };
    }

    /// Clears from the start of the screen through the cursor.
    fn clearstartofscreen(&mut self, bg: u_int) {
        unsafe { write::screen_write_clearstartofscreen(&mut self.context_mut(), bg) };
    }

    /// Sets the scrollable region of the target screen.
    fn scrollregion(&mut self, top: u_int, bottom: u_int) {
        unsafe { write::screen_write_scrollregion(&mut self.context_mut(), top, bottom) };
    }

    /// Deletes rows from the target screen.
    fn deleteline(&mut self, ny: u_int, bg: u_int) {
        unsafe { write::screen_write_deleteline(&mut self.context_mut(), ny, bg) };
    }

    /// Clears the target line.
    fn clearline(&mut self, bg: u_int) {
        write::screen_write_clearline(&mut self.context_mut(), bg);
    }

    /// Clears from the start of the line through the cursor.
    fn clearstartofline(&mut self, bg: u_int) {
        write::screen_write_clearstartofline(&mut self.context_mut(), bg);
    }

    /// Moves the cursor up, scrolling the target region down when needed.
    fn reverseindex(&mut self, bg: u_int) {
        unsafe { write::screen_write_reverseindex(&mut self.context_mut(), bg) };
    }

    /// Inserts rows into the target screen.
    fn insertline(&mut self, ny: u_int, bg: u_int) {
        unsafe { write::screen_write_insertline(&mut self.context_mut(), ny, bg) };
    }

    /// Returns the current cursor position of the target screen.
    fn cursor_position(&self) -> (u_int, u_int) {
        self.with_screen(Screen::cursor)
    }

    /// Returns mutable access to the screen currently being written.
    ///
    /// The returned screen must not be used to bypass writer operations that
    /// maintain the terminal output collection.
    fn screen_mut(&mut self) -> &mut dyn Screen<Grid = crate::grid::RustGrid> {
        assert!(self.active, "screen writer is inactive");
        if let Some(target) = self.shared_target.as_ref() {
            &mut **self.screen_access
                .get_or_insert_with(|| target.borrow_mut())
        } else {
            self.target
                .as_deref_mut()
                .expect("active writer has a target")
        }
    }

    /// Returns the visible size of the target screen.
    fn size(&self) -> (u_int, u_int) {
        self.with_screen(Screen::size)
    }

    /// Draws expanded formatted text through the active writer.
    fn format_draw(
        &mut self,
        base: &grid_cell,
        available: u_int,
        expanded: &[u8],
        ranges: Option<&mut style_ranges>,
        default_colours: c_int,
    ) {
        unsafe {
            super::draw::format_draw(
                &mut self.context_mut(),
                base,
                available,
                expanded,
                ranges,
                default_colours,
            )
        };
    }

    /// Sets a clipboard selection through the active writer.
    fn setselection(&mut self, clip: &CStr, text: &[u_char]) {
        unsafe { write::screen_write_setselection(&mut self.context_mut(), clip, text) };
    }

    /// Clears the target screen's scrollback history.
    fn clearhistory(&mut self) {
        write::screen_write_clearhistory(&mut self.context_mut());
    }

    /// Requests a complete redraw of the target screen.
    fn fullredraw(&mut self) {
        unsafe { write::screen_write_fullredraw(&mut self.context_mut()) };
    }

    /// Sends bytes directly to the terminal.
    fn rawstring(&mut self, data: &[u_char], allow_invisible_panes: c_int) {
        unsafe {
            write::screen_write_rawstring(&mut self.context_mut(), data, allow_invisible_panes)
        };
    }

    /// Switches the target pane to its alternate screen.
    fn alternateon(&mut self, gc: &mut grid_cell, cursor: c_int) {
        unsafe { write::screen_write_alternateon(&mut self.context_mut(), gc, cursor) };
    }

    /// Switches the target pane back from its alternate screen.
    fn alternateoff(&mut self, gc: &mut grid_cell, cursor: c_int) {
        unsafe { write::screen_write_alternateoff(&mut self.context_mut(), gc, cursor) };
    }

    fn stop(&mut self) {
        if self.active {
            unsafe { write::screen_write_stop(&mut self.context_mut()) };
            self.active = false;
            self.state = screen_write_state::default();
            self.target = None;
            self.shared_target = None;
        }
    }
}

impl RustScreenWriteCtx<'_> {
    fn strlen(fmt: &CStr, args: &[FmtArg]) -> size_t {
        unsafe { write::screen_write_strlen(fmt, args) }
    }
}

pub(crate) fn screen_write_ctx_on_screen(s: &mut RustScreen) -> RustScreenWriteCtx<'_> {
    RustScreenWriteCtx::on_screen(s)
}

pub(crate) fn screen_write_ctx_on_pane(wp: &mut (impl crate::WindowPane + ?Sized)) -> RustScreenWriteCtx<'_> {
    RustScreenWriteCtx::on_pane(wp)
}

pub(crate) fn screen_write_ctx_on_pane_base(
    wp: &mut (impl crate::WindowPane + ?Sized),
) -> RustScreenWriteCtx<'_> {
    RustScreenWriteCtx::on_pane_base(wp)
}

pub(crate) fn screen_write_strlen(fmt: &CStr, args: &[FmtArg]) -> size_t {
    RustScreenWriteCtx::strlen(fmt, args)
}

impl Drop for RustScreenWriteCtx<'_> {
    fn drop(&mut self) {
        self.stop();
    }
}
use super::RustScreen;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;
    use crate::tests::test_fixtures::globals;

    #[test]
    #[should_panic(expected = "screen writer is inactive")]
    fn querying_an_inactive_writer_rejects_its_missing_target() {
        let writer = RustScreenWriteCtx::new();
        writer.cursor_position();
    }

    #[test]
    #[should_panic(expected = "screen writer is inactive")]
    fn writing_with_an_inactive_writer_is_rejected() {
        let mut writer = RustScreenWriteCtx::new();
        writer.clearscreen(8);
    }

    #[test]
    fn stopping_a_writer_releases_its_target_and_callback() {
        let _guard = crate::tests::test_fixtures::globals();
        let mut target = RustScreen::new_with_server_options(4, 2, 0);
        let owner = std::rc::Rc::new(());
        let weak = std::rc::Rc::downgrade(&owner);
        let callback = std::rc::Rc::new(move |_: &mut tty_ctx| {
            let _ = &owner;
        });
        let mut writer = RustScreenWriteCtx::new();
        writer.start_callback_owned(&mut target, Some(callback));
        assert!(weak.upgrade().is_some());
        writer.stop();
        assert!(writer.target.is_none());
        assert!(writer.state.wp.is_none());
        assert!(weak.upgrade().is_none());
        writer.stop();
        drop(writer);
        drop(target);
        let mut replacement = RustScreen::new_with_server_options(3, 1, 0);
        let mut writer = RustScreenWriteCtx::on_screen(&mut replacement);
        assert_eq!(writer.size(), (3, 1));
        writer.stop();
    }
    #[test]
    fn shared_screen_writers_allow_observation_between_operations() {
        let _guard = globals();
        let target = ScreenRef::new(RustScreen::new_with_server_options(12, 3, 0));
        let mut writer = RustScreenWriteCtx::on_shared_screen(&target);
        writer.puts(&crate::grid::grid_default_cell, c"hi", &[]);
        assert_eq!(target.borrow().cursor(), (2, 0));
        writer.screen_mut().set_cursor_style(3);
        assert_eq!(writer.cursor_position(), (2, 0));
        assert_eq!(writer.size(), (12, 3));
        writer.cursormove(1, 1, 0);
        assert_eq!(
            target.borrow().cursor_style(),
            crate::screen::SCREEN_CURSOR_UNDERLINE
        );
        writer.puts(&crate::grid::grid_default_cell, c"there", &[]);
        writer.finish();
        let screen = target.borrow();
        assert_eq!(screen.cursor(), (6, 1));
        assert_eq!(RustScreen::grid(&screen).cell(1, 1).data.data[0], b't');
    }

    #[test]
    fn shared_screen_writes_reject_an_outstanding_read_borrow() {
        let _guard = globals();
        let target = ScreenRef::new(RustScreen::new_with_server_options(12, 3, 0));
        let mut writer = RustScreenWriteCtx::on_shared_screen(&target);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _other = RustScreenWriteCtx::on_shared_screen(&target);
            }))
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _mutable = target.borrow_mut();
            }))
            .is_err()
        );
        let read = target.borrow();
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            writer.cursormove(2, 1, 0);
        }));
        assert!(attempt.is_err());
        assert_eq!(read.cursor(), (0, 0));
        drop(read);
        writer.cursormove(2, 1, 0);
        writer.finish();
        assert_eq!(target.borrow_mut().cursor(), (2, 1));
    }
    #[test]
    fn a_panicking_output_callback_releases_the_shared_screen_borrow() {
        let _guard = globals();
        let target = ScreenRef::new(RustScreen::new_with_server_options(12, 3, 0));
        let first = std::cell::Cell::new(true);
        let callback = std::rc::Rc::new(move |_: &mut tty_ctx| {
            if first.replace(false) {
                panic!("output callback failed");
            }
        });
        let mut writer = RustScreenWriteCtx::on_shared_callback(&target, Some(callback));
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            writer.clearscreen(8);
        }));
        assert!(attempt.is_err());
        assert_eq!(target.borrow().cursor(), (0, 0));
        writer.cursormove(2, 1, 0);
        writer.putc(&crate::grid::grid_default_cell, b'x');
        writer.finish();
        let mut screen = target.borrow_mut();
        assert_eq!(screen.grid().cell(2, 1).data.data[0], b'x');
        screen.set_cursor(0, 0);
    }

    #[test]
    fn a_failed_shared_writer_start_releases_its_access_reservation() {
        let _guard = globals();
        let target = ScreenRef::new(RustScreen::new_with_server_options(12, 3, 0));
        let read = target.borrow();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _writer = RustScreenWriteCtx::on_shared_screen(&target);
            }))
            .is_err()
        );
        drop(read);
        let mut writer = RustScreenWriteCtx::on_shared_screen(&target);
        writer.cursormove(1, 1, 0);
        writer.stop();
        assert_eq!(target.borrow_mut().cursor(), (1, 1));
    }
    #[test]
    fn pane_writers_borrow_shown_mode_screens_and_keep_base_writes_separate() {
        use crate::options::OptionsRef;
        use crate::tests::test_fixtures::Target;
                let _guard = globals();
        let mut target = Target::new(40, 12);
        let state = target.state();
        let mut pane = state.pane_ref().unwrap();
        unsafe {
            for mode in [
                WindowMode::Clock,
                WindowMode::Copy,
                WindowMode::View,
                WindowMode::Buffer,
                WindowMode::Client,
                WindowMode::Tree,
                WindowMode::Customize,
            ] {
                pane.get_mut().unwrap().base_mut().set_cursor(0, 0);
                let source = (mode == WindowMode::Copy).then_some(pane.id());
                assert_eq!(
                    (pane.get_mut().unwrap()).set_mode(
                        source.and_then(crate::window::window_pane_find_by_id),
                        mode,
                        Some(&state),
                        None
                    ),
                    0
                );
                let entry = (pane.get().unwrap()).active_mode().unwrap();
                let shared = match &entry.state {
                    WindowModeState::Clock(_) => None,
                    WindowModeState::Copy(data) | WindowModeState::View(data) => {
                        Some(data.screen.clone())
                    }
                    _ => Some(
                        entry
                            .screen.as_ref().unwrap().shared().unwrap()
                            .clone(),
                    ),
                };
                pane.get().unwrap().options_ref().set_string(
                    c"window-style",
                    0,
                    c"#{?pane_in_mode,fg=default,fg=red}",
                    crate::fmt_args![],
                );
                *pane.get_mut().unwrap().flags_mut() |= crate::window::PANE_STYLECHANGED;
                let mut writer = RustScreenWriteCtx::on_pane(pane.get_mut().unwrap());
                writer.cursormove(0, 0, 0);
                writer.putc(&crate::grid::grid_default_cell, b'x');
                if let Some(shared) = shared.as_ref() {
                    let read = shared.borrow();
                    assert_eq!(read.cursor(), (1, 0));
                    assert!(
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            writer.cursormove(2, 1, 0);
                        }))
                        .is_err()
                    );
                    drop(read);
                }
                writer.cursormove(2, 1, 0);
                writer.finish();
                assert_eq!(pane.get().unwrap().screen_ref().cursor(), (2, 1));
                assert_eq!(pane.get().unwrap().base().cursor(), (0, 0));
                let mut writer = RustScreenWriteCtx::on_pane_base(pane.get_mut().unwrap());
                writer.cursormove(3, 2, 0);
                writer.finish();
                assert_eq!(pane.get().unwrap().base().cursor(), (3, 2));
                assert_eq!(pane.get().unwrap().screen_ref().cursor(), (2, 1));
                if let Some(shared) = shared.as_ref() {
                    shared.borrow_mut().set_cursor(0, 0);
                }
                (pane.get_mut().unwrap()).reset_modes();
            }
        }
    }
}
