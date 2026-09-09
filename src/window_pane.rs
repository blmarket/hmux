use std::cell::{RefCell, UnsafeCell};
use std::rc::{Rc, Weak};

use crate::grid::grid_default_cell;
use crate::handle_registry::HandleRegistration;
use crate::screen::RustScreen;
use crate::types::*;
use crate::{
    PaneStyleCache, PaneStyleCells, RustPaneActivityState, RustPaneBorderCache,
    RustPaneCommandState, RustPaneControlColours, RustPaneExitState, RustPaneGeometryState,
    RustPaneIdentity, RustPaneOutputBaseState, RustPaneOutputOffset, RustPaneResizeQueue,
    RustPaneScrollbar, RustPaneScrollbarStyleState, RustPaneSearchState, RustPaneStatusLineState,
    RustPaneStyleCache, RustPaneThemeState,
};

/// A pane allocation owned through a strong reference.
pub struct RustWindowPane {
    registration: RefCell<Option<HandleRegistration<RustWindowPaneWeak>>>,
    window: RefCell<Option<WindowWeak>>,
    id: u32,
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
    id: u32,
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
            .field("id", &self.id)
            .field("alive", &self.is_alive())
            .finish()
    }
}

impl RustWindowPaneRef {
    pub(crate) fn from_pane(pane: Box<window_pane>) -> Self {
        let id = crate::PaneIdentity::pane_id(&*pane);
        let pane = unsafe { Box::from_raw(Box::into_raw(pane).cast::<UnsafeCell<window_pane>>()) };
        Self(Rc::new(RustWindowPane {
            registration: RefCell::new(None),
            window: RefCell::new(None),
            id,
            pane,
        }))
    }

    /// The immutable identity under which this pane was registered.
    pub fn pane_id(&self) -> u32 {
        self.0.id
    }

    pub(crate) fn id(&self) -> u32 {
        self.pane_id()
    }

    pub fn downgrade(&self) -> RustWindowPaneWeak {
        RustWindowPaneWeak {
            id: self.0.id,
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

    pub(crate) fn register_window(&self, window: Option<WindowWeak>) {
        *self.0.window.borrow_mut() = window;
    }

    #[cfg(test)]
    pub(crate) fn window(&self) -> Option<WindowRef> {
        self.0.window.borrow().as_ref()?.upgrade()
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
    pub(crate) unsafe fn as_pane(&self) -> &window_pane {
        unsafe { &*self.0.pane.get() }
    }

    /// # Safety
    /// Exclude other payload access for this borrow, including through
    /// observations and callbacks. The registered ID must not change.
    pub(crate) unsafe fn as_pane_mut(&mut self) -> &mut window_pane {
        unsafe { &mut *self.0.pane.get() }
    }

    /// # Safety
    /// Exclude mutation for this borrow, including through other owners,
    /// observations, and callbacks.
    pub unsafe fn get(&self) -> Option<&window_pane> {
        Some(unsafe { self.as_pane() })
    }

    /// # Safety
    /// Exclude other payload access for this borrow, including through
    /// observations and callbacks. The registered ID must not change.
    pub unsafe fn get_mut(&mut self) -> Option<&mut window_pane> {
        Some(unsafe { self.as_pane_mut() })
    }
}

impl RustWindowPaneWeak {
    /// The immutable identity under which this pane was registered.
    pub fn pane_id(&self) -> u32 {
        self.id
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
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
        pane.window.borrow().as_ref()?.upgrade()
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
    pub(crate) unsafe fn as_pane(&self) -> &window_pane {
        unsafe { self.get() }.expect("the pane has been removed")
    }

    /// # Safety
    /// Prevent mutation and destruction for the returned borrow, including
    /// through other observations and reentrant callbacks.
    pub unsafe fn get(&self) -> Option<&window_pane> {
        unsafe { self.as_ptr().as_ref() }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.allocation, &other.allocation)
    }

    /// # Safety
    /// Exclude all other payload access and destruction for this borrow,
    /// including through observations and callbacks. The ID must not change.
    pub(crate) unsafe fn as_pane_mut(&mut self) -> &mut window_pane {
        unsafe { self.get_mut() }.expect("the pane has been removed")
    }

    /// # Safety
    /// Exclude all other payload access and destruction for this borrow,
    /// including through observations and callbacks. The ID must not change.
    pub unsafe fn get_mut(&mut self) -> Option<&mut window_pane> {
        unsafe { self.as_mut_ptr().as_mut() }
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
    identity: crate::pane_identity::RustPaneIdentity,
    activity: crate::pane_activity::RustPaneActivityState,
    /// The recorded window context, preserved while a pane is between lists.
    /// Registered membership is tracked separately by the pane owner.
    recorded_window: Option<WindowWeak>,
    options: Option<crate::options::RustOptionsRef>,
    geometry: crate::pane_geometry::RustPaneGeometryState,
    flags: core::ffi::c_int,
    scrollbar: crate::pane_scrollbar::RustPaneScrollbar,
    command: crate::pane_command::RustPaneCommandState,
    pid: pid_t,
    tty: [u8; 32],
    exit_state: crate::pane_exit::RustPaneExitState,
    fd: core::ffi::c_int,
    event: Stream,
    offset: crate::pane_output::RustPaneOutputOffset,
    output_base: crate::pane_output_base::RustPaneOutputBaseState,
    resize_queue: crate::pane_resize::RustPaneResizeQueue,
    resize_timer: TimerHandle,
    sync_timer: TimerHandle,
    ictx: Option<InputCtxRef>,
    style_cache: crate::pane_style_cache::RustPaneStyleCache,
    palette: colour_palette,
    theme: crate::pane_theme::RustPaneThemeState,
    border_status_line: style_line_entry,
    pipe_fd: core::ffi::c_int,
    pipe_pid: pid_t,
    pipe_event: Stream,
    pipe_offset: crate::pane_output::RustPaneOutputOffset,
    /// Which screen the pane is showing: its own, or the one the mode at the
    /// front of its mode list draws on.
    shown: PaneScreen,
    base: RustScreen,
    status_screen: RustScreen,
    status_line: crate::pane_status_line::RustPaneStatusLineState,
    modes: window_modes,
    search: crate::pane_search::RustPaneSearchState,
    border_cache: crate::pane_border_cache::RustPaneBorderCache,
    control_colours: crate::pane_control_colours::RustPaneControlColours,
    scrollbar_style_state: crate::pane_scrollbar_style::RustPaneScrollbarStyleState,
    r: visible_ranges,
}

impl crate::pane_identity::PaneIdentity for window_pane {
    fn pane_id(&self) -> u32 {
        crate::pane_identity::PaneIdentity::pane_id(&self.identity)
    }

    fn set_pane_id(&mut self, id: u32) {
        crate::pane_identity::PaneIdentity::set_pane_id(&mut self.identity, id);
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
    fn styles(&self) -> crate::pane_style_cache::PaneStyleCells {
        crate::pane_style_cache::PaneStyleCache::styles(&self.style_cache)
    }

    fn set_styles(&mut self, styles: crate::pane_style_cache::PaneStyleCells) {
        crate::pane_style_cache::PaneStyleCache::set_styles(&mut self.style_cache, styles);
    }
}

impl crate::pane_border_cache::PaneBorderCache for window_pane {
    fn get(
        &self,
        kind: crate::pane_border_cache::PaneBorderKind,
    ) -> Option<crate::types::grid_cell> {
        crate::pane_border_cache::PaneBorderCache::get(&self.border_cache, kind)
    }

    fn insert(
        &mut self,
        kind: crate::pane_border_cache::PaneBorderKind,
        cell: crate::types::grid_cell,
    ) {
        crate::pane_border_cache::PaneBorderCache::insert(&mut self.border_cache, kind, cell);
    }

    fn clear(&mut self) {
        crate::pane_border_cache::PaneBorderCache::clear(&mut self.border_cache);
    }
}

impl crate::pane_control_colours::PaneControlColours for window_pane {
    fn colours(&self) -> crate::pane_control_colours::PaneControlColourPair {
        crate::pane_control_colours::PaneControlColours::colours(&self.control_colours)
    }

    fn set_colours(&mut self, colours: crate::pane_control_colours::PaneControlColourPair) {
        crate::pane_control_colours::PaneControlColours::set_colours(
            &mut self.control_colours,
            colours,
        );
    }

    fn clear(&mut self) {
        crate::pane_control_colours::PaneControlColours::clear(&mut self.control_colours);
    }
}

impl crate::pane_theme::PaneThemeState for window_pane {
    fn theme(&self) -> crate::types::client_theme {
        crate::pane_theme::PaneThemeState::theme(&self.theme)
    }

    fn set_theme(&mut self, theme: crate::types::client_theme) {
        crate::pane_theme::PaneThemeState::set_theme(&mut self.theme, theme);
    }

    fn replace(&mut self, theme: crate::types::client_theme) -> bool {
        crate::pane_theme::PaneThemeState::replace(&mut self.theme, theme)
    }
}

impl crate::pane_search::PaneSearchState for window_pane {
    fn query(&self) -> Option<&core::ffi::CStr> {
        crate::pane_search::PaneSearchState::query(&self.search)
    }

    fn is_regex(&self) -> bool {
        crate::pane_search::PaneSearchState::is_regex(&self.search)
    }

    fn set(&mut self, query: &core::ffi::CStr, regex: bool) {
        crate::pane_search::PaneSearchState::set(&mut self.search, query, regex);
    }

    fn clear(&mut self) {
        crate::pane_search::PaneSearchState::clear(&mut self.search);
    }

    fn matches(&self, query: &core::ffi::CStr, regex: bool) -> bool {
        crate::pane_search::PaneSearchState::matches(&self.search, query, regex)
    }
}

impl crate::pane_scrollbar::PaneScrollbar for window_pane {
    fn slider(&self) -> crate::pane_scrollbar::PaneScrollbarSlider {
        crate::pane_scrollbar::PaneScrollbar::slider(&self.scrollbar)
    }

    fn set_slider(&mut self, slider: crate::pane_scrollbar::PaneScrollbarSlider) {
        crate::pane_scrollbar::PaneScrollbar::set_slider(&mut self.scrollbar, slider);
    }

    fn clear(&mut self) {
        crate::pane_scrollbar::PaneScrollbar::clear(&mut self.scrollbar);
    }
}

impl crate::pane_geometry::PaneGeometryState for window_pane {
    fn geometry(&self) -> crate::pane_geometry::PaneGeometry {
        crate::pane_geometry::PaneGeometryState::geometry(&self.geometry)
    }

    fn set_geometry(&mut self, geometry: crate::pane_geometry::PaneGeometry) {
        crate::pane_geometry::PaneGeometryState::set_geometry(&mut self.geometry, geometry);
    }

    fn set_position(&mut self, x: core::ffi::c_int, y: core::ffi::c_int) {
        crate::pane_geometry::PaneGeometryState::set_position(&mut self.geometry, x, y);
    }

    fn set_size(&mut self, size: crate::pane_resize::PaneSize) {
        crate::pane_geometry::PaneGeometryState::set_size(&mut self.geometry, size);
    }
}

impl crate::pane_exit::PaneExitState for window_pane {
    fn exit_status(&self) -> core::ffi::c_int {
        crate::pane_exit::PaneExitState::exit_status(&self.exit_state)
    }

    fn set_exit_status(&mut self, status: core::ffi::c_int) {
        crate::pane_exit::PaneExitState::set_exit_status(&mut self.exit_state, status);
    }

    fn death_time(&self) -> timeval {
        crate::pane_exit::PaneExitState::death_time(&self.exit_state)
    }

    fn set_death_time(&mut self, time: timeval) {
        crate::pane_exit::PaneExitState::set_death_time(&mut self.exit_state, time);
    }
}

impl crate::pane_command::PaneCommandState for window_pane {
    fn pane_command(&self) -> crate::pane_command::PaneCommand {
        crate::pane_command::PaneCommandState::pane_command(&self.command)
    }

    fn set_pane_command(&mut self, command: &crate::pane_command::PaneCommand) {
        crate::pane_command::PaneCommandState::set_pane_command(&mut self.command, command);
    }

    fn clear_pane_command(&mut self) {
        crate::pane_command::PaneCommandState::clear_pane_command(&mut self.command);
    }
}

impl crate::pane_output_base::PaneOutputBaseState for window_pane {
    fn output_base(&self) -> usize {
        crate::pane_output_base::PaneOutputBaseState::output_base(&self.output_base)
    }

    fn set_output_base(&mut self, position: usize) {
        crate::pane_output_base::PaneOutputBaseState::set_output_base(
            &mut self.output_base,
            position,
        );
    }
}

impl crate::pane_activity::PaneActivityState for window_pane {
    fn activity_point(&self) -> u_int {
        crate::pane_activity::PaneActivityState::activity_point(&self.activity)
    }

    fn mark_active_at(&mut self, point: u_int) {
        crate::pane_activity::PaneActivityState::mark_active_at(&mut self.activity, point);
    }
}

impl crate::pane_status_line::PaneStatusLineState for window_pane {
    fn status_line_width(&self) -> usize {
        crate::pane_status_line::PaneStatusLineState::status_line_width(&self.status_line)
    }

    fn set_status_line_width(&mut self, width: usize) {
        crate::pane_status_line::PaneStatusLineState::set_status_line_width(
            &mut self.status_line,
            width,
        );
    }
}

impl crate::pane_scrollbar_style::PaneScrollbarStyleState for window_pane {
    fn scrollbar_style(&self) -> crate::pane_scrollbar_style::PaneScrollbarStyle {
        crate::pane_scrollbar_style::PaneScrollbarStyleState::scrollbar_style(
            &self.scrollbar_style_state,
        )
    }

    fn set_scrollbar_style(&mut self, style: crate::pane_scrollbar_style::PaneScrollbarStyle) {
        crate::pane_scrollbar_style::PaneScrollbarStyleState::set_scrollbar_style(
            &mut self.scrollbar_style_state,
            style,
        );
    }
}

impl window_pane {
    pub(crate) fn new() -> Self {
        window_pane {
            identity: RustPaneIdentity::default(),
            activity: RustPaneActivityState::default(),
            recorded_window: None,
            options: None,
            geometry: RustPaneGeometryState::default(),
            flags: 0,
            scrollbar: RustPaneScrollbar::default(),
            command: RustPaneCommandState::default(),
            pid: 0,
            tty: [0; 32],
            exit_state: RustPaneExitState::default(),
            fd: -1,
            event: Stream::NONE,
            offset: RustPaneOutputOffset::default(),
            output_base: RustPaneOutputBaseState::default(),
            resize_queue: RustPaneResizeQueue::default(),
            resize_timer: TimerHandle(0),
            sync_timer: TimerHandle(0),
            ictx: None,
            style_cache: {
                let mut cache = RustPaneStyleCache::default();
                cache.set_styles(PaneStyleCells {
                    normal: grid_default_cell,
                    active: grid_default_cell,
                });
                cache
            },
            palette: colour_palette {
                fg: 8,
                bg: 8,
                palette: None,
                default_palette: None,
            },
            theme: RustPaneThemeState::default(),
            border_status_line: style_line_entry::default(),
            pipe_fd: -1,
            pipe_pid: 0,
            pipe_event: Stream::NONE,
            pipe_offset: RustPaneOutputOffset::default(),
            shown: PaneScreen::Base,
            base: RustScreen::default(),
            status_screen: RustScreen::default(),
            status_line: RustPaneStatusLineState::default(),
            modes: window_modes::new(),
            search: RustPaneSearchState::default(),
            border_cache: RustPaneBorderCache::default(),
            control_colours: RustPaneControlColours::default(),
            scrollbar_style_state: RustPaneScrollbarStyleState::default(),
            r: visible_ranges::default(),
        }
    }
}

impl crate::WindowPane for window_pane {
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
        &self.shown
    }
    fn shown_mut(&mut self) -> &mut crate::types::PaneScreen {
        &mut self.shown
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
        self.recorded_window.as_ref().and_then(WindowWeak::upgrade)
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
        if self.shown == PaneScreen::Base || self.modes.is_empty() {
            return Some(ScreenBorrow::Owned(&self.base));
        }
        let mode = self.modes.first()?;
        if !mode.screen_ready {
            return None;
        }
        match &mode.state {
            WindowModeState::Clock(data) => Some(ScreenBorrow::Owned(&data.screen)),
            WindowModeState::Copy(data) | WindowModeState::View(data) => {
                Some(ScreenBorrow::Shared(data.screen.borrow()))
            }
            WindowModeState::Buffer(_)
            | WindowModeState::Client(_)
            | WindowModeState::Tree(_)
            | WindowModeState::Customize(_) => Some(ScreenBorrow::Shared({
                mode.mode_tree_ref.as_ref()?.screen_handle().borrow()
            })),
            WindowModeState::None => None,
        }
    }

    fn set_window_context(&mut self, window: Option<&WindowRef>) {
        self.recorded_window = window.map(WindowRef::downgrade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaneGeometryState;
    use crate::window::PANE_STYLECHANGED;

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
            let pane: &mut window_pane = owner.get_mut().unwrap();
            pane.set_size(crate::PaneSize {
                width: 80,
                height: 24,
            });
            *pane.flags_mut() = PANE_STYLECHANGED;
        }
        unsafe {
            let pane: &window_pane = retained.get().unwrap();
            assert_eq!(pane.geometry().width, 80);
            assert_eq!(pane.geometry().height, 24);
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
