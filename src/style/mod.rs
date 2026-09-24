//! Style parsing, formatting, and application, with attribute and colour conversion.

pub mod attributes;
pub mod colour;
mod parsing;

// Preserve the established family API without glob exports.
pub use self::parsing::{
    args, client, client_file, client_file_cb, client_file_entry, client_files,
    cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds, control_state, environ, format_job_tree,
    format_tree, hyperlinks, input_ctx, input_request, input_requests, key_binding,
    key_binding_entry, key_bindings, key_event, key_table, key_table_entry, layout_cell,
    layout_cell_entry, layout_cells, layout_geometry, menu_data, mouse_event, options,
    options_entry, options_table_entry, overlay_check_cb, overlay_draw_cb, overlay_free_cb,
    overlay_key_cb, overlay_mode_cb, overlay_resize_cb, prompt, redraw_scene, screen, screen_sel,
    screen_titles, screen_write_cline, session, session_entry, session_gentry, spawn_editor_state,
    status_line, style_add, style_apply, style_copy, style_link, style_parse, style_parse_colour,
    style_ranges_clear, style_ranges_free, style_ranges_get_range, style_ranges_init, style_set,
    style_set_scrollbar_style_from_option, style_tostring, tmuxpeer, tty, tty_code, tty_key,
    tty_term, tty_term_entry, visible_range, visible_ranges, window, window_alerts_entry,
    window_entry, window_mode, window_mode_entry, window_mode_entry_entry, window_pane,
    window_pane_modes, window_pane_offset, window_pane_prompt,
    window_pane_resize, window_pane_resize_entry, window_pane_resizes, window_pane_tree_entry, window_panes, window_winlinks, winlink,
    winlink_entry, winlink_sentry, winlink_stack, winlink_wentry, winlinks, C2RustUnnamed_12,
    C2RustUnnamed_13, __INT_MAX__, FORMAT_NOJOBS, PANE_SCROLLBARS_CHARACTER,
    PANE_SCROLLBARS_DEFAULT_PADDING, PANE_SCROLLBARS_DEFAULT_WIDTH, STYLE_PAD_DEFAULT,
    STYLE_WIDTH_DEFAULT, UINT_MAX,
};
