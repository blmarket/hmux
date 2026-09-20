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
    record!(
        "src/cmd_display_menu.rs::menu_choice_cb",
        hmux2::src::cmd_display_menu::menu_choice_cb,
        []
    );
    record!(
        "src/menu.rs::menu_choice_cb",
        hmux2::src::menu::menu_choice_cb,
        []
    );
    record!(
        "src/mode_tree.rs::menu_choice_cb",
        hmux2::src::mode_tree::menu_choice_cb,
        []
    );
    record!(
        "src/alerts.rs::menu_data",
        *mut hmux2::src::alerts::menu_data,
        []
    );
    record!(
        "src/arguments.rs::menu_data",
        *mut hmux2::src::arguments::menu_data,
        []
    );
    record!("src/cfg.rs::menu_data", *mut hmux2::src::cfg::menu_data, []);
    record!(
        "src/client.rs::menu_data",
        *mut hmux2::src::client::menu_data,
        []
    );
    record!("src/cmd.rs::menu_data", *mut hmux2::src::cmd::menu_data, []);
    record!(
        "src/cmd_attach_session.rs::menu_data",
        *mut hmux2::src::cmd_attach_session::menu_data,
        []
    );
    record!(
        "src/cmd_bind_key.rs::menu_data",
        *mut hmux2::src::cmd_bind_key::menu_data,
        []
    );
    record!(
        "src/cmd_break_pane.rs::menu_data",
        *mut hmux2::src::cmd_break_pane::menu_data,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::menu_data",
        *mut hmux2::src::cmd_capture_pane::menu_data,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::menu_data",
        *mut hmux2::src::cmd_choose_tree::menu_data,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::menu_data",
        *mut hmux2::src::cmd_command_prompt::menu_data,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::menu_data",
        *mut hmux2::src::cmd_confirm_before::menu_data,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::menu_data",
        *mut hmux2::src::cmd_copy_mode::menu_data,
        []
    );
    record!(
        "src/cmd_detach_client.rs::menu_data",
        *mut hmux2::src::cmd_detach_client::menu_data,
        []
    );
    record!(
        "src/cmd_display_menu.rs::menu_data",
        *mut hmux2::src::cmd_display_menu::menu_data,
        []
    );
    record!(
        "src/cmd_display_message.rs::menu_data",
        *mut hmux2::src::cmd_display_message::menu_data,
        []
    );
    record!(
        "src/cmd_find.rs::menu_data",
        *mut hmux2::src::cmd_find::menu_data,
        []
    );
    record!(
        "src/cmd_find_window.rs::menu_data",
        *mut hmux2::src::cmd_find_window::menu_data,
        []
    );
    record!(
        "src/cmd_if_shell.rs::menu_data",
        *mut hmux2::src::cmd_if_shell::menu_data,
        []
    );
    record!(
        "src/cmd_join_pane.rs::menu_data",
        *mut hmux2::src::cmd_join_pane::menu_data,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::menu_data",
        *mut hmux2::src::cmd_kill_pane::menu_data,
        []
    );
    record!(
        "src/cmd_kill_session.rs::menu_data",
        *mut hmux2::src::cmd_kill_session::menu_data,
        []
    );
    record!(
        "src/cmd_kill_window.rs::menu_data",
        *mut hmux2::src::cmd_kill_window::menu_data,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::menu_data",
        *mut hmux2::src::cmd_list_buffers::menu_data,
        []
    );
    record!(
        "src/cmd_list_clients.rs::menu_data",
        *mut hmux2::src::cmd_list_clients::menu_data,
        []
    );
    record!(
        "src/cmd_list_commands.rs::menu_data",
        *mut hmux2::src::cmd_list_commands::menu_data,
        []
    );
    record!(
        "src/cmd_list_keys.rs::menu_data",
        *mut hmux2::src::cmd_list_keys::menu_data,
        []
    );
    record!(
        "src/cmd_list_panes.rs::menu_data",
        *mut hmux2::src::cmd_list_panes::menu_data,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::menu_data",
        *mut hmux2::src::cmd_list_sessions::menu_data,
        []
    );
    record!(
        "src/cmd_list_windows.rs::menu_data",
        *mut hmux2::src::cmd_list_windows::menu_data,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::menu_data",
        *mut hmux2::src::cmd_load_buffer::menu_data,
        []
    );
    record!(
        "src/cmd_lock_server.rs::menu_data",
        *mut hmux2::src::cmd_lock_server::menu_data,
        []
    );
    record!(
        "src/cmd_move_window.rs::menu_data",
        *mut hmux2::src::cmd_move_window::menu_data,
        []
    );
    record!(
        "src/cmd_new_session.rs::menu_data",
        *mut hmux2::src::cmd_new_session::menu_data,
        []
    );
    record!(
        "src/cmd_new_window.rs::menu_data",
        *mut hmux2::src::cmd_new_window::menu_data,
        []
    );
    record!(
        "src/cmd_parse.rs::menu_data",
        *mut hmux2::src::cmd_parse::menu_data,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::menu_data",
        *mut hmux2::src::cmd_paste_buffer::menu_data,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::menu_data",
        *mut hmux2::src::cmd_pipe_pane::menu_data,
        []
    );
    record!(
        "src/cmd_queue.rs::menu_data",
        *mut hmux2::src::cmd_queue::menu_data,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::menu_data",
        *mut hmux2::src::cmd_refresh_client::menu_data,
        []
    );
    record!(
        "src/cmd_rename_session.rs::menu_data",
        *mut hmux2::src::cmd_rename_session::menu_data,
        []
    );
    record!(
        "src/cmd_rename_window.rs::menu_data",
        *mut hmux2::src::cmd_rename_window::menu_data,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::menu_data",
        *mut hmux2::src::cmd_resize_pane::menu_data,
        []
    );
    record!(
        "src/cmd_resize_window.rs::menu_data",
        *mut hmux2::src::cmd_resize_window::menu_data,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::menu_data",
        *mut hmux2::src::cmd_respawn_pane::menu_data,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::menu_data",
        *mut hmux2::src::cmd_respawn_window::menu_data,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::menu_data",
        *mut hmux2::src::cmd_rotate_window::menu_data,
        []
    );
    record!(
        "src/cmd_run_shell.rs::menu_data",
        *mut hmux2::src::cmd_run_shell::menu_data,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::menu_data",
        *mut hmux2::src::cmd_save_buffer::menu_data,
        []
    );
    record!(
        "src/cmd_select_layout.rs::menu_data",
        *mut hmux2::src::cmd_select_layout::menu_data,
        []
    );
    record!(
        "src/cmd_select_pane.rs::menu_data",
        *mut hmux2::src::cmd_select_pane::menu_data,
        []
    );
    record!(
        "src/cmd_select_window.rs::menu_data",
        *mut hmux2::src::cmd_select_window::menu_data,
        []
    );
    record!(
        "src/cmd_send_keys.rs::menu_data",
        *mut hmux2::src::cmd_send_keys::menu_data,
        []
    );
    record!(
        "src/cmd_server_access.rs::menu_data",
        *mut hmux2::src::cmd_server_access::menu_data,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::menu_data",
        *mut hmux2::src::cmd_set_buffer::menu_data,
        []
    );
    record!(
        "src/cmd_set_environment.rs::menu_data",
        *mut hmux2::src::cmd_set_environment::menu_data,
        []
    );
    record!(
        "src/cmd_set_option.rs::menu_data",
        *mut hmux2::src::cmd_set_option::menu_data,
        []
    );
    record!(
        "src/cmd_show_environment.rs::menu_data",
        *mut hmux2::src::cmd_show_environment::menu_data,
        []
    );
    record!(
        "src/cmd_show_messages.rs::menu_data",
        *mut hmux2::src::cmd_show_messages::menu_data,
        []
    );
    record!(
        "src/cmd_show_options.rs::menu_data",
        *mut hmux2::src::cmd_show_options::menu_data,
        []
    );
    record!(
        "src/cmd_source_file.rs::menu_data",
        *mut hmux2::src::cmd_source_file::menu_data,
        []
    );
    record!(
        "src/cmd_split_window.rs::menu_data",
        *mut hmux2::src::cmd_split_window::menu_data,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::menu_data",
        *mut hmux2::src::cmd_swap_pane::menu_data,
        []
    );
    record!(
        "src/cmd_swap_window.rs::menu_data",
        *mut hmux2::src::cmd_swap_window::menu_data,
        []
    );
    record!(
        "src/cmd_switch_client.rs::menu_data",
        *mut hmux2::src::cmd_switch_client::menu_data,
        []
    );
    record!(
        "src/cmd_wait_for.rs::menu_data",
        *mut hmux2::src::cmd_wait_for::menu_data,
        []
    );
    record!(
        "src/colour.rs::menu_data",
        *mut hmux2::src::colour::menu_data,
        []
    );
    record!(
        "src/control.rs::menu_data",
        *mut hmux2::src::control::menu_data,
        []
    );
    record!(
        "src/control_notify.rs::menu_data",
        *mut hmux2::src::control_notify::menu_data,
        []
    );
    record!(
        "src/environ.rs::menu_data",
        *mut hmux2::src::environ::menu_data,
        []
    );
    record!(
        "src/events.rs::menu_data",
        *mut hmux2::src::events::menu_data,
        []
    );
    record!(
        "src/events_payload.rs::menu_data",
        *mut hmux2::src::events_payload::menu_data,
        []
    );
    record!(
        "src/file.rs::menu_data",
        *mut hmux2::src::file::menu_data,
        []
    );
    record!(
        "src/format.rs::menu_data",
        *mut hmux2::src::format::menu_data,
        []
    );
    record!(
        "src/format_draw.rs::menu_data",
        *mut hmux2::src::format_draw::menu_data,
        []
    );
    record!(
        "src/hooks.rs::menu_data",
        *mut hmux2::src::hooks::menu_data,
        []
    );
    record!(
        "src/input.rs::menu_data",
        *mut hmux2::src::input::menu_data,
        []
    );
    record!(
        "src/input_keys.rs::menu_data",
        *mut hmux2::src::input_keys::menu_data,
        []
    );
    record!("src/job.rs::menu_data", *mut hmux2::src::job::menu_data, []);
    record!(
        "src/key_bindings.rs::menu_data",
        *mut hmux2::src::key_bindings::menu_data,
        []
    );
    record!(
        "src/layout.rs::menu_data",
        *mut hmux2::src::layout::menu_data,
        []
    );
    record!(
        "src/layout_custom.rs::menu_data",
        *mut hmux2::src::layout_custom::menu_data,
        []
    );
    record!(
        "src/layout_set.rs::menu_data",
        *mut hmux2::src::layout_set::menu_data,
        []
    );
    record!(
        "src/menu.rs::menu_data",
        hmux2::src::menu::menu_data,
        [
            w,
            flags,
            style,
            border_style,
            selected_style,
            style_gc,
            border_style_gc,
            selected_style_gc,
            border_lines,
            fs,
            key,
            m,
            s,
            px,
            py,
            menu,
            choice,
            cb,
            data
        ]
    );
    record!(
        "src/mode_tree.rs::menu_data",
        *mut hmux2::src::mode_tree::menu_data,
        []
    );
    record!(
        "src/monitor.rs::menu_data",
        *mut hmux2::src::monitor::menu_data,
        []
    );
    record!(
        "src/names.rs::menu_data",
        *mut hmux2::src::names::menu_data,
        []
    );
    record!(
        "src/options.rs::menu_data",
        *mut hmux2::src::options::menu_data,
        []
    );
    record!(
        "src/popup.rs::menu_data",
        *mut hmux2::src::popup::menu_data,
        []
    );
    record!(
        "src/prompt.rs::menu_data",
        *mut hmux2::src::prompt::menu_data,
        []
    );
    record!(
        "src/resize.rs::menu_data",
        *mut hmux2::src::resize::menu_data,
        []
    );
    record!(
        "src/screen.rs::menu_data",
        *mut hmux2::src::screen::menu_data,
        []
    );
    record!(
        "src/screen_redraw.rs::menu_data",
        *mut hmux2::src::screen_redraw::menu_data,
        []
    );
    record!(
        "src/screen_write.rs::menu_data",
        *mut hmux2::src::screen_write::menu_data,
        []
    );
    record!(
        "src/server.rs::menu_data",
        *mut hmux2::src::server::menu_data,
        []
    );
    record!(
        "src/server_acl.rs::menu_data",
        *mut hmux2::src::server_acl::menu_data,
        []
    );
    record!(
        "src/server_client.rs::menu_data",
        *mut hmux2::src::server_client::menu_data,
        []
    );
    record!(
        "src/server_fn.rs::menu_data",
        *mut hmux2::src::server_fn::menu_data,
        []
    );
    record!(
        "src/session.rs::menu_data",
        *mut hmux2::src::session::menu_data,
        []
    );
    record!(
        "src/sort.rs::menu_data",
        *mut hmux2::src::sort::menu_data,
        []
    );
    record!(
        "src/spawn.rs::menu_data",
        *mut hmux2::src::spawn::menu_data,
        []
    );
    record!(
        "src/status.rs::menu_data",
        *mut hmux2::src::status::menu_data,
        []
    );
    record!(
        "src/style.rs::menu_data",
        *mut hmux2::src::style::menu_data,
        []
    );
    record!("src/tty.rs::menu_data", *mut hmux2::src::tty::menu_data, []);
    record!(
        "src/tty_acs.rs::menu_data",
        *mut hmux2::src::tty_acs::menu_data,
        []
    );
    record!(
        "src/tty_draw.rs::menu_data",
        *mut hmux2::src::tty_draw::menu_data,
        []
    );
    record!(
        "src/tty_features.rs::menu_data",
        *mut hmux2::src::tty_features::menu_data,
        []
    );
    record!(
        "src/tty_keys.rs::menu_data",
        *mut hmux2::src::tty_keys::menu_data,
        []
    );
    record!(
        "src/tty_term.rs::menu_data",
        *mut hmux2::src::tty_term::menu_data,
        []
    );
    record!(
        "src/window.rs::menu_data",
        *mut hmux2::src::window::menu_data,
        []
    );
    record!(
        "src/window_border.rs::menu_data",
        *mut hmux2::src::window_border::menu_data,
        []
    );
    record!(
        "src/window_buffer.rs::menu_data",
        *mut hmux2::src::window_buffer::menu_data,
        []
    );
    record!(
        "src/window_client.rs::menu_data",
        *mut hmux2::src::window_client::menu_data,
        []
    );
    record!(
        "src/window_clock.rs::menu_data",
        *mut hmux2::src::window_clock::menu_data,
        []
    );
    record!(
        "src/window_copy.rs::menu_data",
        *mut hmux2::src::window_copy::menu_data,
        []
    );
    record!(
        "src/window_customize.rs::menu_data",
        *mut hmux2::src::window_customize::menu_data,
        []
    );
    record!(
        "src/window_panes.rs::menu_data",
        *mut hmux2::src::window_panes::menu_data,
        []
    );
    record!(
        "src/window_switch.rs::menu_data",
        *mut hmux2::src::window_switch::menu_data,
        []
    );
    record!(
        "src/window_tree.rs::menu_data",
        *mut hmux2::src::window_tree::menu_data,
        []
    );
    record!(
        "src/window_visible.rs::menu_data",
        *mut hmux2::src::window_visible::menu_data,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-menu.txt"));
}
