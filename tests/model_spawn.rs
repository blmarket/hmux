//! Layout snapshots for translated declarations and Rust-owned fields.
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
        "src/alerts.rs::spawn_editor_state",
        *mut hmux2::src::alerts::spawn_editor_state,
        []
    );
    record!(
        "src/arguments.rs::spawn_editor_state",
        *mut hmux2::src::arguments::spawn_editor_state,
        []
    );
    record!(
        "src/cfg.rs::spawn_editor_state",
        *mut hmux2::src::cfg::spawn_editor_state,
        []
    );
    record!(
        "src/client.rs::spawn_editor_state",
        *mut hmux2::src::client::spawn_editor_state,
        []
    );
    record!(
        "src/cmd.rs::spawn_editor_state",
        *mut hmux2::src::cmd::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_attach_session.rs::spawn_editor_state",
        *mut hmux2::src::cmd_attach_session::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_bind_key.rs::spawn_editor_state",
        *mut hmux2::src::cmd_bind_key::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_break_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_break_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_capture_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::spawn_editor_state",
        *mut hmux2::src::cmd_choose_tree::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::spawn_editor_state",
        *mut hmux2::src::cmd_command_prompt::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::spawn_editor_state",
        *mut hmux2::src::cmd_confirm_before::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::spawn_editor_state",
        *mut hmux2::src::cmd_copy_mode::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_detach_client.rs::spawn_editor_state",
        *mut hmux2::src::cmd_detach_client::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_display_menu.rs::spawn_editor_state",
        *mut hmux2::src::cmd_display_menu::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_display_message.rs::spawn_editor_state",
        *mut hmux2::src::cmd_display_message::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_find.rs::spawn_editor_state",
        *mut hmux2::src::cmd_find::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_find_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_find_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_if_shell.rs::spawn_editor_state",
        *mut hmux2::src::cmd_if_shell::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_join_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_join_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_kill_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_kill_session.rs::spawn_editor_state",
        *mut hmux2::src::cmd_kill_session::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_kill_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_kill_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_buffers::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_clients.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_clients::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_commands.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_commands::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_keys.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_keys::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_panes.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_panes::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_sessions::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_list_windows.rs::spawn_editor_state",
        *mut hmux2::src::cmd_list_windows::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::spawn_editor_state",
        *mut hmux2::src::cmd_load_buffer::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_lock_server.rs::spawn_editor_state",
        *mut hmux2::src::cmd_lock_server::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_move_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_move_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_new_session.rs::spawn_editor_state",
        *mut hmux2::src::cmd_new_session::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_new_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_new_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_parse.rs::spawn_editor_state",
        *mut hmux2::src::cmd_parse::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::spawn_editor_state",
        *mut hmux2::src::cmd_paste_buffer::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_pipe_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_queue.rs::spawn_editor_state",
        *mut hmux2::src::cmd_queue::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::spawn_editor_state",
        *mut hmux2::src::cmd_refresh_client::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_rename_session.rs::spawn_editor_state",
        *mut hmux2::src::cmd_rename_session::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_rename_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_rename_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_resize_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_resize_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_resize_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_respawn_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_respawn_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_rotate_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_run_shell.rs::spawn_editor_state",
        *mut hmux2::src::cmd_run_shell::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::spawn_editor_state",
        *mut hmux2::src::cmd_save_buffer::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_select_layout.rs::spawn_editor_state",
        *mut hmux2::src::cmd_select_layout::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_select_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_select_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_select_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_select_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_send_keys.rs::spawn_editor_state",
        *mut hmux2::src::cmd_send_keys::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_server_access.rs::spawn_editor_state",
        *mut hmux2::src::cmd_server_access::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::spawn_editor_state",
        *mut hmux2::src::cmd_set_buffer::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_set_environment.rs::spawn_editor_state",
        *mut hmux2::src::cmd_set_environment::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_set_option.rs::spawn_editor_state",
        *mut hmux2::src::cmd_set_option::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_show_environment.rs::spawn_editor_state",
        *mut hmux2::src::cmd_show_environment::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_show_messages.rs::spawn_editor_state",
        *mut hmux2::src::cmd_show_messages::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_show_options.rs::spawn_editor_state",
        *mut hmux2::src::cmd_show_options::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_source_file.rs::spawn_editor_state",
        *mut hmux2::src::cmd_source_file::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_split_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_split_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::spawn_editor_state",
        *mut hmux2::src::cmd_swap_pane::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_swap_window.rs::spawn_editor_state",
        *mut hmux2::src::cmd_swap_window::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_switch_client.rs::spawn_editor_state",
        *mut hmux2::src::cmd_switch_client::spawn_editor_state,
        []
    );
    record!(
        "src/cmd_wait_for.rs::spawn_editor_state",
        *mut hmux2::src::cmd_wait_for::spawn_editor_state,
        []
    );
    record!(
        "src/colour.rs::spawn_editor_state",
        *mut hmux2::src::colour::spawn_editor_state,
        []
    );
    record!(
        "src/control.rs::spawn_editor_state",
        *mut hmux2::src::control::spawn_editor_state,
        []
    );
    record!(
        "src/control_notify.rs::spawn_editor_state",
        *mut hmux2::src::control_notify::spawn_editor_state,
        []
    );
    record!(
        "src/environ.rs::spawn_editor_state",
        *mut hmux2::src::environ::spawn_editor_state,
        []
    );
    record!(
        "src/events.rs::spawn_editor_state",
        *mut hmux2::src::events::spawn_editor_state,
        []
    );
    record!(
        "src/events_payload.rs::spawn_editor_state",
        *mut hmux2::src::events_payload::spawn_editor_state,
        []
    );
    record!(
        "src/file.rs::spawn_editor_state",
        *mut hmux2::src::file::spawn_editor_state,
        []
    );
    record!(
        "src/format.rs::spawn_editor_state",
        *mut hmux2::src::format::spawn_editor_state,
        []
    );
    record!(
        "src/format_draw.rs::spawn_editor_state",
        *mut hmux2::src::format_draw::spawn_editor_state,
        []
    );
    record!(
        "src/hooks.rs::spawn_editor_state",
        *mut hmux2::src::hooks::spawn_editor_state,
        []
    );
    record!(
        "src/input.rs::spawn_editor_state",
        *mut hmux2::src::input::spawn_editor_state,
        []
    );
    record!(
        "src/input_keys.rs::spawn_editor_state",
        *mut hmux2::src::input_keys::spawn_editor_state,
        []
    );
    record!(
        "src/job.rs::spawn_editor_state",
        *mut hmux2::src::job::spawn_editor_state,
        []
    );
    record!(
        "src/key_bindings.rs::spawn_editor_state",
        *mut hmux2::src::key_bindings::spawn_editor_state,
        []
    );
    record!(
        "src/layout.rs::spawn_editor_state",
        *mut hmux2::src::layout::spawn_editor_state,
        []
    );
    record!(
        "src/layout_custom.rs::spawn_editor_state",
        *mut hmux2::src::layout_custom::spawn_editor_state,
        []
    );
    record!(
        "src/layout_set.rs::spawn_editor_state",
        *mut hmux2::src::layout_set::spawn_editor_state,
        []
    );
    record!(
        "src/menu.rs::spawn_editor_state",
        *mut hmux2::src::menu::spawn_editor_state,
        []
    );
    record!(
        "src/mode_tree.rs::spawn_editor_state",
        *mut hmux2::src::mode_tree::spawn_editor_state,
        []
    );
    record!(
        "src/monitor.rs::spawn_editor_state",
        *mut hmux2::src::monitor::spawn_editor_state,
        []
    );
    record!(
        "src/names.rs::spawn_editor_state",
        *mut hmux2::src::names::spawn_editor_state,
        []
    );
    record!(
        "src/options.rs::spawn_editor_state",
        *mut hmux2::src::options::spawn_editor_state,
        []
    );
    record!(
        "src/popup.rs::spawn_editor_state",
        *mut hmux2::src::popup::spawn_editor_state,
        []
    );
    record!(
        "src/prompt.rs::spawn_editor_state",
        *mut hmux2::src::prompt::spawn_editor_state,
        []
    );
    record!(
        "src/resize.rs::spawn_editor_state",
        *mut hmux2::src::resize::spawn_editor_state,
        []
    );
    record!(
        "src/screen.rs::spawn_editor_state",
        *mut hmux2::src::screen::spawn_editor_state,
        []
    );
    record!(
        "src/screen_redraw.rs::spawn_editor_state",
        *mut hmux2::src::screen_redraw::spawn_editor_state,
        []
    );
    record!(
        "src/screen_write.rs::spawn_editor_state",
        *mut hmux2::src::screen_write::spawn_editor_state,
        []
    );
    record!(
        "src/server.rs::spawn_editor_state",
        *mut hmux2::src::server::spawn_editor_state,
        []
    );
    record!(
        "src/server_acl.rs::spawn_editor_state",
        *mut hmux2::src::server_acl::spawn_editor_state,
        []
    );
    record!(
        "src/server_client.rs::spawn_editor_state",
        *mut hmux2::src::server_client::spawn_editor_state,
        []
    );
    record!(
        "src/server_fn.rs::spawn_editor_state",
        *mut hmux2::src::server_fn::spawn_editor_state,
        []
    );
    record!(
        "src/session.rs::spawn_editor_state",
        *mut hmux2::src::session::spawn_editor_state,
        []
    );
    record!(
        "src/sort.rs::spawn_editor_state",
        *mut hmux2::src::sort::spawn_editor_state,
        []
    );
    record!(
        "src/spawn.rs::spawn_editor_state",
        hmux2::src::spawn::spawn_editor_state,
        [path, pid, cb, arg]
    );
    record!(
        "src/status.rs::spawn_editor_state",
        *mut hmux2::src::status::spawn_editor_state,
        []
    );
    record!(
        "src/style.rs::spawn_editor_state",
        *mut hmux2::src::style::spawn_editor_state,
        []
    );
    record!(
        "src/tty.rs::spawn_editor_state",
        *mut hmux2::src::tty::spawn_editor_state,
        []
    );
    record!(
        "src/tty_acs.rs::spawn_editor_state",
        *mut hmux2::src::tty_acs::spawn_editor_state,
        []
    );
    record!(
        "src/tty_draw.rs::spawn_editor_state",
        *mut hmux2::src::tty_draw::spawn_editor_state,
        []
    );
    record!(
        "src/tty_features.rs::spawn_editor_state",
        *mut hmux2::src::tty_features::spawn_editor_state,
        []
    );
    record!(
        "src/tty_keys.rs::spawn_editor_state",
        *mut hmux2::src::tty_keys::spawn_editor_state,
        []
    );
    record!(
        "src/tty_term.rs::spawn_editor_state",
        *mut hmux2::src::tty_term::spawn_editor_state,
        []
    );
    record!(
        "src/window.rs::spawn_editor_state",
        *mut hmux2::src::window::spawn_editor_state,
        []
    );
    record!(
        "src/window_border.rs::spawn_editor_state",
        *mut hmux2::src::window_border::spawn_editor_state,
        []
    );
    record!(
        "src/window_buffer.rs::spawn_editor_state",
        *mut hmux2::src::window_buffer::spawn_editor_state,
        []
    );
    record!(
        "src/window_client.rs::spawn_editor_state",
        *mut hmux2::src::window_client::spawn_editor_state,
        []
    );
    record!(
        "src/window_clock.rs::spawn_editor_state",
        *mut hmux2::src::window_clock::spawn_editor_state,
        []
    );
    record!(
        "src/window_copy.rs::spawn_editor_state",
        *mut hmux2::src::window_copy::spawn_editor_state,
        []
    );
    record!(
        "src/window_customize.rs::spawn_editor_state",
        *mut hmux2::src::window_customize::spawn_editor_state,
        []
    );
    record!(
        "src/window_panes.rs::spawn_editor_state",
        *mut hmux2::src::window_panes::spawn_editor_state,
        []
    );
    record!(
        "src/window_switch.rs::spawn_editor_state",
        *mut hmux2::src::window_switch::spawn_editor_state,
        []
    );
    record!(
        "src/window_tree.rs::spawn_editor_state",
        *mut hmux2::src::window_tree::spawn_editor_state,
        []
    );
    record!(
        "src/window_visible.rs::spawn_editor_state",
        *mut hmux2::src::window_visible::spawn_editor_state,
        []
    );
    record!(
        "src/spawn.rs::spawn_finish_edit_cb",
        hmux2::src::spawn::spawn_finish_edit_cb,
        []
    );
    record!(
        "src/window_buffer.rs::spawn_finish_edit_cb",
        hmux2::src::window_buffer::spawn_finish_edit_cb,
        []
    );
    record!(
        "src/window_customize.rs::spawn_finish_edit_cb",
        hmux2::src::window_customize::spawn_finish_edit_cb,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-spawn.txt"));
}
