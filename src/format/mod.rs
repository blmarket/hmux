//! Format strings: the tree of values a #{...} is expanded against, and the
//! drawing of an expanded string into a line.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod draw;
mod expand;

pub use draw::{format_draw, format_trim_left, format_trim_right, format_width};
pub use expand::{
    format_add, format_add_cb, format_add_tv, format_create, format_create_defaults,
    format_create_from_state, format_create_from_target, format_defaults, format_defaults_pane,
    format_defaults_paste_buffer, format_defaults_window, format_each, format_expand,
    format_expand_time, format_get_pane, format_grid_hyperlink, format_grid_line, format_grid_word,
    format_job, format_job_tree, format_log_debug, format_lost_client, format_merge, format_single,
    format_single_from_state, format_single_from_target, format_tidy_jobs, format_tree,
    format_true,
};

#[cfg(test)]
pub(crate) use expand::{
    FORMAT_BASENAME, FORMAT_CHARACTER, FORMAT_CLIENTS, FORMAT_COLOUR, FORMAT_DIRNAME,
    FORMAT_EXPAND, FORMAT_EXPAND_NOJOBS, FORMAT_EXPAND_TIME, FORMAT_EXPANDTIME, FORMAT_FORCE,
    FORMAT_LAST, FORMAT_LENGTH, FORMAT_LITERAL, FORMAT_LOOP_LIMIT, FORMAT_MAX_PRECISION,
    FORMAT_MAX_REPEAT, FORMAT_MAX_WIDTH, FORMAT_NOJOBS, FORMAT_NONE, FORMAT_NOT, FORMAT_NOT_NOT,
    FORMAT_PANE, FORMAT_PANES, FORMAT_PRETTY, FORMAT_QUOTE_ARGUMENTS, FORMAT_QUOTE_SHELL,
    FORMAT_QUOTE_STYLE, FORMAT_REPEAT, FORMAT_SESSION_NAME, FORMAT_SESSIONS, FORMAT_STATUS,
    FORMAT_TIME_LIMIT, FORMAT_TIMESTRING, FORMAT_TYPE_PANE, FORMAT_TYPE_SESSION,
    FORMAT_TYPE_UNKNOWN, FORMAT_TYPE_WINDOW, FORMAT_VERBOSE, FORMAT_WIDTH, FORMAT_WINDOW,
    FORMAT_WINDOW_NAME, FORMAT_WINDOWS, MODE_BRACKETPASTE, MODE_INSERT, MODE_KCURSOR, MODE_KKEYPAD,
    MODE_MOUSE_BUTTON, MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_SYNC, MODE_WRAP, PANE_STATUSREADY,
    SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_INDEX, format_expand_state,
};
