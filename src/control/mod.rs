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
    control_add_sub, control_all_done, control_continue_pane, control_discard, control_pane_offset,
    control_pause_pane, control_ready, control_remove_sub, control_reset_offsets,
    control_set_pane_off, control_set_pane_on, control_start, control_state, control_stop,
    control_write, control_write_output,
};

#[cfg(test)]
pub(crate) use state::*;
