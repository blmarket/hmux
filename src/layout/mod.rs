//! Layout tree operations, preset arrangements, and custom layout serialization.

mod core;
pub mod custom;
pub mod set;

// Preserve the established family API without glob exports.
pub use self::core::{
    cmdq_item, environ, format_job_tree, format_tree, key_event, layout_add_horizontal_border,
    layout_assign_pane, layout_cell, layout_cell_has_tiled_child, layout_cell_is_tiled,
    layout_close_pane, layout_count_cells, layout_create_cell, layout_destroy_cell,
    layout_fix_offsets, layout_fix_panes, layout_float_pane, layout_floating_args_parse,
    layout_floating_pane, layout_free, layout_get_floating_cell, layout_get_tiled_cell,
    layout_init, layout_make_leaf, layout_make_node, layout_print_cell, layout_resize,
    layout_resize_floating_pane, layout_resize_floating_pane_to, layout_resize_layout,
    layout_resize_pane, layout_resize_pane_to, layout_search_by_border, layout_set_size,
    layout_spread_out, layout_take_leaf, layout_take_leaves, layout_tile_pane, logical_size,
    mouse_event, options, tty_term, PANE_MINIMUM, PANE_SCROLLBARS_ALWAYS, PANE_STATUS_BOTTOM,
    PANE_STATUS_TOP,
};
