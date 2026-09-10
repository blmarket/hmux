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

    /// Borrows the pane's optional option handle.
    fn options(&self) -> &Option<crate::options::RustOptionsRef>;

    /// Mutably borrows the pane's optional option handle.
    fn options_mut(&mut self) -> &mut Option<crate::options::RustOptionsRef>;

    /// Resizes the base and active mode screens and queues the process resize,
    /// cancelling synchronized output. An unchanged size does nothing.
    /// # Safety
    /// Exclude conflicting access while mode resize callbacks execute.
    unsafe fn resize(&mut self, size: crate::PaneSize);

    /// Starts synchronized output, rearming its allocation-bound one-second expiry.
    fn start_sync(&mut self);

    /// Stops synchronized output and disarms its expiry without forcing a redraw.
    fn stop_sync(&mut self);

    /// Configures borrowed I/O in unit fixtures; unavailable in production builds.
    /// # Safety
    /// Prevent conflicting stream/parser use and keep injected resources alive.
    #[cfg(test)]
    unsafe fn configure_test_io(&mut self, setting: crate::window_pane::PaneTestIo);

    /// Returns the pane's child process ID.
    fn process_id(&self) -> crate::types::pid_t;
    /// Returns whether the pane owns an open PTY.
    fn process_active(&self) -> bool;
    /// Returns the current foreground process name from the PTY.
    fn process_name(&self) -> Option<std::ffi::CString>;
    /// Returns the current working directory reported for the PTY.
    fn process_cwd(&self) -> Option<std::ffi::CString>;
    /// Returns the foreground group and session leader, when the PTY is open.
    fn process_groups(&self) -> Option<(Option<crate::types::pid_t>, Option<crate::types::pid_t>)>;
    /// Forks a PTY and installs its PID, descriptor and terminal name together.
    /// # Safety
    /// Follow fork requirements and provide exclusive process access.
    unsafe fn fork_process(&mut self, master: core::ffi::c_int, size: &crate::types::winsize) -> crate::types::pid_t;
    /// Takes a job's PTY, process and terminal identity together.
    fn take_job(&mut self, id: u32) -> bool;
    /// Closes the old PTY/parser, resets modes and screen, and clears exit-display state.
    /// # Safety
    /// Exclude conflicting access while mode teardown callbacks run.
    unsafe fn prepare_respawn(&mut self);
    /// Removes the session record, closes the PTY stream and descriptor, and marks it closed.
    /// # Safety
    /// Exclude conflicting process and stream access.
    unsafe fn close_process(&mut self);
    /// Records the spawned terminal, clears exit state, restores the pre-fork
    /// signal mask, then installs the stream and parser (also for an empty pane).
    /// # Safety
    /// Run in the parent after child setup, with exclusive pane/parser access.
    unsafe fn activate_spawned_process(&mut self, signal_mask: &crate::types::sigset_t);
    /// Writes bytes to the terminal, including protocol replies which bypass input-off.
    fn write_terminal(&self, bytes: &[u8]);
    /// Transfers buffered bytes to the terminal without applying key encoding.
    fn write_terminal_buffer(&self, bytes: &mut crate::reactor::ByteBuffer);
    /// Encodes and writes a key using the selected screen and existing key encoder.
    /// # Safety
    /// Exclude conflicting screen access while selecting its modes.
    unsafe fn write_key(&self, key: crate::types::key_code) -> core::ffi::c_int;
    /// Parses submitted output in the attached or mode-screen context as appropriate.
    /// # Safety
    /// Exclude conflicting pane/parser access during parser and drawing callbacks.
    unsafe fn parse_bytes(&mut self, bytes: crate::reactor::ByteBuffer);

    /// Installs the stream and parser after a job transfer and screen setup.
    /// # Safety
    /// Exclude conflicting pane/parser access; job transfer is already complete.
    unsafe fn activate_transferred_process(&mut self);

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
