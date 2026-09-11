//! Capabilities used by pane consumers.

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
/// ```compile_fail
/// use tmux_c2rs::WindowPane;
/// fn change_flags(pane: &mut dyn WindowPane) { pane.flags_mut(); }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::WindowPane;
/// fn replace_parent(pane: &mut dyn WindowPane) { pane.set_window_context(None); }
/// ```
pub trait WindowPane {
    /// Returns the immutable numeric pane identity.
    fn pane_id(&self) -> u32;

    /// Returns the complete pane rectangle.
    fn geometry(&self) -> crate::PaneGeometry;

    /// Moves the pane without changing its size.
    fn set_position(&mut self, x: core::ffi::c_int, y: core::ffi::c_int);

    /// Returns the retained search text, if any.
    fn search_query(&self) -> Option<&core::ffi::CStr>;

    /// Returns whether the retained search text is a regular expression.
    fn search_is_regex(&self) -> bool;

    /// Replaces the retained search text and interpretation atomically.
    fn set_search(&mut self, query: &core::ffi::CStr, regex: bool);

    /// Returns whether both retained search properties match the arguments.
    fn search_matches(&self, query: &core::ffi::CStr, regex: bool) -> bool;

    /// Returns the activity sequence number.
    fn activity_point(&self) -> core::ffi::c_uint;

    /// Records the pane as active and schedules automatic naming reconsideration.
    fn mark_active_at(&mut self, point: core::ffi::c_uint);

    /// Returns an owned snapshot of the command metadata.
    fn pane_command(&self) -> crate::PaneCommand;

    /// Copies a complete replacement command.
    fn set_pane_command(&mut self, command: &crate::PaneCommand);

    /// Returns the raw wait status.
    fn exit_status(&self) -> core::ffi::c_int;

    /// Returns the recorded death time.
    fn death_time(&self) -> crate::types::timeval;

    /// Returns both cached style cells.
    fn styles(&self) -> crate::PaneStyleCells;

    /// Refreshes both cached styles if invalid, clearing the invalidation before
    /// evaluating formats so recursive observations retain the existing ordering.
    /// # Safety
    /// Exclude conflicting pane access while format callbacks execute.
    unsafe fn refresh_styles(&mut self) -> crate::PaneStyleCells;

    /// Returns both reported colours.
    fn colours(&self) -> crate::PaneControlColourPair;

    /// Parses a control-client colour report and publishes its pair. Malformed reports
    /// retain existing values; terminal query flags follow the parser and no redraw is requested.
    /// # Safety
    /// Exclude conflicting pane/TTY access while the report parser reads client identity.
    unsafe fn report_control_colours(&mut self, tty: &mut crate::types::tty, report: &[u8]);

    /// Returns a copy of the current slider.
    fn slider(&self) -> crate::PaneScrollbarSlider;

    /// Publishes the rendered slider for subsequent scrollbar hit-testing.
    fn publish_slider(&mut self, slider: crate::PaneScrollbarSlider);

    /// Returns the cached scrollbar style.
    fn scrollbar_style(&self) -> crate::PaneScrollbarStyle;

    /// Observes this allocation without retaining it. Unowned implementations return None.
    /// The observation retains its allocation identity after destruction and never
    /// resolves another allocation with the same pane ID.
    fn observation(&self) -> Option<crate::types::RustWindowPaneWeak>;

    /// Borrows the pane's server flags.
    fn flags(&self) -> &core::ffi::c_int;

    /// Changes flags only for explicit unit fixtures; unavailable in production.
    #[cfg(test)]
    fn flags_mut(&mut self) -> &mut core::ffi::c_int;

    /// Schedules content redraw without changing border or status policy.
    fn request_redraw(&mut self);
    /// Schedules scrollbar redraw without requesting a content redraw.
    fn request_scrollbar_redraw(&mut self);
    /// Schedules both pane content and scrollbar redraw.
    fn request_full_redraw(&mut self);
    /// Acknowledges content and scrollbar redraw after all client passes complete.
    fn finish_redraw(&mut self);

    /// Requests reconsideration of automatic naming after pane content/context changes.
    fn name_changed(&mut self);
    /// Acknowledges the naming request after the window's throttle permits evaluation.
    fn finish_name_update(&mut self);
    /// Schedules terminal theme evaluation after attached-client context changes.
    fn request_theme_update(&mut self);
    /// Records the membership marker maintained by the window's stack operations.
    fn set_stack_member(&mut self, member: bool);
    /// Records zoom state selected by the window's complete zoom transition.
    fn set_window_zoomed(&mut self, zoomed: bool);
    /// Changes pane input acceptance; the owning window updates border/status policy.
    fn set_input_enabled(&mut self, enabled: bool);
    /// Configures a process-free pane with hidden cursor and CRLF output semantics.
    fn prepare_empty(&mut self);
    /// Adopts window context and option parent together, invalidating inherited styles/theme.
    fn inherit_window_context(&mut self, window: &crate::types::WindowRef);

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

    /// Records a child wait result and exit flags, returning whether output has drained.
    fn record_process_exit(&mut self, status: core::ffi::c_int) -> bool;
    /// Closes the process and applies remain-on-exit policy, rendering retained status
    /// once. Returns true when the window should remove this pane.
    /// # Safety
    /// Exclude conflicting pane access during close, notifications and rendering.
    unsafe fn finish_process(&mut self, notify: bool) -> bool;

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

    /// Applies pane cache policy after an option changes: window styles invalidate
    /// style/theme, user options invalidate style, pane colours reload their defaults,
    /// and scrollbar style is recomputed. Window redraw/layout policy stays with callers.
    fn option_changed(&mut self, name: &core::ffi::CStr);

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
    /// Installs a transferred popup screen and sizes it to the pane.
    fn adopt_popup_screen(&mut self, screen: crate::screen::RustScreen);
    fn base_mut(&mut self) -> crate::screen::ScreenMut<'_>;

    /// Borrows the pane's status screen.
    fn status_screen(&self) -> &crate::screen::RustScreen;

    /// Retains the recorded window context, including during pane transfers.
    fn window_context(&self) -> Option<crate::types::WindowRef>;

    /// Initializes window context only in unit fixtures.
    #[cfg(test)]
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
