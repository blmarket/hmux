//! The cell store: a grid of lines, the screen's view onto it, a cursor that
//! walks it, and the hyperlink set its extended cells point into.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod line;
pub(crate) mod links;
mod reader;
mod store;
mod view;

use line::grid_line;
pub use links::{Hyperlinks, RustHyperlinks};
pub use reader::{GridReader, RustGrid, RustGridReader};
pub use store::{
    GRID_LINE_WRAPPED, GRID_STRING_EMPTY_CELLS, GRID_STRING_TRIM_SPACES, Grid, grid,
    grid_cells_equal, grid_cells_look_equal, grid_clear_history,
    grid_clear_lines, grid_collect_history, grid_compare, grid_create, grid_default_cell,
    grid_duplicate_lines, grid_get_cell, grid_line_info, grid_peek_info, grid_mark_wrapped, GridLineInfo,
    grid_in_set, grid_line_length, grid_reflow, grid_remove_history, grid_set_cell,
    grid_set_cells, grid_set_padding, grid_set_tab, grid_string_cells, grid_unwrap_position,
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
pub(crate) use store::{
    GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_FLAG_PADDING, GRID_HISTORY, grid_clear,
    grid_move_cells, grid_move_lines, grid_scroll_history,
};
