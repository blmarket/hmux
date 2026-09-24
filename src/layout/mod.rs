//! Layout tree operations, preset arrangements, and custom layout serialization.

mod core;
pub mod custom;
pub mod set;

// Preserve the established family API without glob exports.
pub use self::core::{
    args, client, client_file, client_file_cb, client_file_entry, client_files, cmd_find_state,
    cmd_list, cmdq_item, cmdq_list, cmds, control_state, environ, format_job_tree, format_tree,
    hyperlinks, input_ctx, input_request, input_requests, key_binding, key_binding_entry,
    key_bindings, key_event, key_table, key_table_entry, layout_add_horizontal_border,
    layout_assign_pane, layout_cell, layout_cell_entry, layout_cell_get_neighbour,
    layout_cell_has_tiled_child, layout_cell_is_tiled, layout_cells, layout_close_pane,
    layout_count_cells, layout_create_cell, layout_destroy_cell, layout_fix_offsets,
    layout_fix_panes, layout_floating_args_parse, layout_floating_pane, layout_free,
    layout_free_cell, layout_geometry, layout_get_floating_cell, layout_get_tiled_cell,
    layout_init, layout_insert_tile, layout_make_leaf, layout_make_node, layout_print_cell,
    layout_remove_tile, layout_replace_with_node, layout_resize, layout_resize_adjust,
    layout_resize_floating_pane, layout_resize_floating_pane_to, layout_resize_layout,
    layout_resize_pane, layout_resize_pane_to, layout_resize_set_size, layout_search_by_border,
    layout_set_size, layout_split_check_space, layout_split_floating_cell, layout_split_pane,
    layout_split_sizes, layout_spread_cell, layout_spread_out, menu_data, mouse_event, options,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb, prompt, redraw_scene, screen, screen_sel, screen_titles, screen_write_cline,
    session, session_entry, spawn_editor_state, status_line, tmuxpeer, tty, tty_code, tty_key,
    tty_term, tty_term_entry, visible_range, visible_ranges, window, window_entry, window_mode,
    window_mode_entry, window_pane, window_pane_modes, window_pane_offset, window_pane_prompt,
    window_pane_resize, window_pane_resizes, window_pane_tree_entry, window_panes, window_winlinks,
    winlink, winlink_entry, winlink_stack, winlinks, C2RustUnnamed_12, C2RustUnnamed_13,
    __INT_MAX__, INT_MAX, PANE_MAXIMUM, PANE_MINIMUM, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS,
    PANE_SCROLLBARS_LEFT, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, SPAWN_BEFORE, SPAWN_FLOATOVERZOOM,
    SPAWN_FULLSIZE, SPAWN_HORIZONTAL, SPAWN_SPLIT, SPAWN_ZOOM, UINT_MAX,
};

pub use crate::src::shared::layout::{
    layout_cell_next, layout_cell_prev, layout_cells_first, layout_cells_insert_after,
    layout_cells_insert_before, layout_cells_last, layout_cells_push_back, layout_cells_push_front,
    layout_cells_remove, layout_cells_replace,
};
