//! Grid storage and cell operations, with viewport access and reader traversal.

mod core;
pub mod reader;
pub mod view;

pub(crate) use self::core::grid_create_box;

// Preserve the established family API without glob exports.
pub use self::core::{
    grid_adjust_lines, grid_cell_attr_string, grid_cell_flags_string, grid_cells_equal,
    grid_cells_look_equal, grid_check_is_clear, grid_clear, grid_clear_history, grid_clear_lines,
    grid_collect_history, grid_compare, grid_create, grid_default_cell, grid_destroy,
    grid_duplicate_lines, grid_empty_line, grid_free_lines, grid_get_cell, grid_get_line,
    grid_in_set, grid_line_flags_string, grid_line_length, grid_line_limit, grid_line_time,
    grid_move_cells, grid_move_lines, grid_peek_line, grid_reflow, grid_remove_history,
    grid_scroll_history, grid_scroll_history_region, grid_set_cell, grid_set_cells,
    grid_set_padding, grid_set_tab, grid_string_cells_bytes,
    grid_unwrap_position, grid_wrap_position, hyperlinks, screen, screen_sel, screen_titles,
    screen_write_cline, C2RustUnnamed, C2RustUnnamed_0, C2RustUnnamed_1, __INT_MAX__,
    COLOUR_FLAG_256, COLOUR_FLAG_RGB, COLOUR_FLAG_THEME, UINT_MAX,
};
