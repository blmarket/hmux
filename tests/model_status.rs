//! Sizes, alignments, and every named field offset across re-exported copies.
//! Internal owner fixtures include the hmux-rt handle migration (128 -> 56 byte events).
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
        "src/alerts.rs::status_line",
        hmux2::src::alerts::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/arguments.rs::status_line",
        hmux2::src::arguments::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cfg.rs::status_line",
        hmux2::src::cfg::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/client.rs::status_line",
        hmux2::src::client::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd.rs::status_line",
        hmux2::src::cmd::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_attach_session.rs::status_line",
        hmux2::src::cmd_attach_session::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_bind_key.rs::status_line",
        hmux2::src::cmd_bind_key::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_break_pane.rs::status_line",
        hmux2::src::cmd_break_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_capture_pane.rs::status_line",
        hmux2::src::cmd_capture_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_choose_tree.rs::status_line",
        hmux2::src::cmd_choose_tree::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_command_prompt.rs::status_line",
        hmux2::src::cmd_command_prompt::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_confirm_before.rs::status_line",
        hmux2::src::cmd_confirm_before::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_copy_mode.rs::status_line",
        hmux2::src::cmd_copy_mode::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_detach_client.rs::status_line",
        hmux2::src::cmd_detach_client::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_display_menu.rs::status_line",
        hmux2::src::cmd_display_menu::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_display_message.rs::status_line",
        hmux2::src::cmd_display_message::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_find.rs::status_line",
        hmux2::src::cmd_find::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_find_window.rs::status_line",
        hmux2::src::cmd_find_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_if_shell.rs::status_line",
        hmux2::src::cmd_if_shell::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_join_pane.rs::status_line",
        hmux2::src::cmd_join_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_kill_pane.rs::status_line",
        hmux2::src::cmd_kill_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_kill_session.rs::status_line",
        hmux2::src::cmd_kill_session::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_kill_window.rs::status_line",
        hmux2::src::cmd_kill_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_buffers.rs::status_line",
        hmux2::src::cmd_list_buffers::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_clients.rs::status_line",
        hmux2::src::cmd_list_clients::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_commands.rs::status_line",
        hmux2::src::cmd_list_commands::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_keys.rs::status_line",
        hmux2::src::cmd_list_keys::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_panes.rs::status_line",
        hmux2::src::cmd_list_panes::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_sessions.rs::status_line",
        hmux2::src::cmd_list_sessions::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_list_windows.rs::status_line",
        hmux2::src::cmd_list_windows::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_load_buffer.rs::status_line",
        hmux2::src::cmd_load_buffer::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_lock_server.rs::status_line",
        hmux2::src::cmd_lock_server::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_move_window.rs::status_line",
        hmux2::src::cmd_move_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_new_session.rs::status_line",
        hmux2::src::cmd_new_session::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_new_window.rs::status_line",
        hmux2::src::cmd_new_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_parse.rs::status_line",
        hmux2::src::cmd_parse::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_paste_buffer.rs::status_line",
        hmux2::src::cmd_paste_buffer::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_pipe_pane.rs::status_line",
        hmux2::src::cmd_pipe_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_queue.rs::status_line",
        hmux2::src::cmd_queue::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_refresh_client.rs::status_line",
        hmux2::src::cmd_refresh_client::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_rename_session.rs::status_line",
        hmux2::src::cmd_rename_session::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_rename_window.rs::status_line",
        hmux2::src::cmd_rename_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_resize_pane.rs::status_line",
        hmux2::src::cmd_resize_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_resize_window.rs::status_line",
        hmux2::src::cmd_resize_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_respawn_pane.rs::status_line",
        hmux2::src::cmd_respawn_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_respawn_window.rs::status_line",
        hmux2::src::cmd_respawn_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_rotate_window.rs::status_line",
        hmux2::src::cmd_rotate_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_run_shell.rs::status_line",
        hmux2::src::cmd_run_shell::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_save_buffer.rs::status_line",
        hmux2::src::cmd_save_buffer::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_select_layout.rs::status_line",
        hmux2::src::cmd_select_layout::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_select_pane.rs::status_line",
        hmux2::src::cmd_select_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_select_window.rs::status_line",
        hmux2::src::cmd_select_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_send_keys.rs::status_line",
        hmux2::src::cmd_send_keys::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_server_access.rs::status_line",
        hmux2::src::cmd_server_access::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_set_buffer.rs::status_line",
        hmux2::src::cmd_set_buffer::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_set_environment.rs::status_line",
        hmux2::src::cmd_set_environment::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_set_option.rs::status_line",
        hmux2::src::cmd_set_option::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_show_environment.rs::status_line",
        hmux2::src::cmd_show_environment::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_show_messages.rs::status_line",
        hmux2::src::cmd_show_messages::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_show_options.rs::status_line",
        hmux2::src::cmd_show_options::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_source_file.rs::status_line",
        hmux2::src::cmd_source_file::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_split_window.rs::status_line",
        hmux2::src::cmd_split_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_swap_pane.rs::status_line",
        hmux2::src::cmd_swap_pane::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_swap_window.rs::status_line",
        hmux2::src::cmd_swap_window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_switch_client.rs::status_line",
        hmux2::src::cmd_switch_client::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_wait_for.rs::status_line",
        hmux2::src::cmd_wait_for::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/colour.rs::status_line",
        hmux2::src::colour::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/control.rs::status_line",
        hmux2::src::control::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/control_notify.rs::status_line",
        hmux2::src::control_notify::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/environ.rs::status_line",
        hmux2::src::environ::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/events.rs::status_line",
        hmux2::src::events::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/events_payload.rs::status_line",
        hmux2::src::events_payload::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/file.rs::status_line",
        hmux2::src::file::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/format.rs::status_line",
        hmux2::src::format::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/format_draw.rs::status_line",
        hmux2::src::format_draw::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/hooks.rs::status_line",
        hmux2::src::hooks::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/input.rs::status_line",
        hmux2::src::input::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/input_keys.rs::status_line",
        hmux2::src::input_keys::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/job.rs::status_line",
        hmux2::src::job::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/key_bindings.rs::status_line",
        hmux2::src::key_bindings::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/layout.rs::status_line",
        hmux2::src::layout::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/layout_custom.rs::status_line",
        hmux2::src::layout_custom::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/layout_set.rs::status_line",
        hmux2::src::layout_set::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/menu.rs::status_line",
        hmux2::src::menu::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/mode_tree.rs::status_line",
        hmux2::src::mode_tree::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/monitor.rs::status_line",
        hmux2::src::monitor::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/names.rs::status_line",
        hmux2::src::names::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/options.rs::status_line",
        hmux2::src::options::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/popup.rs::status_line",
        hmux2::src::popup::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/prompt.rs::status_line",
        hmux2::src::prompt::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/resize.rs::status_line",
        hmux2::src::resize::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/screen.rs::status_line",
        hmux2::src::screen::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/screen_redraw.rs::status_line",
        hmux2::src::screen_redraw::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/screen_write.rs::status_line",
        hmux2::src::screen_write::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/server.rs::status_line",
        hmux2::src::server::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/server_acl.rs::status_line",
        hmux2::src::server_acl::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/server_client.rs::status_line",
        hmux2::src::server_client::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/server_fn.rs::status_line",
        hmux2::src::server_fn::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/session.rs::status_line",
        hmux2::src::session::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/sort.rs::status_line",
        hmux2::src::sort::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/spawn.rs::status_line",
        hmux2::src::spawn::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/status.rs::status_line",
        hmux2::src::status::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/style.rs::status_line",
        hmux2::src::style::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/tty.rs::status_line",
        hmux2::src::tty::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/tty_acs.rs::status_line",
        hmux2::src::tty_acs::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/tty_draw.rs::status_line",
        hmux2::src::tty_draw::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/tty_features.rs::status_line",
        hmux2::src::tty_features::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/tty_keys.rs::status_line",
        hmux2::src::tty_keys::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/tty_term.rs::status_line",
        hmux2::src::tty_term::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window.rs::status_line",
        hmux2::src::window::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_border.rs::status_line",
        hmux2::src::window_border::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_buffer.rs::status_line",
        hmux2::src::window_buffer::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_client.rs::status_line",
        hmux2::src::window_client::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_clock.rs::status_line",
        hmux2::src::window_clock::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_copy.rs::status_line",
        hmux2::src::window_copy::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_customize.rs::status_line",
        hmux2::src::window_customize::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_panes.rs::status_line",
        hmux2::src::window_panes::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_switch.rs::status_line",
        hmux2::src::window_switch::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_tree.rs::status_line",
        hmux2::src::window_tree::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/window_visible.rs::status_line",
        hmux2::src::window_visible::status_line,
        [timer, screen, active, references, prompt_cx, style, entries]
    );
    record!(
        "src/cmd_command_prompt.rs::status_prompt_input_cb",
        hmux2::src::cmd_command_prompt::status_prompt_input_cb,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::status_prompt_input_cb",
        hmux2::src::cmd_confirm_before::status_prompt_input_cb,
        []
    );
    record!(
        "src/status.rs::status_prompt_input_cb",
        hmux2::src::status::status_prompt_input_cb,
        []
    );
    record!(
        "src/window.rs::status_prompt_input_cb",
        hmux2::src::window::status_prompt_input_cb,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-status.txt"));
}
