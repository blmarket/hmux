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
    record!("src/alerts.rs::args", *mut hmux2::src::alerts::args, []);
    record!(
        "src/arguments.rs::args",
        hmux2::src::arguments::args,
        [tree, count, values]
    );
    record!("src/cfg.rs::args", *mut hmux2::src::cfg::args, []);
    record!("src/client.rs::args", *mut hmux2::src::client::args, []);
    record!("src/cmd.rs::args", *mut hmux2::src::cmd::args, []);
    record!(
        "src/cmd_attach_session.rs::args",
        *mut hmux2::src::cmd_attach_session::args,
        []
    );
    record!(
        "src/cmd_bind_key.rs::args",
        *mut hmux2::src::cmd_bind_key::args,
        []
    );
    record!(
        "src/cmd_break_pane.rs::args",
        *mut hmux2::src::cmd_break_pane::args,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::args",
        *mut hmux2::src::cmd_capture_pane::args,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::args",
        *mut hmux2::src::cmd_choose_tree::args,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::args",
        *mut hmux2::src::cmd_command_prompt::args,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::args",
        *mut hmux2::src::cmd_confirm_before::args,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::args",
        *mut hmux2::src::cmd_copy_mode::args,
        []
    );
    record!(
        "src/cmd_detach_client.rs::args",
        *mut hmux2::src::cmd_detach_client::args,
        []
    );
    record!(
        "src/cmd_display_menu.rs::args",
        *mut hmux2::src::cmd_display_menu::args,
        []
    );
    record!(
        "src/cmd_display_message.rs::args",
        *mut hmux2::src::cmd_display_message::args,
        []
    );
    record!("src/cmd_find.rs::args", *mut hmux2::src::cmd_find::args, []);
    record!(
        "src/cmd_find_window.rs::args",
        *mut hmux2::src::cmd_find_window::args,
        []
    );
    record!(
        "src/cmd_if_shell.rs::args",
        *mut hmux2::src::cmd_if_shell::args,
        []
    );
    record!(
        "src/cmd_join_pane.rs::args",
        *mut hmux2::src::cmd_join_pane::args,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::args",
        *mut hmux2::src::cmd_kill_pane::args,
        []
    );
    record!(
        "src/cmd_kill_server.rs::args",
        *mut hmux2::src::cmd_kill_server::args,
        []
    );
    record!(
        "src/cmd_kill_session.rs::args",
        *mut hmux2::src::cmd_kill_session::args,
        []
    );
    record!(
        "src/cmd_kill_window.rs::args",
        *mut hmux2::src::cmd_kill_window::args,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::args",
        *mut hmux2::src::cmd_list_buffers::args,
        []
    );
    record!(
        "src/cmd_list_clients.rs::args",
        *mut hmux2::src::cmd_list_clients::args,
        []
    );
    record!(
        "src/cmd_list_commands.rs::args",
        *mut hmux2::src::cmd_list_commands::args,
        []
    );
    record!(
        "src/cmd_list_keys.rs::args",
        *mut hmux2::src::cmd_list_keys::args,
        []
    );
    record!(
        "src/cmd_list_panes.rs::args",
        *mut hmux2::src::cmd_list_panes::args,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::args",
        *mut hmux2::src::cmd_list_sessions::args,
        []
    );
    record!(
        "src/cmd_list_windows.rs::args",
        *mut hmux2::src::cmd_list_windows::args,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::args",
        *mut hmux2::src::cmd_load_buffer::args,
        []
    );
    record!(
        "src/cmd_lock_server.rs::args",
        *mut hmux2::src::cmd_lock_server::args,
        []
    );
    record!(
        "src/cmd_move_window.rs::args",
        *mut hmux2::src::cmd_move_window::args,
        []
    );
    record!(
        "src/cmd_new_session.rs::args",
        *mut hmux2::src::cmd_new_session::args,
        []
    );
    record!(
        "src/cmd_new_window.rs::args",
        *mut hmux2::src::cmd_new_window::args,
        []
    );
    record!(
        "src/cmd_parse.rs::args",
        *mut hmux2::src::cmd_parse::args,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::args",
        *mut hmux2::src::cmd_paste_buffer::args,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::args",
        *mut hmux2::src::cmd_pipe_pane::args,
        []
    );
    record!(
        "src/cmd_queue.rs::args",
        *mut hmux2::src::cmd_queue::args,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::args",
        *mut hmux2::src::cmd_refresh_client::args,
        []
    );
    record!(
        "src/cmd_rename_session.rs::args",
        *mut hmux2::src::cmd_rename_session::args,
        []
    );
    record!(
        "src/cmd_rename_window.rs::args",
        *mut hmux2::src::cmd_rename_window::args,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::args",
        *mut hmux2::src::cmd_resize_pane::args,
        []
    );
    record!(
        "src/cmd_resize_window.rs::args",
        *mut hmux2::src::cmd_resize_window::args,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::args",
        *mut hmux2::src::cmd_respawn_pane::args,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::args",
        *mut hmux2::src::cmd_respawn_window::args,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::args",
        *mut hmux2::src::cmd_rotate_window::args,
        []
    );
    record!(
        "src/cmd_run_shell.rs::args",
        *mut hmux2::src::cmd_run_shell::args,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::args",
        *mut hmux2::src::cmd_save_buffer::args,
        []
    );
    record!(
        "src/cmd_select_layout.rs::args",
        *mut hmux2::src::cmd_select_layout::args,
        []
    );
    record!(
        "src/cmd_select_pane.rs::args",
        *mut hmux2::src::cmd_select_pane::args,
        []
    );
    record!(
        "src/cmd_select_window.rs::args",
        *mut hmux2::src::cmd_select_window::args,
        []
    );
    record!(
        "src/cmd_send_keys.rs::args",
        *mut hmux2::src::cmd_send_keys::args,
        []
    );
    record!(
        "src/cmd_server_access.rs::args",
        *mut hmux2::src::cmd_server_access::args,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::args",
        *mut hmux2::src::cmd_set_buffer::args,
        []
    );
    record!(
        "src/cmd_set_environment.rs::args",
        *mut hmux2::src::cmd_set_environment::args,
        []
    );
    record!(
        "src/cmd_set_option.rs::args",
        *mut hmux2::src::cmd_set_option::args,
        []
    );
    record!(
        "src/cmd_show_environment.rs::args",
        *mut hmux2::src::cmd_show_environment::args,
        []
    );
    record!(
        "src/cmd_show_messages.rs::args",
        *mut hmux2::src::cmd_show_messages::args,
        []
    );
    record!(
        "src/cmd_show_options.rs::args",
        *mut hmux2::src::cmd_show_options::args,
        []
    );
    record!(
        "src/cmd_show_prompt_history.rs::args",
        *mut hmux2::src::cmd_show_prompt_history::args,
        []
    );
    record!(
        "src/cmd_source_file.rs::args",
        *mut hmux2::src::cmd_source_file::args,
        []
    );
    record!(
        "src/cmd_split_window.rs::args",
        *mut hmux2::src::cmd_split_window::args,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::args",
        *mut hmux2::src::cmd_swap_pane::args,
        []
    );
    record!(
        "src/cmd_swap_window.rs::args",
        *mut hmux2::src::cmd_swap_window::args,
        []
    );
    record!(
        "src/cmd_switch_client.rs::args",
        *mut hmux2::src::cmd_switch_client::args,
        []
    );
    record!(
        "src/cmd_unbind_key.rs::args",
        *mut hmux2::src::cmd_unbind_key::args,
        []
    );
    record!(
        "src/cmd_wait_for.rs::args",
        *mut hmux2::src::cmd_wait_for::args,
        []
    );
    record!("src/colour.rs::args", *mut hmux2::src::colour::args, []);
    record!("src/control.rs::args", *mut hmux2::src::control::args, []);
    record!(
        "src/control_notify.rs::args",
        *mut hmux2::src::control_notify::args,
        []
    );
    record!("src/environ.rs::args", *mut hmux2::src::environ::args, []);
    record!("src/events.rs::args", *mut hmux2::src::events::args, []);
    record!(
        "src/events_payload.rs::args",
        *mut hmux2::src::events_payload::args,
        []
    );
    record!("src/file.rs::args", *mut hmux2::src::file::args, []);
    record!("src/format.rs::args", *mut hmux2::src::format::args, []);
    record!(
        "src/format_draw.rs::args",
        *mut hmux2::src::format_draw::args,
        []
    );
    record!("src/hooks.rs::args", *mut hmux2::src::hooks::args, []);
    record!("src/input.rs::args", *mut hmux2::src::input::args, []);
    record!(
        "src/input_keys.rs::args",
        *mut hmux2::src::input_keys::args,
        []
    );
    record!("src/job.rs::args", *mut hmux2::src::job::args, []);
    record!(
        "src/key_bindings.rs::args",
        *mut hmux2::src::key_bindings::args,
        []
    );
    record!("src/layout.rs::args", *mut hmux2::src::layout::args, []);
    record!(
        "src/layout_custom.rs::args",
        *mut hmux2::src::layout_custom::args,
        []
    );
    record!(
        "src/layout_set.rs::args",
        *mut hmux2::src::layout_set::args,
        []
    );
    record!("src/menu.rs::args", *mut hmux2::src::menu::args, []);
    record!(
        "src/mode_tree.rs::args",
        *mut hmux2::src::mode_tree::args,
        []
    );
    record!("src/monitor.rs::args", *mut hmux2::src::monitor::args, []);
    record!("src/names.rs::args", *mut hmux2::src::names::args, []);
    record!("src/options.rs::args", *mut hmux2::src::options::args, []);
    record!("src/popup.rs::args", *mut hmux2::src::popup::args, []);
    record!("src/prompt.rs::args", *mut hmux2::src::prompt::args, []);
    record!("src/resize.rs::args", *mut hmux2::src::resize::args, []);
    record!("src/screen.rs::args", *mut hmux2::src::screen::args, []);
    record!(
        "src/screen_redraw.rs::args",
        *mut hmux2::src::screen_redraw::args,
        []
    );
    record!(
        "src/screen_write.rs::args",
        *mut hmux2::src::screen_write::args,
        []
    );
    record!("src/server.rs::args", *mut hmux2::src::server::args, []);
    record!(
        "src/server_acl.rs::args",
        *mut hmux2::src::server_acl::args,
        []
    );
    record!(
        "src/server_client.rs::args",
        *mut hmux2::src::server_client::args,
        []
    );
    record!(
        "src/server_fn.rs::args",
        *mut hmux2::src::server_fn::args,
        []
    );
    record!("src/session.rs::args", *mut hmux2::src::session::args, []);
    record!("src/sort.rs::args", *mut hmux2::src::sort::args, []);
    record!("src/spawn.rs::args", *mut hmux2::src::spawn::args, []);
    record!("src/status.rs::args", *mut hmux2::src::status::args, []);
    record!("src/style.rs::args", *mut hmux2::src::style::args, []);
    record!("src/tty.rs::args", *mut hmux2::src::tty::args, []);
    record!("src/tty_acs.rs::args", *mut hmux2::src::tty_acs::args, []);
    record!("src/tty_draw.rs::args", *mut hmux2::src::tty_draw::args, []);
    record!(
        "src/tty_features.rs::args",
        *mut hmux2::src::tty_features::args,
        []
    );
    record!("src/tty_keys.rs::args", *mut hmux2::src::tty_keys::args, []);
    record!("src/tty_term.rs::args", *mut hmux2::src::tty_term::args, []);
    record!("src/window.rs::args", *mut hmux2::src::window::args, []);
    record!(
        "src/window_border.rs::args",
        *mut hmux2::src::window_border::args,
        []
    );
    record!(
        "src/window_buffer.rs::args",
        *mut hmux2::src::window_buffer::args,
        []
    );
    record!(
        "src/window_client.rs::args",
        *mut hmux2::src::window_client::args,
        []
    );
    record!(
        "src/window_clock.rs::args",
        *mut hmux2::src::window_clock::args,
        []
    );
    record!(
        "src/window_copy.rs::args",
        *mut hmux2::src::window_copy::args,
        []
    );
    record!(
        "src/window_customize.rs::args",
        *mut hmux2::src::window_customize::args,
        []
    );
    record!(
        "src/window_panes.rs::args",
        *mut hmux2::src::window_panes::args,
        []
    );
    record!(
        "src/window_switch.rs::args",
        *mut hmux2::src::window_switch::args,
        []
    );
    record!(
        "src/window_tree.rs::args",
        *mut hmux2::src::window_tree::args,
        []
    );
    record!(
        "src/window_visible.rs::args",
        *mut hmux2::src::window_visible::args,
        []
    );
    record!(
        "src/arguments.rs::args_entry",
        hmux2::src::arguments::args_entry,
        [flag, values, count, flags, entry]
    );
    record!(
        "src/cmd_queue.rs::args_entry",
        *mut hmux2::src::cmd_queue::args_entry,
        []
    );
    record!(
        "src/arguments.rs::C2RustUnnamed_13",
        hmux2::src::arguments::args_entry_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/arguments.rs::args_parse",
        hmux2::src::arguments::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd.rs::args_parse",
        hmux2::src::cmd::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_attach_session.rs::args_parse",
        hmux2::src::cmd_attach_session::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_bind_key.rs::args_parse",
        hmux2::src::cmd_bind_key::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_break_pane.rs::args_parse",
        hmux2::src::cmd_break_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_capture_pane.rs::args_parse",
        hmux2::src::cmd_capture_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_choose_tree.rs::args_parse",
        hmux2::src::cmd_choose_tree::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_command_prompt.rs::args_parse",
        hmux2::src::cmd_command_prompt::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_confirm_before.rs::args_parse",
        hmux2::src::cmd_confirm_before::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_copy_mode.rs::args_parse",
        hmux2::src::cmd_copy_mode::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_detach_client.rs::args_parse",
        hmux2::src::cmd_detach_client::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_display_menu.rs::args_parse",
        hmux2::src::cmd_display_menu::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_display_message.rs::args_parse",
        hmux2::src::cmd_display_message::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_find_window.rs::args_parse",
        hmux2::src::cmd_find_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_if_shell.rs::args_parse",
        hmux2::src::cmd_if_shell::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_join_pane.rs::args_parse",
        hmux2::src::cmd_join_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_kill_pane.rs::args_parse",
        hmux2::src::cmd_kill_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_kill_server.rs::args_parse",
        hmux2::src::cmd_kill_server::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_kill_session.rs::args_parse",
        hmux2::src::cmd_kill_session::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_kill_window.rs::args_parse",
        hmux2::src::cmd_kill_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_buffers.rs::args_parse",
        hmux2::src::cmd_list_buffers::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_clients.rs::args_parse",
        hmux2::src::cmd_list_clients::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_commands.rs::args_parse",
        hmux2::src::cmd_list_commands::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_keys.rs::args_parse",
        hmux2::src::cmd_list_keys::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_panes.rs::args_parse",
        hmux2::src::cmd_list_panes::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_sessions.rs::args_parse",
        hmux2::src::cmd_list_sessions::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_list_windows.rs::args_parse",
        hmux2::src::cmd_list_windows::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_load_buffer.rs::args_parse",
        hmux2::src::cmd_load_buffer::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_lock_server.rs::args_parse",
        hmux2::src::cmd_lock_server::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_move_window.rs::args_parse",
        hmux2::src::cmd_move_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_new_session.rs::args_parse",
        hmux2::src::cmd_new_session::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_new_window.rs::args_parse",
        hmux2::src::cmd_new_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_paste_buffer.rs::args_parse",
        hmux2::src::cmd_paste_buffer::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_pipe_pane.rs::args_parse",
        hmux2::src::cmd_pipe_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_queue.rs::args_parse",
        hmux2::src::cmd_queue::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_refresh_client.rs::args_parse",
        hmux2::src::cmd_refresh_client::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_rename_session.rs::args_parse",
        hmux2::src::cmd_rename_session::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_rename_window.rs::args_parse",
        hmux2::src::cmd_rename_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_resize_pane.rs::args_parse",
        hmux2::src::cmd_resize_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_resize_window.rs::args_parse",
        hmux2::src::cmd_resize_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_respawn_pane.rs::args_parse",
        hmux2::src::cmd_respawn_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_respawn_window.rs::args_parse",
        hmux2::src::cmd_respawn_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_rotate_window.rs::args_parse",
        hmux2::src::cmd_rotate_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_run_shell.rs::args_parse",
        hmux2::src::cmd_run_shell::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_save_buffer.rs::args_parse",
        hmux2::src::cmd_save_buffer::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_select_layout.rs::args_parse",
        hmux2::src::cmd_select_layout::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_select_pane.rs::args_parse",
        hmux2::src::cmd_select_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_select_window.rs::args_parse",
        hmux2::src::cmd_select_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_send_keys.rs::args_parse",
        hmux2::src::cmd_send_keys::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_server_access.rs::args_parse",
        hmux2::src::cmd_server_access::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_set_buffer.rs::args_parse",
        hmux2::src::cmd_set_buffer::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_set_environment.rs::args_parse",
        hmux2::src::cmd_set_environment::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_set_option.rs::args_parse",
        hmux2::src::cmd_set_option::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_show_environment.rs::args_parse",
        hmux2::src::cmd_show_environment::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_show_messages.rs::args_parse",
        hmux2::src::cmd_show_messages::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_show_options.rs::args_parse",
        hmux2::src::cmd_show_options::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_show_prompt_history.rs::args_parse",
        hmux2::src::cmd_show_prompt_history::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_source_file.rs::args_parse",
        hmux2::src::cmd_source_file::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_split_window.rs::args_parse",
        hmux2::src::cmd_split_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_swap_pane.rs::args_parse",
        hmux2::src::cmd_swap_pane::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_swap_window.rs::args_parse",
        hmux2::src::cmd_swap_window::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_switch_client.rs::args_parse",
        hmux2::src::cmd_switch_client::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_unbind_key.rs::args_parse",
        hmux2::src::cmd_unbind_key::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/cmd_wait_for.rs::args_parse",
        hmux2::src::cmd_wait_for::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/prompt.rs::args_parse",
        hmux2::src::prompt::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/window_copy.rs::args_parse",
        hmux2::src::window_copy::args_parse,
        [template, lower, upper, cb]
    );
    record!(
        "src/arguments.rs::args_parse_cb",
        hmux2::src::arguments::args_parse_cb,
        []
    );
    record!(
        "src/cmd.rs::args_parse_cb",
        hmux2::src::cmd::args_parse_cb,
        []
    );
    record!(
        "src/cmd_attach_session.rs::args_parse_cb",
        hmux2::src::cmd_attach_session::args_parse_cb,
        []
    );
    record!(
        "src/cmd_bind_key.rs::args_parse_cb",
        hmux2::src::cmd_bind_key::args_parse_cb,
        []
    );
    record!(
        "src/cmd_break_pane.rs::args_parse_cb",
        hmux2::src::cmd_break_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::args_parse_cb",
        hmux2::src::cmd_capture_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::args_parse_cb",
        hmux2::src::cmd_choose_tree::args_parse_cb,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::args_parse_cb",
        hmux2::src::cmd_command_prompt::args_parse_cb,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::args_parse_cb",
        hmux2::src::cmd_confirm_before::args_parse_cb,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::args_parse_cb",
        hmux2::src::cmd_copy_mode::args_parse_cb,
        []
    );
    record!(
        "src/cmd_detach_client.rs::args_parse_cb",
        hmux2::src::cmd_detach_client::args_parse_cb,
        []
    );
    record!(
        "src/cmd_display_menu.rs::args_parse_cb",
        hmux2::src::cmd_display_menu::args_parse_cb,
        []
    );
    record!(
        "src/cmd_display_message.rs::args_parse_cb",
        hmux2::src::cmd_display_message::args_parse_cb,
        []
    );
    record!(
        "src/cmd_find_window.rs::args_parse_cb",
        hmux2::src::cmd_find_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_if_shell.rs::args_parse_cb",
        hmux2::src::cmd_if_shell::args_parse_cb,
        []
    );
    record!(
        "src/cmd_join_pane.rs::args_parse_cb",
        hmux2::src::cmd_join_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::args_parse_cb",
        hmux2::src::cmd_kill_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_kill_server.rs::args_parse_cb",
        hmux2::src::cmd_kill_server::args_parse_cb,
        []
    );
    record!(
        "src/cmd_kill_session.rs::args_parse_cb",
        hmux2::src::cmd_kill_session::args_parse_cb,
        []
    );
    record!(
        "src/cmd_kill_window.rs::args_parse_cb",
        hmux2::src::cmd_kill_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::args_parse_cb",
        hmux2::src::cmd_list_buffers::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_clients.rs::args_parse_cb",
        hmux2::src::cmd_list_clients::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_commands.rs::args_parse_cb",
        hmux2::src::cmd_list_commands::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_keys.rs::args_parse_cb",
        hmux2::src::cmd_list_keys::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_panes.rs::args_parse_cb",
        hmux2::src::cmd_list_panes::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::args_parse_cb",
        hmux2::src::cmd_list_sessions::args_parse_cb,
        []
    );
    record!(
        "src/cmd_list_windows.rs::args_parse_cb",
        hmux2::src::cmd_list_windows::args_parse_cb,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::args_parse_cb",
        hmux2::src::cmd_load_buffer::args_parse_cb,
        []
    );
    record!(
        "src/cmd_lock_server.rs::args_parse_cb",
        hmux2::src::cmd_lock_server::args_parse_cb,
        []
    );
    record!(
        "src/cmd_move_window.rs::args_parse_cb",
        hmux2::src::cmd_move_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_new_session.rs::args_parse_cb",
        hmux2::src::cmd_new_session::args_parse_cb,
        []
    );
    record!(
        "src/cmd_new_window.rs::args_parse_cb",
        hmux2::src::cmd_new_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::args_parse_cb",
        hmux2::src::cmd_paste_buffer::args_parse_cb,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::args_parse_cb",
        hmux2::src::cmd_pipe_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_queue.rs::args_parse_cb",
        hmux2::src::cmd_queue::args_parse_cb,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::args_parse_cb",
        hmux2::src::cmd_refresh_client::args_parse_cb,
        []
    );
    record!(
        "src/cmd_rename_session.rs::args_parse_cb",
        hmux2::src::cmd_rename_session::args_parse_cb,
        []
    );
    record!(
        "src/cmd_rename_window.rs::args_parse_cb",
        hmux2::src::cmd_rename_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::args_parse_cb",
        hmux2::src::cmd_resize_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_resize_window.rs::args_parse_cb",
        hmux2::src::cmd_resize_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::args_parse_cb",
        hmux2::src::cmd_respawn_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::args_parse_cb",
        hmux2::src::cmd_respawn_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::args_parse_cb",
        hmux2::src::cmd_rotate_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_run_shell.rs::args_parse_cb",
        hmux2::src::cmd_run_shell::args_parse_cb,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::args_parse_cb",
        hmux2::src::cmd_save_buffer::args_parse_cb,
        []
    );
    record!(
        "src/cmd_select_layout.rs::args_parse_cb",
        hmux2::src::cmd_select_layout::args_parse_cb,
        []
    );
    record!(
        "src/cmd_select_pane.rs::args_parse_cb",
        hmux2::src::cmd_select_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_select_window.rs::args_parse_cb",
        hmux2::src::cmd_select_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_send_keys.rs::args_parse_cb",
        hmux2::src::cmd_send_keys::args_parse_cb,
        []
    );
    record!(
        "src/cmd_server_access.rs::args_parse_cb",
        hmux2::src::cmd_server_access::args_parse_cb,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::args_parse_cb",
        hmux2::src::cmd_set_buffer::args_parse_cb,
        []
    );
    record!(
        "src/cmd_set_environment.rs::args_parse_cb",
        hmux2::src::cmd_set_environment::args_parse_cb,
        []
    );
    record!(
        "src/cmd_set_option.rs::args_parse_cb",
        hmux2::src::cmd_set_option::args_parse_cb,
        []
    );
    record!(
        "src/cmd_show_environment.rs::args_parse_cb",
        hmux2::src::cmd_show_environment::args_parse_cb,
        []
    );
    record!(
        "src/cmd_show_messages.rs::args_parse_cb",
        hmux2::src::cmd_show_messages::args_parse_cb,
        []
    );
    record!(
        "src/cmd_show_options.rs::args_parse_cb",
        hmux2::src::cmd_show_options::args_parse_cb,
        []
    );
    record!(
        "src/cmd_show_prompt_history.rs::args_parse_cb",
        hmux2::src::cmd_show_prompt_history::args_parse_cb,
        []
    );
    record!(
        "src/cmd_source_file.rs::args_parse_cb",
        hmux2::src::cmd_source_file::args_parse_cb,
        []
    );
    record!(
        "src/cmd_split_window.rs::args_parse_cb",
        hmux2::src::cmd_split_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::args_parse_cb",
        hmux2::src::cmd_swap_pane::args_parse_cb,
        []
    );
    record!(
        "src/cmd_swap_window.rs::args_parse_cb",
        hmux2::src::cmd_swap_window::args_parse_cb,
        []
    );
    record!(
        "src/cmd_switch_client.rs::args_parse_cb",
        hmux2::src::cmd_switch_client::args_parse_cb,
        []
    );
    record!(
        "src/cmd_unbind_key.rs::args_parse_cb",
        hmux2::src::cmd_unbind_key::args_parse_cb,
        []
    );
    record!(
        "src/cmd_wait_for.rs::args_parse_cb",
        hmux2::src::cmd_wait_for::args_parse_cb,
        []
    );
    record!(
        "src/prompt.rs::args_parse_cb",
        hmux2::src::prompt::args_parse_cb,
        []
    );
    record!(
        "src/window_copy.rs::args_parse_cb",
        hmux2::src::window_copy::args_parse_cb,
        []
    );
    record!(
        "src/arguments.rs::args_tree",
        hmux2::src::arguments::args_tree,
        [entries]
    );
    record!(
        "src/arguments.rs::args_value",
        hmux2::src::arguments::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/client.rs::args_value",
        hmux2::src::client::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd.rs::args_value",
        hmux2::src::cmd::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_bind_key.rs::args_value",
        hmux2::src::cmd_bind_key::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_display_menu.rs::args_value",
        hmux2::src::cmd_display_menu::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_find_window.rs::args_value",
        hmux2::src::cmd_find_window::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_new_session.rs::args_value",
        hmux2::src::cmd_new_session::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_new_window.rs::args_value",
        hmux2::src::cmd_new_window::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_parse.rs::args_value",
        hmux2::src::cmd_parse::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_queue.rs::args_value",
        hmux2::src::cmd_queue::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_refresh_client.rs::args_value",
        hmux2::src::cmd_refresh_client::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_respawn_pane.rs::args_value",
        hmux2::src::cmd_respawn_pane::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_respawn_window.rs::args_value",
        hmux2::src::cmd_respawn_window::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/cmd_split_window.rs::args_value",
        hmux2::src::cmd_split_window::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/server_client.rs::args_value",
        hmux2::src::server_client::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/window_copy.rs::args_value",
        hmux2::src::window_copy::args_value,
        [payload, cached, entry]
    );
    record!(
        "src/arguments.rs::C2RustUnnamed_11",
        hmux2::src::arguments::args_value_entry,
        [owner, index]
    );
    record!(
        "src/client.rs::C2RustUnnamed_48",
        hmux2::src::client::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd.rs::C2RustUnnamed_36",
        hmux2::src::cmd::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_bind_key.rs::C2RustUnnamed_36",
        hmux2::src::cmd_bind_key::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_display_menu.rs::C2RustUnnamed_35",
        hmux2::src::cmd_display_menu::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_find_window.rs::C2RustUnnamed_35",
        hmux2::src::cmd_find_window::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_37",
        hmux2::src::cmd_new_session::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_new_window.rs::C2RustUnnamed_35",
        hmux2::src::cmd_new_window::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_parse.rs::C2RustUnnamed_37",
        hmux2::src::cmd_parse::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_queue.rs::C2RustUnnamed_37",
        hmux2::src::cmd_queue::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_refresh_client.rs::C2RustUnnamed_35",
        hmux2::src::cmd_refresh_client::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_respawn_pane.rs::C2RustUnnamed_35",
        hmux2::src::cmd_respawn_pane::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_respawn_window.rs::C2RustUnnamed_35",
        hmux2::src::cmd_respawn_window::args_value_entry,
        [owner, index]
    );
    record!(
        "src/cmd_split_window.rs::C2RustUnnamed_35",
        hmux2::src::cmd_split_window::args_value_entry,
        [owner, index]
    );
    record!(
        "src/server_client.rs::C2RustUnnamed_38",
        hmux2::src::server_client::args_value_entry,
        [owner, index]
    );
    record!(
        "src/window_copy.rs::C2RustUnnamed_38",
        hmux2::src::window_copy::args_value_entry,
        [owner, index]
    );
    record!(
        "src/arguments.rs::args_values",
        hmux2::src::arguments::args_values,
        [first, storage]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-arguments.txt"));
}
