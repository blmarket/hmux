//! The pane modes: the dispatch a pane in a mode goes through, the modes
//! themselves, and the tree widget the choosing ones are built on.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod buffer;
mod client;
mod clock;
mod copy;
mod customize;
mod dispatch;
mod tree;
mod widget;

pub use buffer::{window_buffer_editdata, window_buffer_modedata};
pub use client::{window_client_itemdata, window_client_modedata};
pub use clock::{window_clock_mode_data, window_clock_table};
pub use copy::{
    window_copy_add, window_copy_get_current_offset, window_copy_get_hyperlink,
    window_copy_get_line, window_copy_get_word, window_copy_mode_data, window_copy_pagedown,
    window_copy_pageup, window_copy_scroll, window_copy_set_line_numbers, window_copy_start_drag,
};
pub use customize::{window_customize_itemdata, window_customize_modedata};
pub use tree::{window_tree_itemdata, window_tree_modedata};
pub use widget::{mode_tree_data, mode_tree_item, mode_tree_menu};

#[cfg(test)]
pub(crate) use copy::{
    CURSORDRAG_ENDSEL, CURSORDRAG_NONE, CURSORDRAG_SEL, LINE_SEL_LEFT_RIGHT, LINE_SEL_NONE,
    LINE_SEL_RIGHT_LEFT, RECENTRE_BOTTOM, RECENTRE_MIDDLE, RECENTRE_TOP, SEL_CHAR, SEL_LINE,
    SEL_WORD, WINDOW_COPY_CMD_CANCEL, WINDOW_COPY_CMD_CLEAR_ALWAYS,
    WINDOW_COPY_CMD_CLEAR_EMACS_ONLY, WINDOW_COPY_CMD_CLEAR_NEVER, WINDOW_COPY_CMD_NOTHING,
    WINDOW_COPY_CMD_REDRAW, WINDOW_COPY_DRAG_REPEAT_TIME, WINDOW_COPY_JUMPBACKWARD,
    WINDOW_COPY_JUMPFORWARD, WINDOW_COPY_JUMPTOBACKWARD, WINDOW_COPY_JUMPTOFORWARD,
    WINDOW_COPY_LINE_NUMBERS_ABSOLUTE, WINDOW_COPY_LINE_NUMBERS_DEFAULT,
    WINDOW_COPY_LINE_NUMBERS_HYBRID, WINDOW_COPY_LINE_NUMBERS_OFF,
    WINDOW_COPY_LINE_NUMBERS_RELATIVE, WINDOW_COPY_OFF, WINDOW_COPY_SEARCH_ALL_TIMEOUT,
    WINDOW_COPY_SEARCH_MAX_LINE, WINDOW_COPY_SEARCH_TIMEOUT, WINDOW_COPY_SEARCHDOWN,
    WINDOW_COPY_SEARCHUP, window_copy_backing,
};
#[cfg(test)]
pub(crate) use customize::{
    WINDOW_CUSTOMIZE_DEFAULT_FORMAT, WINDOW_CUSTOMIZE_GLOBAL_SESSION,
    WINDOW_CUSTOMIZE_GLOBAL_WINDOW, WINDOW_CUSTOMIZE_KEY, WINDOW_CUSTOMIZE_NONE,
    WINDOW_CUSTOMIZE_PANE, WINDOW_CUSTOMIZE_RESET, WINDOW_CUSTOMIZE_SERVER,
    WINDOW_CUSTOMIZE_SESSION, WINDOW_CUSTOMIZE_UNSET, WINDOW_CUSTOMIZE_WINDOW,
};
pub(crate) use customize::{
    window_customize_change_current_callback, window_customize_change_tagged_callback,
    window_customize_set_command_callback, window_customize_set_note_callback,
    window_customize_set_option_callback,
};
#[cfg(test)]
pub(crate) use tree::{
    WINDOW_TREE_DEFAULT_COMMAND, WINDOW_TREE_DEFAULT_FORMAT, WINDOW_TREE_DEFAULT_KEY_FORMAT,
    WINDOW_TREE_NONE, WINDOW_TREE_PANE, WINDOW_TREE_SESSION, WINDOW_TREE_WINDOW,
};
pub(crate) use tree::{
    window_tree_command_callback, window_tree_kill_current_callback,
    window_tree_kill_tagged_callback,
};
#[cfg(test)]
pub(crate) use widget::{mode_tree_expand_current, mode_tree_get_current};
pub(crate) use widget::{mode_tree_filter_callback, mode_tree_search_callback};
