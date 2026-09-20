//! Layouts of every original grid-storage copy, measured before replacing definitions.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_grid_storage_copies_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, $canonical:ty, [$($field:ident),*]) => {
            let _: Option<$canonical> = None::<$ty>;
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!(
        "src/alerts.rs::C2RustUnnamed_12",
        hmux2::src::alerts::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/alerts.rs::C2RustUnnamed_13",
        hmux2::src::alerts::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/arguments.rs::C2RustUnnamed_20",
        hmux2::src::arguments::C2RustUnnamed_20,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/arguments.rs::C2RustUnnamed_21",
        hmux2::src::arguments::C2RustUnnamed_21,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cfg.rs::C2RustUnnamed_12",
        hmux2::src::cfg::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cfg.rs::C2RustUnnamed_13",
        hmux2::src::cfg::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/client.rs::C2RustUnnamed_25",
        hmux2::src::client::C2RustUnnamed_25,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/client.rs::C2RustUnnamed_26",
        hmux2::src::client::C2RustUnnamed_26,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd.rs::C2RustUnnamed_12",
        hmux2::src::cmd::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd.rs::C2RustUnnamed_13",
        hmux2::src::cmd::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_attach_session.rs::C2RustUnnamed_12",
        hmux2::src::cmd_attach_session::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_attach_session.rs::C2RustUnnamed_13",
        hmux2::src::cmd_attach_session::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_bind_key.rs::C2RustUnnamed_12",
        hmux2::src::cmd_bind_key::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_bind_key.rs::C2RustUnnamed_13",
        hmux2::src::cmd_bind_key::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_break_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_break_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_break_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_break_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_capture_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_capture_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_capture_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_capture_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_choose_tree.rs::C2RustUnnamed_12",
        hmux2::src::cmd_choose_tree::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_choose_tree.rs::C2RustUnnamed_13",
        hmux2::src::cmd_choose_tree::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_command_prompt.rs::C2RustUnnamed_12",
        hmux2::src::cmd_command_prompt::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_command_prompt.rs::C2RustUnnamed_13",
        hmux2::src::cmd_command_prompt::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_confirm_before.rs::C2RustUnnamed_12",
        hmux2::src::cmd_confirm_before::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_confirm_before.rs::C2RustUnnamed_13",
        hmux2::src::cmd_confirm_before::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_copy_mode.rs::C2RustUnnamed_12",
        hmux2::src::cmd_copy_mode::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_copy_mode.rs::C2RustUnnamed_13",
        hmux2::src::cmd_copy_mode::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_detach_client.rs::C2RustUnnamed_12",
        hmux2::src::cmd_detach_client::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_detach_client.rs::C2RustUnnamed_13",
        hmux2::src::cmd_detach_client::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_display_menu.rs::C2RustUnnamed_12",
        hmux2::src::cmd_display_menu::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_display_menu.rs::C2RustUnnamed_13",
        hmux2::src::cmd_display_menu::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_display_message.rs::C2RustUnnamed_12",
        hmux2::src::cmd_display_message::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_display_message.rs::C2RustUnnamed_13",
        hmux2::src::cmd_display_message::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_find.rs::C2RustUnnamed_12",
        hmux2::src::cmd_find::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_find.rs::C2RustUnnamed_13",
        hmux2::src::cmd_find::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_find_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_find_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_find_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_find_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_if_shell.rs::C2RustUnnamed_12",
        hmux2::src::cmd_if_shell::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_if_shell.rs::C2RustUnnamed_13",
        hmux2::src::cmd_if_shell::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_join_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_join_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_join_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_join_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_kill_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_kill_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_kill_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_kill_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_kill_session.rs::C2RustUnnamed_12",
        hmux2::src::cmd_kill_session::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_kill_session.rs::C2RustUnnamed_13",
        hmux2::src::cmd_kill_session::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_kill_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_kill_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_kill_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_kill_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_buffers.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_buffers::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_buffers.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_buffers::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_clients.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_clients::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_clients.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_clients::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_commands.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_commands::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_commands.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_commands::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_keys.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_keys::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_keys.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_keys::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_panes.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_panes::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_panes.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_panes::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_sessions.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_sessions::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_sessions.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_sessions::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_list_windows.rs::C2RustUnnamed_12",
        hmux2::src::cmd_list_windows::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_list_windows.rs::C2RustUnnamed_13",
        hmux2::src::cmd_list_windows::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_load_buffer.rs::C2RustUnnamed_12",
        hmux2::src::cmd_load_buffer::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_load_buffer.rs::C2RustUnnamed_13",
        hmux2::src::cmd_load_buffer::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_lock_server.rs::C2RustUnnamed_12",
        hmux2::src::cmd_lock_server::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_lock_server.rs::C2RustUnnamed_13",
        hmux2::src::cmd_lock_server::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_move_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_move_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_move_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_move_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_12",
        hmux2::src::cmd_new_session::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_13",
        hmux2::src::cmd_new_session::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_new_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_new_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_new_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_new_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_parse.rs::C2RustUnnamed_13",
        hmux2::src::cmd_parse::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_parse.rs::C2RustUnnamed_14",
        hmux2::src::cmd_parse::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_paste_buffer.rs::C2RustUnnamed_12",
        hmux2::src::cmd_paste_buffer::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_paste_buffer.rs::C2RustUnnamed_13",
        hmux2::src::cmd_paste_buffer::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_pipe_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_pipe_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_pipe_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_pipe_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_queue.rs::C2RustUnnamed_12",
        hmux2::src::cmd_queue::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_queue.rs::C2RustUnnamed_13",
        hmux2::src::cmd_queue::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_refresh_client.rs::C2RustUnnamed_12",
        hmux2::src::cmd_refresh_client::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_refresh_client.rs::C2RustUnnamed_13",
        hmux2::src::cmd_refresh_client::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_rename_session.rs::C2RustUnnamed_12",
        hmux2::src::cmd_rename_session::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_rename_session.rs::C2RustUnnamed_13",
        hmux2::src::cmd_rename_session::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_rename_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_rename_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_rename_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_rename_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_resize_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_resize_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_resize_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_resize_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_resize_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_resize_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_resize_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_resize_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_respawn_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_respawn_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_respawn_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_respawn_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_respawn_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_respawn_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_respawn_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_respawn_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_rotate_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_rotate_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_rotate_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_rotate_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_run_shell.rs::C2RustUnnamed_12",
        hmux2::src::cmd_run_shell::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_run_shell.rs::C2RustUnnamed_13",
        hmux2::src::cmd_run_shell::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_save_buffer.rs::C2RustUnnamed_12",
        hmux2::src::cmd_save_buffer::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_save_buffer.rs::C2RustUnnamed_13",
        hmux2::src::cmd_save_buffer::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_select_layout.rs::C2RustUnnamed_12",
        hmux2::src::cmd_select_layout::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_select_layout.rs::C2RustUnnamed_13",
        hmux2::src::cmd_select_layout::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_select_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_select_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_select_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_select_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_select_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_select_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_select_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_select_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_send_keys.rs::C2RustUnnamed_12",
        hmux2::src::cmd_send_keys::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_send_keys.rs::C2RustUnnamed_13",
        hmux2::src::cmd_send_keys::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_server_access.rs::C2RustUnnamed_12",
        hmux2::src::cmd_server_access::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_server_access.rs::C2RustUnnamed_13",
        hmux2::src::cmd_server_access::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_set_buffer.rs::C2RustUnnamed_12",
        hmux2::src::cmd_set_buffer::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_set_buffer.rs::C2RustUnnamed_13",
        hmux2::src::cmd_set_buffer::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_set_environment.rs::C2RustUnnamed_12",
        hmux2::src::cmd_set_environment::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_set_environment.rs::C2RustUnnamed_13",
        hmux2::src::cmd_set_environment::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_set_option.rs::C2RustUnnamed_12",
        hmux2::src::cmd_set_option::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_set_option.rs::C2RustUnnamed_13",
        hmux2::src::cmd_set_option::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_show_environment.rs::C2RustUnnamed_12",
        hmux2::src::cmd_show_environment::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_show_environment.rs::C2RustUnnamed_13",
        hmux2::src::cmd_show_environment::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_show_messages.rs::C2RustUnnamed_12",
        hmux2::src::cmd_show_messages::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_show_messages.rs::C2RustUnnamed_13",
        hmux2::src::cmd_show_messages::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_show_options.rs::C2RustUnnamed_12",
        hmux2::src::cmd_show_options::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_show_options.rs::C2RustUnnamed_13",
        hmux2::src::cmd_show_options::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_source_file.rs::C2RustUnnamed_13",
        hmux2::src::cmd_source_file::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_source_file.rs::C2RustUnnamed_14",
        hmux2::src::cmd_source_file::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_split_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_split_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_split_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_split_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_swap_pane.rs::C2RustUnnamed_12",
        hmux2::src::cmd_swap_pane::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_swap_pane.rs::C2RustUnnamed_13",
        hmux2::src::cmd_swap_pane::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_swap_window.rs::C2RustUnnamed_12",
        hmux2::src::cmd_swap_window::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_swap_window.rs::C2RustUnnamed_13",
        hmux2::src::cmd_swap_window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_switch_client.rs::C2RustUnnamed_12",
        hmux2::src::cmd_switch_client::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_switch_client.rs::C2RustUnnamed_13",
        hmux2::src::cmd_switch_client::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/cmd_wait_for.rs::C2RustUnnamed_12",
        hmux2::src::cmd_wait_for::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/cmd_wait_for.rs::C2RustUnnamed_13",
        hmux2::src::cmd_wait_for::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/colour.rs::C2RustUnnamed_13",
        hmux2::src::colour::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/colour.rs::C2RustUnnamed_14",
        hmux2::src::colour::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/control.rs::C2RustUnnamed_12",
        hmux2::src::control::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/control.rs::C2RustUnnamed_13",
        hmux2::src::control::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/control_notify.rs::C2RustUnnamed_12",
        hmux2::src::control_notify::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/control_notify.rs::C2RustUnnamed_13",
        hmux2::src::control_notify::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/environ.rs::C2RustUnnamed_12",
        hmux2::src::environ::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/environ.rs::C2RustUnnamed_13",
        hmux2::src::environ::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/events.rs::C2RustUnnamed_12",
        hmux2::src::events::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/events.rs::C2RustUnnamed_13",
        hmux2::src::events::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/events_payload.rs::C2RustUnnamed_12",
        hmux2::src::events_payload::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/events_payload.rs::C2RustUnnamed_13",
        hmux2::src::events_payload::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/file.rs::C2RustUnnamed_13",
        hmux2::src::file::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/file.rs::C2RustUnnamed_14",
        hmux2::src::file::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/format.rs::C2RustUnnamed_13",
        hmux2::src::format::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/format.rs::C2RustUnnamed_14",
        hmux2::src::format::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/format_draw.rs::C2RustUnnamed_12",
        hmux2::src::format_draw::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/format_draw.rs::C2RustUnnamed_13",
        hmux2::src::format_draw::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/grid.rs::C2RustUnnamed",
        hmux2::src::grid::C2RustUnnamed,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/grid.rs::C2RustUnnamed_0",
        hmux2::src::grid::C2RustUnnamed_0,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/grid_reader.rs::C2RustUnnamed",
        hmux2::src::grid_reader::C2RustUnnamed,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/grid_reader.rs::C2RustUnnamed_0",
        hmux2::src::grid_reader::C2RustUnnamed_0,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/grid_view.rs::C2RustUnnamed",
        hmux2::src::grid_view::C2RustUnnamed,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/grid_view.rs::C2RustUnnamed_0",
        hmux2::src::grid_view::C2RustUnnamed_0,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/hooks.rs::C2RustUnnamed_12",
        hmux2::src::hooks::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/hooks.rs::C2RustUnnamed_13",
        hmux2::src::hooks::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/input.rs::C2RustUnnamed_12",
        hmux2::src::input::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/input.rs::C2RustUnnamed_13",
        hmux2::src::input::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/input_keys.rs::C2RustUnnamed_12",
        hmux2::src::input_keys::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/input_keys.rs::C2RustUnnamed_13",
        hmux2::src::input_keys::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/job.rs::C2RustUnnamed_13",
        hmux2::src::job::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/job.rs::C2RustUnnamed_14",
        hmux2::src::job::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/key_bindings.rs::C2RustUnnamed_12",
        hmux2::src::key_bindings::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/key_bindings.rs::C2RustUnnamed_13",
        hmux2::src::key_bindings::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/layout.rs::C2RustUnnamed_12",
        hmux2::src::layout::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/layout.rs::C2RustUnnamed_13",
        hmux2::src::layout::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/layout_custom.rs::C2RustUnnamed_13",
        hmux2::src::layout_custom::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/layout_custom.rs::C2RustUnnamed_14",
        hmux2::src::layout_custom::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/layout_set.rs::C2RustUnnamed_12",
        hmux2::src::layout_set::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/layout_set.rs::C2RustUnnamed_13",
        hmux2::src::layout_set::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/menu.rs::C2RustUnnamed_12",
        hmux2::src::menu::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/menu.rs::C2RustUnnamed_13",
        hmux2::src::menu::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/mode_tree.rs::C2RustUnnamed_12",
        hmux2::src::mode_tree::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/mode_tree.rs::C2RustUnnamed_13",
        hmux2::src::mode_tree::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/monitor.rs::C2RustUnnamed_12",
        hmux2::src::monitor::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/monitor.rs::C2RustUnnamed_13",
        hmux2::src::monitor::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/names.rs::C2RustUnnamed_13",
        hmux2::src::names::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/names.rs::C2RustUnnamed_14",
        hmux2::src::names::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/options.rs::C2RustUnnamed_13",
        hmux2::src::options::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/options.rs::C2RustUnnamed_14",
        hmux2::src::options::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/popup.rs::C2RustUnnamed_12",
        hmux2::src::popup::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/popup.rs::C2RustUnnamed_13",
        hmux2::src::popup::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/prompt.rs::C2RustUnnamed_12",
        hmux2::src::prompt::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/prompt.rs::C2RustUnnamed_13",
        hmux2::src::prompt::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/resize.rs::C2RustUnnamed_12",
        hmux2::src::resize::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/resize.rs::C2RustUnnamed_13",
        hmux2::src::resize::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/screen.rs::C2RustUnnamed_12",
        hmux2::src::screen::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/screen.rs::C2RustUnnamed_13",
        hmux2::src::screen::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_12",
        hmux2::src::screen_redraw::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_13",
        hmux2::src::screen_redraw::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/screen_write.rs::C2RustUnnamed_15",
        hmux2::src::screen_write::C2RustUnnamed_15,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/screen_write.rs::C2RustUnnamed_16",
        hmux2::src::screen_write::C2RustUnnamed_16,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/server.rs::C2RustUnnamed_13",
        hmux2::src::server::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/server.rs::C2RustUnnamed_14",
        hmux2::src::server::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/server_acl.rs::C2RustUnnamed_12",
        hmux2::src::server_acl::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/server_acl.rs::C2RustUnnamed_13",
        hmux2::src::server_acl::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/server_client.rs::C2RustUnnamed_13",
        hmux2::src::server_client::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/server_client.rs::C2RustUnnamed_14",
        hmux2::src::server_client::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/server_fn.rs::C2RustUnnamed_12",
        hmux2::src::server_fn::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/server_fn.rs::C2RustUnnamed_13",
        hmux2::src::server_fn::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/session.rs::C2RustUnnamed_12",
        hmux2::src::session::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/session.rs::C2RustUnnamed_13",
        hmux2::src::session::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/sort.rs::C2RustUnnamed_12",
        hmux2::src::sort::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/sort.rs::C2RustUnnamed_13",
        hmux2::src::sort::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/spawn.rs::C2RustUnnamed_12",
        hmux2::src::spawn::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/spawn.rs::C2RustUnnamed_13",
        hmux2::src::spawn::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/status.rs::C2RustUnnamed_12",
        hmux2::src::status::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/status.rs::C2RustUnnamed_13",
        hmux2::src::status::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/style.rs::C2RustUnnamed_12",
        hmux2::src::style::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/style.rs::C2RustUnnamed_13",
        hmux2::src::style::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/tty.rs::C2RustUnnamed",
        hmux2::src::tty::C2RustUnnamed,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/tty.rs::C2RustUnnamed_0",
        hmux2::src::tty::C2RustUnnamed_0,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/tty_acs.rs::C2RustUnnamed_12",
        hmux2::src::tty_acs::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/tty_acs.rs::C2RustUnnamed_13",
        hmux2::src::tty_acs::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/tty_draw.rs::C2RustUnnamed_12",
        hmux2::src::tty_draw::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/tty_draw.rs::C2RustUnnamed_13",
        hmux2::src::tty_draw::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/tty_features.rs::C2RustUnnamed",
        hmux2::src::tty_features::C2RustUnnamed,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/tty_features.rs::C2RustUnnamed_0",
        hmux2::src::tty_features::C2RustUnnamed_0,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/tty_keys.rs::C2RustUnnamed_13",
        hmux2::src::tty_keys::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/tty_keys.rs::C2RustUnnamed_14",
        hmux2::src::tty_keys::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/tty_term.rs::C2RustUnnamed",
        hmux2::src::tty_term::C2RustUnnamed,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/tty_term.rs::C2RustUnnamed_0",
        hmux2::src::tty_term::C2RustUnnamed_0,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window.rs::C2RustUnnamed_13",
        hmux2::src::window::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window.rs::C2RustUnnamed_14",
        hmux2::src::window::C2RustUnnamed_14,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_border.rs::C2RustUnnamed_12",
        hmux2::src::window_border::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_border.rs::C2RustUnnamed_13",
        hmux2::src::window_border::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_buffer.rs::C2RustUnnamed_12",
        hmux2::src::window_buffer::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_buffer.rs::C2RustUnnamed_13",
        hmux2::src::window_buffer::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_client.rs::C2RustUnnamed_12",
        hmux2::src::window_client::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_client.rs::C2RustUnnamed_13",
        hmux2::src::window_client::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_clock.rs::C2RustUnnamed_12",
        hmux2::src::window_clock::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_clock.rs::C2RustUnnamed_13",
        hmux2::src::window_clock::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_copy.rs::C2RustUnnamed_12",
        hmux2::src::window_copy::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_copy.rs::C2RustUnnamed_13",
        hmux2::src::window_copy::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_customize.rs::C2RustUnnamed_12",
        hmux2::src::window_customize::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_customize.rs::C2RustUnnamed_13",
        hmux2::src::window_customize::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_panes.rs::C2RustUnnamed_12",
        hmux2::src::window_panes::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_panes.rs::C2RustUnnamed_13",
        hmux2::src::window_panes::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_switch.rs::C2RustUnnamed_12",
        hmux2::src::window_switch::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_switch.rs::C2RustUnnamed_13",
        hmux2::src::window_switch::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_12",
        hmux2::src::window_tree::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_13",
        hmux2::src::window_tree::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    record!(
        "src/window_visible.rs::C2RustUnnamed_12",
        hmux2::src::window_visible::C2RustUnnamed_12,
        hmux2::src::shared::grid::grid_cell_entry_storage,
        [offset, data]
    );
    record!(
        "src/window_visible.rs::C2RustUnnamed_13",
        hmux2::src::window_visible::C2RustUnnamed_13,
        hmux2::src::shared::grid::grid_cell_entry_data,
        [attr, fg, bg, data]
    );
    assert_eq!(
        records.join("\n") + "\n",
        include_str!("fixtures/grid-storage-layout.txt")
    );
}
