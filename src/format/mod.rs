//! Format strings and the tree of values a `#{...}` expression expands against.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

pub(crate) mod expand;

pub use crate::screen::{format_trim_left, format_trim_right, format_width};
pub use expand::{
    format_add, format_add_cb, format_add_tv, format_create, format_create_defaults,
    format_create_from_state, format_create_from_target, format_defaults, format_defaults_pane,
    format_defaults_paste_buffer, format_defaults_window, format_each, format_expand,
    format_expand_time, format_free_jobs, format_grid_hyperlink, format_grid_line,
    format_grid_word, format_job, format_job_tree, format_log_debug, format_merge, format_single,
    format_single_from_state, format_single_from_target, format_tidy_jobs, format_tree,
    format_true,
};

pub(crate) use expand::FORMAT_TYPE_PANE;
pub(crate) use expand::{
    format_create_for_client, format_create_from_state_for_client, format_defaults_for_handles,
    format_defaults_for_link, format_defaults_for_link_pane, format_defaults_for_session,
};

#[cfg(test)]
pub(crate) use expand::{
    FORMAT_NONE, MODE_BRACKETPASTE, MODE_INSERT, MODE_KCURSOR, MODE_KKEYPAD, MODE_MOUSE_BUTTON,
    MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_SYNC, MODE_WRAP,
};
