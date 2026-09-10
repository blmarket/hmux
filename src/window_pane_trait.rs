//! Capabilities used by pane consumers.

use crate::{
    PaneActivityState, PaneBorderCache, PaneCommandState, PaneControlColours, PaneExitState,
    PaneGeometryState, PaneIdentity, PaneScrollbar,
    PaneScrollbarStyleState, PaneSearchState, PaneStyleCache, PaneThemeState,
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
    + PaneStyleCache
    + PaneThemeState
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

    /// Borrows the pane's optional option handle.
    fn options(&self) -> &Option<crate::options::RustOptionsRef>;

    /// Mutably borrows the pane's optional option handle.
    fn options_mut(&mut self) -> &mut Option<crate::options::RustOptionsRef>;

    /// Borrows the pane's pseudo-terminal stream handle.
    fn event(&self) -> &crate::reactor::Stream;

    /// Mutably borrows the pane's pseudo-terminal stream handle.
    fn event_mut(&mut self) -> &mut crate::reactor::Stream;

    /// Resizes the base and active mode screens and queues the process resize,
    /// cancelling synchronized output. An unchanged size does nothing.
    /// # Safety
    /// Exclude conflicting access while mode resize callbacks execute.
    unsafe fn resize(&mut self, size: crate::PaneSize);

    /// Starts synchronized output, rearming its allocation-bound one-second expiry.
    fn start_sync(&mut self);

    /// Stops synchronized output and disarms its expiry without forcing a redraw.
    fn stop_sync(&mut self);

    /// Borrows the pane's input parser handle.
    fn ictx(&self) -> &Option<crate::input::InputCtxRef>;

    /// Mutably borrows the pane's input parser handle.
    fn ictx_mut(&mut self) -> &mut Option<crate::input::InputCtxRef>;

    /// Installs the PTY stream callbacks and parser, then enables input and output.
    /// # Safety
    /// The pane owns a valid PTY and has no existing live parser or stream.
    unsafe fn initialize_io(&mut self);

    /// Returns the connected pipe process ID, or None when no pipe is open.
    fn pipe_process(&self) -> Option<crate::types::pid_t>;

    /// Copies the current application-output reader position.
    fn output_position(&self) -> crate::RustPaneOutputOffset;

    /// Returns bytes retained after a reader's position.
    fn unread_output(&self, position: &crate::RustPaneOutputOffset) -> crate::reactor::ByteBuffer;

    /// Returns the number of retained bytes after a reader's position.
    fn unread_output_len(&self, position: &crate::RustPaneOutputOffset) -> usize;

    /// Advances an independent reader by at most the available retained bytes.
    fn advance_output(&self, position: &mut crate::RustPaneOutputOffset, size: usize);

    /// Reclaims output consumed by the parser, pipe, and attached control clients,
    /// rebasing their positions together on wrap and updating read backpressure.
    /// # Safety
    /// Exclude conflicting pane and client access while retained offsets change.
    unsafe fn maintain_output(&mut self);

    /// Parses newly retained output and advances the pane's parser position.
    /// # Safety
    /// Exclude conflicting pane and parser access during input callbacks.
    unsafe fn parse_output(&mut self);

    /// Tests whether exited process and pipe output have both drained.
    fn destroy_ready(&self) -> bool;

    /// Borrows the pane's colour palette.
    fn palette(&self) -> &crate::types::colour_palette;

    /// Mutably borrows the pane's colour palette.
    fn palette_mut(&mut self) -> &mut crate::types::colour_palette;

    /// Returns the displayed width of the cached border status.
    fn status_line_width(&self) -> usize;

    /// Publishes a rendered border status and its hit ranges atomically, returning
    /// whether its screen contents changed. Width may be zero for hidden status.
    fn publish_border_status(&mut self, width: usize, screen: crate::screen::RustScreen,
        ranges: crate::types::style_ranges, expanded: std::ffi::CString) -> bool;

    /// Looks up the cached border-status hit range at a status-relative column.
    fn border_status_range(&self, x: u32) -> Option<crate::types::style_range>;

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
