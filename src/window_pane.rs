use crate::entity_id::next_entity_id;
use crate::ffi::{gethostname, getpid, kill, close, utempter_remove_record};
use crate::handle_registry::HandleRegistry;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::reactor::Timer;
use crate::style::{ColourEngine, RustColourEngine, pane_scrollbar_style_from_option, style_ranges_free};
use crate::window::{window_pane_set_window, window_pane_set_window_ref, PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::{WindowPane, PaneIdentity, PaneGeometryState, PaneScrollbarStyleState, PaneSearchState, PaneResizeQueue, PaneCommandState};
use std::cell::Cell;
use libc::SIGCHLD;
use std::cell::{RefCell, UnsafeCell};
use std::rc::{Rc, Weak};

use crate::grid::grid_default_cell;
use crate::handle_registry::HandleRegistration;
use crate::screen::RustScreen;
use crate::{Screen, PaneOutputOffset, WindowDimensionsState};
use crate::fmt_args;
use crate::log::log_debug;
use crate::types::*;
use crate::{
    PaneCommand, PaneControlColourPair, PaneGeometry, PaneScrollbarSlider,
    PaneScrollbarStyle, PaneSize, PaneStyleCells,
};
use core::ffi::{c_int, CStr};
use std::ffi::CString;

/// A pane allocation owned through a strong reference.
struct RustWindowPane {
    registration: RefCell<Option<HandleRegistration<RustWindowPaneWeak>>>,
    pane: Box<UnsafeCell<window_pane>>,
}

/// A strong pane owner, held by its window or a transfer operation.
///
/// Command targets and callbacks use `RustWindowPaneWeak` so they cannot
/// postpone destruction. Payload access remains explicit and unsafe.
///
/// ```compile_fail
/// use tmux_c2rs::types::RustWindowPaneRef;
/// fn shared(owner: &RustWindowPaneRef) -> &impl tmux_c2rs::WindowPane { owner }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::types::RustWindowPaneRef;
/// fn exclusive(owner: &mut RustWindowPaneRef) -> &mut impl tmux_c2rs::WindowPane { owner }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::types::window_pane;
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::window_pane::window_pane;
/// ```
///
/// ```compile_fail
/// let owner = tmux_c2rs::types::RustWindowPaneRef::detached();
/// let _ = owner.as_ptr();
/// ```
///
/// ```compile_fail
/// let owner = tmux_c2rs::types::RustWindowPaneRef::detached();
/// let pane = unsafe { owner.get().unwrap() };
/// let _ = &pane.base;
/// ```
#[derive(Clone)]
pub struct RustWindowPaneRef(Rc<RustWindowPane>);

/// A non-owning pane observation that detects destruction.
///
/// Cloning retains neither the pane payload nor its window. Borrowing through
/// an observation is unsafe because it does not keep the allocation alive.
#[derive(Clone)]
pub struct RustWindowPaneWeak {
    allocation: Weak<RustWindowPane>,
}

impl PartialEq for RustWindowPaneWeak {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for RustWindowPaneWeak {}

impl std::fmt::Debug for RustWindowPaneWeak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustWindowPaneWeak")
            .field("id", &self.upgrade().map(|pane| pane.pane_id()))
            .field("alive", &self.is_alive())
            .finish()
    }
}

impl RustWindowPaneRef {
    fn from_pane(pane: Box<UnsafeCell<window_pane>>) -> Self {
        Self(Rc::new_cyclic(|allocation| {
            unsafe { (*pane.get()).observation = Some(RustWindowPaneWeak { allocation: allocation.clone() }) };
            RustWindowPane { registration: RefCell::new(None), pane }
        }))
    }

    /// The immutable identity under which this pane was registered.
    pub fn pane_id(&self) -> u32 {
        unsafe { (*self.0.pane.get()).id }
    }

    pub(crate) fn id(&self) -> u32 {
        self.pane_id()
    }

    pub fn downgrade(&self) -> RustWindowPaneWeak {
        RustWindowPaneWeak {
            allocation: Rc::downgrade(&self.0),
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    fn install_registration(&self, registration: HandleRegistration<RustWindowPaneWeak>) {
        *self.0.registration.borrow_mut() = Some(registration);
    }

    fn unregister(&self) {
        self.0.registration.borrow_mut().take();
    }

    #[cfg(test)]
    pub(crate) fn window(&self) -> Option<WindowRef> {
        self.downgrade().window()
    }

    /// Consumes the sole owner at the server's pane destruction point.
    fn into_pane(self) -> Box<UnsafeCell<window_pane>> {
        Rc::try_unwrap(self.0)
            .ok()
            .expect("pane destruction requires sole ownership")
            .pane
    }

    /// # Safety
    /// Exclude mutation for this borrow, including through other owners,
    /// observations, and callbacks.
    pub(crate) unsafe fn as_pane(&self) -> &(dyn WindowPane + 'static) {
        unsafe { &*self.0.pane.get() }
    }

    /// # Safety
    /// Exclude other payload access for this borrow, including through
    /// observations and callbacks. The registered ID must not change.
    pub(crate) unsafe fn as_pane_mut(&mut self) -> &mut (dyn WindowPane + 'static) {
        unsafe { &mut *self.0.pane.get() }
    }

    /// # Safety
    /// Exclude mutation for this borrow, including through other owners,
    /// observations, and callbacks.
    pub unsafe fn get(&self) -> Option<&(dyn WindowPane + 'static)> {
        Some(unsafe { self.as_pane() })
    }

    /// # Safety
    /// Exclude other payload access for this borrow, including through
    /// observations and callbacks. The registered ID must not change.
    pub unsafe fn get_mut(&mut self) -> Option<&mut (dyn WindowPane + 'static)> {
        Some(unsafe { self.as_pane_mut() })
    }
}

impl RustWindowPaneWeak {
    /// Returns the ID while the pane is alive.
    pub fn pane_id(&self) -> Option<u32> {
        self.upgrade().map(|pane| pane.pane_id())
    }

    pub(crate) fn id(&self) -> u32 {
        self.pane_id().expect("the pane is alive")
    }

    pub fn upgrade(&self) -> Option<RustWindowPaneRef> {
        self.allocation.upgrade().map(RustWindowPaneRef)
    }

    pub fn is_alive(&self) -> bool {
        self.allocation.strong_count() != 0
    }

    /// The current window, including changes made by pane moves and swaps.
    pub(crate) fn window(&self) -> Option<WindowRef> {
        let pane = self.allocation.upgrade()?;
        unsafe { (*pane.pane.get()).window.as_ref()?.upgrade() }
    }

    pub(crate) fn listed_window(&self) -> Option<WindowRef> {
        self.window().filter(|window| window.contains_pane(self))
    }

    /// # Safety
    /// Prevent mutation and destruction for the returned borrow, including
    /// through other observations and reentrant callbacks.
    pub(crate) unsafe fn as_pane(&self) -> &(dyn WindowPane + 'static) {
        unsafe { self.get() }.expect("the pane has been removed")
    }

    /// # Safety
    /// Prevent mutation and destruction for the returned borrow, including
    /// through other observations and reentrant callbacks.
    pub unsafe fn get(&self) -> Option<&(dyn WindowPane + 'static)> {
        let allocation = self.allocation.upgrade()?;
        Some(unsafe { &*allocation.pane.get() })
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.allocation, &other.allocation)
    }

    /// # Safety
    /// Exclude all other payload access and destruction for this borrow,
    /// including through observations and callbacks. The ID must not change.
    pub(crate) unsafe fn as_pane_mut(&mut self) -> &mut (dyn WindowPane + 'static) {
        unsafe { self.get_mut() }.expect("the pane has been removed")
    }

    /// # Safety
    /// Exclude all other payload access and destruction for this borrow,
    /// including through observations and callbacks. The ID must not change.
    pub unsafe fn get_mut(&mut self) -> Option<&mut (dyn WindowPane + 'static)> {
        let allocation = self.allocation.upgrade()?;
        Some(unsafe { &mut *allocation.pane.get() })
    }
}

/// Pane storage accessed through [`crate::WindowPane`] and its capabilities.
#[derive(Default)]
struct window_pane {
    observation: Option<RustWindowPaneWeak>,
    id: u32,
    active_point: u_int,
    /// The window backlink, retained during transfers and teardown.
    window: Option<WindowWeak>,
    options: Option<crate::options::RustOptionsRef>,
    sx: u_int,
    sy: u_int,
    xoff: c_int,
    yoff: c_int,
    flags: core::ffi::c_int,
    sb_slider_y: u_int,
    sb_slider_h: u_int,
    argv: Vec<CString>,
    shell: Option<CString>,
    cwd: Option<CString>,
    pid: pid_t,
    tty: [u8; 32],
    status: c_int,
    dead_time: timeval,
    fd: core::ffi::c_int,
    event: Stream,
    offset: crate::pane_output::RustPaneOutputOffset,
    base_offset: usize,
    resize_queue: crate::pane_resize::RustPaneResizeQueue,
    resize_timer: TimerHandle,
    sync_timer: TimerHandle,
    ictx: Option<InputCtxRef>,
    cached_gc: grid_cell,
    cached_active_gc: grid_cell,
    palette: colour_palette,
    last_theme: client_theme,
    border_status_line: style_line_entry,
    pipe_fd: core::ffi::c_int,
    pipe_pid: pid_t,
    pipe_event: Stream,
    pipe_offset: crate::pane_output::RustPaneOutputOffset,
    /// Which screen the pane is showing: its own, or the one the mode at the
    /// front of its mode list draws on.
    screen: PaneScreen,
    base: RustScreen,
    status_screen: RustScreen,
    status_size: usize,
    modes: window_modes,
    searchstr: Option<CString>,
    searchregex: bool,
    control_bg: Option<c_int>,
    control_fg: Option<c_int>,
    scrollbar_style: PaneScrollbarStyle,
}

impl crate::pane_identity::PaneIdentity for window_pane {
    fn pane_id(&self) -> u32 {
        self.id
    }

}


impl crate::pane_style_cache::PaneStyleCache for window_pane {
    fn styles(&self) -> PaneStyleCells {
        PaneStyleCells { cached_gc: self.cached_gc, cached_active_gc: self.cached_active_gc }
    }
    unsafe fn refresh_styles(&mut self) -> PaneStyleCells {
        if self.flags & PANE_STYLECHANGED != 0 { unsafe { render::refresh_styles(self) }; }
        crate::PaneStyleCache::styles(self)
    }

}

impl crate::pane_control_colours::PaneControlColours for window_pane {
    fn colours(&self) -> PaneControlColourPair {
        PaneControlColourPair { control_fg: self.control_fg, control_bg: self.control_bg }
    }
    fn set_colours(&mut self, colours: PaneControlColourPair) {
        self.control_fg = colours.control_fg; self.control_bg = colours.control_bg;
    }
    fn clear(&mut self) { self.control_fg = None; self.control_bg = None; }
}

impl crate::pane_search::PaneSearchState for window_pane {
    fn query(&self) -> Option<&CStr> { self.searchstr.as_deref() }
    fn is_regex(&self) -> bool { self.searchregex }
    fn set(&mut self, query: &CStr, regex: bool) {
        self.searchstr = Some(query.to_owned()); self.searchregex = regex;
    }
    fn clear(&mut self) { self.searchstr = None; self.searchregex = false; }
    fn matches(&self, query: &CStr, regex: bool) -> bool {
        self.searchregex == regex && self.searchstr.as_deref() == Some(query)
    }
}

impl crate::pane_scrollbar::PaneScrollbar for window_pane {
    fn slider(&self) -> PaneScrollbarSlider {
        PaneScrollbarSlider { sb_slider_y: self.sb_slider_y, sb_slider_h: self.sb_slider_h }
    }
    fn set_slider(&mut self, slider: PaneScrollbarSlider) { self.sb_slider_y = slider.sb_slider_y; self.sb_slider_h = slider.sb_slider_h; }
    fn clear(&mut self) { self.sb_slider_y = 0; self.sb_slider_h = 0; }
}

impl crate::pane_geometry::PaneGeometryState for window_pane {
    fn geometry(&self) -> PaneGeometry {
        PaneGeometry { xoff: self.xoff, yoff: self.yoff, sx: self.sx, sy: self.sy }
    }
    fn set_position(&mut self, x: c_int, y: c_int) { self.xoff = x; self.yoff = y; }
}

impl crate::pane_exit::PaneExitState for window_pane {
    fn exit_status(&self) -> c_int { self.status }
    fn death_time(&self) -> timeval { self.dead_time }
}

impl crate::pane_command::PaneCommandState for window_pane {
    fn pane_command(&self) -> PaneCommand {
        PaneCommand { argv: self.argv.clone(), shell: self.shell.clone(), cwd: self.cwd.clone() }
    }
    fn set_pane_command(&mut self, command: &PaneCommand) {
        self.argv.clone_from(&command.argv); self.shell.clone_from(&command.shell); self.cwd.clone_from(&command.cwd);
    }
    fn clear_pane_command(&mut self) { self.argv.clear(); self.shell = None; self.cwd = None; }
}


impl crate::pane_activity::PaneActivityState for window_pane {
    fn activity_point(&self) -> u_int { self.active_point }
    fn mark_active_at(&mut self, point: u_int) { self.active_point = point; }
}


impl crate::pane_scrollbar_style::PaneScrollbarStyleState for window_pane {
    fn scrollbar_style(&self) -> PaneScrollbarStyle { self.scrollbar_style }
    fn set_scrollbar_style(&mut self, style: PaneScrollbarStyle) { self.scrollbar_style = style; }
}

impl window_pane {
    pub(crate) fn new() -> Self {
        window_pane {
            fd: -1,
            pipe_fd: -1,
            cached_gc: grid_default_cell,
            cached_active_gc: grid_default_cell,
            palette: colour_palette {
                fg: 8,
                bg: 8,
                palette: None,
                default_palette: None,
            },
            ..Default::default()
        }
    }
}

impl crate::WindowPane for window_pane {
    fn observation(&self) -> Option<RustWindowPaneWeak> {
        self.observation.clone()
    }

    fn flags(&self) -> &core::ffi::c_int {
        &self.flags
    }
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
    unsafe fn resize(&mut self, size: PaneSize) {
        let old = PaneSize { width: self.sx, height: self.sy };
        if old == size { return; }
        self.stop_sync();
        self.resize_queue.record(old, size);
        self.sx = size.width;
        self.sy = size.height;
        let reflow = !self.base.is_alternate();
        unsafe { crate::screen::screen_resize(&mut self.base, size.width, size.height, reflow as c_int) };
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
            PaneTestSetup::Descriptor(fd) => self.fd = fd,
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
        unsafe { crate::screen::screen_reinit(&mut self.base) };
        if let Some(context) = self.ictx.take() { unsafe { context.close() }; }
        self.flags &= !(crate::consts::PANE_STATUSREADY | crate::consts::PANE_STATUSDRAWN);
    }
    unsafe fn close_process(&mut self) {
        if self.fd != -1 {
            unsafe { utempter_remove_record(self.fd); kill(getpid(), SIGCHLD); }
            self.event.free();
            self.event = Stream::NONE;
            unsafe { close(self.fd) };
            self.fd = -1;
        }
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
    unsafe fn parse_output(&mut self) {
        let input = self.unread_output(&self.offset);
        let size = input.len();
        unsafe { self.parse_bytes(input) };
        let mut offset = self.offset;
        self.advance_output(&mut offset, size);
        self.offset = offset;
    }
    fn destroy_ready(&self) -> bool {
        let mut remaining: c_int = 0;
        if self.pipe_fd != -1 && self.pipe_event.output_len() != 0 { return false; }
        if unsafe { crate::ffi::ioctl(self.fd, crate::window::FIONREAD as core::ffi::c_ulong, &raw mut remaining) } != -1 && remaining > 0 { return false; }
        self.flags & crate::window::PANE_EXITED != 0
    }
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
    fn base_mut(&mut self) -> &mut crate::screen::RustScreen {
        &mut self.base
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

    fn set_window_context(&mut self, window: Option<&WindowRef>) {
        self.window = window.map(WindowRef::downgrade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaneGeometryState;
    use crate::window::PANE_STYLECHANGED;

    #[test]
    fn observations_remain_bound_to_the_original_allocation() {
        let owner = RustWindowPaneRef::from_pane(Box::default());
        let observed = unsafe { owner.get().unwrap().observation().unwrap() };
        assert_eq!(observed, owner.downgrade());
        let id = owner.pane_id();
        drop(owner);
        let replacement = RustWindowPaneRef::from_pane(Box::default());
        assert_eq!(id, replacement.pane_id());
        assert_ne!(observed, replacement.downgrade());
        assert!(!observed.is_alive());
        assert!(unsafe { observed.get() }.is_none());
        assert!(window_pane::default().observation().is_none());
    }

    #[test]
    fn weak_upgrade_shares_the_strong_owner_without_changing_identity() {
        let owner = RustWindowPaneRef::from_pane(Box::default());
        let observed = owner.downgrade();
        let retained = observed.upgrade().unwrap();
        assert!(owner.ptr_eq(&retained));
        drop(owner);
        assert!(observed.is_alive());
        drop(retained);
        assert!(!observed.is_alive());
        assert!(observed.upgrade().is_none());
    }

    #[test]
    fn observers_expire_when_the_owner_is_dropped() {
        let mut owner = RustWindowPaneRef::from_pane(Box::default());
        let mut retained = owner.downgrade();
        unsafe {
            let pane: &mut dyn WindowPane = owner.get_mut().unwrap();
            pane.configure_test(PaneTestSetup::Size(PaneSize { width: 80, height: 24 }));
            *pane.flags_mut() = PANE_STYLECHANGED;
        }
        unsafe {
            let pane: &dyn WindowPane = retained.get().unwrap();
            assert_eq!(pane.geometry().sx, 80);
            assert_eq!(pane.geometry().sy, 24);
            assert_eq!(*pane.flags(), PANE_STYLECHANGED);
        }
        drop(owner);
        unsafe {
            assert!(retained.get().is_none());
            assert!(retained.get_mut().is_none());
        }
        assert!(!retained.is_alive());
    }
}

pub(crate) struct GlobalPaneIndex {
    panes: HandleRegistry<RustWindowPaneWeak>,
}

impl GlobalPaneIndex {
    pub(crate) fn find(&self, id: u_int) -> Option<RustWindowPaneWeak> {
        self.panes
            .get(id as usize)
            .filter(RustWindowPaneWeak::is_alive)
    }

    pub(crate) fn ids(&self) -> Vec<u_int> {
        self.panes
            .keys()
            .into_iter()
            .map(|id| id as u_int)
            .collect()
    }
}

impl GlobalPaneIndex {
    pub(crate) fn register(&self, reference: RustWindowPaneRef) -> RustWindowPaneRef {
        let registration = self
            .panes
            .register(reference.pane_id() as usize, reference.downgrade());
        reference.install_registration(registration);
        reference
    }
}

impl RustWindowPaneRef {
    /// Creates an unregistered pane without process or stream resources.
    pub fn detached() -> Self {
        Self::from_pane(Box::new(UnsafeCell::new(window_pane { fd: -1, pipe_fd: -1, ..Default::default() })))
    }

    pub(crate) fn register_owner(self) -> Self {
        GLOBAL_PANE_INDEX.with(|index| index.register(self))
    }
}

pub(crate) const GLOBAL_PANE_INDEX: crate::server_state::LocalField<GlobalPaneIndex> =
    crate::server_state::LocalField::new(|state| &state.global_pane_index);
pub(crate) const next_window_pane_id: crate::server_state::LocalField<Cell<Option<u_int>>> =
    crate::server_state::LocalField::new(|state| &state.next_window_pane_id);
pub(crate) unsafe fn window_pane_create(
    w: &mut window,
    sx: u_int,
    sy: u_int,
    hlimit: u_int,
) -> RustWindowPaneRef {
    unsafe {
        let fresh2 = next_entity_id(&next_window_pane_id);
        let mut host: [core::ffi::c_char; 65] = [0; 65];
        let wp_box = Box::new(UnsafeCell::new(window_pane::new()));
        let wp = wp_box.get();
        window_pane_set_window(&mut *wp, Some(w));
        (*wp).options = Some(RustOptionsEngine.create(Some((*w).options_ref())));
        *(*wp).flags_mut() = PANE_STYLECHANGED;
        (*wp).id = fresh2;
        (*wp).fd = -(1 as core::ffi::c_int);
        (*wp).sx = sx;
        (*wp).sy = sy;
        (*wp).pipe_fd = -(1 as core::ffi::c_int);
        let scrollbar_style = pane_scrollbar_style_from_option((*wp).options_ref());
        (*wp).set_scrollbar_style(scrollbar_style);
        RustColourEngine.init_palette(&mut (*wp).palette);
        (*wp).reload_palette();
        *(*wp).base_mut() = RustScreen::new_with_server_options(sx, sy, hlimit);
        (*wp).screen = PaneScreen::Base;
        (*wp).update_default_cursor();
        (*wp).status_screen =
            RustScreen::new_with_server_options(1 as u_int, 1 as u_int, 0 as u_int);
        if gethostname(
            &raw mut host as *mut core::ffi::c_char,
            size_of::<[core::ffi::c_char; 65]>() as size_t,
        ) == 0 as core::ffi::c_int
        {
            (*wp)
                .base_mut()
                .set_title(CStr::from_ptr(host.as_ptr()), 0 as core::ffi::c_int);
        }
        RustWindowPaneRef::from_pane(wp_box).register_owner()
    }
}
/// Tears down and frees the pane at the end of destruction. Observers do not
/// postpone resource or allocation release.
pub(crate) unsafe fn window_pane_destroy(pane: RustWindowPaneRef) {
    unsafe {
        let wp = &mut *pane.0.pane.get();
        (*wp).reset_modes();
        PaneSearchState::clear(wp);
        wp.close_process();
        if let Some(ictx) = wp.ictx.take() {
            ictx.close();
        }
        if wp.pipe_fd != -(1 as core::ffi::c_int) {
            wp.pipe_event.free();
            close(wp.pipe_fd);
            wp.pipe_fd = -1;
        }
        wp.resize_timer.disarm();
        wp.sync_timer.disarm();
        wp.resize_queue.clear();
        pane.unregister();
        if let Some(oo) = wp.options.take() {
            RustOptionsEngine.destroy(oo);
        }
        wp.clear_pane_command();
        RustColourEngine.free_palette(Some(&mut wp.palette));
        style_ranges_free(&mut wp.border_status_line.ranges);
        wp.border_status_line.expanded = None;
        window_pane_set_window_ref(wp, None);
        drop(pane.into_pane());
    }
}
impl GlobalPaneIndex {
    pub(crate) fn new() -> Self {
        Self {
            panes: HandleRegistry::new(),
        }
    }
}

/// Unique test construction storage. No observation exists until this owner is
/// consumed, so its capability borrows cannot expose a shared owning handle.
#[cfg(test)]
pub(crate) struct PaneAllocation(Box<UnsafeCell<window_pane>>);

#[cfg(test)]
impl Default for PaneAllocation {
    fn default() -> Self {
        Self(Box::new(UnsafeCell::new(window_pane { fd: -1, pipe_fd: -1, ..Default::default() })))
    }
}

#[cfg(test)]
impl PaneAllocation {
    pub(crate) fn set_output_position(&mut self, position: usize) { self.0.get_mut().offset = crate::RustPaneOutputOffset::at(position); }
    pub(crate) fn set_pane_id(&mut self, id: u32) { self.0.get_mut().id = id; }
    pub(crate) fn into_owner(self) -> RustWindowPaneRef { RustWindowPaneRef::from_pane(self.0) }
}

#[cfg(test)]
impl std::ops::Deref for PaneAllocation {
    type Target = dyn WindowPane;
    fn deref(&self) -> &Self::Target { unsafe { &*self.0.get() } }
}

#[cfg(test)]
impl std::ops::DerefMut for PaneAllocation {
    fn deref_mut(&mut self) -> &mut Self::Target { self.0.get_mut() }
}

impl window_pane {
    fn expire_sync(&mut self) {
        self.sync_timer.disarm();
        if self.base.mode() & crate::screen::MODE_SYNC != 0 {
            self.base.set_mode(self.base.mode() & !crate::screen::MODE_SYNC);
            self.flags |= crate::window::PANE_REDRAW;
        }
    }
}

#[cfg(test)]
pub(crate) fn sync_timer_for_test(pane: &dyn WindowPane) -> TimerHandle {
    let owner = pane.observation().unwrap().upgrade().unwrap();
    unsafe { (*owner.0.pane.get()).sync_timer }
}

#[cfg(test)]
pub(crate) fn expire_sync_for_test(pane: &mut dyn WindowPane) {
    let owner = pane.observation().unwrap().upgrade().unwrap();
    unsafe { (&mut *owner.0.pane.get()).expire_sync() };
}

impl RustWindowPaneWeak {
    /// Delivers a coalesced process resize when the retry delay has elapsed.
    /// # Safety
    /// Exclude conflicting pane access and window changes during delivery.
    pub(crate) unsafe fn deliver_pending_resize(&self) {
        let Some(owner) = self.upgrade() else { return };
        let pane = unsafe { &mut *owner.0.pane.get() };
        if pane.resize_queue.is_empty() || pane.resize_timer.is_armed() { return; }
        if !pane.resize_timer.is_set() {
            let observed = self.clone();
            pane.resize_timer.set_callback(move || {
                if let Some(owner) = observed.upgrade() {
                    unsafe { (*owner.0.pane.get()).resize_timer.disarm() };
                }
            });
        }
        let step = pane.resize_queue.next_step().expect("the resize queue is not empty");
        drop(owner);
        unsafe { self.send_process_resize(step.size.width, step.size.height) };
        if let Some(owner) = self.upgrade() {
            unsafe { (*owner.0.pane.get()).resize_timer.arm(timeval::from_usecs(step.retry_after.as_micros() as __suseconds_t)) };
        }
    }
}

#[cfg(test)]
pub(crate) fn resize_timer_for_test(pane: &dyn WindowPane) -> TimerHandle {
    let owner = pane.observation().unwrap().upgrade().unwrap();
    unsafe { (*owner.0.pane.get()).resize_timer }
}

mod output;
mod pipe;
pub(crate) use pipe::PanePipePair;

impl RustWindowPaneWeak {
    unsafe fn payload_mut(&mut self) -> Option<&mut window_pane> {
        let allocation = self.allocation.upgrade()?;
        Some(unsafe { &mut *allocation.pane.get() })
    }
}
fn on_pane_owned(
    id: u_int,
    body: impl Fn(&mut window_pane) + 'static,
) -> std::rc::Rc<dyn Fn(Stream)> {
    let observed = crate::window::window_pane_find_by_id(id);
    std::rc::Rc::new(move |_stream| unsafe {
        if let Some(mut pane) = observed.clone()
            && pane.listed_window().is_some()
            && let Some(wp) = pane.payload_mut()
        {
            body(wp);
        }
    })
}

/// The same, for the callback a failed stream makes.
fn on_pane_error_owned(
    id: u_int,
    body: impl Fn(&mut window_pane) + 'static,
) -> std::rc::Rc<dyn Fn(Stream, core::ffi::c_short)> {
    let observed = crate::window::window_pane_find_by_id(id);
    std::rc::Rc::new(move |_stream, _what| unsafe {
        if let Some(mut pane) = observed.clone()
            && pane.listed_window().is_some()
            && let Some(wp) = pane.payload_mut()
        {
            body(wp);
        }
    })
}

pub(crate) fn on_pane(id: u_int, body: impl Fn(&mut dyn WindowPane) + 'static) -> Rc<dyn Fn(Stream)> {
    on_pane_owned(id, move |pane| body(pane))
}
pub(crate) fn on_pane_error(id: u_int, body: impl Fn(&mut dyn WindowPane) + 'static) -> Rc<dyn Fn(Stream, core::ffi::c_short)> {
    on_pane_error_owned(id, move |pane| body(pane))
}

#[cfg(test)]
pub(crate) unsafe fn install_pipe_for_test(pane: &mut dyn WindowPane, fd: c_int) {
    let owner = pane.observation().unwrap().upgrade().unwrap();
    unsafe { (*owner.0.pane.get()).pipe_fd = fd };
}

mod io;
#[cfg(test)]
pub(crate) use io::send_line;

#[cfg(test)]
pub enum PaneTestSetup {
    Descriptor(c_int),
    Styles(PaneStyleCells),
    Palette(colour_palette),
    Geometry(PaneGeometry),
    Size(PaneSize),
    Options(Option<RustOptionsRef>),
    Stream(Stream),
    Parser(Option<crate::input::InputCtxRef>),
    Terminal([u8; 32]),
}

impl RustWindowPaneWeak {
pub(crate) unsafe fn send_process_resize(&self, sx: u_int, sy: u_int) {
    unsafe {
        let pane = self;
        let Some(allocation) = pane.allocation.upgrade() else { return };
        let wp = &*allocation.pane.get();
        let mut ws = winsize::default();
        if !wp.process_active() {
            return;
        }
        let Some(window) = pane.window() else { return };
        let w = window.as_window();
        if !w.panes.iter().any(|owner| owner.downgrade().ptr_eq(pane)) {
            return;
        }
        log_debug(
            c"%s: %%%u resize to %u,%u",
            fmt_args![c"window_pane_send_resize", wp.pane_id(), sx, sy],
        );
        ws.ws_col = sx as core::ffi::c_ushort;
        ws.ws_row = sy as core::ffi::c_ushort;
        ws.ws_xpixel =
            w.dimensions().pixels.width.wrapping_mul(ws.ws_col as u_int) as core::ffi::c_ushort;
        ws.ws_ypixel = w
            .dimensions()
            .pixels
            .height
            .wrapping_mul(ws.ws_row as u_int) as core::ffi::c_ushort;
        if crate::ffi::ioctl(wp.fd, crate::window::TIOCSWINSZ as core::ffi::c_ulong, &raw mut ws)
            == -(1 as core::ffi::c_int)
        {
            crate::log::fatal(c"ioctl failed", crate::fmt_args![]);
        }
    }
}

}

impl window_pane {
    unsafe fn initialize_io(&mut self) {
        crate::tmux::setblocking(self.fd, 0);
        self.event = Stream::new(self.fd,
            Some(on_pane_owned(self.id, output::window_pane_read_callback)), None,
            Some(on_pane_error_owned(self.id, output::window_pane_error_callback)));
        if self.event.is_none() { crate::log::fatalx(c"out of memory", crate::fmt_args![]); }
        self.ictx = Some(unsafe { crate::input::InputCtxRef::create(crate::input::InputOwner::Pane(self.id), self.event) });
        self.event.enable(crate::reactor::Interest::ReadWrite);
    }
}

mod modes;

/// Which screen a pane is showing.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
enum PaneScreen {
    /// The pane's own screen, which it holds itself.
    #[default]
    Base,
    /// The screen the mode at the front of the pane's mode list draws on.
    Mode,
}

mod render;
mod process_exit;
