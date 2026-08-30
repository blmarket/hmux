//! The cell store: a grid of lines, the screen's view onto it, a cursor that
//! walks it, and the hyperlink set its extended cells point into.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod line;
mod links;
mod reader;
mod store;
mod view;

pub use line::grid_line;
pub(crate) use links::{HyperlinksRef, hyperlinks_put};
pub use links::{hyperlinks, hyperlinks_get};
pub use reader::{
    grid_reader_cursor_back_to_indentation, grid_reader_cursor_end_of_line,
    grid_reader_cursor_jump, grid_reader_cursor_jump_back, grid_reader_cursor_left,
    grid_reader_cursor_next_word, grid_reader_cursor_next_word_end,
    grid_reader_cursor_previous_word, grid_reader_cursor_right, grid_reader_cursor_start_of_line,
    grid_reader_get_cursor, grid_reader_in_set, grid_reader_start,
};
pub use store::{
    GRID_LINE_WRAPPED, GRID_STRING_EMPTY_CELLS, GRID_STRING_TRIM_SPACES, grid_adjust_lines,
    grid_cells_equal, grid_cells_look_equal, grid_clear_history, grid_clear_lines,
    grid_collect_history, grid_compare, grid_create, grid_default_cell, grid_duplicate_lines,
    grid_empty_line, grid_get_cell, grid_get_line, grid_in_set, grid_line_length, grid_peek_line,
    grid_reflow, grid_remove_history, grid_set_tab, grid_string_cells, grid_unwrap_position,
    grid_wrap_position,
};
pub use view::{
    grid_view_clear, grid_view_clear_history, grid_view_delete_cells, grid_view_delete_lines,
    grid_view_delete_lines_region, grid_view_get_cell, grid_view_insert_cells,
    grid_view_insert_lines, grid_view_insert_lines_region, grid_view_scroll_region_down,
    grid_view_scroll_region_up, grid_view_set_cell, grid_view_set_cells, grid_view_set_padding,
    grid_view_string_cells,
};

#[cfg(test)]
pub(crate) use links::{
    MAX_HYPERLINKS, RB_BLACK, RB_NEGINF, RB_RED, VIS_CSTYLE, VIS_OCTAL, hyperlinks_by_inner_tree,
    hyperlinks_by_uri_tree, hyperlinks_uri_key,
};
#[cfg(test)]
pub(crate) use store::{
    GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_FLAG_PADDING, GRID_HISTORY, grid_clear, grid_destroy,
    grid_move_cells, grid_move_lines, grid_scroll_history, grid_set_cell, grid_set_cells,
    grid_set_padding,
};
