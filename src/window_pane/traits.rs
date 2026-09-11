use super::*;

impl crate::pane_identity::PaneIdentity for window_pane {
    fn pane_id(&self) -> u32 {
        self.id
    }

}

impl crate::pane_search::PaneSearchState for window_pane {
    fn query(&self) -> Option<&CStr> { self.searchstr.as_deref() }
    fn is_regex(&self) -> bool { self.searchregex }
    fn set(&mut self, query: &CStr, regex: bool) {
        self.searchstr = Some(query.to_owned()); self.searchregex = regex;
    }
    fn matches(&self, query: &CStr, regex: bool) -> bool {
        self.searchregex == regex && self.searchstr.as_deref() == Some(query)
    }
}

impl crate::pane_geometry::PaneGeometryState for window_pane {
    fn geometry(&self) -> PaneGeometry {
        PaneGeometry { xoff: self.xoff, yoff: self.yoff, sx: self.sx, sy: self.sy }
    }
    fn set_position(&mut self, x: c_int, y: c_int) { self.xoff = x; self.yoff = y; }
}

impl crate::WindowPane for window_pane {

    fn activity_point(&self) -> u_int { self.active_point }
    fn mark_active_at(&mut self, point: u_int) { self.active_point = point; self.name_changed(); }

    fn pane_command(&self) -> PaneCommand {
        PaneCommand { argv: self.argv.clone(), shell: self.shell.clone(), cwd: self.cwd.clone() }
    }
    fn set_pane_command(&mut self, command: &PaneCommand) {
        self.argv.clone_from(&command.argv); self.shell.clone_from(&command.shell); self.cwd.clone_from(&command.cwd);
    }

    fn exit_status(&self) -> c_int { self.status }
    fn death_time(&self) -> timeval { self.dead_time }

    fn styles(&self) -> PaneStyleCells {
        PaneStyleCells { cached_gc: self.cached_gc, cached_active_gc: self.cached_active_gc }
    }
    unsafe fn refresh_styles(&mut self) -> PaneStyleCells {
        if self.flags & PANE_STYLECHANGED != 0 { unsafe { render::refresh_styles(self) }; }
        self.styles()
    }

    fn colours(&self) -> PaneControlColourPair {
        PaneControlColourPair { control_fg: self.control_fg, control_bg: self.control_bg }
    }
    unsafe fn report_control_colours(&mut self, tty: &mut tty, report: &[u8]) {
        let mut foreground = self.control_fg.unwrap_or(-1);
        let mut background = self.control_bg.unwrap_or(-1);
        let mut size = 0;
        unsafe { crate::tty::tty_keys_colours(tty, report, &mut size, &mut foreground, &mut background) };
        self.control_fg = (foreground != -1).then_some(foreground);
        self.control_bg = (background != -1).then_some(background);
    }

    fn slider(&self) -> PaneScrollbarSlider {
        PaneScrollbarSlider { sb_slider_y: self.sb_slider_y, sb_slider_h: self.sb_slider_h }
    }
    fn publish_slider(&mut self, slider: PaneScrollbarSlider) { self.sb_slider_y = slider.sb_slider_y; self.sb_slider_h = slider.sb_slider_h; }

    fn scrollbar_style(&self) -> PaneScrollbarStyle { self.scrollbar_style }

    fn observation(&self) -> Option<RustWindowPaneWeak> {
        self.observation.clone()
    }

    fn flags(&self) -> &core::ffi::c_int {
        &self.flags
    }
    #[cfg(test)]
    fn flags_mut(&mut self) -> &mut core::ffi::c_int {
        &mut self.flags
    }
    fn request_redraw(&mut self) { self.flags |= crate::consts::PANE_REDRAW; }
    fn request_scrollbar_redraw(&mut self) { self.flags |= crate::consts::PANE_REDRAWSCROLLBAR; }
    fn request_full_redraw(&mut self) {
        self.flags |= crate::consts::PANE_REDRAW | crate::consts::PANE_REDRAWSCROLLBAR;
    }
    fn finish_redraw(&mut self) {
        self.flags &= !(crate::consts::PANE_REDRAW | crate::consts::PANE_REDRAWSCROLLBAR);
    }
    fn name_changed(&mut self) { self.flags |= crate::consts::PANE_CHANGED; }
    fn finish_name_update(&mut self) { self.flags &= !crate::consts::PANE_CHANGED; }
    fn request_theme_update(&mut self) { self.flags |= PANE_THEMECHANGED; }
    fn set_stack_member(&mut self, member: bool) {
        if member { self.flags |= crate::window::PANE_VISITED; }
        else { self.flags &= !crate::window::PANE_VISITED; }
    }
    fn set_window_zoomed(&mut self, zoomed: bool) {
        if zoomed { self.flags |= crate::consts::PANE_ZOOMED; }
        else { self.flags &= !crate::consts::PANE_ZOOMED; }
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        if enabled { self.flags &= !crate::consts::PANE_INPUTOFF; }
        else { self.flags |= crate::consts::PANE_INPUTOFF; }
    }
    fn prepare_empty(&mut self) {
        self.flags |= crate::consts::PANE_EMPTY;
        self.base.set_mode((self.base.mode() & !crate::consts::MODE_CURSOR) | crate::consts::MODE_CRLF);
    }
    fn inherit_window_context(&mut self, window: &WindowRef) {
        self.window = Some(window.downgrade());
        self.options_ref().set_parent(Some(&window.options()));
        self.flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
    }
    unsafe fn resize(&mut self, size: PaneSize) {
        let old = PaneSize { width: self.sx, height: self.sy };
        if old == size { return; }
        self.stop_sync();
        self.resize_queue.record(old, size);
        self.sx = size.width;
        self.sy = size.height;
        let reflow = !self.base.is_alternate();
        unsafe { (&mut self.base).resize(size.width, size.height, reflow as c_int) };
        if let Some(mode) = self.modes.first_mut() {
            unsafe { mode.resize(size.width, size.height) };
        }
    }
    fn start_sync(&mut self) {
        self.base.set_mode(self.base.mode() | crate::screen::MODE_SYNC);
        if !self.sync_timer.is_set() {
            let pane = self.observation.clone().expect("a syncing pane is owned");
            self.sync_timer.set_callback(move || {
                if let Some(owner) = pane.upgrade() {
                    unsafe { (&mut *owner.0.pane.get()).expire_sync() };
                }
            });
        }
        self.sync_timer.arm(timeval::from_secs(1));
    }
    fn stop_sync(&mut self) {
        self.sync_timer.disarm();
        self.base.set_mode(self.base.mode() & !crate::screen::MODE_SYNC);
    }

    #[cfg(test)]
    unsafe fn configure_test(&mut self, setting: PaneTestSetup) {
        match setting {
            PaneTestSetup::Screen(screen) => self.base = screen,
            PaneTestSetup::Descriptor(fd) => self.fd = fd,
            PaneTestSetup::ScrollbarStyle(style) => self.scrollbar_style = style,
            PaneTestSetup::Styles(styles) => { self.cached_gc = styles.cached_gc; self.cached_active_gc = styles.cached_active_gc; }
            PaneTestSetup::Palette(palette) => self.palette = palette,
            PaneTestSetup::Geometry(rect) => { self.xoff = rect.xoff; self.yoff = rect.yoff; self.sx = rect.sx; self.sy = rect.sy; }
            PaneTestSetup::Size(size) => { self.sx = size.width; self.sy = size.height; }
            PaneTestSetup::Options(options) => {
                if let Some(old) = self.options.take() { RustOptionsEngine.destroy(old); }
                self.options = options;
            }
            PaneTestSetup::Stream(stream) => self.event = stream,
            PaneTestSetup::Parser(parser) => {
                if let Some(old) = self.ictx.take() { unsafe { old.close() }; }
                self.ictx = parser;
            }
            PaneTestSetup::Terminal(name) => self.tty = name,
        }
    }
    fn process_id(&self) -> pid_t { self.pid }
    fn process_active(&self) -> bool { self.fd != -1 }
    fn process_name(&self) -> Option<CString> { crate::osdep_linux::osdep_get_name(self.fd) }
    fn process_cwd(&self) -> Option<CString> { crate::osdep_linux::osdep_get_cwd(self.fd) }
    fn process_groups(&self) -> Option<(Option<pid_t>, Option<pid_t>)> {
        if self.fd == -1 { return None; }
        let group = unsafe { libc::tcgetpgrp(self.fd) };
        let leader = unsafe { libc::tcgetsid(self.fd) };
        Some(((group > 0).then_some(group), (leader > 0).then_some(leader)))
    }
    unsafe fn fork_process(&mut self, master: c_int, size: &winsize) -> pid_t {
        let result = unsafe { crate::compat::fdforkpty(master, None, Some(size)) };
        self.pid = result.pid;
        self.fd = if result.pid == -1 { -1 } else { result.master_fd };
        self.tty = result.tty_name;
        self.pid
    }
    fn take_job(&mut self, id: u32) -> bool {
        let Some((fd, pid)) = crate::job::job_transfer(id, Some(&mut self.tty)) else { return false };
        self.fd = fd;
        self.pid = pid;
        true
    }
    unsafe fn prepare_respawn(&mut self) {
        if self.fd != -1 {
            self.event.free();
            self.event = Stream::NONE;
            unsafe { close(self.fd) };
            self.fd = -1;
        }
        unsafe { self.reset_modes() };
        let extended_keys = crate::tmux::global_options.get().as_ref()
            .expect("global options are initialized").number(c"extended-keys");
        self.base.reinit_with_extended_keys(extended_keys);
        if let Some(context) = self.ictx.take() { unsafe { context.close() }; }
        self.flags &= !(crate::consts::PANE_STATUSREADY | crate::consts::PANE_STATUSDRAWN);
    }

    unsafe fn activate_spawned_process(&mut self, signal_mask: &sigset_t) {
        if self.flags & crate::window::PANE_EMPTY == 0 {
            let name = crate::xmalloc::xasprintf(c"tmux(%lu).%%%u", crate::fmt_args![unsafe { getpid() } as core::ffi::c_long, self.id]);
            unsafe { crate::ffi::utempter_add_record(self.fd, name.as_ptr() as *mut core::ffi::c_char); kill(getpid(), SIGCHLD); }
        }
        self.flags &= !crate::window::PANE_EXITED;
        unsafe { crate::ffi::sigprocmask(crate::consts::SIG_SETMASK, signal_mask, core::ptr::null_mut()); self.initialize_io(); }
    }
    fn write_terminal(&self, bytes: &[u8]) { self.event.write(bytes); }
    fn write_terminal_buffer(&self, bytes: &mut crate::reactor::ByteBuffer) { self.event.write_buffer(bytes); }
    unsafe fn write_key(&self, key: key_code) -> c_int { unsafe { crate::input::input_key(&self.screen_ref(), self.event, key) } }
    unsafe fn parse_bytes(&mut self, bytes: crate::reactor::ByteBuffer) { unsafe { output::parse_bytes(self, bytes) } }
    unsafe fn activate_transferred_process(&mut self) { unsafe { self.initialize_io() }; }
    fn pipe_process(&self) -> Option<pid_t> { (self.pipe_fd != -1).then_some(self.pipe_pid) }
    fn output_position(&self) -> crate::RustPaneOutputOffset { self.offset }
    fn unread_output_len(&self, position: &crate::RustPaneOutputOffset) -> usize {
        use crate::PaneOutputOffset;
        self.event.input_len().wrapping_sub(position.position().wrapping_sub(self.base_offset))
    }
    fn unread_output(&self, position: &crate::RustPaneOutputOffset) -> crate::reactor::ByteBuffer {
        use crate::PaneOutputOffset;
        let used = position.position().wrapping_sub(self.base_offset);
        let size = self.unread_output_len(position);
        self.event.with_input(|buffer| buffer.slice(used, size)).unwrap_or_default()
    }
    fn advance_output(&self, position: &mut crate::RustPaneOutputOffset, size: usize) {
        use crate::PaneOutputOffset;
        position.advance(size.min(self.unread_output_len(position)));
    }
    unsafe fn maintain_output(&mut self) { unsafe { output::maintain_output(self) } }

    fn record_process_exit(&mut self, status: c_int) -> bool {
        self.status = status;
        self.flags |= crate::consts::PANE_STATUSREADY | crate::window::PANE_EXITED;
        log_debug(c"%%%u exited", fmt_args![self.id]);
        self.destroy_ready()
    }
    unsafe fn finish_process(&mut self, notify: bool) -> bool {
        unsafe { process_exit::finish_process(self, notify) }
    }

    unsafe fn acknowledge_theme(&mut self) -> client_theme {
        let theme = unsafe { crate::window::window_pane_get_theme(Some(self)) };
        self.last_theme = theme;
        self.flags &= !PANE_THEMECHANGED;
        theme
    }
    unsafe fn set_theme_updates(&mut self, enabled: bool) {
        let mode = self.base.mode();
        self.base.set_mode(if enabled { mode | crate::consts::MODE_THEME_UPDATES }
            else { mode & !crate::consts::MODE_THEME_UPDATES });
        if enabled { unsafe { self.acknowledge_theme() }; }
        else { self.flags &= !PANE_THEMECHANGED; }
    }
    unsafe fn send_theme_update(&mut self) { unsafe { render::send_theme_update(self) }; }
    #[cfg(test)]
    fn theme(&self) -> client_theme { self.last_theme }

    fn option_changed(&mut self, name: &CStr) {
        if name == c"window-style" || name == c"window-active-style" {
            self.flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        } else if name.to_bytes().first() == Some(&b'@') {
            self.flags |= PANE_STYLECHANGED;
        } else if name == c"pane-colours" {
            self.reload_palette();
        } else if name == c"pane-scrollbars-style" {
            self.scrollbar_style = unsafe { pane_scrollbar_style_from_option(self.options_ref()) };
        }
    }
    fn palette_snapshot(&self) -> colour_palette { self.palette.clone() }
    fn palette_colour(&self, colour: c_int) -> c_int { RustColourEngine.get_palette(Some(&self.palette), colour) }
    fn set_palette_colour(&mut self, index: c_int, colour: c_int) -> c_int { RustColourEngine.set_palette(Some(&mut self.palette), index, colour) }
    fn clear_palette(&mut self) { RustColourEngine.clear_palette(Some(&mut self.palette)); }
    fn reload_palette(&mut self) { self.options_ref().clone().load_pane_colours(Some(&mut self.palette)); }
    fn set_palette_default(&mut self, foreground: bool, colour: c_int) {
        if foreground { self.palette.fg = colour; } else { self.palette.bg = colour; self.flags |= crate::window::PANE_THEMECHANGED; }
        self.flags |= PANE_STYLECHANGED;
    }
    fn status_line_width(&self) -> usize { self.status_size }
    fn publish_border_status(&mut self, width: usize, screen: crate::screen::RustScreen,
        ranges: style_ranges, expanded: std::ffi::CString) -> bool {
        let changed = !self.status_screen.is_initialized()
            || !crate::Grid::content_eq(screen.grid(), self.status_screen.grid());
        self.status_size = width;
        self.status_screen = screen;
        self.border_status_line = style_line_entry { ranges, expanded: Some(expanded) };
        changed
    }
    fn border_status_range(&self, x: u32) -> Option<style_range> {
        crate::style::style_ranges_get_range(&self.border_status_line.ranges, x)
    }
    #[cfg(test)]
    unsafe fn take_test_mode(&mut self) -> Option<Box<window_mode_entry>> {
        if self.modes.is_empty() { None } else { Some(self.modes.remove(0)) }
    }
    #[cfg(test)]
    unsafe fn insert_test_mode(&mut self, entry: Box<window_mode_entry>) { self.modes.insert(0, entry); }
    fn showing_base(&self) -> bool { self.screen == PaneScreen::Base || self.modes.is_empty() }
    fn mode_count(&self) -> usize { self.modes.len() }
    fn active_mode(&self) -> Option<&window_mode_entry> { self.modes.first().map(Box::as_ref) }
    fn active_mode_mut(&mut self) -> Option<crate::modes::ModeContext<'_>> { self.modes.first_mut().map(|entry| crate::modes::ModeContext::new(entry)) }
    fn find_mode_mut(&mut self, mode: WindowMode) -> Option<crate::modes::ModeContext<'_>> {
        self.modes.iter_mut().find(|entry| entry.mode() == mode).map(|entry| crate::modes::ModeContext::new(entry))
    }
    unsafe fn set_mode(&mut self, source: Option<RustWindowPaneWeak>, mode: WindowMode,
        target: Option<&cmd_find_state>, args: Option<&crate::args::RustArguments>) -> c_int {
        unsafe { modes::set_mode(self, source, mode, target, args) }
    }
    unsafe fn reset_mode(&mut self) { unsafe { modes::reset_mode(self) }; }
    unsafe fn reset_modes(&mut self) { unsafe { modes::reset_modes(self) }; }
    fn update_default_cursor(&mut self) { modes::update_default_cursor(self); }
    fn base(&self) -> &crate::screen::RustScreen {
        &self.base
    }
    fn adopt_popup_screen(&mut self, screen: RustScreen) {
        self.base = screen;
        self.base.resize(self.sx, self.sy, 1);
    }
    fn base_mut(&mut self) -> crate::screen::ScreenMut<'_> {
        crate::screen::ScreenMut::new(&mut self.base)
    }

    fn status_screen(&self) -> &crate::screen::RustScreen {
        &self.status_screen
    }

    /// Retains the recorded window context, including during pane transfers.
    fn window_context(&self) -> Option<WindowRef> {
        self.window.as_ref().and_then(WindowWeak::upgrade)
    }

    /// Borrows the terminal device name within the pane's fixed storage.
    fn terminal_name(&self) -> &core::ffi::CStr {
        core::ffi::CStr::from_bytes_until_nul(&self.tty)
            .expect("the pane terminal name is terminated")
    }

    /// The pane's shared option handle.
    fn options_ref(&self) -> &RustOptionsRef {
        self.options.as_ref().expect("options are initialized")
    }

    /// Borrows the shown screen, falling back to the base when the mode list is empty.
    fn screen_ref(&self) -> ScreenBorrow<'_> {
        self.try_screen_ref().expect("shown mode has a screen")
    }

    /// Borrows the shown screen when the current mode has initialized it.
    fn try_screen_ref(&self) -> Option<ScreenBorrow<'_>> {
        if self.screen == PaneScreen::Base || self.modes.is_empty() {
            return Some(ScreenBorrow::Owned(&self.base));
        }
        self.modes.first()?.shown_screen()
    }

    #[cfg(test)]
    fn set_window_context(&mut self, window: Option<&WindowRef>) {
        self.window = window.map(WindowRef::downgrade);
    }
}

