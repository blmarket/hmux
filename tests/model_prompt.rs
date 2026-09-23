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
    record!("src/alerts.rs::prompt", *mut hmux2::src::alerts::prompt, []);
    record!(
        "src/arguments.rs::prompt",
        *mut hmux2::src::arguments::prompt,
        []
    );
    record!("src/cfg.rs::prompt", *mut hmux2::src::cfg::prompt, []);
    record!("src/client.rs::prompt", *mut hmux2::src::client::prompt, []);
    record!("src/cmd.rs::prompt", *mut hmux2::src::cmd::prompt, []);
    record!(
        "src/cmd_attach_session.rs::prompt",
        *mut hmux2::src::cmd_attach_session::prompt,
        []
    );
    record!(
        "src/cmd_bind_key.rs::prompt",
        *mut hmux2::src::cmd_bind_key::prompt,
        []
    );
    record!(
        "src/cmd_break_pane.rs::prompt",
        *mut hmux2::src::cmd_break_pane::prompt,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::prompt",
        *mut hmux2::src::cmd_capture_pane::prompt,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::prompt",
        *mut hmux2::src::cmd_choose_tree::prompt,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::prompt",
        *mut hmux2::src::cmd_command_prompt::prompt,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::prompt",
        *mut hmux2::src::cmd_confirm_before::prompt,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::prompt",
        *mut hmux2::src::cmd_copy_mode::prompt,
        []
    );
    record!(
        "src/cmd_detach_client.rs::prompt",
        *mut hmux2::src::cmd_detach_client::prompt,
        []
    );
    record!(
        "src/cmd_display_menu.rs::prompt",
        *mut hmux2::src::cmd_display_menu::prompt,
        []
    );
    record!(
        "src/cmd_display_message.rs::prompt",
        *mut hmux2::src::cmd_display_message::prompt,
        []
    );
    record!(
        "src/cmd_find.rs::prompt",
        *mut hmux2::src::cmd_find::prompt,
        []
    );
    record!(
        "src/cmd_find_window.rs::prompt",
        *mut hmux2::src::cmd_find_window::prompt,
        []
    );
    record!(
        "src/cmd_if_shell.rs::prompt",
        *mut hmux2::src::cmd_if_shell::prompt,
        []
    );
    record!(
        "src/cmd_join_pane.rs::prompt",
        *mut hmux2::src::cmd_join_pane::prompt,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::prompt",
        *mut hmux2::src::cmd_kill_pane::prompt,
        []
    );
    record!(
        "src/cmd_kill_session.rs::prompt",
        *mut hmux2::src::cmd_kill_session::prompt,
        []
    );
    record!(
        "src/cmd_kill_window.rs::prompt",
        *mut hmux2::src::cmd_kill_window::prompt,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::prompt",
        *mut hmux2::src::cmd_list_buffers::prompt,
        []
    );
    record!(
        "src/cmd_list_clients.rs::prompt",
        *mut hmux2::src::cmd_list_clients::prompt,
        []
    );
    record!(
        "src/cmd_list_commands.rs::prompt",
        *mut hmux2::src::cmd_list_commands::prompt,
        []
    );
    record!(
        "src/cmd_list_keys.rs::prompt",
        *mut hmux2::src::cmd_list_keys::prompt,
        []
    );
    record!(
        "src/cmd_list_panes.rs::prompt",
        *mut hmux2::src::cmd_list_panes::prompt,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::prompt",
        *mut hmux2::src::cmd_list_sessions::prompt,
        []
    );
    record!(
        "src/cmd_list_windows.rs::prompt",
        *mut hmux2::src::cmd_list_windows::prompt,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::prompt",
        *mut hmux2::src::cmd_load_buffer::prompt,
        []
    );
    record!(
        "src/cmd_lock_server.rs::prompt",
        *mut hmux2::src::cmd_lock_server::prompt,
        []
    );
    record!(
        "src/cmd_move_window.rs::prompt",
        *mut hmux2::src::cmd_move_window::prompt,
        []
    );
    record!(
        "src/cmd_new_session.rs::prompt",
        *mut hmux2::src::cmd_new_session::prompt,
        []
    );
    record!(
        "src/cmd_new_window.rs::prompt",
        *mut hmux2::src::cmd_new_window::prompt,
        []
    );
    record!(
        "src/cmd_parse.rs::prompt",
        *mut hmux2::src::cmd_parse::prompt,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::prompt",
        *mut hmux2::src::cmd_paste_buffer::prompt,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::prompt",
        *mut hmux2::src::cmd_pipe_pane::prompt,
        []
    );
    record!(
        "src/cmd_queue.rs::prompt",
        *mut hmux2::src::cmd_queue::prompt,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::prompt",
        *mut hmux2::src::cmd_refresh_client::prompt,
        []
    );
    record!(
        "src/cmd_rename_session.rs::prompt",
        *mut hmux2::src::cmd_rename_session::prompt,
        []
    );
    record!(
        "src/cmd_rename_window.rs::prompt",
        *mut hmux2::src::cmd_rename_window::prompt,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::prompt",
        *mut hmux2::src::cmd_resize_pane::prompt,
        []
    );
    record!(
        "src/cmd_resize_window.rs::prompt",
        *mut hmux2::src::cmd_resize_window::prompt,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::prompt",
        *mut hmux2::src::cmd_respawn_pane::prompt,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::prompt",
        *mut hmux2::src::cmd_respawn_window::prompt,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::prompt",
        *mut hmux2::src::cmd_rotate_window::prompt,
        []
    );
    record!(
        "src/cmd_run_shell.rs::prompt",
        *mut hmux2::src::cmd_run_shell::prompt,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::prompt",
        *mut hmux2::src::cmd_save_buffer::prompt,
        []
    );
    record!(
        "src/cmd_select_layout.rs::prompt",
        *mut hmux2::src::cmd_select_layout::prompt,
        []
    );
    record!(
        "src/cmd_select_pane.rs::prompt",
        *mut hmux2::src::cmd_select_pane::prompt,
        []
    );
    record!(
        "src/cmd_select_window.rs::prompt",
        *mut hmux2::src::cmd_select_window::prompt,
        []
    );
    record!(
        "src/cmd_send_keys.rs::prompt",
        *mut hmux2::src::cmd_send_keys::prompt,
        []
    );
    record!(
        "src/cmd_server_access.rs::prompt",
        *mut hmux2::src::cmd_server_access::prompt,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::prompt",
        *mut hmux2::src::cmd_set_buffer::prompt,
        []
    );
    record!(
        "src/cmd_set_environment.rs::prompt",
        *mut hmux2::src::cmd_set_environment::prompt,
        []
    );
    record!(
        "src/cmd_set_option.rs::prompt",
        *mut hmux2::src::cmd_set_option::prompt,
        []
    );
    record!(
        "src/cmd_show_environment.rs::prompt",
        *mut hmux2::src::cmd_show_environment::prompt,
        []
    );
    record!(
        "src/cmd_show_messages.rs::prompt",
        *mut hmux2::src::cmd_show_messages::prompt,
        []
    );
    record!(
        "src/cmd_show_options.rs::prompt",
        *mut hmux2::src::cmd_show_options::prompt,
        []
    );
    record!(
        "src/cmd_source_file.rs::prompt",
        *mut hmux2::src::cmd_source_file::prompt,
        []
    );
    record!(
        "src/cmd_split_window.rs::prompt",
        *mut hmux2::src::cmd_split_window::prompt,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::prompt",
        *mut hmux2::src::cmd_swap_pane::prompt,
        []
    );
    record!(
        "src/cmd_swap_window.rs::prompt",
        *mut hmux2::src::cmd_swap_window::prompt,
        []
    );
    record!(
        "src/cmd_switch_client.rs::prompt",
        *mut hmux2::src::cmd_switch_client::prompt,
        []
    );
    record!(
        "src/cmd_wait_for.rs::prompt",
        *mut hmux2::src::cmd_wait_for::prompt,
        []
    );
    record!("src/colour.rs::prompt", *mut hmux2::src::colour::prompt, []);
    record!(
        "src/control.rs::prompt",
        *mut hmux2::src::control::prompt,
        []
    );
    record!(
        "src/control_notify.rs::prompt",
        *mut hmux2::src::control_notify::prompt,
        []
    );
    record!(
        "src/environ.rs::prompt",
        *mut hmux2::src::environ::prompt,
        []
    );
    record!("src/events.rs::prompt", *mut hmux2::src::events::prompt, []);
    record!(
        "src/events_payload.rs::prompt",
        *mut hmux2::src::events_payload::prompt,
        []
    );
    record!("src/file.rs::prompt", *mut hmux2::src::file::prompt, []);
    record!("src/format.rs::prompt", *mut hmux2::src::format::prompt, []);
    record!(
        "src/format_draw.rs::prompt",
        *mut hmux2::src::format_draw::prompt,
        []
    );
    record!("src/hooks.rs::prompt", *mut hmux2::src::hooks::prompt, []);
    record!("src/input.rs::prompt", *mut hmux2::src::input::prompt, []);
    record!(
        "src/input_keys.rs::prompt",
        *mut hmux2::src::input_keys::prompt,
        []
    );
    record!("src/job.rs::prompt", *mut hmux2::src::job::prompt, []);
    record!(
        "src/key_bindings.rs::prompt",
        *mut hmux2::src::key_bindings::prompt,
        []
    );
    record!("src/layout.rs::prompt", *mut hmux2::src::layout::prompt, []);
    record!(
        "src/layout_custom.rs::prompt",
        *mut hmux2::src::layout_custom::prompt,
        []
    );
    record!(
        "src/layout_set.rs::prompt",
        *mut hmux2::src::layout_set::prompt,
        []
    );
    record!("src/menu.rs::prompt", *mut hmux2::src::menu::prompt, []);
    record!(
        "src/mode_tree.rs::prompt",
        *mut hmux2::src::mode_tree::prompt,
        []
    );
    record!(
        "src/monitor.rs::prompt",
        *mut hmux2::src::monitor::prompt,
        []
    );
    record!("src/names.rs::prompt", *mut hmux2::src::names::prompt, []);
    record!(
        "src/options.rs::prompt",
        *mut hmux2::src::options::prompt,
        []
    );
    record!("src/popup.rs::prompt", *mut hmux2::src::popup::prompt, []);
    record!(
        "src/prompt.rs::prompt",
        hmux2::src::prompt::prompt,
        [
            string,
            buffer,
            state,
            last,
            index,
            inputcb,
            freecb,
            data,
            message_format,
            keys,
            word_separators,
            style,
            command_style,
            style_str,
            command_style_str,
            cstyle,
            command_cstyle,
            ccolour,
            command_ccolour,
            cmode,
            command_cmode,
            type_0,
            flags,
            closed,
            hindex,
            copied,
            completion
        ]
    );
    record!("src/resize.rs::prompt", *mut hmux2::src::resize::prompt, []);
    record!("src/screen.rs::prompt", *mut hmux2::src::screen::prompt, []);
    record!(
        "src/screen_redraw.rs::prompt",
        *mut hmux2::src::screen_redraw::prompt,
        []
    );
    record!(
        "src/screen_write.rs::prompt",
        *mut hmux2::src::screen_write::prompt,
        []
    );
    record!("src/server.rs::prompt", *mut hmux2::src::server::prompt, []);
    record!(
        "src/server_acl.rs::prompt",
        *mut hmux2::src::server_acl::prompt,
        []
    );
    record!(
        "src/server_client.rs::prompt",
        *mut hmux2::src::server_client::prompt,
        []
    );
    record!(
        "src/server_fn.rs::prompt",
        *mut hmux2::src::server_fn::prompt,
        []
    );
    record!(
        "src/session.rs::prompt",
        *mut hmux2::src::session::prompt,
        []
    );
    record!("src/sort.rs::prompt", *mut hmux2::src::sort::prompt, []);
    record!("src/spawn.rs::prompt", *mut hmux2::src::spawn::prompt, []);
    record!("src/status.rs::prompt", *mut hmux2::src::status::prompt, []);
    record!("src/style.rs::prompt", *mut hmux2::src::style::prompt, []);
    record!("src/tty.rs::prompt", *mut hmux2::src::tty::prompt, []);
    record!(
        "src/tty_acs.rs::prompt",
        *mut hmux2::src::tty_acs::prompt,
        []
    );
    record!(
        "src/tty_draw.rs::prompt",
        *mut hmux2::src::tty_draw::prompt,
        []
    );
    record!(
        "src/tty_features.rs::prompt",
        *mut hmux2::src::tty_features::prompt,
        []
    );
    record!(
        "src/tty_keys.rs::prompt",
        *mut hmux2::src::tty_keys::prompt,
        []
    );
    record!(
        "src/tty_term.rs::prompt",
        *mut hmux2::src::tty_term::prompt,
        []
    );
    record!("src/window.rs::prompt", *mut hmux2::src::window::prompt, []);
    record!(
        "src/window_border.rs::prompt",
        *mut hmux2::src::window_border::prompt,
        []
    );
    record!(
        "src/window_buffer.rs::prompt",
        *mut hmux2::src::window_buffer::prompt,
        []
    );
    record!(
        "src/window_client.rs::prompt",
        *mut hmux2::src::window_client::prompt,
        []
    );
    record!(
        "src/window_clock.rs::prompt",
        *mut hmux2::src::window_clock::prompt,
        []
    );
    record!(
        "src/window_copy.rs::prompt",
        *mut hmux2::src::window_copy::prompt,
        []
    );
    record!(
        "src/window_customize.rs::prompt",
        *mut hmux2::src::window_customize::prompt,
        []
    );
    record!(
        "src/window_panes.rs::prompt",
        *mut hmux2::src::window_panes::prompt,
        []
    );
    record!(
        "src/window_switch.rs::prompt",
        *mut hmux2::src::window_switch::prompt,
        []
    );
    record!(
        "src/window_tree.rs::prompt",
        *mut hmux2::src::window_tree::prompt,
        []
    );
    record!(
        "src/window_visible.rs::prompt",
        *mut hmux2::src::window_visible::prompt,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-prompt.txt"));
}
