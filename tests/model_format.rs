//! Frozen C layouts, excluding the migrated format-job cache and its nodes.
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
    record!(
        "src/format.rs::format_cb",
        hmux2::src::format::format_cb,
        []
    );
    record!(
        "src/window_copy.rs::format_cb",
        hmux2::src::window_copy::format_cb,
        []
    );
    record!(
        "src/format.rs::format_entry",
        hmux2::src::format::format_entry,
        [key, value, time, cb, entry]
    );
    record!(
        "src/format.rs::C2RustUnnamed_30",
        hmux2::src::format::format_entry_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/format.rs::format_entry_tree",
        hmux2::src::format::format_entry_tree,
        [rbh_root]
    );
    record!(
        "src/alerts.rs::format_job_tree",
        *mut hmux2::src::alerts::format_job_tree,
        []
    );
    record!(
        "src/arguments.rs::format_job_tree",
        *mut hmux2::src::arguments::format_job_tree,
        []
    );
    record!(
        "src/cfg.rs::format_job_tree",
        *mut hmux2::src::cfg::format_job_tree,
        []
    );
    record!(
        "src/client.rs::format_job_tree",
        *mut hmux2::src::client::format_job_tree,
        []
    );
    record!(
        "src/cmd.rs::format_job_tree",
        *mut hmux2::src::cmd::format_job_tree,
        []
    );
    record!(
        "src/cmd_attach_session.rs::format_job_tree",
        *mut hmux2::src::cmd_attach_session::format_job_tree,
        []
    );
    record!(
        "src/cmd_bind_key.rs::format_job_tree",
        *mut hmux2::src::cmd_bind_key::format_job_tree,
        []
    );
    record!(
        "src/cmd_break_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_break_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_capture_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::format_job_tree",
        *mut hmux2::src::cmd_choose_tree::format_job_tree,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::format_job_tree",
        *mut hmux2::src::cmd_command_prompt::format_job_tree,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::format_job_tree",
        *mut hmux2::src::cmd_confirm_before::format_job_tree,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::format_job_tree",
        *mut hmux2::src::cmd_copy_mode::format_job_tree,
        []
    );
    record!(
        "src/cmd_detach_client.rs::format_job_tree",
        *mut hmux2::src::cmd_detach_client::format_job_tree,
        []
    );
    record!(
        "src/cmd_display_menu.rs::format_job_tree",
        *mut hmux2::src::cmd_display_menu::format_job_tree,
        []
    );
    record!(
        "src/cmd_display_message.rs::format_job_tree",
        *mut hmux2::src::cmd_display_message::format_job_tree,
        []
    );
    record!(
        "src/cmd_find.rs::format_job_tree",
        *mut hmux2::src::cmd_find::format_job_tree,
        []
    );
    record!(
        "src/cmd_find_window.rs::format_job_tree",
        *mut hmux2::src::cmd_find_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_if_shell.rs::format_job_tree",
        *mut hmux2::src::cmd_if_shell::format_job_tree,
        []
    );
    record!(
        "src/cmd_join_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_join_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_kill_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_kill_session.rs::format_job_tree",
        *mut hmux2::src::cmd_kill_session::format_job_tree,
        []
    );
    record!(
        "src/cmd_kill_window.rs::format_job_tree",
        *mut hmux2::src::cmd_kill_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::format_job_tree",
        *mut hmux2::src::cmd_list_buffers::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_clients.rs::format_job_tree",
        *mut hmux2::src::cmd_list_clients::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_commands.rs::format_job_tree",
        *mut hmux2::src::cmd_list_commands::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_keys.rs::format_job_tree",
        *mut hmux2::src::cmd_list_keys::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_panes.rs::format_job_tree",
        *mut hmux2::src::cmd_list_panes::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::format_job_tree",
        *mut hmux2::src::cmd_list_sessions::format_job_tree,
        []
    );
    record!(
        "src/cmd_list_windows.rs::format_job_tree",
        *mut hmux2::src::cmd_list_windows::format_job_tree,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::format_job_tree",
        *mut hmux2::src::cmd_load_buffer::format_job_tree,
        []
    );
    record!(
        "src/cmd_lock_server.rs::format_job_tree",
        *mut hmux2::src::cmd_lock_server::format_job_tree,
        []
    );
    record!(
        "src/cmd_move_window.rs::format_job_tree",
        *mut hmux2::src::cmd_move_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_new_session.rs::format_job_tree",
        *mut hmux2::src::cmd_new_session::format_job_tree,
        []
    );
    record!(
        "src/cmd_new_window.rs::format_job_tree",
        *mut hmux2::src::cmd_new_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_parse.rs::format_job_tree",
        *mut hmux2::src::cmd_parse::format_job_tree,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::format_job_tree",
        *mut hmux2::src::cmd_paste_buffer::format_job_tree,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_pipe_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_queue.rs::format_job_tree",
        *mut hmux2::src::cmd_queue::format_job_tree,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::format_job_tree",
        *mut hmux2::src::cmd_refresh_client::format_job_tree,
        []
    );
    record!(
        "src/cmd_rename_session.rs::format_job_tree",
        *mut hmux2::src::cmd_rename_session::format_job_tree,
        []
    );
    record!(
        "src/cmd_rename_window.rs::format_job_tree",
        *mut hmux2::src::cmd_rename_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_resize_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_resize_window.rs::format_job_tree",
        *mut hmux2::src::cmd_resize_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_respawn_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::format_job_tree",
        *mut hmux2::src::cmd_respawn_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::format_job_tree",
        *mut hmux2::src::cmd_rotate_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_run_shell.rs::format_job_tree",
        *mut hmux2::src::cmd_run_shell::format_job_tree,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::format_job_tree",
        *mut hmux2::src::cmd_save_buffer::format_job_tree,
        []
    );
    record!(
        "src/cmd_select_layout.rs::format_job_tree",
        *mut hmux2::src::cmd_select_layout::format_job_tree,
        []
    );
    record!(
        "src/cmd_select_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_select_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_select_window.rs::format_job_tree",
        *mut hmux2::src::cmd_select_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_send_keys.rs::format_job_tree",
        *mut hmux2::src::cmd_send_keys::format_job_tree,
        []
    );
    record!(
        "src/cmd_server_access.rs::format_job_tree",
        *mut hmux2::src::cmd_server_access::format_job_tree,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::format_job_tree",
        *mut hmux2::src::cmd_set_buffer::format_job_tree,
        []
    );
    record!(
        "src/cmd_set_environment.rs::format_job_tree",
        *mut hmux2::src::cmd_set_environment::format_job_tree,
        []
    );
    record!(
        "src/cmd_set_option.rs::format_job_tree",
        *mut hmux2::src::cmd_set_option::format_job_tree,
        []
    );
    record!(
        "src/cmd_show_environment.rs::format_job_tree",
        *mut hmux2::src::cmd_show_environment::format_job_tree,
        []
    );
    record!(
        "src/cmd_show_messages.rs::format_job_tree",
        *mut hmux2::src::cmd_show_messages::format_job_tree,
        []
    );
    record!(
        "src/cmd_show_options.rs::format_job_tree",
        *mut hmux2::src::cmd_show_options::format_job_tree,
        []
    );
    record!(
        "src/cmd_source_file.rs::format_job_tree",
        *mut hmux2::src::cmd_source_file::format_job_tree,
        []
    );
    record!(
        "src/cmd_split_window.rs::format_job_tree",
        *mut hmux2::src::cmd_split_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::format_job_tree",
        *mut hmux2::src::cmd_swap_pane::format_job_tree,
        []
    );
    record!(
        "src/cmd_swap_window.rs::format_job_tree",
        *mut hmux2::src::cmd_swap_window::format_job_tree,
        []
    );
    record!(
        "src/cmd_switch_client.rs::format_job_tree",
        *mut hmux2::src::cmd_switch_client::format_job_tree,
        []
    );
    record!(
        "src/cmd_wait_for.rs::format_job_tree",
        *mut hmux2::src::cmd_wait_for::format_job_tree,
        []
    );
    record!(
        "src/colour.rs::format_job_tree",
        *mut hmux2::src::colour::format_job_tree,
        []
    );
    record!(
        "src/control.rs::format_job_tree",
        *mut hmux2::src::control::format_job_tree,
        []
    );
    record!(
        "src/control_notify.rs::format_job_tree",
        *mut hmux2::src::control_notify::format_job_tree,
        []
    );
    record!(
        "src/environ.rs::format_job_tree",
        *mut hmux2::src::environ::format_job_tree,
        []
    );
    record!(
        "src/events.rs::format_job_tree",
        *mut hmux2::src::events::format_job_tree,
        []
    );
    record!(
        "src/events_payload.rs::format_job_tree",
        *mut hmux2::src::events_payload::format_job_tree,
        []
    );
    record!(
        "src/file.rs::format_job_tree",
        *mut hmux2::src::file::format_job_tree,
        []
    );
    record!(
        "src/format_draw.rs::format_job_tree",
        *mut hmux2::src::format_draw::format_job_tree,
        []
    );
    record!(
        "src/hooks.rs::format_job_tree",
        *mut hmux2::src::hooks::format_job_tree,
        []
    );
    record!(
        "src/input.rs::format_job_tree",
        *mut hmux2::src::input::format_job_tree,
        []
    );
    record!(
        "src/input_keys.rs::format_job_tree",
        *mut hmux2::src::input_keys::format_job_tree,
        []
    );
    record!(
        "src/job.rs::format_job_tree",
        *mut hmux2::src::job::format_job_tree,
        []
    );
    record!(
        "src/key_bindings.rs::format_job_tree",
        *mut hmux2::src::key_bindings::format_job_tree,
        []
    );
    record!(
        "src/layout.rs::format_job_tree",
        *mut hmux2::src::layout::format_job_tree,
        []
    );
    record!(
        "src/layout_custom.rs::format_job_tree",
        *mut hmux2::src::layout_custom::format_job_tree,
        []
    );
    record!(
        "src/layout_set.rs::format_job_tree",
        *mut hmux2::src::layout_set::format_job_tree,
        []
    );
    record!(
        "src/menu.rs::format_job_tree",
        *mut hmux2::src::menu::format_job_tree,
        []
    );
    record!(
        "src/mode_tree.rs::format_job_tree",
        *mut hmux2::src::mode_tree::format_job_tree,
        []
    );
    record!(
        "src/monitor.rs::format_job_tree",
        *mut hmux2::src::monitor::format_job_tree,
        []
    );
    record!(
        "src/names.rs::format_job_tree",
        *mut hmux2::src::names::format_job_tree,
        []
    );
    record!(
        "src/options.rs::format_job_tree",
        *mut hmux2::src::options::format_job_tree,
        []
    );
    record!(
        "src/popup.rs::format_job_tree",
        *mut hmux2::src::popup::format_job_tree,
        []
    );
    record!(
        "src/prompt.rs::format_job_tree",
        *mut hmux2::src::prompt::format_job_tree,
        []
    );
    record!(
        "src/resize.rs::format_job_tree",
        *mut hmux2::src::resize::format_job_tree,
        []
    );
    record!(
        "src/screen.rs::format_job_tree",
        *mut hmux2::src::screen::format_job_tree,
        []
    );
    record!(
        "src/screen_redraw.rs::format_job_tree",
        *mut hmux2::src::screen_redraw::format_job_tree,
        []
    );
    record!(
        "src/screen_write.rs::format_job_tree",
        *mut hmux2::src::screen_write::format_job_tree,
        []
    );
    record!(
        "src/server.rs::format_job_tree",
        *mut hmux2::src::server::format_job_tree,
        []
    );
    record!(
        "src/server_acl.rs::format_job_tree",
        *mut hmux2::src::server_acl::format_job_tree,
        []
    );
    record!(
        "src/server_client.rs::format_job_tree",
        *mut hmux2::src::server_client::format_job_tree,
        []
    );
    record!(
        "src/server_fn.rs::format_job_tree",
        *mut hmux2::src::server_fn::format_job_tree,
        []
    );
    record!(
        "src/session.rs::format_job_tree",
        *mut hmux2::src::session::format_job_tree,
        []
    );
    record!(
        "src/sort.rs::format_job_tree",
        *mut hmux2::src::sort::format_job_tree,
        []
    );
    record!(
        "src/spawn.rs::format_job_tree",
        *mut hmux2::src::spawn::format_job_tree,
        []
    );
    record!(
        "src/status.rs::format_job_tree",
        *mut hmux2::src::status::format_job_tree,
        []
    );
    record!(
        "src/style.rs::format_job_tree",
        *mut hmux2::src::style::format_job_tree,
        []
    );
    record!(
        "src/tty.rs::format_job_tree",
        *mut hmux2::src::tty::format_job_tree,
        []
    );
    record!(
        "src/tty_acs.rs::format_job_tree",
        *mut hmux2::src::tty_acs::format_job_tree,
        []
    );
    record!(
        "src/tty_draw.rs::format_job_tree",
        *mut hmux2::src::tty_draw::format_job_tree,
        []
    );
    record!(
        "src/tty_features.rs::format_job_tree",
        *mut hmux2::src::tty_features::format_job_tree,
        []
    );
    record!(
        "src/tty_keys.rs::format_job_tree",
        *mut hmux2::src::tty_keys::format_job_tree,
        []
    );
    record!(
        "src/tty_term.rs::format_job_tree",
        *mut hmux2::src::tty_term::format_job_tree,
        []
    );
    record!(
        "src/window.rs::format_job_tree",
        *mut hmux2::src::window::format_job_tree,
        []
    );
    record!(
        "src/window_border.rs::format_job_tree",
        *mut hmux2::src::window_border::format_job_tree,
        []
    );
    record!(
        "src/window_buffer.rs::format_job_tree",
        *mut hmux2::src::window_buffer::format_job_tree,
        []
    );
    record!(
        "src/window_client.rs::format_job_tree",
        *mut hmux2::src::window_client::format_job_tree,
        []
    );
    record!(
        "src/window_clock.rs::format_job_tree",
        *mut hmux2::src::window_clock::format_job_tree,
        []
    );
    record!(
        "src/window_copy.rs::format_job_tree",
        *mut hmux2::src::window_copy::format_job_tree,
        []
    );
    record!(
        "src/window_customize.rs::format_job_tree",
        *mut hmux2::src::window_customize::format_job_tree,
        []
    );
    record!(
        "src/window_panes.rs::format_job_tree",
        *mut hmux2::src::window_panes::format_job_tree,
        []
    );
    record!(
        "src/window_switch.rs::format_job_tree",
        *mut hmux2::src::window_switch::format_job_tree,
        []
    );
    record!(
        "src/window_tree.rs::format_job_tree",
        *mut hmux2::src::window_tree::format_job_tree,
        []
    );
    record!(
        "src/window_visible.rs::format_job_tree",
        *mut hmux2::src::window_visible::format_job_tree,
        []
    );
    record!(
        "src/alerts.rs::format_tree",
        *mut hmux2::src::alerts::format_tree,
        []
    );
    record!(
        "src/arguments.rs::format_tree",
        *mut hmux2::src::arguments::format_tree,
        []
    );
    record!(
        "src/cfg.rs::format_tree",
        *mut hmux2::src::cfg::format_tree,
        []
    );
    record!(
        "src/client.rs::format_tree",
        *mut hmux2::src::client::format_tree,
        []
    );
    record!(
        "src/cmd.rs::format_tree",
        *mut hmux2::src::cmd::format_tree,
        []
    );
    record!(
        "src/cmd_attach_session.rs::format_tree",
        *mut hmux2::src::cmd_attach_session::format_tree,
        []
    );
    record!(
        "src/cmd_bind_key.rs::format_tree",
        *mut hmux2::src::cmd_bind_key::format_tree,
        []
    );
    record!(
        "src/cmd_break_pane.rs::format_tree",
        *mut hmux2::src::cmd_break_pane::format_tree,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::format_tree",
        *mut hmux2::src::cmd_capture_pane::format_tree,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::format_tree",
        *mut hmux2::src::cmd_choose_tree::format_tree,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::format_tree",
        *mut hmux2::src::cmd_command_prompt::format_tree,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::format_tree",
        *mut hmux2::src::cmd_confirm_before::format_tree,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::format_tree",
        *mut hmux2::src::cmd_copy_mode::format_tree,
        []
    );
    record!(
        "src/cmd_detach_client.rs::format_tree",
        *mut hmux2::src::cmd_detach_client::format_tree,
        []
    );
    record!(
        "src/cmd_display_menu.rs::format_tree",
        *mut hmux2::src::cmd_display_menu::format_tree,
        []
    );
    record!(
        "src/cmd_display_message.rs::format_tree",
        *mut hmux2::src::cmd_display_message::format_tree,
        []
    );
    record!(
        "src/cmd_find.rs::format_tree",
        *mut hmux2::src::cmd_find::format_tree,
        []
    );
    record!(
        "src/cmd_find_window.rs::format_tree",
        *mut hmux2::src::cmd_find_window::format_tree,
        []
    );
    record!(
        "src/cmd_if_shell.rs::format_tree",
        *mut hmux2::src::cmd_if_shell::format_tree,
        []
    );
    record!(
        "src/cmd_join_pane.rs::format_tree",
        *mut hmux2::src::cmd_join_pane::format_tree,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::format_tree",
        *mut hmux2::src::cmd_kill_pane::format_tree,
        []
    );
    record!(
        "src/cmd_kill_session.rs::format_tree",
        *mut hmux2::src::cmd_kill_session::format_tree,
        []
    );
    record!(
        "src/cmd_kill_window.rs::format_tree",
        *mut hmux2::src::cmd_kill_window::format_tree,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::format_tree",
        *mut hmux2::src::cmd_list_buffers::format_tree,
        []
    );
    record!(
        "src/cmd_list_clients.rs::format_tree",
        *mut hmux2::src::cmd_list_clients::format_tree,
        []
    );
    record!(
        "src/cmd_list_commands.rs::format_tree",
        *mut hmux2::src::cmd_list_commands::format_tree,
        []
    );
    record!(
        "src/cmd_list_keys.rs::format_tree",
        *mut hmux2::src::cmd_list_keys::format_tree,
        []
    );
    record!(
        "src/cmd_list_panes.rs::format_tree",
        *mut hmux2::src::cmd_list_panes::format_tree,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::format_tree",
        *mut hmux2::src::cmd_list_sessions::format_tree,
        []
    );
    record!(
        "src/cmd_list_windows.rs::format_tree",
        *mut hmux2::src::cmd_list_windows::format_tree,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::format_tree",
        *mut hmux2::src::cmd_load_buffer::format_tree,
        []
    );
    record!(
        "src/cmd_lock_server.rs::format_tree",
        *mut hmux2::src::cmd_lock_server::format_tree,
        []
    );
    record!(
        "src/cmd_move_window.rs::format_tree",
        *mut hmux2::src::cmd_move_window::format_tree,
        []
    );
    record!(
        "src/cmd_new_session.rs::format_tree",
        *mut hmux2::src::cmd_new_session::format_tree,
        []
    );
    record!(
        "src/cmd_new_window.rs::format_tree",
        *mut hmux2::src::cmd_new_window::format_tree,
        []
    );
    record!(
        "src/cmd_parse.rs::format_tree",
        *mut hmux2::src::cmd_parse::format_tree,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::format_tree",
        *mut hmux2::src::cmd_paste_buffer::format_tree,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::format_tree",
        *mut hmux2::src::cmd_pipe_pane::format_tree,
        []
    );
    record!(
        "src/cmd_queue.rs::format_tree",
        *mut hmux2::src::cmd_queue::format_tree,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::format_tree",
        *mut hmux2::src::cmd_refresh_client::format_tree,
        []
    );
    record!(
        "src/cmd_rename_session.rs::format_tree",
        *mut hmux2::src::cmd_rename_session::format_tree,
        []
    );
    record!(
        "src/cmd_rename_window.rs::format_tree",
        *mut hmux2::src::cmd_rename_window::format_tree,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::format_tree",
        *mut hmux2::src::cmd_resize_pane::format_tree,
        []
    );
    record!(
        "src/cmd_resize_window.rs::format_tree",
        *mut hmux2::src::cmd_resize_window::format_tree,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::format_tree",
        *mut hmux2::src::cmd_respawn_pane::format_tree,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::format_tree",
        *mut hmux2::src::cmd_respawn_window::format_tree,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::format_tree",
        *mut hmux2::src::cmd_rotate_window::format_tree,
        []
    );
    record!(
        "src/cmd_run_shell.rs::format_tree",
        *mut hmux2::src::cmd_run_shell::format_tree,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::format_tree",
        *mut hmux2::src::cmd_save_buffer::format_tree,
        []
    );
    record!(
        "src/cmd_select_layout.rs::format_tree",
        *mut hmux2::src::cmd_select_layout::format_tree,
        []
    );
    record!(
        "src/cmd_select_pane.rs::format_tree",
        *mut hmux2::src::cmd_select_pane::format_tree,
        []
    );
    record!(
        "src/cmd_select_window.rs::format_tree",
        *mut hmux2::src::cmd_select_window::format_tree,
        []
    );
    record!(
        "src/cmd_send_keys.rs::format_tree",
        *mut hmux2::src::cmd_send_keys::format_tree,
        []
    );
    record!(
        "src/cmd_server_access.rs::format_tree",
        *mut hmux2::src::cmd_server_access::format_tree,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::format_tree",
        *mut hmux2::src::cmd_set_buffer::format_tree,
        []
    );
    record!(
        "src/cmd_set_environment.rs::format_tree",
        *mut hmux2::src::cmd_set_environment::format_tree,
        []
    );
    record!(
        "src/cmd_set_option.rs::format_tree",
        *mut hmux2::src::cmd_set_option::format_tree,
        []
    );
    record!(
        "src/cmd_show_environment.rs::format_tree",
        *mut hmux2::src::cmd_show_environment::format_tree,
        []
    );
    record!(
        "src/cmd_show_messages.rs::format_tree",
        *mut hmux2::src::cmd_show_messages::format_tree,
        []
    );
    record!(
        "src/cmd_show_options.rs::format_tree",
        *mut hmux2::src::cmd_show_options::format_tree,
        []
    );
    record!(
        "src/cmd_source_file.rs::format_tree",
        *mut hmux2::src::cmd_source_file::format_tree,
        []
    );
    record!(
        "src/cmd_split_window.rs::format_tree",
        *mut hmux2::src::cmd_split_window::format_tree,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::format_tree",
        *mut hmux2::src::cmd_swap_pane::format_tree,
        []
    );
    record!(
        "src/cmd_swap_window.rs::format_tree",
        *mut hmux2::src::cmd_swap_window::format_tree,
        []
    );
    record!(
        "src/cmd_switch_client.rs::format_tree",
        *mut hmux2::src::cmd_switch_client::format_tree,
        []
    );
    record!(
        "src/cmd_wait_for.rs::format_tree",
        *mut hmux2::src::cmd_wait_for::format_tree,
        []
    );
    record!(
        "src/colour.rs::format_tree",
        *mut hmux2::src::colour::format_tree,
        []
    );
    record!(
        "src/control.rs::format_tree",
        *mut hmux2::src::control::format_tree,
        []
    );
    record!(
        "src/control_notify.rs::format_tree",
        *mut hmux2::src::control_notify::format_tree,
        []
    );
    record!(
        "src/environ.rs::format_tree",
        *mut hmux2::src::environ::format_tree,
        []
    );
    record!(
        "src/events.rs::format_tree",
        *mut hmux2::src::events::format_tree,
        []
    );
    record!(
        "src/events_payload.rs::format_tree",
        *mut hmux2::src::events_payload::format_tree,
        []
    );
    record!(
        "src/file.rs::format_tree",
        *mut hmux2::src::file::format_tree,
        []
    );
    record!(
        "src/format.rs::format_tree",
        hmux2::src::format::format_tree,
        [type_0, c, s, wl, w, wp, pb, item, client, flags, tag, m, tree]
    );
    record!(
        "src/format_draw.rs::format_tree",
        *mut hmux2::src::format_draw::format_tree,
        []
    );
    record!(
        "src/hooks.rs::format_tree",
        *mut hmux2::src::hooks::format_tree,
        []
    );
    record!(
        "src/input.rs::format_tree",
        *mut hmux2::src::input::format_tree,
        []
    );
    record!(
        "src/input_keys.rs::format_tree",
        *mut hmux2::src::input_keys::format_tree,
        []
    );
    record!(
        "src/job.rs::format_tree",
        *mut hmux2::src::job::format_tree,
        []
    );
    record!(
        "src/key_bindings.rs::format_tree",
        *mut hmux2::src::key_bindings::format_tree,
        []
    );
    record!(
        "src/layout.rs::format_tree",
        *mut hmux2::src::layout::format_tree,
        []
    );
    record!(
        "src/layout_custom.rs::format_tree",
        *mut hmux2::src::layout_custom::format_tree,
        []
    );
    record!(
        "src/layout_set.rs::format_tree",
        *mut hmux2::src::layout_set::format_tree,
        []
    );
    record!(
        "src/menu.rs::format_tree",
        *mut hmux2::src::menu::format_tree,
        []
    );
    record!(
        "src/mode_tree.rs::format_tree",
        *mut hmux2::src::mode_tree::format_tree,
        []
    );
    record!(
        "src/monitor.rs::format_tree",
        *mut hmux2::src::monitor::format_tree,
        []
    );
    record!(
        "src/names.rs::format_tree",
        *mut hmux2::src::names::format_tree,
        []
    );
    record!(
        "src/options.rs::format_tree",
        *mut hmux2::src::options::format_tree,
        []
    );
    record!(
        "src/popup.rs::format_tree",
        *mut hmux2::src::popup::format_tree,
        []
    );
    record!(
        "src/prompt.rs::format_tree",
        *mut hmux2::src::prompt::format_tree,
        []
    );
    record!(
        "src/resize.rs::format_tree",
        *mut hmux2::src::resize::format_tree,
        []
    );
    record!(
        "src/screen.rs::format_tree",
        *mut hmux2::src::screen::format_tree,
        []
    );
    record!(
        "src/screen_redraw.rs::format_tree",
        *mut hmux2::src::screen_redraw::format_tree,
        []
    );
    record!(
        "src/screen_write.rs::format_tree",
        *mut hmux2::src::screen_write::format_tree,
        []
    );
    record!(
        "src/server.rs::format_tree",
        *mut hmux2::src::server::format_tree,
        []
    );
    record!(
        "src/server_acl.rs::format_tree",
        *mut hmux2::src::server_acl::format_tree,
        []
    );
    record!(
        "src/server_client.rs::format_tree",
        *mut hmux2::src::server_client::format_tree,
        []
    );
    record!(
        "src/server_fn.rs::format_tree",
        *mut hmux2::src::server_fn::format_tree,
        []
    );
    record!(
        "src/session.rs::format_tree",
        *mut hmux2::src::session::format_tree,
        []
    );
    record!(
        "src/sort.rs::format_tree",
        *mut hmux2::src::sort::format_tree,
        []
    );
    record!(
        "src/spawn.rs::format_tree",
        *mut hmux2::src::spawn::format_tree,
        []
    );
    record!(
        "src/status.rs::format_tree",
        *mut hmux2::src::status::format_tree,
        []
    );
    record!(
        "src/style.rs::format_tree",
        *mut hmux2::src::style::format_tree,
        []
    );
    record!(
        "src/tty.rs::format_tree",
        *mut hmux2::src::tty::format_tree,
        []
    );
    record!(
        "src/tty_acs.rs::format_tree",
        *mut hmux2::src::tty_acs::format_tree,
        []
    );
    record!(
        "src/tty_draw.rs::format_tree",
        *mut hmux2::src::tty_draw::format_tree,
        []
    );
    record!(
        "src/tty_features.rs::format_tree",
        *mut hmux2::src::tty_features::format_tree,
        []
    );
    record!(
        "src/tty_keys.rs::format_tree",
        *mut hmux2::src::tty_keys::format_tree,
        []
    );
    record!(
        "src/tty_term.rs::format_tree",
        *mut hmux2::src::tty_term::format_tree,
        []
    );
    record!(
        "src/window.rs::format_tree",
        *mut hmux2::src::window::format_tree,
        []
    );
    record!(
        "src/window_border.rs::format_tree",
        *mut hmux2::src::window_border::format_tree,
        []
    );
    record!(
        "src/window_buffer.rs::format_tree",
        *mut hmux2::src::window_buffer::format_tree,
        []
    );
    record!(
        "src/window_client.rs::format_tree",
        *mut hmux2::src::window_client::format_tree,
        []
    );
    record!(
        "src/window_clock.rs::format_tree",
        *mut hmux2::src::window_clock::format_tree,
        []
    );
    record!(
        "src/window_copy.rs::format_tree",
        *mut hmux2::src::window_copy::format_tree,
        []
    );
    record!(
        "src/window_customize.rs::format_tree",
        *mut hmux2::src::window_customize::format_tree,
        []
    );
    record!(
        "src/window_panes.rs::format_tree",
        *mut hmux2::src::window_panes::format_tree,
        []
    );
    record!(
        "src/window_switch.rs::format_tree",
        *mut hmux2::src::window_switch::format_tree,
        []
    );
    record!(
        "src/window_tree.rs::format_tree",
        *mut hmux2::src::window_tree::format_tree,
        []
    );
    record!(
        "src/window_visible.rs::format_tree",
        *mut hmux2::src::window_visible::format_tree,
        []
    );
    record!(
        "src/format.rs::format_type",
        hmux2::src::format::format_type,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-format.txt"));
}
