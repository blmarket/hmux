//! Capabilities used by pane consumers.

use crate::{
    PaneActivityState, PaneCommandState, PaneControlColours, PaneExitState,
    PaneGeometryState, PaneIdentity, PaneScrollbar,
    PaneScrollbarStyleState, PaneSearchState, PaneStyleCache,
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
    + PaneSearchState
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
    unsafe fn configure_test(&mut self, setting: crate::window_pane::PaneTestSetup);

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

    /// Derives and acknowledges the current terminal theme, clearing pending updates.
    /// # Safety
    /// Exclude conflicting pane/client access during style and theme evaluation.
    unsafe fn acknowledge_theme(&mut self) -> crate::types::client_theme;
    /// Enables or disables base-screen theme updates and acknowledges pending state.
    /// # Safety
    /// Exclude conflicting pane/client access during theme evaluation.
    unsafe fn set_theme_updates(&mut self, enabled: bool);
    /// Sends a changed terminal theme when the selected screen enables updates.
    /// An unchanged theme, exited process or disabled mode leaves the update pending.
    /// # Safety
    /// Exclude conflicting pane/client access during style evaluation and terminal writes.
    unsafe fn send_theme_update(&mut self);
    #[cfg(test)]
    fn theme(&self) -> crate::types::client_theme;

    /// Copies the palette for a rendering context without sharing mutable storage.
    fn palette_snapshot(&self) -> crate::types::colour_palette;
    /// Resolves a colour through the chosen palette engine.
    fn palette_colour(&self, colour: core::ffi::c_int) -> core::ffi::c_int;
    /// Updates an indexed palette entry, reporting the engine's change result.
    /// Drawing-context redraw policy remains with the caller; no pane flags change.
    fn set_palette_colour(&mut self, index: core::ffi::c_int, colour: core::ffi::c_int) -> core::ffi::c_int;
    /// Clears application palette overrides with the chosen engine's reset semantics.
    /// Drawing-context redraw policy remains with the caller; no pane flags change.
    fn clear_palette(&mut self);
    /// Reloads palette defaults from the pane's current options without changing flags;
    /// option and window transitions schedule their own redraws.
    fn reload_palette(&mut self);
    /// Updates a default colour and marks style/theme refresh as appropriate.
    fn set_palette_default(&mut self, foreground: bool, colour: core::ffi::c_int);

    /// Returns the displayed width of the cached border status.
    fn status_line_width(&self) -> usize;

    /// Publishes a rendered border status and its hit ranges atomically, returning
    /// whether its screen contents changed. Width may be zero for hidden status.
    fn publish_border_status(&mut self, width: usize, screen: crate::screen::RustScreen,
        ranges: crate::types::style_ranges, expanded: std::ffi::CString) -> bool;

    /// Looks up the cached border-status hit range at a status-relative column.
    fn border_status_range(&self, x: u32) -> Option<crate::types::style_range>;

    /// Takes a fixture mode to exercise delayed callbacks after pane destruction.
    /// # Safety
    /// Unit fixtures must complete teardown; unavailable in production builds.
    #[cfg(test)]
    unsafe fn take_test_mode(&mut self) -> Option<Box<crate::types::window_mode_entry>>;
    /// Installs an uninitialized fixture mode for parser guard tests.
    /// # Safety
    /// Unit fixtures must remove it before normal teardown.
    #[cfg(test)]
    unsafe fn insert_test_mode(&mut self, entry: Box<crate::types::window_mode_entry>);

    /// Reports whether the base screen is selected, including an empty mode stack.
    fn showing_base(&self) -> bool;

    /// Returns the number of open modes without exposing their stack.
    fn mode_count(&self) -> usize;
    /// Borrows the active mode's engine state, including pending initialization.
    fn active_mode(&self) -> Option<&crate::types::window_mode_entry>;
    /// Borrows the active mode engine without granting stack membership control.
    fn active_mode_mut(&mut self) -> Option<crate::modes::ModeContext<'_>>;
    /// Finds an open mode engine for its context-specific callbacks.
    fn find_mode_mut(&mut self, mode: crate::types::WindowMode) -> Option<crate::modes::ModeContext<'_>>;
    /// Starts a mode or promotes an existing one, then updates layout and notifications.
    /// # Safety
    /// Exclude conflicting pane/mode access during initialization and notifications.
    unsafe fn set_mode(&mut self, source: Option<crate::types::RustWindowPaneWeak>, mode: crate::types::WindowMode,
        target: Option<&crate::types::cmd_find_state>, args: Option<&crate::args::RustArguments>) -> core::ffi::c_int;
    /// Frees the active mode, selects/resizes its successor, and updates layout.
    /// # Safety
    /// Exclude conflicting access during mode teardown and notifications.
    unsafe fn reset_mode(&mut self);
    /// Frees all modes through the same transition sequence as individual removals.
    /// # Safety
    /// Exclude conflicting access during mode teardown and notifications.
    unsafe fn reset_modes(&mut self);
    /// Updates the default cursor on the selected screen from pane options.
    fn update_default_cursor(&mut self);

    /// Borrows the pane's base screen.
    fn base(&self) -> &crate::screen::RustScreen;

    /// Mutably borrows the pane's base screen.
    fn base_mut(&mut self) -> &mut crate::screen::RustScreen;

    /// Borrows the pane's status screen.
    fn status_screen(&self) -> &crate::screen::RustScreen;

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
