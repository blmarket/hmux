//! Control mode: the state a control client is driven through, and the
//! notifications it is sent.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod notify;
mod state;

pub use notify::{
    control_notify_client_detached, control_notify_client_session_changed,
    control_notify_pane_mode_changed, control_notify_paste_buffer_changed,
    control_notify_paste_buffer_deleted, control_notify_session_closed,
    control_notify_session_created, control_notify_session_renamed,
    control_notify_session_window_changed, control_notify_window_layout_changed,
    control_notify_window_linked, control_notify_window_pane_changed,
    control_notify_window_renamed, control_notify_window_unlinked,
};
pub use state::{
    control_add_sub, control_all_done, control_block, control_continue_pane, control_discard,
    control_pane, control_pane_offset, control_pane_offset_mut, control_pause_pane, control_ready,
    control_remove_sub, control_reset_offsets, control_set_pane_off, control_set_pane_on,
    control_start, control_state, control_stop, control_write, control_write_output,
};

#[cfg(test)]
pub(crate) use state::*;

impl crate::types::ClientRef {
    /// Replaces the named control subscription, copying its name and format and
    /// resetting its previous observations. The existing selector and signed ID
    /// representation are preserved, including the session selector's unused ID.
    /// Arms the one-second subscription timer when needed; its callback retains
    /// only a weak client handle. No format expansion or callback runs inline.
    ///
    /// # Safety
    /// Run on the server thread for an initialized control client, without
    /// outstanding client or control-state payload borrows. The selector must
    /// be one of the control protocol's subscription kinds.
    pub(crate) unsafe fn set_control_subscription(
        &mut self,
        name: &std::ffi::CStr,
        selector: crate::types::control_sub_type,
        id: core::ffi::c_int,
        format: &std::ffi::CStr,
    ) {
        unsafe { control_add_sub(self.as_client_mut(), name, selector, id, format) }
    }

    /// Removes a named control subscription, if present, and disarms the timer
    /// when no subscriptions remain. No callback runs and no borrow escapes.
    ///
    /// # Safety
    /// Run on the server thread for an initialized control client, without
    /// outstanding client or control-state payload borrows.
    pub(crate) unsafe fn remove_control_subscription(&mut self, name: &std::ffi::CStr) {
        unsafe { control_remove_sub(self.as_client_mut(), name) }
    }
}

impl crate::types::ClientRef {
    /// Enables output for an existing disabled control-pane entry and starts
    /// both output offsets at the pane's current offset. Otherwise does nothing.
    /// A dead observation does nothing; a live pane is retained only for this
    /// synchronous operation, without changing registration or window ownership.
    /// No command callbacks run; control output is delivered by the event loop.
    ///
    /// # Safety
    /// Run on the server thread with no conflicting client, control-state or
    /// pane payload access. The client must have initialized control state.
    pub(crate) unsafe fn control_set_pane_on(&mut self, pane: &crate::types::RustWindowPaneWeak) {
        unsafe {
            if let Some(pane) = pane.upgrade() {
                control_set_pane_on(self.as_client_mut(), pane.as_pane());
            }
        }
    }

    /// Disables output for this client's pane, creating its tracking entry if
    /// needed, resetting both offsets and discarding queued pane output.
    /// A dead observation does nothing; a live pane is retained only for this
    /// synchronous operation, without changing registration or window ownership.
    /// No command callbacks run; control output is delivered by the event loop.
    ///
    /// # Safety
    /// Run on the server thread with no conflicting client, control-state or
    /// pane payload access. The client must have initialized control state.
    pub(crate) unsafe fn control_set_pane_off(&mut self, pane: &crate::types::RustWindowPaneWeak) {
        unsafe {
            if let Some(pane) = pane.upgrade() {
                control_set_pane_off(self.as_client_mut(), pane.as_pane());
            }
        }
    }

    /// Continues an existing paused control-pane entry from the pane's current
    /// offset and queues one `%continue` line. Otherwise does nothing.
    /// A dead observation does nothing; a live pane is retained only for this
    /// synchronous operation, without changing registration or window ownership.
    /// No command callbacks run; control output is delivered by the event loop.
    ///
    /// # Safety
    /// Run on the server thread with no conflicting client, control-state or
    /// pane payload access. The client must have initialized control state.
    pub(crate) unsafe fn control_continue_pane(&mut self, pane: &crate::types::RustWindowPaneWeak) {
        unsafe {
            if let Some(pane) = pane.upgrade() {
                control_continue_pane(self.as_client_mut(), pane.as_pane());
            }
        }
    }

    /// Pauses this client's pane, creating its tracking entry if needed. On the
    /// first pause, discards queued pane output and queues one `%pause` line.
    /// A dead observation does nothing; a live pane is retained only for this
    /// synchronous operation, without changing registration or window ownership.
    /// No command callbacks run; control output is delivered by the event loop.
    ///
    /// # Safety
    /// Run on the server thread with no conflicting client, control-state or
    /// pane payload access. The client must have initialized control state.
    pub(crate) unsafe fn control_pause_pane(&mut self, pane: &crate::types::RustWindowPaneWeak) {
        unsafe {
            if let Some(pane) = pane.upgrade() {
                control_pause_pane(self.as_client_mut(), pane.as_pane());
            }
        }
    }
}
