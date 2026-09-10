//! Capabilities used by pane consumers.

use crate::{
    PaneActivityState, PaneBorderCache, PaneCommandState, PaneControlColours, PaneExitState,
    PaneGeometryState, PaneIdentity, PaneOutputBaseState, PaneResizeQueue, PaneScrollbar,
    PaneScrollbarStyleState, PaneSearchState, PaneStatusLineState, PaneStyleCache, PaneThemeState,
};

/// State and engine capabilities used to interact with a window pane.
///
/// Pane allocations and their shared lifetime are managed by owning handles.
///
/// ```
/// use tmux_c2rs::{PaneGeometry, WindowPane};
///
/// fn describe(pane: &impl WindowPane) -> (u32, PaneGeometry) {
///     (pane.pane_id(), pane.geometry())
/// }
/// ```
pub trait WindowPane:
    PaneIdentity
    + PaneActivityState
    + PaneGeometryState
    + PaneScrollbar
    + PaneCommandState
    + PaneExitState
    + PaneOutputBaseState
    + PaneResizeQueue
    + PaneStyleCache
    + PaneThemeState
    + PaneStatusLineState
    + PaneSearchState
    + PaneBorderCache
    + PaneControlColours
    + PaneScrollbarStyleState
{
    /// Observes this allocation without retaining it. Unowned implementations return None.
    /// The observation retains its allocation identity after destruction and never
    /// resolves another allocation with the same pane ID.
    fn observation(&self) -> Option<crate::types::RustWindowPaneWeak>;

    /// Borrows the pane's server flags.
    fn flags(&self) -> &core::ffi::c_int;

    /// Mutably borrows the pane's server flags.
    fn flags_mut(&mut self) -> &mut core::ffi::c_int;

    /// Borrows the pane's process ID.
    fn pid(&self) -> &crate::types::pid_t;

    /// Mutably borrows the pane's process ID.
    fn pid_mut(&mut self) -> &mut crate::types::pid_t;

    /// Borrows the pane's fixed terminal device-name storage.
    fn tty(&self) -> &[u8; 32];

    /// Mutably borrows the pane's fixed terminal device-name storage.
    fn tty_mut(&mut self) -> &mut [u8; 32];

    /// Borrows the pane's pseudo-terminal descriptor.
    fn fd(&self) -> &core::ffi::c_int;

    /// Mutably borrows the pane's pseudo-terminal descriptor.
    fn fd_mut(&mut self) -> &mut core::ffi::c_int;

    /// Borrows the pane's pipe descriptor.
    fn pipe_fd(&self) -> &core::ffi::c_int;

    /// Mutably borrows the pane's pipe descriptor.
    fn pipe_fd_mut(&mut self) -> &mut core::ffi::c_int;

    /// Borrows the pane's pipe process ID.
    fn pipe_pid(&self) -> &crate::types::pid_t;

    /// Mutably borrows the pane's pipe process ID.
    fn pipe_pid_mut(&mut self) -> &mut crate::types::pid_t;
    /// Borrows the pane's optional option handle.
    fn options(&self) -> &Option<crate::options::RustOptionsRef>;

    /// Mutably borrows the pane's optional option handle.
    fn options_mut(&mut self) -> &mut Option<crate::options::RustOptionsRef>;

    /// Borrows the pane's pseudo-terminal stream handle.
    fn event(&self) -> &crate::reactor::Stream;

    /// Mutably borrows the pane's pseudo-terminal stream handle.
    fn event_mut(&mut self) -> &mut crate::reactor::Stream;

    /// Borrows the pane's output read position.
    fn offset(&self) -> &crate::pane_output::RustPaneOutputOffset;

    /// Mutably borrows the pane's output read position.
    fn offset_mut(&mut self) -> &mut crate::pane_output::RustPaneOutputOffset;

    /// Borrows the pane's resize timer.
    fn resize_timer(&self) -> &crate::reactor::TimerHandle;

    /// Mutably borrows the pane's resize timer.
    fn resize_timer_mut(&mut self) -> &mut crate::reactor::TimerHandle;

    /// Borrows the pane's synchronized-output timer.
    fn sync_timer(&self) -> &crate::reactor::TimerHandle;

    /// Mutably borrows the pane's synchronized-output timer.
    fn sync_timer_mut(&mut self) -> &mut crate::reactor::TimerHandle;

    /// Borrows the pane's input parser handle.
    fn ictx(&self) -> &Option<crate::input::InputCtxRef>;

    /// Mutably borrows the pane's input parser handle.
    fn ictx_mut(&mut self) -> &mut Option<crate::input::InputCtxRef>;

    /// Borrows the pane's pipe stream handle.
    fn pipe_event(&self) -> &crate::reactor::Stream;

    /// Mutably borrows the pane's pipe stream handle.
    fn pipe_event_mut(&mut self) -> &mut crate::reactor::Stream;

    /// Borrows the pane's pipe read position.
    fn pipe_offset(&self) -> &crate::pane_output::RustPaneOutputOffset;

    /// Mutably borrows the pane's pipe read position.
    fn pipe_offset_mut(&mut self) -> &mut crate::pane_output::RustPaneOutputOffset;
    /// Borrows the pane's colour palette.
    fn palette(&self) -> &crate::types::colour_palette;

    /// Mutably borrows the pane's colour palette.
    fn palette_mut(&mut self) -> &mut crate::types::colour_palette;

    /// Borrows the pane's border status line.
    fn border_status_line(&self) -> &crate::types::style_line_entry;

    /// Mutably borrows the pane's border status line.
    fn border_status_line_mut(&mut self) -> &mut crate::types::style_line_entry;

    /// Borrows the pane's screen selector.
    fn shown(&self) -> &crate::types::PaneScreen;

    /// Mutably borrows the pane's screen selector.
    fn shown_mut(&mut self) -> &mut crate::types::PaneScreen;

    /// Borrows the pane's base screen.
    fn base(&self) -> &crate::screen::RustScreen;

    /// Mutably borrows the pane's base screen.
    fn base_mut(&mut self) -> &mut crate::screen::RustScreen;

    /// Borrows the pane's status screen.
    fn status_screen(&self) -> &crate::screen::RustScreen;

    /// Mutably borrows the pane's status screen.
    fn status_screen_mut(&mut self) -> &mut crate::screen::RustScreen;

    /// Borrows the pane's mode stack.
    fn modes(&self) -> &crate::types::window_modes;

    /// Mutably borrows the pane's mode stack.
    fn modes_mut(&mut self) -> &mut crate::types::window_modes;

    /// Borrows the pane's visible ranges.
    fn r(&self) -> &crate::types::visible_ranges;

    /// Mutably borrows the pane's visible ranges.
    fn r_mut(&mut self) -> &mut crate::types::visible_ranges;
    /// Retains the recorded window context, including during pane transfers.
    fn window_context(&self) -> Option<crate::types::WindowRef>;

    /// Records window context without retaining the window allocation.
    fn set_window_context(&mut self, window: Option<&crate::types::WindowRef>);

    /// Borrows the terminal device name within the pane's fixed storage.
    fn terminal_name(&self) -> &core::ffi::CStr;

    /// Borrows the initialized pane option handle.
    fn options_ref(&self) -> &crate::options::RustOptionsRef;

    /// Borrows the shown screen, falling back to the base when the mode list is empty.
    fn screen_ref(&self) -> crate::types::ScreenBorrow<'_>;

    /// Borrows the shown screen when the current mode has initialized it.
    fn try_screen_ref(&self) -> Option<crate::types::ScreenBorrow<'_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_window_pane<T: WindowPane>() {}

    #[test]
    fn server_pane_implements_the_aggregate_contract() {
        assert_window_pane::<crate::types::window_pane>();
    }
}
