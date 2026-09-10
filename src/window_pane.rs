use crate::entity_id::next_entity_id;
use crate::ffi::{gethostname, getpid, kill, close, utempter_remove_record};
use crate::handle_registry::HandleRegistry;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::reactor::Timer;
use crate::style::{ColourEngine, RustColourEngine, pane_scrollbar_style_from_option, style_ranges_free};
use crate::window::{window_pane_set_window, window_pane_set_window_ref, window_pane_reset_mode_all, window_pane_default_cursor, PANE_STYLECHANGED};
use crate::{WindowPane, PaneIdentity, PaneGeometryState, PaneScrollbarStyleState, PaneSearchState, PaneResizeQueue, PaneCommandState};
use std::cell::Cell;
use libc::SIGCHLD;
use std::cell::{RefCell, UnsafeCell};
use std::rc::{Rc, Weak};

use crate::grid::grid_default_cell;
use crate::handle_registry::HandleRegistration;
use crate::screen::RustScreen;
use crate::types::*;
use crate::{
    PaneBorderKind, PaneCommand, PaneControlColourPair, PaneGeometry, PaneScrollbarSlider,
    PaneScrollbarStyle, PaneSize, PaneStyleCells,
};
use core::ffi::{c_int, CStr};
use std::ffi::CString;

/// A pane allocation owned through a strong reference.
pub struct RustWindowPane {
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
    pub(crate) fn from_pane(pane: Box<window_pane>) -> Self {
        let pane = unsafe { Box::from_raw(Box::into_raw(pane).cast::<UnsafeCell<window_pane>>()) };
        Self(Rc::new_cyclic(|allocation| {
            unsafe { (*pane.get()).observation = Some(RustWindowPaneWeak { allocation: allocation.clone() }) };
            RustWindowPane { registration: RefCell::new(None), pane }
        }))
    }

    /// The immutable identity under which this pane was registered.
    pub fn pane_id(&self) -> u32 {
        unsafe { (*self.as_ptr()).id }
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

    pub(crate) fn register(&self, registration: HandleRegistration<RustWindowPaneWeak>) {
        *self.0.registration.borrow_mut() = Some(registration);
    }

    pub(crate) fn unregister(&self) {
        self.0.registration.borrow_mut().take();
    }

    #[cfg(test)]
    pub(crate) fn window(&self) -> Option<WindowRef> {
        self.downgrade().window()
    }

    pub(crate) fn as_ptr(&self) -> *const window_pane {
        self.0.pane.get().cast_const()
    }

    pub(crate) fn as_mut_ptr(&self) -> *mut window_pane {
        self.0.pane.get()
    }

    /// Consumes the sole owner at the server's pane destruction point.
    pub(crate) fn into_pane(self) -> Box<UnsafeCell<window_pane>> {
        Rc::try_unwrap(self.0)
            .ok()
            .expect("pane destruction requires sole ownership")
            .pane
    }

    /// # Safety
    /// Exclude mutation for this borrow, including through other owners,
    /// observations, and callbacks.
    pub(crate) unsafe fn as_pane(&self) -> &dyn WindowPane {
        unsafe { &*self.0.pane.get() }
    }

    /// # Safety
    /// Exclude other payload access for this borrow, including through
    /// observations and callbacks. The registered ID must not change.
    pub(crate) unsafe fn as_pane_mut(&mut self) -> &mut dyn WindowPane {
        unsafe { &mut *self.0.pane.get() }
    }

    /// # Safety
    /// Exclude mutation for this borrow, including through other owners,
    /// observations, and callbacks.
    pub unsafe fn get(&self) -> Option<&dyn WindowPane> {
        Some(unsafe { self.as_pane() })
    }

    /// # Safety
    /// Exclude other payload access for this borrow, including through
    /// observations and callbacks. The registered ID must not change.
    pub unsafe fn get_mut(&mut self) -> Option<&mut dyn WindowPane> {
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

    /// The compatibility address, or null after destruction.
    /// Obtaining it grants no payload borrow and does not retain the allocation.
    pub(crate) fn as_ptr(&self) -> *const window_pane {
        self.as_mut_ptr().cast_const()
    }

    /// The compatibility address, or null after destruction.
    /// Obtaining it grants no payload borrow and does not retain the allocation.
    pub(crate) fn as_mut_ptr(&self) -> *mut window_pane {
        self.allocation
            .upgrade()
            .map_or(std::ptr::null_mut(), |pane| pane.pane.get())
    }

    /// # Safety
    /// Prevent mutation and destruction for the returned borrow, including
    /// through other observations and reentrant callbacks.
    pub(crate) unsafe fn as_pane(&self) -> &dyn WindowPane {
        unsafe { self.get() }.expect("the pane has been removed")
    }

    /// # Safety
    /// Prevent mutation and destruction for the returned borrow, including
    /// through other observations and reentrant callbacks.
    pub unsafe fn get(&self) -> Option<&dyn WindowPane> {
        unsafe { self.as_ptr().as_ref().map(|pane| pane as &dyn WindowPane) }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.allocation, &other.allocation)
    }

    /// # Safety
    /// Exclude all other payload access and destruction for this borrow,
    /// including through observations and callbacks. The ID must not change.
    pub(crate) unsafe fn as_pane_mut(&mut self) -> &mut dyn WindowPane {
        unsafe { self.get_mut() }.expect("the pane has been removed")
    }

    /// # Safety
    /// Exclude all other payload access and destruction for this borrow,
    /// including through observations and callbacks. The ID must not change.
    pub unsafe fn get_mut(&mut self) -> Option<&mut dyn WindowPane> {
        unsafe { self.as_mut_ptr().as_mut().map(|pane| pane as &mut dyn WindowPane) }
    }
}

/// Pane storage accessed through [`crate::WindowPane`] and its capabilities.
///
/// ```compile_fail
/// use tmux_c2rs::types::window_pane;
/// let mut pane = window_pane::default();
/// pane.flags = 1;
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::types::window_pane;
/// let pane = window_pane::default();
/// let _ = &pane.base;
/// ```
#[derive(Default)]
#[repr(C)]
pub struct window_pane {
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
    border_gc: Option<grid_cell>,
    active_border_gc: Option<grid_cell>,
    control_bg: Option<c_int>,
    control_fg: Option<c_int>,
    scrollbar_style: PaneScrollbarStyle,
    r: visible_ranges,
}

impl crate::pane_identity::PaneIdentity for window_pane {
    fn pane_id(&self) -> u32 {
        self.id
    }

    fn set_pane_id(&mut self, id: u32) {
        self.id = id;
    }
}

impl crate::pane_resize::PaneResizeQueue for window_pane {
    fn is_empty(&self) -> bool {
        crate::pane_resize::PaneResizeQueue::is_empty(&self.resize_queue)
    }

    fn clear(&mut self) {
        crate::pane_resize::PaneResizeQueue::clear(&mut self.resize_queue);
    }

    fn record(&mut self, old: crate::pane_resize::PaneSize, new: crate::pane_resize::PaneSize) {
        crate::pane_resize::PaneResizeQueue::record(&mut self.resize_queue, old, new);
    }

    fn next_step(&mut self) -> Option<crate::pane_resize::PaneResizeStep> {
        crate::pane_resize::PaneResizeQueue::next_step(&mut self.resize_queue)
    }
}

impl crate::pane_style_cache::PaneStyleCache for window_pane {
    fn styles(&self) -> PaneStyleCells {
        PaneStyleCells { cached_gc: self.cached_gc, cached_active_gc: self.cached_active_gc }
    }
    fn set_styles(&mut self, styles: PaneStyleCells) {
        self.cached_gc = styles.cached_gc;
        self.cached_active_gc = styles.cached_active_gc;
    }
}

impl crate::pane_border_cache::PaneBorderCache for window_pane {
    fn get(&self, kind: PaneBorderKind) -> Option<grid_cell> {
        match kind { PaneBorderKind::Normal => self.border_gc, PaneBorderKind::Active => self.active_border_gc }
    }
    fn insert(&mut self, kind: PaneBorderKind, cell: grid_cell) {
        match kind { PaneBorderKind::Normal => self.border_gc = Some(cell), PaneBorderKind::Active => self.active_border_gc = Some(cell) }
    }
    fn clear(&mut self) { self.border_gc = None; self.active_border_gc = None; }
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

impl crate::pane_theme::PaneThemeState for window_pane {
    fn theme(&self) -> client_theme { self.last_theme }
    fn set_theme(&mut self, theme: client_theme) { self.last_theme = theme; }
    fn replace(&mut self, theme: client_theme) -> bool {
        let changed = self.last_theme != theme;
        self.last_theme = theme;
        changed
    }
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
    fn set_geometry(&mut self, geometry: PaneGeometry) {
        self.xoff = geometry.xoff; self.yoff = geometry.yoff; self.sx = geometry.sx; self.sy = geometry.sy;
    }
    fn set_position(&mut self, x: c_int, y: c_int) { self.xoff = x; self.yoff = y; }
    fn set_size(&mut self, size: PaneSize) { self.sx = size.width; self.sy = size.height; }
}

impl crate::pane_exit::PaneExitState for window_pane {
    fn exit_status(&self) -> c_int { self.status }
    fn set_exit_status(&mut self, status: c_int) { self.status = status; }
    fn death_time(&self) -> timeval { self.dead_time }
    fn set_death_time(&mut self, time: timeval) { self.dead_time = time; }
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

impl crate::pane_output_base::PaneOutputBaseState for window_pane {
    fn output_base(&self) -> usize { self.base_offset }
    fn set_output_base(&mut self, position: usize) { self.base_offset = position; }
}

impl crate::pane_activity::PaneActivityState for window_pane {
    fn activity_point(&self) -> u_int { self.active_point }
    fn mark_active_at(&mut self, point: u_int) { self.active_point = point; }
}

impl crate::pane_status_line::PaneStatusLineState for window_pane {
    fn status_line_width(&self) -> usize { self.status_size }
    fn set_status_line_width(&mut self, width: usize) { self.status_size = width; }
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
    fn pid(&self) -> &crate::types::pid_t {
        &self.pid
    }
    fn pid_mut(&mut self) -> &mut crate::types::pid_t {
        &mut self.pid
    }
    fn tty(&self) -> &[u8; 32] {
        &self.tty
    }
    fn tty_mut(&mut self) -> &mut [u8; 32] {
        &mut self.tty
    }
    fn fd(&self) -> &core::ffi::c_int {
        &self.fd
    }
    fn fd_mut(&mut self) -> &mut core::ffi::c_int {
        &mut self.fd
    }
    fn pipe_fd(&self) -> &core::ffi::c_int {
        &self.pipe_fd
    }
    fn pipe_fd_mut(&mut self) -> &mut core::ffi::c_int {
        &mut self.pipe_fd
    }
    fn pipe_pid(&self) -> &crate::types::pid_t {
        &self.pipe_pid
    }
    fn pipe_pid_mut(&mut self) -> &mut crate::types::pid_t {
        &mut self.pipe_pid
    }
    fn options(&self) -> &Option<crate::options::RustOptionsRef> {
        &self.options
    }
    fn options_mut(&mut self) -> &mut Option<crate::options::RustOptionsRef> {
        &mut self.options
    }

    fn event(&self) -> &crate::reactor::Stream {
        &self.event
    }
    fn event_mut(&mut self) -> &mut crate::reactor::Stream {
        &mut self.event
    }

    fn offset(&self) -> &crate::pane_output::RustPaneOutputOffset {
        &self.offset
    }
    fn offset_mut(&mut self) -> &mut crate::pane_output::RustPaneOutputOffset {
        &mut self.offset
    }

    fn resize_timer(&self) -> &crate::reactor::TimerHandle {
        &self.resize_timer
    }
    fn resize_timer_mut(&mut self) -> &mut crate::reactor::TimerHandle {
        &mut self.resize_timer
    }

    fn sync_timer(&self) -> &crate::reactor::TimerHandle {
        &self.sync_timer
    }
    fn sync_timer_mut(&mut self) -> &mut crate::reactor::TimerHandle {
        &mut self.sync_timer
    }

    fn ictx(&self) -> &Option<crate::input::InputCtxRef> {
        &self.ictx
    }
    fn ictx_mut(&mut self) -> &mut Option<crate::input::InputCtxRef> {
        &mut self.ictx
    }

    fn pipe_event(&self) -> &crate::reactor::Stream {
        &self.pipe_event
    }
    fn pipe_event_mut(&mut self) -> &mut crate::reactor::Stream {
        &mut self.pipe_event
    }

    fn pipe_offset(&self) -> &crate::pane_output::RustPaneOutputOffset {
        &self.pipe_offset
    }
    fn pipe_offset_mut(&mut self) -> &mut crate::pane_output::RustPaneOutputOffset {
        &mut self.pipe_offset
    }
    fn palette(&self) -> &crate::types::colour_palette {
        &self.palette
    }
    fn palette_mut(&mut self) -> &mut crate::types::colour_palette {
        &mut self.palette
    }

    fn border_status_line(&self) -> &crate::types::style_line_entry {
        &self.border_status_line
    }
    fn border_status_line_mut(&mut self) -> &mut crate::types::style_line_entry {
        &mut self.border_status_line
    }

    fn shown(&self) -> &crate::types::PaneScreen {
        &self.screen
    }
    fn shown_mut(&mut self) -> &mut crate::types::PaneScreen {
        &mut self.screen
    }

    fn base(&self) -> &crate::screen::RustScreen {
        &self.base
    }
    fn base_mut(&mut self) -> &mut crate::screen::RustScreen {
        &mut self.base
    }

    fn status_screen(&self) -> &crate::screen::RustScreen {
        &self.status_screen
    }
    fn status_screen_mut(&mut self) -> &mut crate::screen::RustScreen {
        &mut self.status_screen
    }

    fn modes(&self) -> &crate::types::window_modes {
        &self.modes
    }
    fn modes_mut(&mut self) -> &mut crate::types::window_modes {
        &mut self.modes
    }

    fn r(&self) -> &crate::types::visible_ranges {
        &self.r
    }
    fn r_mut(&mut self) -> &mut crate::types::visible_ranges {
        &mut self.r
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
        let mode = self.modes.first()?;
        match mode.screen.as_ref()? {
            ModeScreen::Clock => {
                let WindowModeState::Clock(data) = &mode.state else { return None };
                Some(ScreenBorrow::Owned(&data.screen))
            }
            ModeScreen::Shared(screen) => Some(ScreenBorrow::Shared(screen.borrow())),
        }
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
        let owner = RustWindowPaneRef::from_pane(Box::new(window_pane::default()));
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
            pane.set_size(crate::PaneSize {
                width: 80,
                height: 24,
            });
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
        assert!(retained.as_ptr().is_null());
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
    pub(crate) fn register(&self, pane: Box<window_pane>) -> RustWindowPaneRef {
        let reference = RustWindowPaneRef::from_pane(pane);
        let registration = self
            .panes
            .register(reference.pane_id() as usize, reference.downgrade());
        reference.register(registration);
        reference
    }
}

impl RustWindowPaneRef {
    pub(crate) fn new(pane: Box<window_pane>) -> Self {
        GLOBAL_PANE_INDEX.with(|index| index.register(pane))
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
        let mut wp_box = Box::new(window_pane::new());
        let wp = &raw mut *wp_box;
        window_pane_set_window(&mut *wp, Some(w));
        *(*wp).options_mut() = Some(RustOptionsEngine.create(Some((*w).options_ref())));
        *(*wp).flags_mut() = PANE_STYLECHANGED;
        (*wp).set_pane_id(fresh2);
        *(*wp).fd_mut() = -(1 as core::ffi::c_int);
        (*wp).set_size(PaneSize {
            width: sx,
            height: sy,
        });
        *(*wp).pipe_fd_mut() = -(1 as core::ffi::c_int);
        let scrollbar_style = pane_scrollbar_style_from_option((*wp).options_ref());
        (*wp).set_scrollbar_style(scrollbar_style);
        RustColourEngine.init_palette((*wp).palette_mut());
        ((*wp).options_ref()).load_pane_colours(Some((*wp).palette_mut()));
        *(*wp).base_mut() = RustScreen::new_with_server_options(sx, sy, hlimit);
        *(*wp).shown_mut() = PaneScreen::Base;
        window_pane_default_cursor(&mut *wp);
        *(*wp).status_screen_mut() =
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
        RustWindowPaneRef::new(wp_box)
    }
}
/// Tears down and frees the pane at the end of destruction. Observers do not
/// postpone resource or allocation release.
pub(crate) unsafe fn window_pane_destroy(pane: RustWindowPaneRef) {
    unsafe {
        let wp = &mut *pane.as_mut_ptr();
        window_pane_reset_mode_all(&mut *wp);
        PaneSearchState::clear(wp);
        if *wp.fd() != -(1 as core::ffi::c_int) {
            utempter_remove_record(*wp.fd());
            kill(getpid(), SIGCHLD);
            wp.event().free();
            close(*wp.fd());
            *wp.fd_mut() = -1;
        }
        if let Some(ictx) = wp.ictx_mut().take() {
            ictx.close();
        }
        wp.r_mut().ranges.clear();
        if *wp.pipe_fd() != -(1 as core::ffi::c_int) {
            wp.pipe_event().free();
            close(*wp.pipe_fd());
            *wp.pipe_fd_mut() = -1;
        }
        wp.resize_timer_mut().disarm();
        wp.sync_timer_mut().disarm();
        PaneResizeQueue::clear(wp);
        pane.unregister();
        if let Some(oo) = wp.options_mut().take() {
            RustOptionsEngine.destroy(oo);
        }
        wp.clear_pane_command();
        RustColourEngine.free_palette(Some(wp.palette_mut()));
        style_ranges_free(&mut wp.border_status_line_mut().ranges);
        wp.border_status_line_mut().expanded = None;
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
