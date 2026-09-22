//! Environment layouts, including the migrated Rust index and owner pointer.
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
        "src/alerts.rs::environ",
        *mut hmux2::src::alerts::environ,
        []
    );
    record!(
        "src/arguments.rs::environ",
        *mut hmux2::src::arguments::environ,
        []
    );
    record!("src/cfg.rs::environ", *mut hmux2::src::cfg::environ, []);
    record!(
        "src/client.rs::environ",
        *mut hmux2::src::client::environ,
        []
    );
    record!("src/cmd.rs::environ", *mut hmux2::src::cmd::environ, []);
    record!(
        "src/cmd_attach_session.rs::environ",
        *mut hmux2::src::cmd_attach_session::environ,
        []
    );
    record!(
        "src/cmd_bind_key.rs::environ",
        *mut hmux2::src::cmd_bind_key::environ,
        []
    );
    record!(
        "src/cmd_break_pane.rs::environ",
        *mut hmux2::src::cmd_break_pane::environ,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::environ",
        *mut hmux2::src::cmd_capture_pane::environ,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::environ",
        *mut hmux2::src::cmd_choose_tree::environ,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::environ",
        *mut hmux2::src::cmd_command_prompt::environ,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::environ",
        *mut hmux2::src::cmd_confirm_before::environ,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::environ",
        *mut hmux2::src::cmd_copy_mode::environ,
        []
    );
    record!(
        "src/cmd_detach_client.rs::environ",
        *mut hmux2::src::cmd_detach_client::environ,
        []
    );
    record!(
        "src/cmd_display_menu.rs::environ",
        *mut hmux2::src::cmd_display_menu::environ,
        []
    );
    record!(
        "src/cmd_display_message.rs::environ",
        *mut hmux2::src::cmd_display_message::environ,
        []
    );
    record!(
        "src/cmd_find.rs::environ",
        *mut hmux2::src::cmd_find::environ,
        []
    );
    record!(
        "src/cmd_find_window.rs::environ",
        *mut hmux2::src::cmd_find_window::environ,
        []
    );
    record!(
        "src/cmd_if_shell.rs::environ",
        *mut hmux2::src::cmd_if_shell::environ,
        []
    );
    record!(
        "src/cmd_join_pane.rs::environ",
        *mut hmux2::src::cmd_join_pane::environ,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::environ",
        *mut hmux2::src::cmd_kill_pane::environ,
        []
    );
    record!(
        "src/cmd_kill_session.rs::environ",
        *mut hmux2::src::cmd_kill_session::environ,
        []
    );
    record!(
        "src/cmd_kill_window.rs::environ",
        *mut hmux2::src::cmd_kill_window::environ,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::environ",
        *mut hmux2::src::cmd_list_buffers::environ,
        []
    );
    record!(
        "src/cmd_list_clients.rs::environ",
        *mut hmux2::src::cmd_list_clients::environ,
        []
    );
    record!(
        "src/cmd_list_commands.rs::environ",
        *mut hmux2::src::cmd_list_commands::environ,
        []
    );
    record!(
        "src/cmd_list_keys.rs::environ",
        *mut hmux2::src::cmd_list_keys::environ,
        []
    );
    record!(
        "src/cmd_list_panes.rs::environ",
        *mut hmux2::src::cmd_list_panes::environ,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::environ",
        *mut hmux2::src::cmd_list_sessions::environ,
        []
    );
    record!(
        "src/cmd_list_windows.rs::environ",
        *mut hmux2::src::cmd_list_windows::environ,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::environ",
        *mut hmux2::src::cmd_load_buffer::environ,
        []
    );
    record!(
        "src/cmd_lock_server.rs::environ",
        *mut hmux2::src::cmd_lock_server::environ,
        []
    );
    record!(
        "src/cmd_move_window.rs::environ",
        *mut hmux2::src::cmd_move_window::environ,
        []
    );
    record!(
        "src/cmd_new_session.rs::environ",
        *mut hmux2::src::cmd_new_session::environ,
        []
    );
    record!(
        "src/cmd_new_window.rs::environ",
        *mut hmux2::src::cmd_new_window::environ,
        []
    );
    record!(
        "src/cmd_parse.rs::environ",
        *mut hmux2::src::cmd_parse::environ,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::environ",
        *mut hmux2::src::cmd_paste_buffer::environ,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::environ",
        *mut hmux2::src::cmd_pipe_pane::environ,
        []
    );
    record!(
        "src/cmd_queue.rs::environ",
        *mut hmux2::src::cmd_queue::environ,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::environ",
        *mut hmux2::src::cmd_refresh_client::environ,
        []
    );
    record!(
        "src/cmd_rename_session.rs::environ",
        *mut hmux2::src::cmd_rename_session::environ,
        []
    );
    record!(
        "src/cmd_rename_window.rs::environ",
        *mut hmux2::src::cmd_rename_window::environ,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::environ",
        *mut hmux2::src::cmd_resize_pane::environ,
        []
    );
    record!(
        "src/cmd_resize_window.rs::environ",
        *mut hmux2::src::cmd_resize_window::environ,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::environ",
        *mut hmux2::src::cmd_respawn_pane::environ,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::environ",
        *mut hmux2::src::cmd_respawn_window::environ,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::environ",
        *mut hmux2::src::cmd_rotate_window::environ,
        []
    );
    record!(
        "src/cmd_run_shell.rs::environ",
        *mut hmux2::src::cmd_run_shell::environ,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::environ",
        *mut hmux2::src::cmd_save_buffer::environ,
        []
    );
    record!(
        "src/cmd_select_layout.rs::environ",
        *mut hmux2::src::cmd_select_layout::environ,
        []
    );
    record!(
        "src/cmd_select_pane.rs::environ",
        *mut hmux2::src::cmd_select_pane::environ,
        []
    );
    record!(
        "src/cmd_select_window.rs::environ",
        *mut hmux2::src::cmd_select_window::environ,
        []
    );
    record!(
        "src/cmd_send_keys.rs::environ",
        *mut hmux2::src::cmd_send_keys::environ,
        []
    );
    record!(
        "src/cmd_server_access.rs::environ",
        *mut hmux2::src::cmd_server_access::environ,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::environ",
        *mut hmux2::src::cmd_set_buffer::environ,
        []
    );
    record!(
        "src/cmd_set_environment.rs::environ",
        *mut hmux2::src::cmd_set_environment::environ,
        []
    );
    record!(
        "src/cmd_set_option.rs::environ",
        *mut hmux2::src::cmd_set_option::environ,
        []
    );
    record!(
        "src/cmd_show_environment.rs::environ",
        *mut hmux2::src::cmd_show_environment::environ,
        []
    );
    record!(
        "src/cmd_show_messages.rs::environ",
        *mut hmux2::src::cmd_show_messages::environ,
        []
    );
    record!(
        "src/cmd_show_options.rs::environ",
        *mut hmux2::src::cmd_show_options::environ,
        []
    );
    record!(
        "src/cmd_source_file.rs::environ",
        *mut hmux2::src::cmd_source_file::environ,
        []
    );
    record!(
        "src/cmd_split_window.rs::environ",
        *mut hmux2::src::cmd_split_window::environ,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::environ",
        *mut hmux2::src::cmd_swap_pane::environ,
        []
    );
    record!(
        "src/cmd_swap_window.rs::environ",
        *mut hmux2::src::cmd_swap_window::environ,
        []
    );
    record!(
        "src/cmd_switch_client.rs::environ",
        *mut hmux2::src::cmd_switch_client::environ,
        []
    );
    record!(
        "src/cmd_wait_for.rs::environ",
        *mut hmux2::src::cmd_wait_for::environ,
        []
    );
    record!(
        "src/colour.rs::environ",
        *mut hmux2::src::colour::environ,
        []
    );
    record!(
        "src/control.rs::environ",
        *mut hmux2::src::control::environ,
        []
    );
    record!(
        "src/control_notify.rs::environ",
        *mut hmux2::src::control_notify::environ,
        []
    );
    record!(
        "src/environ.rs::environ",
        hmux2::src::environ::environ,
        [entries]
    );
    record!(
        "src/events.rs::environ",
        *mut hmux2::src::events::environ,
        []
    );
    record!(
        "src/events_payload.rs::environ",
        *mut hmux2::src::events_payload::environ,
        []
    );
    record!("src/file.rs::environ", *mut hmux2::src::file::environ, []);
    record!(
        "src/format.rs::environ",
        *mut hmux2::src::format::environ,
        []
    );
    record!(
        "src/format_draw.rs::environ",
        *mut hmux2::src::format_draw::environ,
        []
    );
    record!("src/hooks.rs::environ", *mut hmux2::src::hooks::environ, []);
    record!("src/input.rs::environ", *mut hmux2::src::input::environ, []);
    record!(
        "src/input_keys.rs::environ",
        *mut hmux2::src::input_keys::environ,
        []
    );
    record!("src/job.rs::environ", *mut hmux2::src::job::environ, []);
    record!(
        "src/key_bindings.rs::environ",
        *mut hmux2::src::key_bindings::environ,
        []
    );
    record!(
        "src/layout.rs::environ",
        *mut hmux2::src::layout::environ,
        []
    );
    record!(
        "src/layout_custom.rs::environ",
        *mut hmux2::src::layout_custom::environ,
        []
    );
    record!(
        "src/layout_set.rs::environ",
        *mut hmux2::src::layout_set::environ,
        []
    );
    record!("src/menu.rs::environ", *mut hmux2::src::menu::environ, []);
    record!(
        "src/mode_tree.rs::environ",
        *mut hmux2::src::mode_tree::environ,
        []
    );
    record!(
        "src/monitor.rs::environ",
        *mut hmux2::src::monitor::environ,
        []
    );
    record!("src/names.rs::environ", *mut hmux2::src::names::environ, []);
    record!(
        "src/options.rs::environ",
        *mut hmux2::src::options::environ,
        []
    );
    record!("src/popup.rs::environ", *mut hmux2::src::popup::environ, []);
    record!(
        "src/prompt.rs::environ",
        *mut hmux2::src::prompt::environ,
        []
    );
    record!(
        "src/resize.rs::environ",
        *mut hmux2::src::resize::environ,
        []
    );
    record!(
        "src/screen.rs::environ",
        *mut hmux2::src::screen::environ,
        []
    );
    record!(
        "src/screen_redraw.rs::environ",
        *mut hmux2::src::screen_redraw::environ,
        []
    );
    record!(
        "src/screen_write.rs::environ",
        *mut hmux2::src::screen_write::environ,
        []
    );
    record!(
        "src/server.rs::environ",
        *mut hmux2::src::server::environ,
        []
    );
    record!(
        "src/server_acl.rs::environ",
        *mut hmux2::src::server_acl::environ,
        []
    );
    record!(
        "src/server_client.rs::environ",
        *mut hmux2::src::server_client::environ,
        []
    );
    record!(
        "src/server_fn.rs::environ",
        *mut hmux2::src::server_fn::environ,
        []
    );
    record!(
        "src/session.rs::environ",
        *mut hmux2::src::session::environ,
        []
    );
    record!("src/sort.rs::environ", *mut hmux2::src::sort::environ, []);
    record!("src/spawn.rs::environ", *mut hmux2::src::spawn::environ, []);
    record!(
        "src/status.rs::environ",
        *mut hmux2::src::status::environ,
        []
    );
    record!("src/style.rs::environ", *mut hmux2::src::style::environ, []);
    record!("src/tmux.rs::environ", *mut hmux2::src::tmux::environ, []);
    record!("src/tty.rs::environ", *mut hmux2::src::tty::environ, []);
    record!(
        "src/tty_acs.rs::environ",
        *mut hmux2::src::tty_acs::environ,
        []
    );
    record!(
        "src/tty_draw.rs::environ",
        *mut hmux2::src::tty_draw::environ,
        []
    );
    record!(
        "src/tty_features.rs::environ",
        *mut hmux2::src::tty_features::environ,
        []
    );
    record!(
        "src/tty_keys.rs::environ",
        *mut hmux2::src::tty_keys::environ,
        []
    );
    record!(
        "src/tty_term.rs::environ",
        *mut hmux2::src::tty_term::environ,
        []
    );
    record!(
        "src/window.rs::environ",
        *mut hmux2::src::window::environ,
        []
    );
    record!(
        "src/window_border.rs::environ",
        *mut hmux2::src::window_border::environ,
        []
    );
    record!(
        "src/window_buffer.rs::environ",
        *mut hmux2::src::window_buffer::environ,
        []
    );
    record!(
        "src/window_client.rs::environ",
        *mut hmux2::src::window_client::environ,
        []
    );
    record!(
        "src/window_clock.rs::environ",
        *mut hmux2::src::window_clock::environ,
        []
    );
    record!(
        "src/window_copy.rs::environ",
        *mut hmux2::src::window_copy::environ,
        []
    );
    record!(
        "src/window_customize.rs::environ",
        *mut hmux2::src::window_customize::environ,
        []
    );
    record!(
        "src/window_panes.rs::environ",
        *mut hmux2::src::window_panes::environ,
        []
    );
    record!(
        "src/window_switch.rs::environ",
        *mut hmux2::src::window_switch::environ,
        []
    );
    record!(
        "src/window_tree.rs::environ",
        *mut hmux2::src::window_tree::environ,
        []
    );
    record!(
        "src/window_visible.rs::environ",
        *mut hmux2::src::window_visible::environ,
        []
    );
    record!(
        "src/cmd_find.rs::environ_entry",
        hmux2::src::cmd_find::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/cmd_parse.rs::environ_entry",
        hmux2::src::cmd_parse::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/cmd_show_environment.rs::environ_entry",
        hmux2::src::cmd_show_environment::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/environ.rs::environ_entry",
        hmux2::src::environ::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/format.rs::environ_entry",
        hmux2::src::format::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/server_client.rs::environ_entry",
        hmux2::src::server_client::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/spawn.rs::environ_entry",
        hmux2::src::spawn::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/tmux.rs::environ_entry",
        hmux2::src::tmux::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/tty_term.rs::environ_entry",
        hmux2::src::tty_term::environ_entry,
        [name, value, flags, owner]
    );
    record!(
        "src/window_customize.rs::environ_entry",
        hmux2::src::window_customize::environ_entry,
        [name, value, flags, owner]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-environment.txt"));
}
