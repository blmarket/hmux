//! Frozen pre-migration sizes, alignments, and every named field offset.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_copies_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, [$($field:ident),*]) => {
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!("src/arguments.rs::cmd", *mut hmux2::src::arguments::cmd, []);
    record!(
        "src/cmd.rs::cmd",
        hmux2::src::cmd::cmd,
        [entry, args, group, file, line, parse_flags, qentry]
    );
    record!(
        "src/cmd_attach_session.rs::cmd",
        *mut hmux2::src::cmd_attach_session::cmd,
        []
    );
    record!(
        "src/cmd_bind_key.rs::cmd",
        *mut hmux2::src::cmd_bind_key::cmd,
        []
    );
    record!(
        "src/cmd_break_pane.rs::cmd",
        *mut hmux2::src::cmd_break_pane::cmd,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::cmd",
        *mut hmux2::src::cmd_capture_pane::cmd,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::cmd",
        *mut hmux2::src::cmd_choose_tree::cmd,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::cmd",
        *mut hmux2::src::cmd_command_prompt::cmd,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::cmd",
        *mut hmux2::src::cmd_confirm_before::cmd,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::cmd",
        *mut hmux2::src::cmd_copy_mode::cmd,
        []
    );
    record!(
        "src/cmd_detach_client.rs::cmd",
        *mut hmux2::src::cmd_detach_client::cmd,
        []
    );
    record!(
        "src/cmd_display_menu.rs::cmd",
        *mut hmux2::src::cmd_display_menu::cmd,
        []
    );
    record!(
        "src/cmd_display_message.rs::cmd",
        *mut hmux2::src::cmd_display_message::cmd,
        []
    );
    record!(
        "src/cmd_find_window.rs::cmd",
        *mut hmux2::src::cmd_find_window::cmd,
        []
    );
    record!(
        "src/cmd_if_shell.rs::cmd",
        *mut hmux2::src::cmd_if_shell::cmd,
        []
    );
    record!(
        "src/cmd_join_pane.rs::cmd",
        *mut hmux2::src::cmd_join_pane::cmd,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::cmd",
        *mut hmux2::src::cmd_kill_pane::cmd,
        []
    );
    record!(
        "src/cmd_kill_server.rs::cmd",
        *mut hmux2::src::cmd_kill_server::cmd,
        []
    );
    record!(
        "src/cmd_kill_session.rs::cmd",
        *mut hmux2::src::cmd_kill_session::cmd,
        []
    );
    record!(
        "src/cmd_kill_window.rs::cmd",
        *mut hmux2::src::cmd_kill_window::cmd,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::cmd",
        *mut hmux2::src::cmd_list_buffers::cmd,
        []
    );
    record!(
        "src/cmd_list_clients.rs::cmd",
        *mut hmux2::src::cmd_list_clients::cmd,
        []
    );
    record!(
        "src/cmd_list_commands.rs::cmd",
        *mut hmux2::src::cmd_list_commands::cmd,
        []
    );
    record!(
        "src/cmd_list_keys.rs::cmd",
        *mut hmux2::src::cmd_list_keys::cmd,
        []
    );
    record!(
        "src/cmd_list_panes.rs::cmd",
        *mut hmux2::src::cmd_list_panes::cmd,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::cmd",
        *mut hmux2::src::cmd_list_sessions::cmd,
        []
    );
    record!(
        "src/cmd_list_windows.rs::cmd",
        *mut hmux2::src::cmd_list_windows::cmd,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::cmd",
        *mut hmux2::src::cmd_load_buffer::cmd,
        []
    );
    record!(
        "src/cmd_lock_server.rs::cmd",
        *mut hmux2::src::cmd_lock_server::cmd,
        []
    );
    record!(
        "src/cmd_move_window.rs::cmd",
        *mut hmux2::src::cmd_move_window::cmd,
        []
    );
    record!(
        "src/cmd_new_session.rs::cmd",
        *mut hmux2::src::cmd_new_session::cmd,
        []
    );
    record!(
        "src/cmd_new_window.rs::cmd",
        *mut hmux2::src::cmd_new_window::cmd,
        []
    );
    record!("src/cmd_parse.rs::cmd", *mut hmux2::src::cmd_parse::cmd, []);
    record!(
        "src/cmd_paste_buffer.rs::cmd",
        *mut hmux2::src::cmd_paste_buffer::cmd,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::cmd",
        *mut hmux2::src::cmd_pipe_pane::cmd,
        []
    );
    record!("src/cmd_queue.rs::cmd", *mut hmux2::src::cmd_queue::cmd, []);
    record!(
        "src/cmd_refresh_client.rs::cmd",
        *mut hmux2::src::cmd_refresh_client::cmd,
        []
    );
    record!(
        "src/cmd_rename_session.rs::cmd",
        *mut hmux2::src::cmd_rename_session::cmd,
        []
    );
    record!(
        "src/cmd_rename_window.rs::cmd",
        *mut hmux2::src::cmd_rename_window::cmd,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::cmd",
        *mut hmux2::src::cmd_resize_pane::cmd,
        []
    );
    record!(
        "src/cmd_resize_window.rs::cmd",
        *mut hmux2::src::cmd_resize_window::cmd,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::cmd",
        *mut hmux2::src::cmd_respawn_pane::cmd,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::cmd",
        *mut hmux2::src::cmd_respawn_window::cmd,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::cmd",
        *mut hmux2::src::cmd_rotate_window::cmd,
        []
    );
    record!(
        "src/cmd_run_shell.rs::cmd",
        *mut hmux2::src::cmd_run_shell::cmd,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::cmd",
        *mut hmux2::src::cmd_save_buffer::cmd,
        []
    );
    record!(
        "src/cmd_select_layout.rs::cmd",
        *mut hmux2::src::cmd_select_layout::cmd,
        []
    );
    record!(
        "src/cmd_select_pane.rs::cmd",
        *mut hmux2::src::cmd_select_pane::cmd,
        []
    );
    record!(
        "src/cmd_select_window.rs::cmd",
        *mut hmux2::src::cmd_select_window::cmd,
        []
    );
    record!(
        "src/cmd_send_keys.rs::cmd",
        *mut hmux2::src::cmd_send_keys::cmd,
        []
    );
    record!(
        "src/cmd_server_access.rs::cmd",
        *mut hmux2::src::cmd_server_access::cmd,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::cmd",
        *mut hmux2::src::cmd_set_buffer::cmd,
        []
    );
    record!(
        "src/cmd_set_environment.rs::cmd",
        *mut hmux2::src::cmd_set_environment::cmd,
        []
    );
    record!(
        "src/cmd_set_option.rs::cmd",
        *mut hmux2::src::cmd_set_option::cmd,
        []
    );
    record!(
        "src/cmd_show_environment.rs::cmd",
        *mut hmux2::src::cmd_show_environment::cmd,
        []
    );
    record!(
        "src/cmd_show_messages.rs::cmd",
        *mut hmux2::src::cmd_show_messages::cmd,
        []
    );
    record!(
        "src/cmd_show_options.rs::cmd",
        *mut hmux2::src::cmd_show_options::cmd,
        []
    );
    record!(
        "src/cmd_show_prompt_history.rs::cmd",
        *mut hmux2::src::cmd_show_prompt_history::cmd,
        []
    );
    record!(
        "src/cmd_source_file.rs::cmd",
        *mut hmux2::src::cmd_source_file::cmd,
        []
    );
    record!(
        "src/cmd_split_window.rs::cmd",
        *mut hmux2::src::cmd_split_window::cmd,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::cmd",
        *mut hmux2::src::cmd_swap_pane::cmd,
        []
    );
    record!(
        "src/cmd_swap_window.rs::cmd",
        *mut hmux2::src::cmd_swap_window::cmd,
        []
    );
    record!(
        "src/cmd_switch_client.rs::cmd",
        *mut hmux2::src::cmd_switch_client::cmd,
        []
    );
    record!(
        "src/cmd_unbind_key.rs::cmd",
        *mut hmux2::src::cmd_unbind_key::cmd,
        []
    );
    record!(
        "src/cmd_wait_for.rs::cmd",
        *mut hmux2::src::cmd_wait_for::cmd,
        []
    );
    record!("src/prompt.rs::cmd", *mut hmux2::src::prompt::cmd, []);
    record!(
        "src/window_panes.rs::cmd",
        *mut hmux2::src::window_panes::cmd,
        []
    );
    record!(
        "src/arguments.rs::cmd_entry",
        hmux2::src::arguments::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd.rs::cmd_entry",
        hmux2::src::cmd::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_attach_session.rs::cmd_entry",
        hmux2::src::cmd_attach_session::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_bind_key.rs::cmd_entry",
        hmux2::src::cmd_bind_key::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_break_pane.rs::cmd_entry",
        hmux2::src::cmd_break_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_capture_pane.rs::cmd_entry",
        hmux2::src::cmd_capture_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_choose_tree.rs::cmd_entry",
        hmux2::src::cmd_choose_tree::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_command_prompt.rs::cmd_entry",
        hmux2::src::cmd_command_prompt::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_confirm_before.rs::cmd_entry",
        hmux2::src::cmd_confirm_before::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_copy_mode.rs::cmd_entry",
        hmux2::src::cmd_copy_mode::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_detach_client.rs::cmd_entry",
        hmux2::src::cmd_detach_client::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_display_menu.rs::cmd_entry",
        hmux2::src::cmd_display_menu::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_display_message.rs::cmd_entry",
        hmux2::src::cmd_display_message::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_find_window.rs::cmd_entry",
        hmux2::src::cmd_find_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_if_shell.rs::cmd_entry",
        hmux2::src::cmd_if_shell::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_join_pane.rs::cmd_entry",
        hmux2::src::cmd_join_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_kill_pane.rs::cmd_entry",
        hmux2::src::cmd_kill_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_kill_server.rs::cmd_entry",
        hmux2::src::cmd_kill_server::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_kill_session.rs::cmd_entry",
        hmux2::src::cmd_kill_session::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_kill_window.rs::cmd_entry",
        hmux2::src::cmd_kill_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_buffers.rs::cmd_entry",
        hmux2::src::cmd_list_buffers::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_clients.rs::cmd_entry",
        hmux2::src::cmd_list_clients::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_commands.rs::cmd_entry",
        hmux2::src::cmd_list_commands::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_keys.rs::cmd_entry",
        hmux2::src::cmd_list_keys::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_panes.rs::cmd_entry",
        hmux2::src::cmd_list_panes::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_sessions.rs::cmd_entry",
        hmux2::src::cmd_list_sessions::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_list_windows.rs::cmd_entry",
        hmux2::src::cmd_list_windows::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_load_buffer.rs::cmd_entry",
        hmux2::src::cmd_load_buffer::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_lock_server.rs::cmd_entry",
        hmux2::src::cmd_lock_server::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_move_window.rs::cmd_entry",
        hmux2::src::cmd_move_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_new_session.rs::cmd_entry",
        hmux2::src::cmd_new_session::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_new_window.rs::cmd_entry",
        hmux2::src::cmd_new_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_paste_buffer.rs::cmd_entry",
        hmux2::src::cmd_paste_buffer::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_pipe_pane.rs::cmd_entry",
        hmux2::src::cmd_pipe_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_queue.rs::cmd_entry",
        hmux2::src::cmd_queue::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_refresh_client.rs::cmd_entry",
        hmux2::src::cmd_refresh_client::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_rename_session.rs::cmd_entry",
        hmux2::src::cmd_rename_session::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_rename_window.rs::cmd_entry",
        hmux2::src::cmd_rename_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_resize_pane.rs::cmd_entry",
        hmux2::src::cmd_resize_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_resize_window.rs::cmd_entry",
        hmux2::src::cmd_resize_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_respawn_pane.rs::cmd_entry",
        hmux2::src::cmd_respawn_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_respawn_window.rs::cmd_entry",
        hmux2::src::cmd_respawn_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_rotate_window.rs::cmd_entry",
        hmux2::src::cmd_rotate_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_run_shell.rs::cmd_entry",
        hmux2::src::cmd_run_shell::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_save_buffer.rs::cmd_entry",
        hmux2::src::cmd_save_buffer::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_select_layout.rs::cmd_entry",
        hmux2::src::cmd_select_layout::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_select_pane.rs::cmd_entry",
        hmux2::src::cmd_select_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_select_window.rs::cmd_entry",
        hmux2::src::cmd_select_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_send_keys.rs::cmd_entry",
        hmux2::src::cmd_send_keys::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_server_access.rs::cmd_entry",
        hmux2::src::cmd_server_access::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_set_buffer.rs::cmd_entry",
        hmux2::src::cmd_set_buffer::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_set_environment.rs::cmd_entry",
        hmux2::src::cmd_set_environment::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_set_option.rs::cmd_entry",
        hmux2::src::cmd_set_option::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_show_environment.rs::cmd_entry",
        hmux2::src::cmd_show_environment::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_show_messages.rs::cmd_entry",
        hmux2::src::cmd_show_messages::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_show_options.rs::cmd_entry",
        hmux2::src::cmd_show_options::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_show_prompt_history.rs::cmd_entry",
        hmux2::src::cmd_show_prompt_history::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_source_file.rs::cmd_entry",
        hmux2::src::cmd_source_file::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_split_window.rs::cmd_entry",
        hmux2::src::cmd_split_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_swap_pane.rs::cmd_entry",
        hmux2::src::cmd_swap_pane::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_swap_window.rs::cmd_entry",
        hmux2::src::cmd_swap_window::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_switch_client.rs::cmd_entry",
        hmux2::src::cmd_switch_client::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_unbind_key.rs::cmd_entry",
        hmux2::src::cmd_unbind_key::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/cmd_wait_for.rs::cmd_entry",
        hmux2::src::cmd_wait_for::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/prompt.rs::cmd_entry",
        hmux2::src::prompt::cmd_entry,
        [name, alias, args, usage, source, target, flags, exec]
    );
    record!(
        "src/arguments.rs::cmd_entry_flag",
        hmux2::src::arguments::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd.rs::cmd_entry_flag",
        hmux2::src::cmd::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_attach_session.rs::cmd_entry_flag",
        hmux2::src::cmd_attach_session::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_bind_key.rs::cmd_entry_flag",
        hmux2::src::cmd_bind_key::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_break_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_break_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_capture_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_capture_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_choose_tree.rs::cmd_entry_flag",
        hmux2::src::cmd_choose_tree::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_command_prompt.rs::cmd_entry_flag",
        hmux2::src::cmd_command_prompt::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_confirm_before.rs::cmd_entry_flag",
        hmux2::src::cmd_confirm_before::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_copy_mode.rs::cmd_entry_flag",
        hmux2::src::cmd_copy_mode::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_detach_client.rs::cmd_entry_flag",
        hmux2::src::cmd_detach_client::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_display_menu.rs::cmd_entry_flag",
        hmux2::src::cmd_display_menu::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_display_message.rs::cmd_entry_flag",
        hmux2::src::cmd_display_message::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_find_window.rs::cmd_entry_flag",
        hmux2::src::cmd_find_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_if_shell.rs::cmd_entry_flag",
        hmux2::src::cmd_if_shell::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_join_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_join_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_kill_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_kill_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_kill_server.rs::cmd_entry_flag",
        hmux2::src::cmd_kill_server::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_kill_session.rs::cmd_entry_flag",
        hmux2::src::cmd_kill_session::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_kill_window.rs::cmd_entry_flag",
        hmux2::src::cmd_kill_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_buffers.rs::cmd_entry_flag",
        hmux2::src::cmd_list_buffers::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_clients.rs::cmd_entry_flag",
        hmux2::src::cmd_list_clients::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_commands.rs::cmd_entry_flag",
        hmux2::src::cmd_list_commands::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_keys.rs::cmd_entry_flag",
        hmux2::src::cmd_list_keys::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_panes.rs::cmd_entry_flag",
        hmux2::src::cmd_list_panes::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_sessions.rs::cmd_entry_flag",
        hmux2::src::cmd_list_sessions::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_list_windows.rs::cmd_entry_flag",
        hmux2::src::cmd_list_windows::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_load_buffer.rs::cmd_entry_flag",
        hmux2::src::cmd_load_buffer::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_lock_server.rs::cmd_entry_flag",
        hmux2::src::cmd_lock_server::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_move_window.rs::cmd_entry_flag",
        hmux2::src::cmd_move_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_new_session.rs::cmd_entry_flag",
        hmux2::src::cmd_new_session::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_new_window.rs::cmd_entry_flag",
        hmux2::src::cmd_new_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_paste_buffer.rs::cmd_entry_flag",
        hmux2::src::cmd_paste_buffer::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_pipe_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_pipe_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_queue.rs::cmd_entry_flag",
        hmux2::src::cmd_queue::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_refresh_client.rs::cmd_entry_flag",
        hmux2::src::cmd_refresh_client::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_rename_session.rs::cmd_entry_flag",
        hmux2::src::cmd_rename_session::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_rename_window.rs::cmd_entry_flag",
        hmux2::src::cmd_rename_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_resize_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_resize_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_resize_window.rs::cmd_entry_flag",
        hmux2::src::cmd_resize_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_respawn_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_respawn_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_respawn_window.rs::cmd_entry_flag",
        hmux2::src::cmd_respawn_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_rotate_window.rs::cmd_entry_flag",
        hmux2::src::cmd_rotate_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_run_shell.rs::cmd_entry_flag",
        hmux2::src::cmd_run_shell::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_save_buffer.rs::cmd_entry_flag",
        hmux2::src::cmd_save_buffer::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_select_layout.rs::cmd_entry_flag",
        hmux2::src::cmd_select_layout::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_select_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_select_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_select_window.rs::cmd_entry_flag",
        hmux2::src::cmd_select_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_send_keys.rs::cmd_entry_flag",
        hmux2::src::cmd_send_keys::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_server_access.rs::cmd_entry_flag",
        hmux2::src::cmd_server_access::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_set_buffer.rs::cmd_entry_flag",
        hmux2::src::cmd_set_buffer::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_set_environment.rs::cmd_entry_flag",
        hmux2::src::cmd_set_environment::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_set_option.rs::cmd_entry_flag",
        hmux2::src::cmd_set_option::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_show_environment.rs::cmd_entry_flag",
        hmux2::src::cmd_show_environment::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_show_messages.rs::cmd_entry_flag",
        hmux2::src::cmd_show_messages::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_show_options.rs::cmd_entry_flag",
        hmux2::src::cmd_show_options::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_show_prompt_history.rs::cmd_entry_flag",
        hmux2::src::cmd_show_prompt_history::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_source_file.rs::cmd_entry_flag",
        hmux2::src::cmd_source_file::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_split_window.rs::cmd_entry_flag",
        hmux2::src::cmd_split_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_swap_pane.rs::cmd_entry_flag",
        hmux2::src::cmd_swap_pane::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_swap_window.rs::cmd_entry_flag",
        hmux2::src::cmd_swap_window::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_switch_client.rs::cmd_entry_flag",
        hmux2::src::cmd_switch_client::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_unbind_key.rs::cmd_entry_flag",
        hmux2::src::cmd_unbind_key::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/cmd_wait_for.rs::cmd_entry_flag",
        hmux2::src::cmd_wait_for::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/prompt.rs::cmd_entry_flag",
        hmux2::src::prompt::cmd_entry_flag,
        [flag, type_0, flags]
    );
    record!(
        "src/alerts.rs::cmd_find_state",
        hmux2::src::alerts::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/arguments.rs::cmd_find_state",
        hmux2::src::arguments::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cfg.rs::cmd_find_state",
        hmux2::src::cfg::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/client.rs::cmd_find_state",
        hmux2::src::client::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd.rs::cmd_find_state",
        hmux2::src::cmd::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_attach_session.rs::cmd_find_state",
        hmux2::src::cmd_attach_session::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_bind_key.rs::cmd_find_state",
        hmux2::src::cmd_bind_key::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_break_pane.rs::cmd_find_state",
        hmux2::src::cmd_break_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_capture_pane.rs::cmd_find_state",
        hmux2::src::cmd_capture_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_choose_tree.rs::cmd_find_state",
        hmux2::src::cmd_choose_tree::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_command_prompt.rs::cmd_find_state",
        hmux2::src::cmd_command_prompt::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_confirm_before.rs::cmd_find_state",
        hmux2::src::cmd_confirm_before::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_copy_mode.rs::cmd_find_state",
        hmux2::src::cmd_copy_mode::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_detach_client.rs::cmd_find_state",
        hmux2::src::cmd_detach_client::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_display_menu.rs::cmd_find_state",
        hmux2::src::cmd_display_menu::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_display_message.rs::cmd_find_state",
        hmux2::src::cmd_display_message::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_find.rs::cmd_find_state",
        hmux2::src::cmd_find::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_find_window.rs::cmd_find_state",
        hmux2::src::cmd_find_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_if_shell.rs::cmd_find_state",
        hmux2::src::cmd_if_shell::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_join_pane.rs::cmd_find_state",
        hmux2::src::cmd_join_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_kill_pane.rs::cmd_find_state",
        hmux2::src::cmd_kill_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_kill_session.rs::cmd_find_state",
        hmux2::src::cmd_kill_session::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_kill_window.rs::cmd_find_state",
        hmux2::src::cmd_kill_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_buffers.rs::cmd_find_state",
        hmux2::src::cmd_list_buffers::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_clients.rs::cmd_find_state",
        hmux2::src::cmd_list_clients::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_commands.rs::cmd_find_state",
        hmux2::src::cmd_list_commands::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_keys.rs::cmd_find_state",
        hmux2::src::cmd_list_keys::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_panes.rs::cmd_find_state",
        hmux2::src::cmd_list_panes::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_sessions.rs::cmd_find_state",
        hmux2::src::cmd_list_sessions::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_list_windows.rs::cmd_find_state",
        hmux2::src::cmd_list_windows::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_load_buffer.rs::cmd_find_state",
        hmux2::src::cmd_load_buffer::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_lock_server.rs::cmd_find_state",
        hmux2::src::cmd_lock_server::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_move_window.rs::cmd_find_state",
        hmux2::src::cmd_move_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_new_session.rs::cmd_find_state",
        hmux2::src::cmd_new_session::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_new_window.rs::cmd_find_state",
        hmux2::src::cmd_new_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_parse.rs::cmd_find_state",
        hmux2::src::cmd_parse::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_paste_buffer.rs::cmd_find_state",
        hmux2::src::cmd_paste_buffer::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_pipe_pane.rs::cmd_find_state",
        hmux2::src::cmd_pipe_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_queue.rs::cmd_find_state",
        hmux2::src::cmd_queue::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_refresh_client.rs::cmd_find_state",
        hmux2::src::cmd_refresh_client::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_rename_session.rs::cmd_find_state",
        hmux2::src::cmd_rename_session::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_rename_window.rs::cmd_find_state",
        hmux2::src::cmd_rename_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_resize_pane.rs::cmd_find_state",
        hmux2::src::cmd_resize_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_resize_window.rs::cmd_find_state",
        hmux2::src::cmd_resize_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_respawn_pane.rs::cmd_find_state",
        hmux2::src::cmd_respawn_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_respawn_window.rs::cmd_find_state",
        hmux2::src::cmd_respawn_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_rotate_window.rs::cmd_find_state",
        hmux2::src::cmd_rotate_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_run_shell.rs::cmd_find_state",
        hmux2::src::cmd_run_shell::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_save_buffer.rs::cmd_find_state",
        hmux2::src::cmd_save_buffer::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_select_layout.rs::cmd_find_state",
        hmux2::src::cmd_select_layout::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_select_pane.rs::cmd_find_state",
        hmux2::src::cmd_select_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_select_window.rs::cmd_find_state",
        hmux2::src::cmd_select_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_send_keys.rs::cmd_find_state",
        hmux2::src::cmd_send_keys::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_server_access.rs::cmd_find_state",
        hmux2::src::cmd_server_access::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_set_buffer.rs::cmd_find_state",
        hmux2::src::cmd_set_buffer::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_set_environment.rs::cmd_find_state",
        hmux2::src::cmd_set_environment::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_set_option.rs::cmd_find_state",
        hmux2::src::cmd_set_option::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_show_environment.rs::cmd_find_state",
        hmux2::src::cmd_show_environment::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_show_messages.rs::cmd_find_state",
        hmux2::src::cmd_show_messages::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_show_options.rs::cmd_find_state",
        hmux2::src::cmd_show_options::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_source_file.rs::cmd_find_state",
        hmux2::src::cmd_source_file::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_split_window.rs::cmd_find_state",
        hmux2::src::cmd_split_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_swap_pane.rs::cmd_find_state",
        hmux2::src::cmd_swap_pane::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_swap_window.rs::cmd_find_state",
        hmux2::src::cmd_swap_window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_switch_client.rs::cmd_find_state",
        hmux2::src::cmd_switch_client::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/cmd_wait_for.rs::cmd_find_state",
        hmux2::src::cmd_wait_for::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/colour.rs::cmd_find_state",
        hmux2::src::colour::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/control.rs::cmd_find_state",
        hmux2::src::control::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/control_notify.rs::cmd_find_state",
        hmux2::src::control_notify::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/environ.rs::cmd_find_state",
        hmux2::src::environ::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/events.rs::cmd_find_state",
        hmux2::src::events::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/events_payload.rs::cmd_find_state",
        hmux2::src::events_payload::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/file.rs::cmd_find_state",
        hmux2::src::file::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/format.rs::cmd_find_state",
        hmux2::src::format::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/format_draw.rs::cmd_find_state",
        hmux2::src::format_draw::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/hooks.rs::cmd_find_state",
        hmux2::src::hooks::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/input.rs::cmd_find_state",
        hmux2::src::input::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/input_keys.rs::cmd_find_state",
        hmux2::src::input_keys::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/job.rs::cmd_find_state",
        hmux2::src::job::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/key_bindings.rs::cmd_find_state",
        hmux2::src::key_bindings::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/layout.rs::cmd_find_state",
        hmux2::src::layout::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/layout_custom.rs::cmd_find_state",
        hmux2::src::layout_custom::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/layout_set.rs::cmd_find_state",
        hmux2::src::layout_set::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/menu.rs::cmd_find_state",
        hmux2::src::menu::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/mode_tree.rs::cmd_find_state",
        hmux2::src::mode_tree::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/monitor.rs::cmd_find_state",
        hmux2::src::monitor::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/names.rs::cmd_find_state",
        hmux2::src::names::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/options.rs::cmd_find_state",
        hmux2::src::options::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/popup.rs::cmd_find_state",
        hmux2::src::popup::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/prompt.rs::cmd_find_state",
        hmux2::src::prompt::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/resize.rs::cmd_find_state",
        hmux2::src::resize::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/screen.rs::cmd_find_state",
        hmux2::src::screen::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/screen_redraw.rs::cmd_find_state",
        hmux2::src::screen_redraw::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/screen_write.rs::cmd_find_state",
        hmux2::src::screen_write::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/server.rs::cmd_find_state",
        hmux2::src::server::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/server_acl.rs::cmd_find_state",
        hmux2::src::server_acl::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/server_client.rs::cmd_find_state",
        hmux2::src::server_client::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/server_fn.rs::cmd_find_state",
        hmux2::src::server_fn::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/session.rs::cmd_find_state",
        hmux2::src::session::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/sort.rs::cmd_find_state",
        hmux2::src::sort::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/spawn.rs::cmd_find_state",
        hmux2::src::spawn::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/status.rs::cmd_find_state",
        hmux2::src::status::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/style.rs::cmd_find_state",
        hmux2::src::style::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/tty.rs::cmd_find_state",
        hmux2::src::tty::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/tty_acs.rs::cmd_find_state",
        hmux2::src::tty_acs::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/tty_draw.rs::cmd_find_state",
        hmux2::src::tty_draw::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/tty_features.rs::cmd_find_state",
        hmux2::src::tty_features::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/tty_keys.rs::cmd_find_state",
        hmux2::src::tty_keys::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/tty_term.rs::cmd_find_state",
        hmux2::src::tty_term::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window.rs::cmd_find_state",
        hmux2::src::window::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_border.rs::cmd_find_state",
        hmux2::src::window_border::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_buffer.rs::cmd_find_state",
        hmux2::src::window_buffer::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_client.rs::cmd_find_state",
        hmux2::src::window_client::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_clock.rs::cmd_find_state",
        hmux2::src::window_clock::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_copy.rs::cmd_find_state",
        hmux2::src::window_copy::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_customize.rs::cmd_find_state",
        hmux2::src::window_customize::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_panes.rs::cmd_find_state",
        hmux2::src::window_panes::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_switch.rs::cmd_find_state",
        hmux2::src::window_switch::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_tree.rs::cmd_find_state",
        hmux2::src::window_tree::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/window_visible.rs::cmd_find_state",
        hmux2::src::window_visible::cmd_find_state,
        [flags, current, s, wl, w, wp, idx]
    );
    record!(
        "src/alerts.rs::cmd_list",
        hmux2::src::alerts::cmd_list,
        [references, group, list]
    );
    record!(
        "src/arguments.rs::cmd_list",
        hmux2::src::arguments::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cfg.rs::cmd_list",
        hmux2::src::cfg::cmd_list,
        [references, group, list]
    );
    record!(
        "src/client.rs::cmd_list",
        hmux2::src::client::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd.rs::cmd_list",
        hmux2::src::cmd::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_attach_session.rs::cmd_list",
        hmux2::src::cmd_attach_session::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_bind_key.rs::cmd_list",
        hmux2::src::cmd_bind_key::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_break_pane.rs::cmd_list",
        hmux2::src::cmd_break_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_capture_pane.rs::cmd_list",
        hmux2::src::cmd_capture_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_choose_tree.rs::cmd_list",
        hmux2::src::cmd_choose_tree::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_command_prompt.rs::cmd_list",
        hmux2::src::cmd_command_prompt::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_confirm_before.rs::cmd_list",
        hmux2::src::cmd_confirm_before::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_copy_mode.rs::cmd_list",
        hmux2::src::cmd_copy_mode::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_detach_client.rs::cmd_list",
        hmux2::src::cmd_detach_client::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_display_menu.rs::cmd_list",
        hmux2::src::cmd_display_menu::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_display_message.rs::cmd_list",
        hmux2::src::cmd_display_message::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_find.rs::cmd_list",
        hmux2::src::cmd_find::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_find_window.rs::cmd_list",
        hmux2::src::cmd_find_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_if_shell.rs::cmd_list",
        hmux2::src::cmd_if_shell::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_join_pane.rs::cmd_list",
        hmux2::src::cmd_join_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_kill_pane.rs::cmd_list",
        hmux2::src::cmd_kill_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_kill_session.rs::cmd_list",
        hmux2::src::cmd_kill_session::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_kill_window.rs::cmd_list",
        hmux2::src::cmd_kill_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_buffers.rs::cmd_list",
        hmux2::src::cmd_list_buffers::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_clients.rs::cmd_list",
        hmux2::src::cmd_list_clients::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_commands.rs::cmd_list",
        hmux2::src::cmd_list_commands::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_keys.rs::cmd_list",
        hmux2::src::cmd_list_keys::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_panes.rs::cmd_list",
        hmux2::src::cmd_list_panes::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_sessions.rs::cmd_list",
        hmux2::src::cmd_list_sessions::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_list_windows.rs::cmd_list",
        hmux2::src::cmd_list_windows::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_load_buffer.rs::cmd_list",
        hmux2::src::cmd_load_buffer::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_lock_server.rs::cmd_list",
        hmux2::src::cmd_lock_server::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_move_window.rs::cmd_list",
        hmux2::src::cmd_move_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_new_session.rs::cmd_list",
        hmux2::src::cmd_new_session::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_new_window.rs::cmd_list",
        hmux2::src::cmd_new_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_parse.rs::cmd_list",
        hmux2::src::cmd_parse::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_paste_buffer.rs::cmd_list",
        hmux2::src::cmd_paste_buffer::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_pipe_pane.rs::cmd_list",
        hmux2::src::cmd_pipe_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_queue.rs::cmd_list",
        hmux2::src::cmd_queue::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_refresh_client.rs::cmd_list",
        hmux2::src::cmd_refresh_client::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_rename_session.rs::cmd_list",
        hmux2::src::cmd_rename_session::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_rename_window.rs::cmd_list",
        hmux2::src::cmd_rename_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_resize_pane.rs::cmd_list",
        hmux2::src::cmd_resize_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_resize_window.rs::cmd_list",
        hmux2::src::cmd_resize_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_respawn_pane.rs::cmd_list",
        hmux2::src::cmd_respawn_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_respawn_window.rs::cmd_list",
        hmux2::src::cmd_respawn_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_rotate_window.rs::cmd_list",
        hmux2::src::cmd_rotate_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_run_shell.rs::cmd_list",
        hmux2::src::cmd_run_shell::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_save_buffer.rs::cmd_list",
        hmux2::src::cmd_save_buffer::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_select_layout.rs::cmd_list",
        hmux2::src::cmd_select_layout::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_select_pane.rs::cmd_list",
        hmux2::src::cmd_select_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_select_window.rs::cmd_list",
        hmux2::src::cmd_select_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_send_keys.rs::cmd_list",
        hmux2::src::cmd_send_keys::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_server_access.rs::cmd_list",
        hmux2::src::cmd_server_access::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_set_buffer.rs::cmd_list",
        hmux2::src::cmd_set_buffer::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_set_environment.rs::cmd_list",
        hmux2::src::cmd_set_environment::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_set_option.rs::cmd_list",
        hmux2::src::cmd_set_option::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_show_environment.rs::cmd_list",
        hmux2::src::cmd_show_environment::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_show_messages.rs::cmd_list",
        hmux2::src::cmd_show_messages::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_show_options.rs::cmd_list",
        hmux2::src::cmd_show_options::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_source_file.rs::cmd_list",
        hmux2::src::cmd_source_file::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_split_window.rs::cmd_list",
        hmux2::src::cmd_split_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_swap_pane.rs::cmd_list",
        hmux2::src::cmd_swap_pane::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_swap_window.rs::cmd_list",
        hmux2::src::cmd_swap_window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_switch_client.rs::cmd_list",
        hmux2::src::cmd_switch_client::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_unbind_key.rs::cmd_list",
        hmux2::src::cmd_unbind_key::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd_wait_for.rs::cmd_list",
        hmux2::src::cmd_wait_for::cmd_list,
        [references, group, list]
    );
    record!(
        "src/colour.rs::cmd_list",
        hmux2::src::colour::cmd_list,
        [references, group, list]
    );
    record!(
        "src/control.rs::cmd_list",
        hmux2::src::control::cmd_list,
        [references, group, list]
    );
    record!(
        "src/control_notify.rs::cmd_list",
        hmux2::src::control_notify::cmd_list,
        [references, group, list]
    );
    record!(
        "src/environ.rs::cmd_list",
        hmux2::src::environ::cmd_list,
        [references, group, list]
    );
    record!(
        "src/events.rs::cmd_list",
        hmux2::src::events::cmd_list,
        [references, group, list]
    );
    record!(
        "src/events_payload.rs::cmd_list",
        hmux2::src::events_payload::cmd_list,
        [references, group, list]
    );
    record!(
        "src/file.rs::cmd_list",
        hmux2::src::file::cmd_list,
        [references, group, list]
    );
    record!(
        "src/format.rs::cmd_list",
        hmux2::src::format::cmd_list,
        [references, group, list]
    );
    record!(
        "src/format_draw.rs::cmd_list",
        hmux2::src::format_draw::cmd_list,
        [references, group, list]
    );
    record!(
        "src/hooks.rs::cmd_list",
        hmux2::src::hooks::cmd_list,
        [references, group, list]
    );
    record!(
        "src/input.rs::cmd_list",
        hmux2::src::input::cmd_list,
        [references, group, list]
    );
    record!(
        "src/input_keys.rs::cmd_list",
        hmux2::src::input_keys::cmd_list,
        [references, group, list]
    );
    record!(
        "src/job.rs::cmd_list",
        hmux2::src::job::cmd_list,
        [references, group, list]
    );
    record!(
        "src/key_bindings.rs::cmd_list",
        hmux2::src::key_bindings::cmd_list,
        [references, group, list]
    );
    record!(
        "src/layout.rs::cmd_list",
        hmux2::src::layout::cmd_list,
        [references, group, list]
    );
    record!(
        "src/layout_custom.rs::cmd_list",
        hmux2::src::layout_custom::cmd_list,
        [references, group, list]
    );
    record!(
        "src/layout_set.rs::cmd_list",
        hmux2::src::layout_set::cmd_list,
        [references, group, list]
    );
    record!(
        "src/menu.rs::cmd_list",
        hmux2::src::menu::cmd_list,
        [references, group, list]
    );
    record!(
        "src/mode_tree.rs::cmd_list",
        hmux2::src::mode_tree::cmd_list,
        [references, group, list]
    );
    record!(
        "src/monitor.rs::cmd_list",
        hmux2::src::monitor::cmd_list,
        [references, group, list]
    );
    record!(
        "src/names.rs::cmd_list",
        hmux2::src::names::cmd_list,
        [references, group, list]
    );
    record!(
        "src/options.rs::cmd_list",
        hmux2::src::options::cmd_list,
        [references, group, list]
    );
    record!(
        "src/popup.rs::cmd_list",
        hmux2::src::popup::cmd_list,
        [references, group, list]
    );
    record!(
        "src/prompt.rs::cmd_list",
        hmux2::src::prompt::cmd_list,
        [references, group, list]
    );
    record!(
        "src/resize.rs::cmd_list",
        hmux2::src::resize::cmd_list,
        [references, group, list]
    );
    record!(
        "src/screen.rs::cmd_list",
        hmux2::src::screen::cmd_list,
        [references, group, list]
    );
    record!(
        "src/screen_redraw.rs::cmd_list",
        hmux2::src::screen_redraw::cmd_list,
        [references, group, list]
    );
    record!(
        "src/screen_write.rs::cmd_list",
        hmux2::src::screen_write::cmd_list,
        [references, group, list]
    );
    record!(
        "src/server.rs::cmd_list",
        hmux2::src::server::cmd_list,
        [references, group, list]
    );
    record!(
        "src/server_acl.rs::cmd_list",
        hmux2::src::server_acl::cmd_list,
        [references, group, list]
    );
    record!(
        "src/server_client.rs::cmd_list",
        hmux2::src::server_client::cmd_list,
        [references, group, list]
    );
    record!(
        "src/server_fn.rs::cmd_list",
        hmux2::src::server_fn::cmd_list,
        [references, group, list]
    );
    record!(
        "src/session.rs::cmd_list",
        hmux2::src::session::cmd_list,
        [references, group, list]
    );
    record!(
        "src/sort.rs::cmd_list",
        hmux2::src::sort::cmd_list,
        [references, group, list]
    );
    record!(
        "src/spawn.rs::cmd_list",
        hmux2::src::spawn::cmd_list,
        [references, group, list]
    );
    record!(
        "src/status.rs::cmd_list",
        hmux2::src::status::cmd_list,
        [references, group, list]
    );
    record!(
        "src/style.rs::cmd_list",
        hmux2::src::style::cmd_list,
        [references, group, list]
    );
    record!(
        "src/tty.rs::cmd_list",
        hmux2::src::tty::cmd_list,
        [references, group, list]
    );
    record!(
        "src/tty_acs.rs::cmd_list",
        hmux2::src::tty_acs::cmd_list,
        [references, group, list]
    );
    record!(
        "src/tty_draw.rs::cmd_list",
        hmux2::src::tty_draw::cmd_list,
        [references, group, list]
    );
    record!(
        "src/tty_features.rs::cmd_list",
        hmux2::src::tty_features::cmd_list,
        [references, group, list]
    );
    record!(
        "src/tty_keys.rs::cmd_list",
        hmux2::src::tty_keys::cmd_list,
        [references, group, list]
    );
    record!(
        "src/tty_term.rs::cmd_list",
        hmux2::src::tty_term::cmd_list,
        [references, group, list]
    );
    record!(
        "src/utf8.rs::cmd_list",
        hmux2::src::utf8::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window.rs::cmd_list",
        hmux2::src::window::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_border.rs::cmd_list",
        hmux2::src::window_border::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_buffer.rs::cmd_list",
        hmux2::src::window_buffer::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_client.rs::cmd_list",
        hmux2::src::window_client::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_clock.rs::cmd_list",
        hmux2::src::window_clock::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_copy.rs::cmd_list",
        hmux2::src::window_copy::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_customize.rs::cmd_list",
        hmux2::src::window_customize::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_panes.rs::cmd_list",
        hmux2::src::window_panes::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_switch.rs::cmd_list",
        hmux2::src::window_switch::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_tree.rs::cmd_list",
        hmux2::src::window_tree::cmd_list,
        [references, group, list]
    );
    record!(
        "src/window_visible.rs::cmd_list",
        hmux2::src::window_visible::cmd_list,
        [references, group, list]
    );
    record!(
        "src/cmd.rs::C2RustUnnamed_33",
        hmux2::src::cmd::cmd_qentry,
        [tqe_next, tqe_prev]
    );
    record!("src/cfg.rs::cmdq_cb", hmux2::src::cfg::cmdq_cb, []);
    record!(
        "src/cmd_queue.rs::cmdq_cb",
        hmux2::src::cmd_queue::cmdq_cb,
        []
    );
    record!(
        "src/cmd_source_file.rs::cmdq_cb",
        hmux2::src::cmd_source_file::cmdq_cb,
        []
    );
    record!("src/control.rs::cmdq_cb", hmux2::src::control::cmdq_cb, []);
    record!(
        "src/key_bindings.rs::cmdq_cb",
        hmux2::src::key_bindings::cmdq_cb,
        []
    );
    record!(
        "src/mode_tree.rs::cmdq_cb",
        hmux2::src::mode_tree::cmdq_cb,
        []
    );
    record!(
        "src/server_client.rs::cmdq_cb",
        hmux2::src::server_client::cmdq_cb,
        []
    );
    record!("src/status.rs::cmdq_cb", hmux2::src::status::cmdq_cb, []);
    record!(
        "src/window_tree.rs::cmdq_cb",
        hmux2::src::window_tree::cmdq_cb,
        []
    );
    record!(
        "src/alerts.rs::cmdq_item",
        *mut hmux2::src::alerts::cmdq_item,
        []
    );
    record!(
        "src/arguments.rs::cmdq_item",
        *mut hmux2::src::arguments::cmdq_item,
        []
    );
    record!("src/cfg.rs::cmdq_item", *mut hmux2::src::cfg::cmdq_item, []);
    record!(
        "src/client.rs::cmdq_item",
        *mut hmux2::src::client::cmdq_item,
        []
    );
    record!("src/cmd.rs::cmdq_item", *mut hmux2::src::cmd::cmdq_item, []);
    record!(
        "src/cmd_attach_session.rs::cmdq_item",
        *mut hmux2::src::cmd_attach_session::cmdq_item,
        []
    );
    record!(
        "src/cmd_bind_key.rs::cmdq_item",
        *mut hmux2::src::cmd_bind_key::cmdq_item,
        []
    );
    record!(
        "src/cmd_break_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_break_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_capture_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::cmdq_item",
        *mut hmux2::src::cmd_choose_tree::cmdq_item,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::cmdq_item",
        *mut hmux2::src::cmd_command_prompt::cmdq_item,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::cmdq_item",
        *mut hmux2::src::cmd_confirm_before::cmdq_item,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::cmdq_item",
        *mut hmux2::src::cmd_copy_mode::cmdq_item,
        []
    );
    record!(
        "src/cmd_detach_client.rs::cmdq_item",
        *mut hmux2::src::cmd_detach_client::cmdq_item,
        []
    );
    record!(
        "src/cmd_display_menu.rs::cmdq_item",
        *mut hmux2::src::cmd_display_menu::cmdq_item,
        []
    );
    record!(
        "src/cmd_display_message.rs::cmdq_item",
        *mut hmux2::src::cmd_display_message::cmdq_item,
        []
    );
    record!(
        "src/cmd_find.rs::cmdq_item",
        *mut hmux2::src::cmd_find::cmdq_item,
        []
    );
    record!(
        "src/cmd_find_window.rs::cmdq_item",
        *mut hmux2::src::cmd_find_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_if_shell.rs::cmdq_item",
        *mut hmux2::src::cmd_if_shell::cmdq_item,
        []
    );
    record!(
        "src/cmd_join_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_join_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_kill_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_kill_server.rs::cmdq_item",
        *mut hmux2::src::cmd_kill_server::cmdq_item,
        []
    );
    record!(
        "src/cmd_kill_session.rs::cmdq_item",
        *mut hmux2::src::cmd_kill_session::cmdq_item,
        []
    );
    record!(
        "src/cmd_kill_window.rs::cmdq_item",
        *mut hmux2::src::cmd_kill_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::cmdq_item",
        *mut hmux2::src::cmd_list_buffers::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_clients.rs::cmdq_item",
        *mut hmux2::src::cmd_list_clients::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_commands.rs::cmdq_item",
        *mut hmux2::src::cmd_list_commands::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_keys.rs::cmdq_item",
        *mut hmux2::src::cmd_list_keys::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_panes.rs::cmdq_item",
        *mut hmux2::src::cmd_list_panes::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::cmdq_item",
        *mut hmux2::src::cmd_list_sessions::cmdq_item,
        []
    );
    record!(
        "src/cmd_list_windows.rs::cmdq_item",
        *mut hmux2::src::cmd_list_windows::cmdq_item,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::cmdq_item",
        *mut hmux2::src::cmd_load_buffer::cmdq_item,
        []
    );
    record!(
        "src/cmd_lock_server.rs::cmdq_item",
        *mut hmux2::src::cmd_lock_server::cmdq_item,
        []
    );
    record!(
        "src/cmd_move_window.rs::cmdq_item",
        *mut hmux2::src::cmd_move_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_new_session.rs::cmdq_item",
        *mut hmux2::src::cmd_new_session::cmdq_item,
        []
    );
    record!(
        "src/cmd_new_window.rs::cmdq_item",
        *mut hmux2::src::cmd_new_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_parse.rs::cmdq_item",
        *mut hmux2::src::cmd_parse::cmdq_item,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::cmdq_item",
        *mut hmux2::src::cmd_paste_buffer::cmdq_item,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_pipe_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_queue.rs::cmdq_item",
        hmux2::src::cmd_queue::cmdq_item,
        [
            name,
            queue,
            next,
            client,
            target_client,
            type_0,
            group,
            number,
            time,
            flags,
            state,
            source,
            target,
            cmdlist,
            cmd,
            cb,
            data,
            entry
        ]
    );
    record!(
        "src/cmd_refresh_client.rs::cmdq_item",
        *mut hmux2::src::cmd_refresh_client::cmdq_item,
        []
    );
    record!(
        "src/cmd_rename_session.rs::cmdq_item",
        *mut hmux2::src::cmd_rename_session::cmdq_item,
        []
    );
    record!(
        "src/cmd_rename_window.rs::cmdq_item",
        *mut hmux2::src::cmd_rename_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_resize_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_resize_window.rs::cmdq_item",
        *mut hmux2::src::cmd_resize_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_respawn_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::cmdq_item",
        *mut hmux2::src::cmd_respawn_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::cmdq_item",
        *mut hmux2::src::cmd_rotate_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_run_shell.rs::cmdq_item",
        *mut hmux2::src::cmd_run_shell::cmdq_item,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::cmdq_item",
        *mut hmux2::src::cmd_save_buffer::cmdq_item,
        []
    );
    record!(
        "src/cmd_select_layout.rs::cmdq_item",
        *mut hmux2::src::cmd_select_layout::cmdq_item,
        []
    );
    record!(
        "src/cmd_select_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_select_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_select_window.rs::cmdq_item",
        *mut hmux2::src::cmd_select_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_send_keys.rs::cmdq_item",
        *mut hmux2::src::cmd_send_keys::cmdq_item,
        []
    );
    record!(
        "src/cmd_server_access.rs::cmdq_item",
        *mut hmux2::src::cmd_server_access::cmdq_item,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::cmdq_item",
        *mut hmux2::src::cmd_set_buffer::cmdq_item,
        []
    );
    record!(
        "src/cmd_set_environment.rs::cmdq_item",
        *mut hmux2::src::cmd_set_environment::cmdq_item,
        []
    );
    record!(
        "src/cmd_set_option.rs::cmdq_item",
        *mut hmux2::src::cmd_set_option::cmdq_item,
        []
    );
    record!(
        "src/cmd_show_environment.rs::cmdq_item",
        *mut hmux2::src::cmd_show_environment::cmdq_item,
        []
    );
    record!(
        "src/cmd_show_messages.rs::cmdq_item",
        *mut hmux2::src::cmd_show_messages::cmdq_item,
        []
    );
    record!(
        "src/cmd_show_options.rs::cmdq_item",
        *mut hmux2::src::cmd_show_options::cmdq_item,
        []
    );
    record!(
        "src/cmd_show_prompt_history.rs::cmdq_item",
        *mut hmux2::src::cmd_show_prompt_history::cmdq_item,
        []
    );
    record!(
        "src/cmd_source_file.rs::cmdq_item",
        *mut hmux2::src::cmd_source_file::cmdq_item,
        []
    );
    record!(
        "src/cmd_split_window.rs::cmdq_item",
        *mut hmux2::src::cmd_split_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::cmdq_item",
        *mut hmux2::src::cmd_swap_pane::cmdq_item,
        []
    );
    record!(
        "src/cmd_swap_window.rs::cmdq_item",
        *mut hmux2::src::cmd_swap_window::cmdq_item,
        []
    );
    record!(
        "src/cmd_switch_client.rs::cmdq_item",
        *mut hmux2::src::cmd_switch_client::cmdq_item,
        []
    );
    record!(
        "src/cmd_unbind_key.rs::cmdq_item",
        *mut hmux2::src::cmd_unbind_key::cmdq_item,
        []
    );
    record!(
        "src/cmd_wait_for.rs::cmdq_item",
        *mut hmux2::src::cmd_wait_for::cmdq_item,
        []
    );
    record!(
        "src/colour.rs::cmdq_item",
        *mut hmux2::src::colour::cmdq_item,
        []
    );
    record!(
        "src/control.rs::cmdq_item",
        *mut hmux2::src::control::cmdq_item,
        []
    );
    record!(
        "src/control_notify.rs::cmdq_item",
        *mut hmux2::src::control_notify::cmdq_item,
        []
    );
    record!(
        "src/environ.rs::cmdq_item",
        *mut hmux2::src::environ::cmdq_item,
        []
    );
    record!(
        "src/events.rs::cmdq_item",
        *mut hmux2::src::events::cmdq_item,
        []
    );
    record!(
        "src/events_payload.rs::cmdq_item",
        *mut hmux2::src::events_payload::cmdq_item,
        []
    );
    record!(
        "src/file.rs::cmdq_item",
        *mut hmux2::src::file::cmdq_item,
        []
    );
    record!(
        "src/format.rs::cmdq_item",
        *mut hmux2::src::format::cmdq_item,
        []
    );
    record!(
        "src/format_draw.rs::cmdq_item",
        *mut hmux2::src::format_draw::cmdq_item,
        []
    );
    record!(
        "src/hooks.rs::cmdq_item",
        *mut hmux2::src::hooks::cmdq_item,
        []
    );
    record!(
        "src/input.rs::cmdq_item",
        *mut hmux2::src::input::cmdq_item,
        []
    );
    record!(
        "src/input_keys.rs::cmdq_item",
        *mut hmux2::src::input_keys::cmdq_item,
        []
    );
    record!("src/job.rs::cmdq_item", *mut hmux2::src::job::cmdq_item, []);
    record!(
        "src/key_bindings.rs::cmdq_item",
        *mut hmux2::src::key_bindings::cmdq_item,
        []
    );
    record!(
        "src/layout.rs::cmdq_item",
        *mut hmux2::src::layout::cmdq_item,
        []
    );
    record!(
        "src/layout_custom.rs::cmdq_item",
        *mut hmux2::src::layout_custom::cmdq_item,
        []
    );
    record!(
        "src/layout_set.rs::cmdq_item",
        *mut hmux2::src::layout_set::cmdq_item,
        []
    );
    record!(
        "src/menu.rs::cmdq_item",
        *mut hmux2::src::menu::cmdq_item,
        []
    );
    record!(
        "src/mode_tree.rs::cmdq_item",
        *mut hmux2::src::mode_tree::cmdq_item,
        []
    );
    record!(
        "src/monitor.rs::cmdq_item",
        *mut hmux2::src::monitor::cmdq_item,
        []
    );
    record!(
        "src/names.rs::cmdq_item",
        *mut hmux2::src::names::cmdq_item,
        []
    );
    record!(
        "src/options.rs::cmdq_item",
        *mut hmux2::src::options::cmdq_item,
        []
    );
    record!(
        "src/popup.rs::cmdq_item",
        *mut hmux2::src::popup::cmdq_item,
        []
    );
    record!(
        "src/prompt.rs::cmdq_item",
        *mut hmux2::src::prompt::cmdq_item,
        []
    );
    record!(
        "src/resize.rs::cmdq_item",
        *mut hmux2::src::resize::cmdq_item,
        []
    );
    record!(
        "src/screen.rs::cmdq_item",
        *mut hmux2::src::screen::cmdq_item,
        []
    );
    record!(
        "src/screen_redraw.rs::cmdq_item",
        *mut hmux2::src::screen_redraw::cmdq_item,
        []
    );
    record!(
        "src/screen_write.rs::cmdq_item",
        *mut hmux2::src::screen_write::cmdq_item,
        []
    );
    record!(
        "src/server.rs::cmdq_item",
        *mut hmux2::src::server::cmdq_item,
        []
    );
    record!(
        "src/server_acl.rs::cmdq_item",
        *mut hmux2::src::server_acl::cmdq_item,
        []
    );
    record!(
        "src/server_client.rs::cmdq_item",
        *mut hmux2::src::server_client::cmdq_item,
        []
    );
    record!(
        "src/server_fn.rs::cmdq_item",
        *mut hmux2::src::server_fn::cmdq_item,
        []
    );
    record!(
        "src/session.rs::cmdq_item",
        *mut hmux2::src::session::cmdq_item,
        []
    );
    record!(
        "src/sort.rs::cmdq_item",
        *mut hmux2::src::sort::cmdq_item,
        []
    );
    record!(
        "src/spawn.rs::cmdq_item",
        *mut hmux2::src::spawn::cmdq_item,
        []
    );
    record!(
        "src/status.rs::cmdq_item",
        *mut hmux2::src::status::cmdq_item,
        []
    );
    record!(
        "src/style.rs::cmdq_item",
        *mut hmux2::src::style::cmdq_item,
        []
    );
    record!("src/tty.rs::cmdq_item", *mut hmux2::src::tty::cmdq_item, []);
    record!(
        "src/tty_acs.rs::cmdq_item",
        *mut hmux2::src::tty_acs::cmdq_item,
        []
    );
    record!(
        "src/tty_draw.rs::cmdq_item",
        *mut hmux2::src::tty_draw::cmdq_item,
        []
    );
    record!(
        "src/tty_features.rs::cmdq_item",
        *mut hmux2::src::tty_features::cmdq_item,
        []
    );
    record!(
        "src/tty_keys.rs::cmdq_item",
        *mut hmux2::src::tty_keys::cmdq_item,
        []
    );
    record!(
        "src/tty_term.rs::cmdq_item",
        *mut hmux2::src::tty_term::cmdq_item,
        []
    );
    record!(
        "src/window.rs::cmdq_item",
        *mut hmux2::src::window::cmdq_item,
        []
    );
    record!(
        "src/window_border.rs::cmdq_item",
        *mut hmux2::src::window_border::cmdq_item,
        []
    );
    record!(
        "src/window_buffer.rs::cmdq_item",
        *mut hmux2::src::window_buffer::cmdq_item,
        []
    );
    record!(
        "src/window_client.rs::cmdq_item",
        *mut hmux2::src::window_client::cmdq_item,
        []
    );
    record!(
        "src/window_clock.rs::cmdq_item",
        *mut hmux2::src::window_clock::cmdq_item,
        []
    );
    record!(
        "src/window_copy.rs::cmdq_item",
        *mut hmux2::src::window_copy::cmdq_item,
        []
    );
    record!(
        "src/window_customize.rs::cmdq_item",
        *mut hmux2::src::window_customize::cmdq_item,
        []
    );
    record!(
        "src/window_panes.rs::cmdq_item",
        *mut hmux2::src::window_panes::cmdq_item,
        []
    );
    record!(
        "src/window_switch.rs::cmdq_item",
        *mut hmux2::src::window_switch::cmdq_item,
        []
    );
    record!(
        "src/window_tree.rs::cmdq_item",
        *mut hmux2::src::window_tree::cmdq_item,
        []
    );
    record!(
        "src/window_visible.rs::cmdq_item",
        *mut hmux2::src::window_visible::cmdq_item,
        []
    );
    record!(
        "src/cmd_queue.rs::C2RustUnnamed_29",
        hmux2::src::cmd_queue::cmdq_item_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_queue.rs::cmdq_item_list",
        hmux2::src::cmd_queue::cmdq_item_list,
        [tqh_first, tqh_last]
    );
    record!(
        "src/alerts.rs::cmdq_list",
        *mut hmux2::src::alerts::cmdq_list,
        []
    );
    record!(
        "src/arguments.rs::cmdq_list",
        *mut hmux2::src::arguments::cmdq_list,
        []
    );
    record!("src/cfg.rs::cmdq_list", *mut hmux2::src::cfg::cmdq_list, []);
    record!(
        "src/client.rs::cmdq_list",
        *mut hmux2::src::client::cmdq_list,
        []
    );
    record!("src/cmd.rs::cmdq_list", *mut hmux2::src::cmd::cmdq_list, []);
    record!(
        "src/cmd_attach_session.rs::cmdq_list",
        *mut hmux2::src::cmd_attach_session::cmdq_list,
        []
    );
    record!(
        "src/cmd_bind_key.rs::cmdq_list",
        *mut hmux2::src::cmd_bind_key::cmdq_list,
        []
    );
    record!(
        "src/cmd_break_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_break_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_capture_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::cmdq_list",
        *mut hmux2::src::cmd_choose_tree::cmdq_list,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::cmdq_list",
        *mut hmux2::src::cmd_command_prompt::cmdq_list,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::cmdq_list",
        *mut hmux2::src::cmd_confirm_before::cmdq_list,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::cmdq_list",
        *mut hmux2::src::cmd_copy_mode::cmdq_list,
        []
    );
    record!(
        "src/cmd_detach_client.rs::cmdq_list",
        *mut hmux2::src::cmd_detach_client::cmdq_list,
        []
    );
    record!(
        "src/cmd_display_menu.rs::cmdq_list",
        *mut hmux2::src::cmd_display_menu::cmdq_list,
        []
    );
    record!(
        "src/cmd_display_message.rs::cmdq_list",
        *mut hmux2::src::cmd_display_message::cmdq_list,
        []
    );
    record!(
        "src/cmd_find.rs::cmdq_list",
        *mut hmux2::src::cmd_find::cmdq_list,
        []
    );
    record!(
        "src/cmd_find_window.rs::cmdq_list",
        *mut hmux2::src::cmd_find_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_if_shell.rs::cmdq_list",
        *mut hmux2::src::cmd_if_shell::cmdq_list,
        []
    );
    record!(
        "src/cmd_join_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_join_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_kill_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_kill_session.rs::cmdq_list",
        *mut hmux2::src::cmd_kill_session::cmdq_list,
        []
    );
    record!(
        "src/cmd_kill_window.rs::cmdq_list",
        *mut hmux2::src::cmd_kill_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::cmdq_list",
        *mut hmux2::src::cmd_list_buffers::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_clients.rs::cmdq_list",
        *mut hmux2::src::cmd_list_clients::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_commands.rs::cmdq_list",
        *mut hmux2::src::cmd_list_commands::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_keys.rs::cmdq_list",
        *mut hmux2::src::cmd_list_keys::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_panes.rs::cmdq_list",
        *mut hmux2::src::cmd_list_panes::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::cmdq_list",
        *mut hmux2::src::cmd_list_sessions::cmdq_list,
        []
    );
    record!(
        "src/cmd_list_windows.rs::cmdq_list",
        *mut hmux2::src::cmd_list_windows::cmdq_list,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::cmdq_list",
        *mut hmux2::src::cmd_load_buffer::cmdq_list,
        []
    );
    record!(
        "src/cmd_lock_server.rs::cmdq_list",
        *mut hmux2::src::cmd_lock_server::cmdq_list,
        []
    );
    record!(
        "src/cmd_move_window.rs::cmdq_list",
        *mut hmux2::src::cmd_move_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_new_session.rs::cmdq_list",
        *mut hmux2::src::cmd_new_session::cmdq_list,
        []
    );
    record!(
        "src/cmd_new_window.rs::cmdq_list",
        *mut hmux2::src::cmd_new_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_parse.rs::cmdq_list",
        *mut hmux2::src::cmd_parse::cmdq_list,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::cmdq_list",
        *mut hmux2::src::cmd_paste_buffer::cmdq_list,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_pipe_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_queue.rs::cmdq_list",
        hmux2::src::cmd_queue::cmdq_list,
        [item, list]
    );
    record!(
        "src/cmd_refresh_client.rs::cmdq_list",
        *mut hmux2::src::cmd_refresh_client::cmdq_list,
        []
    );
    record!(
        "src/cmd_rename_session.rs::cmdq_list",
        *mut hmux2::src::cmd_rename_session::cmdq_list,
        []
    );
    record!(
        "src/cmd_rename_window.rs::cmdq_list",
        *mut hmux2::src::cmd_rename_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_resize_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_resize_window.rs::cmdq_list",
        *mut hmux2::src::cmd_resize_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_respawn_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::cmdq_list",
        *mut hmux2::src::cmd_respawn_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::cmdq_list",
        *mut hmux2::src::cmd_rotate_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_run_shell.rs::cmdq_list",
        *mut hmux2::src::cmd_run_shell::cmdq_list,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::cmdq_list",
        *mut hmux2::src::cmd_save_buffer::cmdq_list,
        []
    );
    record!(
        "src/cmd_select_layout.rs::cmdq_list",
        *mut hmux2::src::cmd_select_layout::cmdq_list,
        []
    );
    record!(
        "src/cmd_select_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_select_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_select_window.rs::cmdq_list",
        *mut hmux2::src::cmd_select_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_send_keys.rs::cmdq_list",
        *mut hmux2::src::cmd_send_keys::cmdq_list,
        []
    );
    record!(
        "src/cmd_server_access.rs::cmdq_list",
        *mut hmux2::src::cmd_server_access::cmdq_list,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::cmdq_list",
        *mut hmux2::src::cmd_set_buffer::cmdq_list,
        []
    );
    record!(
        "src/cmd_set_environment.rs::cmdq_list",
        *mut hmux2::src::cmd_set_environment::cmdq_list,
        []
    );
    record!(
        "src/cmd_set_option.rs::cmdq_list",
        *mut hmux2::src::cmd_set_option::cmdq_list,
        []
    );
    record!(
        "src/cmd_show_environment.rs::cmdq_list",
        *mut hmux2::src::cmd_show_environment::cmdq_list,
        []
    );
    record!(
        "src/cmd_show_messages.rs::cmdq_list",
        *mut hmux2::src::cmd_show_messages::cmdq_list,
        []
    );
    record!(
        "src/cmd_show_options.rs::cmdq_list",
        *mut hmux2::src::cmd_show_options::cmdq_list,
        []
    );
    record!(
        "src/cmd_source_file.rs::cmdq_list",
        *mut hmux2::src::cmd_source_file::cmdq_list,
        []
    );
    record!(
        "src/cmd_split_window.rs::cmdq_list",
        *mut hmux2::src::cmd_split_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::cmdq_list",
        *mut hmux2::src::cmd_swap_pane::cmdq_list,
        []
    );
    record!(
        "src/cmd_swap_window.rs::cmdq_list",
        *mut hmux2::src::cmd_swap_window::cmdq_list,
        []
    );
    record!(
        "src/cmd_switch_client.rs::cmdq_list",
        *mut hmux2::src::cmd_switch_client::cmdq_list,
        []
    );
    record!(
        "src/cmd_wait_for.rs::cmdq_list",
        *mut hmux2::src::cmd_wait_for::cmdq_list,
        []
    );
    record!(
        "src/colour.rs::cmdq_list",
        *mut hmux2::src::colour::cmdq_list,
        []
    );
    record!(
        "src/control.rs::cmdq_list",
        *mut hmux2::src::control::cmdq_list,
        []
    );
    record!(
        "src/control_notify.rs::cmdq_list",
        *mut hmux2::src::control_notify::cmdq_list,
        []
    );
    record!(
        "src/environ.rs::cmdq_list",
        *mut hmux2::src::environ::cmdq_list,
        []
    );
    record!(
        "src/events.rs::cmdq_list",
        *mut hmux2::src::events::cmdq_list,
        []
    );
    record!(
        "src/events_payload.rs::cmdq_list",
        *mut hmux2::src::events_payload::cmdq_list,
        []
    );
    record!(
        "src/file.rs::cmdq_list",
        *mut hmux2::src::file::cmdq_list,
        []
    );
    record!(
        "src/format.rs::cmdq_list",
        *mut hmux2::src::format::cmdq_list,
        []
    );
    record!(
        "src/format_draw.rs::cmdq_list",
        *mut hmux2::src::format_draw::cmdq_list,
        []
    );
    record!(
        "src/hooks.rs::cmdq_list",
        *mut hmux2::src::hooks::cmdq_list,
        []
    );
    record!(
        "src/input.rs::cmdq_list",
        *mut hmux2::src::input::cmdq_list,
        []
    );
    record!(
        "src/input_keys.rs::cmdq_list",
        *mut hmux2::src::input_keys::cmdq_list,
        []
    );
    record!("src/job.rs::cmdq_list", *mut hmux2::src::job::cmdq_list, []);
    record!(
        "src/key_bindings.rs::cmdq_list",
        *mut hmux2::src::key_bindings::cmdq_list,
        []
    );
    record!(
        "src/layout.rs::cmdq_list",
        *mut hmux2::src::layout::cmdq_list,
        []
    );
    record!(
        "src/layout_custom.rs::cmdq_list",
        *mut hmux2::src::layout_custom::cmdq_list,
        []
    );
    record!(
        "src/layout_set.rs::cmdq_list",
        *mut hmux2::src::layout_set::cmdq_list,
        []
    );
    record!(
        "src/menu.rs::cmdq_list",
        *mut hmux2::src::menu::cmdq_list,
        []
    );
    record!(
        "src/mode_tree.rs::cmdq_list",
        *mut hmux2::src::mode_tree::cmdq_list,
        []
    );
    record!(
        "src/monitor.rs::cmdq_list",
        *mut hmux2::src::monitor::cmdq_list,
        []
    );
    record!(
        "src/names.rs::cmdq_list",
        *mut hmux2::src::names::cmdq_list,
        []
    );
    record!(
        "src/options.rs::cmdq_list",
        *mut hmux2::src::options::cmdq_list,
        []
    );
    record!(
        "src/popup.rs::cmdq_list",
        *mut hmux2::src::popup::cmdq_list,
        []
    );
    record!(
        "src/prompt.rs::cmdq_list",
        *mut hmux2::src::prompt::cmdq_list,
        []
    );
    record!(
        "src/resize.rs::cmdq_list",
        *mut hmux2::src::resize::cmdq_list,
        []
    );
    record!(
        "src/screen.rs::cmdq_list",
        *mut hmux2::src::screen::cmdq_list,
        []
    );
    record!(
        "src/screen_redraw.rs::cmdq_list",
        *mut hmux2::src::screen_redraw::cmdq_list,
        []
    );
    record!(
        "src/screen_write.rs::cmdq_list",
        *mut hmux2::src::screen_write::cmdq_list,
        []
    );
    record!(
        "src/server.rs::cmdq_list",
        *mut hmux2::src::server::cmdq_list,
        []
    );
    record!(
        "src/server_acl.rs::cmdq_list",
        *mut hmux2::src::server_acl::cmdq_list,
        []
    );
    record!(
        "src/server_client.rs::cmdq_list",
        *mut hmux2::src::server_client::cmdq_list,
        []
    );
    record!(
        "src/server_fn.rs::cmdq_list",
        *mut hmux2::src::server_fn::cmdq_list,
        []
    );
    record!(
        "src/session.rs::cmdq_list",
        *mut hmux2::src::session::cmdq_list,
        []
    );
    record!(
        "src/sort.rs::cmdq_list",
        *mut hmux2::src::sort::cmdq_list,
        []
    );
    record!(
        "src/spawn.rs::cmdq_list",
        *mut hmux2::src::spawn::cmdq_list,
        []
    );
    record!(
        "src/status.rs::cmdq_list",
        *mut hmux2::src::status::cmdq_list,
        []
    );
    record!(
        "src/style.rs::cmdq_list",
        *mut hmux2::src::style::cmdq_list,
        []
    );
    record!("src/tty.rs::cmdq_list", *mut hmux2::src::tty::cmdq_list, []);
    record!(
        "src/tty_acs.rs::cmdq_list",
        *mut hmux2::src::tty_acs::cmdq_list,
        []
    );
    record!(
        "src/tty_draw.rs::cmdq_list",
        *mut hmux2::src::tty_draw::cmdq_list,
        []
    );
    record!(
        "src/tty_features.rs::cmdq_list",
        *mut hmux2::src::tty_features::cmdq_list,
        []
    );
    record!(
        "src/tty_keys.rs::cmdq_list",
        *mut hmux2::src::tty_keys::cmdq_list,
        []
    );
    record!(
        "src/tty_term.rs::cmdq_list",
        *mut hmux2::src::tty_term::cmdq_list,
        []
    );
    record!(
        "src/window.rs::cmdq_list",
        *mut hmux2::src::window::cmdq_list,
        []
    );
    record!(
        "src/window_border.rs::cmdq_list",
        *mut hmux2::src::window_border::cmdq_list,
        []
    );
    record!(
        "src/window_buffer.rs::cmdq_list",
        *mut hmux2::src::window_buffer::cmdq_list,
        []
    );
    record!(
        "src/window_client.rs::cmdq_list",
        *mut hmux2::src::window_client::cmdq_list,
        []
    );
    record!(
        "src/window_clock.rs::cmdq_list",
        *mut hmux2::src::window_clock::cmdq_list,
        []
    );
    record!(
        "src/window_copy.rs::cmdq_list",
        *mut hmux2::src::window_copy::cmdq_list,
        []
    );
    record!(
        "src/window_customize.rs::cmdq_list",
        *mut hmux2::src::window_customize::cmdq_list,
        []
    );
    record!(
        "src/window_panes.rs::cmdq_list",
        *mut hmux2::src::window_panes::cmdq_list,
        []
    );
    record!(
        "src/window_switch.rs::cmdq_list",
        *mut hmux2::src::window_switch::cmdq_list,
        []
    );
    record!(
        "src/window_tree.rs::cmdq_list",
        *mut hmux2::src::window_tree::cmdq_list,
        []
    );
    record!(
        "src/window_visible.rs::cmdq_list",
        *mut hmux2::src::window_visible::cmdq_list,
        []
    );
    record!(
        "src/cfg.rs::cmdq_state",
        *mut hmux2::src::cfg::cmdq_state,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::cmdq_state",
        *mut hmux2::src::cmd_command_prompt::cmdq_state,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::cmdq_state",
        *mut hmux2::src::cmd_confirm_before::cmdq_state,
        []
    );
    record!(
        "src/cmd_if_shell.rs::cmdq_state",
        *mut hmux2::src::cmd_if_shell::cmdq_state,
        []
    );
    record!(
        "src/cmd_parse.rs::cmdq_state",
        *mut hmux2::src::cmd_parse::cmdq_state,
        []
    );
    record!(
        "src/cmd_queue.rs::cmdq_state",
        hmux2::src::cmd_queue::cmdq_state,
        [references, flags, formats, event, current]
    );
    record!(
        "src/cmd_run_shell.rs::cmdq_state",
        *mut hmux2::src::cmd_run_shell::cmdq_state,
        []
    );
    record!(
        "src/control.rs::cmdq_state",
        *mut hmux2::src::control::cmdq_state,
        []
    );
    record!(
        "src/hooks.rs::cmdq_state",
        *mut hmux2::src::hooks::cmdq_state,
        []
    );
    record!(
        "src/key_bindings.rs::cmdq_state",
        *mut hmux2::src::key_bindings::cmdq_state,
        []
    );
    record!(
        "src/menu.rs::cmdq_state",
        *mut hmux2::src::menu::cmdq_state,
        []
    );
    record!(
        "src/mode_tree.rs::cmdq_state",
        *mut hmux2::src::mode_tree::cmdq_state,
        []
    );
    record!(
        "src/server_client.rs::cmdq_state",
        *mut hmux2::src::server_client::cmdq_state,
        []
    );
    record!(
        "src/window_panes.rs::cmdq_state",
        *mut hmux2::src::window_panes::cmdq_state,
        []
    );
    record!(
        "src/window_switch.rs::cmdq_state",
        *mut hmux2::src::window_switch::cmdq_state,
        []
    );
    record!(
        "src/cmd_queue.rs::cmdq_type",
        hmux2::src::cmd_queue::cmdq_type,
        []
    );
    record!("src/alerts.rs::cmds", *mut hmux2::src::alerts::cmds, []);
    record!(
        "src/arguments.rs::cmds",
        *mut hmux2::src::arguments::cmds,
        []
    );
    record!("src/cfg.rs::cmds", *mut hmux2::src::cfg::cmds, []);
    record!("src/client.rs::cmds", *mut hmux2::src::client::cmds, []);
    record!(
        "src/cmd.rs::cmds",
        hmux2::src::cmd::cmds,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_attach_session.rs::cmds",
        *mut hmux2::src::cmd_attach_session::cmds,
        []
    );
    record!(
        "src/cmd_bind_key.rs::cmds",
        *mut hmux2::src::cmd_bind_key::cmds,
        []
    );
    record!(
        "src/cmd_break_pane.rs::cmds",
        *mut hmux2::src::cmd_break_pane::cmds,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::cmds",
        *mut hmux2::src::cmd_capture_pane::cmds,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::cmds",
        *mut hmux2::src::cmd_choose_tree::cmds,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::cmds",
        *mut hmux2::src::cmd_command_prompt::cmds,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::cmds",
        *mut hmux2::src::cmd_confirm_before::cmds,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::cmds",
        *mut hmux2::src::cmd_copy_mode::cmds,
        []
    );
    record!(
        "src/cmd_detach_client.rs::cmds",
        *mut hmux2::src::cmd_detach_client::cmds,
        []
    );
    record!(
        "src/cmd_display_menu.rs::cmds",
        *mut hmux2::src::cmd_display_menu::cmds,
        []
    );
    record!(
        "src/cmd_display_message.rs::cmds",
        *mut hmux2::src::cmd_display_message::cmds,
        []
    );
    record!("src/cmd_find.rs::cmds", *mut hmux2::src::cmd_find::cmds, []);
    record!(
        "src/cmd_find_window.rs::cmds",
        *mut hmux2::src::cmd_find_window::cmds,
        []
    );
    record!(
        "src/cmd_if_shell.rs::cmds",
        *mut hmux2::src::cmd_if_shell::cmds,
        []
    );
    record!(
        "src/cmd_join_pane.rs::cmds",
        *mut hmux2::src::cmd_join_pane::cmds,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::cmds",
        *mut hmux2::src::cmd_kill_pane::cmds,
        []
    );
    record!(
        "src/cmd_kill_session.rs::cmds",
        *mut hmux2::src::cmd_kill_session::cmds,
        []
    );
    record!(
        "src/cmd_kill_window.rs::cmds",
        *mut hmux2::src::cmd_kill_window::cmds,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::cmds",
        *mut hmux2::src::cmd_list_buffers::cmds,
        []
    );
    record!(
        "src/cmd_list_clients.rs::cmds",
        *mut hmux2::src::cmd_list_clients::cmds,
        []
    );
    record!(
        "src/cmd_list_commands.rs::cmds",
        *mut hmux2::src::cmd_list_commands::cmds,
        []
    );
    record!(
        "src/cmd_list_keys.rs::cmds",
        *mut hmux2::src::cmd_list_keys::cmds,
        []
    );
    record!(
        "src/cmd_list_panes.rs::cmds",
        *mut hmux2::src::cmd_list_panes::cmds,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::cmds",
        *mut hmux2::src::cmd_list_sessions::cmds,
        []
    );
    record!(
        "src/cmd_list_windows.rs::cmds",
        *mut hmux2::src::cmd_list_windows::cmds,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::cmds",
        *mut hmux2::src::cmd_load_buffer::cmds,
        []
    );
    record!(
        "src/cmd_lock_server.rs::cmds",
        *mut hmux2::src::cmd_lock_server::cmds,
        []
    );
    record!(
        "src/cmd_move_window.rs::cmds",
        *mut hmux2::src::cmd_move_window::cmds,
        []
    );
    record!(
        "src/cmd_new_session.rs::cmds",
        *mut hmux2::src::cmd_new_session::cmds,
        []
    );
    record!(
        "src/cmd_new_window.rs::cmds",
        *mut hmux2::src::cmd_new_window::cmds,
        []
    );
    record!(
        "src/cmd_parse.rs::cmds",
        *mut hmux2::src::cmd_parse::cmds,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::cmds",
        *mut hmux2::src::cmd_paste_buffer::cmds,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::cmds",
        *mut hmux2::src::cmd_pipe_pane::cmds,
        []
    );
    record!(
        "src/cmd_queue.rs::cmds",
        *mut hmux2::src::cmd_queue::cmds,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::cmds",
        *mut hmux2::src::cmd_refresh_client::cmds,
        []
    );
    record!(
        "src/cmd_rename_session.rs::cmds",
        *mut hmux2::src::cmd_rename_session::cmds,
        []
    );
    record!(
        "src/cmd_rename_window.rs::cmds",
        *mut hmux2::src::cmd_rename_window::cmds,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::cmds",
        *mut hmux2::src::cmd_resize_pane::cmds,
        []
    );
    record!(
        "src/cmd_resize_window.rs::cmds",
        *mut hmux2::src::cmd_resize_window::cmds,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::cmds",
        *mut hmux2::src::cmd_respawn_pane::cmds,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::cmds",
        *mut hmux2::src::cmd_respawn_window::cmds,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::cmds",
        *mut hmux2::src::cmd_rotate_window::cmds,
        []
    );
    record!(
        "src/cmd_run_shell.rs::cmds",
        *mut hmux2::src::cmd_run_shell::cmds,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::cmds",
        *mut hmux2::src::cmd_save_buffer::cmds,
        []
    );
    record!(
        "src/cmd_select_layout.rs::cmds",
        *mut hmux2::src::cmd_select_layout::cmds,
        []
    );
    record!(
        "src/cmd_select_pane.rs::cmds",
        *mut hmux2::src::cmd_select_pane::cmds,
        []
    );
    record!(
        "src/cmd_select_window.rs::cmds",
        *mut hmux2::src::cmd_select_window::cmds,
        []
    );
    record!(
        "src/cmd_send_keys.rs::cmds",
        *mut hmux2::src::cmd_send_keys::cmds,
        []
    );
    record!(
        "src/cmd_server_access.rs::cmds",
        *mut hmux2::src::cmd_server_access::cmds,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::cmds",
        *mut hmux2::src::cmd_set_buffer::cmds,
        []
    );
    record!(
        "src/cmd_set_environment.rs::cmds",
        *mut hmux2::src::cmd_set_environment::cmds,
        []
    );
    record!(
        "src/cmd_set_option.rs::cmds",
        *mut hmux2::src::cmd_set_option::cmds,
        []
    );
    record!(
        "src/cmd_show_environment.rs::cmds",
        *mut hmux2::src::cmd_show_environment::cmds,
        []
    );
    record!(
        "src/cmd_show_messages.rs::cmds",
        *mut hmux2::src::cmd_show_messages::cmds,
        []
    );
    record!(
        "src/cmd_show_options.rs::cmds",
        *mut hmux2::src::cmd_show_options::cmds,
        []
    );
    record!(
        "src/cmd_source_file.rs::cmds",
        *mut hmux2::src::cmd_source_file::cmds,
        []
    );
    record!(
        "src/cmd_split_window.rs::cmds",
        *mut hmux2::src::cmd_split_window::cmds,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::cmds",
        *mut hmux2::src::cmd_swap_pane::cmds,
        []
    );
    record!(
        "src/cmd_swap_window.rs::cmds",
        *mut hmux2::src::cmd_swap_window::cmds,
        []
    );
    record!(
        "src/cmd_switch_client.rs::cmds",
        *mut hmux2::src::cmd_switch_client::cmds,
        []
    );
    record!(
        "src/cmd_unbind_key.rs::cmds",
        *mut hmux2::src::cmd_unbind_key::cmds,
        []
    );
    record!(
        "src/cmd_wait_for.rs::cmds",
        *mut hmux2::src::cmd_wait_for::cmds,
        []
    );
    record!("src/colour.rs::cmds", *mut hmux2::src::colour::cmds, []);
    record!("src/control.rs::cmds", *mut hmux2::src::control::cmds, []);
    record!(
        "src/control_notify.rs::cmds",
        *mut hmux2::src::control_notify::cmds,
        []
    );
    record!("src/environ.rs::cmds", *mut hmux2::src::environ::cmds, []);
    record!("src/events.rs::cmds", *mut hmux2::src::events::cmds, []);
    record!(
        "src/events_payload.rs::cmds",
        *mut hmux2::src::events_payload::cmds,
        []
    );
    record!("src/file.rs::cmds", *mut hmux2::src::file::cmds, []);
    record!("src/format.rs::cmds", *mut hmux2::src::format::cmds, []);
    record!(
        "src/format_draw.rs::cmds",
        *mut hmux2::src::format_draw::cmds,
        []
    );
    record!("src/hooks.rs::cmds", *mut hmux2::src::hooks::cmds, []);
    record!("src/input.rs::cmds", *mut hmux2::src::input::cmds, []);
    record!(
        "src/input_keys.rs::cmds",
        *mut hmux2::src::input_keys::cmds,
        []
    );
    record!("src/job.rs::cmds", *mut hmux2::src::job::cmds, []);
    record!(
        "src/key_bindings.rs::cmds",
        *mut hmux2::src::key_bindings::cmds,
        []
    );
    record!("src/layout.rs::cmds", *mut hmux2::src::layout::cmds, []);
    record!(
        "src/layout_custom.rs::cmds",
        *mut hmux2::src::layout_custom::cmds,
        []
    );
    record!(
        "src/layout_set.rs::cmds",
        *mut hmux2::src::layout_set::cmds,
        []
    );
    record!("src/menu.rs::cmds", *mut hmux2::src::menu::cmds, []);
    record!(
        "src/mode_tree.rs::cmds",
        *mut hmux2::src::mode_tree::cmds,
        []
    );
    record!("src/monitor.rs::cmds", *mut hmux2::src::monitor::cmds, []);
    record!("src/names.rs::cmds", *mut hmux2::src::names::cmds, []);
    record!("src/options.rs::cmds", *mut hmux2::src::options::cmds, []);
    record!("src/popup.rs::cmds", *mut hmux2::src::popup::cmds, []);
    record!("src/prompt.rs::cmds", *mut hmux2::src::prompt::cmds, []);
    record!("src/resize.rs::cmds", *mut hmux2::src::resize::cmds, []);
    record!("src/screen.rs::cmds", *mut hmux2::src::screen::cmds, []);
    record!(
        "src/screen_redraw.rs::cmds",
        *mut hmux2::src::screen_redraw::cmds,
        []
    );
    record!(
        "src/screen_write.rs::cmds",
        *mut hmux2::src::screen_write::cmds,
        []
    );
    record!("src/server.rs::cmds", *mut hmux2::src::server::cmds, []);
    record!(
        "src/server_acl.rs::cmds",
        *mut hmux2::src::server_acl::cmds,
        []
    );
    record!(
        "src/server_client.rs::cmds",
        *mut hmux2::src::server_client::cmds,
        []
    );
    record!(
        "src/server_fn.rs::cmds",
        *mut hmux2::src::server_fn::cmds,
        []
    );
    record!("src/session.rs::cmds", *mut hmux2::src::session::cmds, []);
    record!("src/sort.rs::cmds", *mut hmux2::src::sort::cmds, []);
    record!("src/spawn.rs::cmds", *mut hmux2::src::spawn::cmds, []);
    record!("src/status.rs::cmds", *mut hmux2::src::status::cmds, []);
    record!("src/style.rs::cmds", *mut hmux2::src::style::cmds, []);
    record!("src/tty.rs::cmds", *mut hmux2::src::tty::cmds, []);
    record!("src/tty_acs.rs::cmds", *mut hmux2::src::tty_acs::cmds, []);
    record!("src/tty_draw.rs::cmds", *mut hmux2::src::tty_draw::cmds, []);
    record!(
        "src/tty_features.rs::cmds",
        *mut hmux2::src::tty_features::cmds,
        []
    );
    record!("src/tty_keys.rs::cmds", *mut hmux2::src::tty_keys::cmds, []);
    record!("src/tty_term.rs::cmds", *mut hmux2::src::tty_term::cmds, []);
    record!("src/utf8.rs::cmds", *mut hmux2::src::utf8::cmds, []);
    record!("src/window.rs::cmds", *mut hmux2::src::window::cmds, []);
    record!(
        "src/window_border.rs::cmds",
        *mut hmux2::src::window_border::cmds,
        []
    );
    record!(
        "src/window_buffer.rs::cmds",
        *mut hmux2::src::window_buffer::cmds,
        []
    );
    record!(
        "src/window_client.rs::cmds",
        *mut hmux2::src::window_client::cmds,
        []
    );
    record!(
        "src/window_clock.rs::cmds",
        *mut hmux2::src::window_clock::cmds,
        []
    );
    record!(
        "src/window_copy.rs::cmds",
        *mut hmux2::src::window_copy::cmds,
        []
    );
    record!(
        "src/window_customize.rs::cmds",
        *mut hmux2::src::window_customize::cmds,
        []
    );
    record!(
        "src/window_panes.rs::cmds",
        *mut hmux2::src::window_panes::cmds,
        []
    );
    record!(
        "src/window_switch.rs::cmds",
        *mut hmux2::src::window_switch::cmds,
        []
    );
    record!(
        "src/window_tree.rs::cmds",
        *mut hmux2::src::window_tree::cmds,
        []
    );
    record!(
        "src/window_visible.rs::cmds",
        *mut hmux2::src::window_visible::cmds,
        []
    );
    record!(
        "src/cmd_wait_for.rs::wait_item",
        hmux2::src::cmd_wait_for::wait_item,
        [item]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-command.txt"));
}
