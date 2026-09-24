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
        "src/control.rs::control_block",
        hmux2::src::control::control_block,
        [size, line, t]
    );
    record!(
        "src/control.rs::control_pane",
        hmux2::src::control::control_pane,
        [pane, offset, queued, flags, pending_flag, blocks, entry]
    );
    record!(
        "src/control.rs::C2RustUnnamed_41",
        hmux2::src::control::control_pane_entry,
        [owner]
    );
    record!(
        "src/control.rs::control_panes",
        hmux2::src::control::control_panes,
        [storage]
    );
    record!(
        "src/alerts.rs::control_state",
        *mut hmux2::src::alerts::control_state,
        []
    );
    record!(
        "src/arguments.rs::control_state",
        *mut hmux2::src::arguments::control_state,
        []
    );
    record!(
        "src/cfg.rs::control_state",
        *mut hmux2::src::cfg::control_state,
        []
    );
    record!(
        "src/client.rs::control_state",
        *mut hmux2::src::client::control_state,
        []
    );
    record!(
        "src/cmd.rs::control_state",
        *mut hmux2::src::cmd::control_state,
        []
    );
    record!(
        "src/cmd_attach_session.rs::control_state",
        *mut hmux2::src::cmd_attach_session::control_state,
        []
    );
    record!(
        "src/cmd_bind_key.rs::control_state",
        *mut hmux2::src::cmd_bind_key::control_state,
        []
    );
    record!(
        "src/cmd_break_pane.rs::control_state",
        *mut hmux2::src::cmd_break_pane::control_state,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::control_state",
        *mut hmux2::src::cmd_capture_pane::control_state,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::control_state",
        *mut hmux2::src::cmd_choose_tree::control_state,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::control_state",
        *mut hmux2::src::cmd_command_prompt::control_state,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::control_state",
        *mut hmux2::src::cmd_confirm_before::control_state,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::control_state",
        *mut hmux2::src::cmd_copy_mode::control_state,
        []
    );
    record!(
        "src/cmd_detach_client.rs::control_state",
        *mut hmux2::src::cmd_detach_client::control_state,
        []
    );
    record!(
        "src/cmd_display_menu.rs::control_state",
        *mut hmux2::src::cmd_display_menu::control_state,
        []
    );
    record!(
        "src/cmd_display_message.rs::control_state",
        *mut hmux2::src::cmd_display_message::control_state,
        []
    );
    record!(
        "src/cmd_find.rs::control_state",
        *mut hmux2::src::cmd_find::control_state,
        []
    );
    record!(
        "src/cmd_find_window.rs::control_state",
        *mut hmux2::src::cmd_find_window::control_state,
        []
    );
    record!(
        "src/cmd_if_shell.rs::control_state",
        *mut hmux2::src::cmd_if_shell::control_state,
        []
    );
    record!(
        "src/cmd_join_pane.rs::control_state",
        *mut hmux2::src::cmd_join_pane::control_state,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::control_state",
        *mut hmux2::src::cmd_kill_pane::control_state,
        []
    );
    record!(
        "src/cmd_kill_session.rs::control_state",
        *mut hmux2::src::cmd_kill_session::control_state,
        []
    );
    record!(
        "src/cmd_kill_window.rs::control_state",
        *mut hmux2::src::cmd_kill_window::control_state,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::control_state",
        *mut hmux2::src::cmd_list_buffers::control_state,
        []
    );
    record!(
        "src/cmd_list_clients.rs::control_state",
        *mut hmux2::src::cmd_list_clients::control_state,
        []
    );
    record!(
        "src/cmd_list_commands.rs::control_state",
        *mut hmux2::src::cmd_list_commands::control_state,
        []
    );
    record!(
        "src/cmd_list_keys.rs::control_state",
        *mut hmux2::src::cmd_list_keys::control_state,
        []
    );
    record!(
        "src/cmd_list_panes.rs::control_state",
        *mut hmux2::src::cmd_list_panes::control_state,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::control_state",
        *mut hmux2::src::cmd_list_sessions::control_state,
        []
    );
    record!(
        "src/cmd_list_windows.rs::control_state",
        *mut hmux2::src::cmd_list_windows::control_state,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::control_state",
        *mut hmux2::src::cmd_load_buffer::control_state,
        []
    );
    record!(
        "src/cmd_lock_server.rs::control_state",
        *mut hmux2::src::cmd_lock_server::control_state,
        []
    );
    record!(
        "src/cmd_move_window.rs::control_state",
        *mut hmux2::src::cmd_move_window::control_state,
        []
    );
    record!(
        "src/cmd_new_session.rs::control_state",
        *mut hmux2::src::cmd_new_session::control_state,
        []
    );
    record!(
        "src/cmd_new_window.rs::control_state",
        *mut hmux2::src::cmd_new_window::control_state,
        []
    );
    record!(
        "src/cmd_parse.rs::control_state",
        *mut hmux2::src::cmd_parse::control_state,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::control_state",
        *mut hmux2::src::cmd_paste_buffer::control_state,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::control_state",
        *mut hmux2::src::cmd_pipe_pane::control_state,
        []
    );
    record!(
        "src/cmd_queue.rs::control_state",
        *mut hmux2::src::cmd_queue::control_state,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::control_state",
        *mut hmux2::src::cmd_refresh_client::control_state,
        []
    );
    record!(
        "src/cmd_rename_session.rs::control_state",
        *mut hmux2::src::cmd_rename_session::control_state,
        []
    );
    record!(
        "src/cmd_rename_window.rs::control_state",
        *mut hmux2::src::cmd_rename_window::control_state,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::control_state",
        *mut hmux2::src::cmd_resize_pane::control_state,
        []
    );
    record!(
        "src/cmd_resize_window.rs::control_state",
        *mut hmux2::src::cmd_resize_window::control_state,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::control_state",
        *mut hmux2::src::cmd_respawn_pane::control_state,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::control_state",
        *mut hmux2::src::cmd_respawn_window::control_state,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::control_state",
        *mut hmux2::src::cmd_rotate_window::control_state,
        []
    );
    record!(
        "src/cmd_run_shell.rs::control_state",
        *mut hmux2::src::cmd_run_shell::control_state,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::control_state",
        *mut hmux2::src::cmd_save_buffer::control_state,
        []
    );
    record!(
        "src/cmd_select_layout.rs::control_state",
        *mut hmux2::src::cmd_select_layout::control_state,
        []
    );
    record!(
        "src/cmd_select_pane.rs::control_state",
        *mut hmux2::src::cmd_select_pane::control_state,
        []
    );
    record!(
        "src/cmd_select_window.rs::control_state",
        *mut hmux2::src::cmd_select_window::control_state,
        []
    );
    record!(
        "src/cmd_send_keys.rs::control_state",
        *mut hmux2::src::cmd_send_keys::control_state,
        []
    );
    record!(
        "src/cmd_server_access.rs::control_state",
        *mut hmux2::src::cmd_server_access::control_state,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::control_state",
        *mut hmux2::src::cmd_set_buffer::control_state,
        []
    );
    record!(
        "src/cmd_set_environment.rs::control_state",
        *mut hmux2::src::cmd_set_environment::control_state,
        []
    );
    record!(
        "src/cmd_set_option.rs::control_state",
        *mut hmux2::src::cmd_set_option::control_state,
        []
    );
    record!(
        "src/cmd_show_environment.rs::control_state",
        *mut hmux2::src::cmd_show_environment::control_state,
        []
    );
    record!(
        "src/cmd_show_messages.rs::control_state",
        *mut hmux2::src::cmd_show_messages::control_state,
        []
    );
    record!(
        "src/cmd_show_options.rs::control_state",
        *mut hmux2::src::cmd_show_options::control_state,
        []
    );
    record!(
        "src/cmd_source_file.rs::control_state",
        *mut hmux2::src::cmd_source_file::control_state,
        []
    );
    record!(
        "src/cmd_split_window.rs::control_state",
        *mut hmux2::src::cmd_split_window::control_state,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::control_state",
        *mut hmux2::src::cmd_swap_pane::control_state,
        []
    );
    record!(
        "src/cmd_swap_window.rs::control_state",
        *mut hmux2::src::cmd_swap_window::control_state,
        []
    );
    record!(
        "src/cmd_switch_client.rs::control_state",
        *mut hmux2::src::cmd_switch_client::control_state,
        []
    );
    record!(
        "src/cmd_wait_for.rs::control_state",
        *mut hmux2::src::cmd_wait_for::control_state,
        []
    );
    record!(
        "src/colour.rs::control_state",
        *mut hmux2::src::colour::control_state,
        []
    );
    record!(
        "src/control.rs::control_state",
        hmux2::src::control::control_state,
        [
            panes,
            windows,
            pending_count,
            queued_reply_bytes,
            read_event,
            write_event,
            subs,
            guard_depth
        ]
    );
    record!(
        "src/control_notify.rs::control_state",
        *mut hmux2::src::control_notify::control_state,
        []
    );
    record!(
        "src/environ.rs::control_state",
        *mut hmux2::src::environ::control_state,
        []
    );
    record!(
        "src/events.rs::control_state",
        *mut hmux2::src::events::control_state,
        []
    );
    record!(
        "src/events_payload.rs::control_state",
        *mut hmux2::src::events_payload::control_state,
        []
    );
    record!(
        "src/file.rs::control_state",
        *mut hmux2::src::file::control_state,
        []
    );
    record!(
        "src/format.rs::control_state",
        *mut hmux2::src::format::control_state,
        []
    );
    record!(
        "src/format_draw.rs::control_state",
        *mut hmux2::src::format_draw::control_state,
        []
    );
    record!(
        "src/hooks.rs::control_state",
        *mut hmux2::src::hooks::control_state,
        []
    );
    record!(
        "src/input.rs::control_state",
        *mut hmux2::src::input::control_state,
        []
    );
    record!(
        "src/input_keys.rs::control_state",
        *mut hmux2::src::input_keys::control_state,
        []
    );
    record!(
        "src/job.rs::control_state",
        *mut hmux2::src::job::control_state,
        []
    );
    record!(
        "src/key_bindings.rs::control_state",
        *mut hmux2::src::key_bindings::control_state,
        []
    );
    record!(
        "src/layout.rs::control_state",
        *mut hmux2::src::layout::control_state,
        []
    );
    record!(
        "src/layout_custom.rs::control_state",
        *mut hmux2::src::layout_custom::control_state,
        []
    );
    record!(
        "src/layout_set.rs::control_state",
        *mut hmux2::src::layout_set::control_state,
        []
    );
    record!(
        "src/menu.rs::control_state",
        *mut hmux2::src::menu::control_state,
        []
    );
    record!(
        "src/mode_tree.rs::control_state",
        *mut hmux2::src::mode_tree::control_state,
        []
    );
    record!(
        "src/monitor.rs::control_state",
        *mut hmux2::src::monitor::control_state,
        []
    );
    record!(
        "src/names.rs::control_state",
        *mut hmux2::src::names::control_state,
        []
    );
    record!(
        "src/options.rs::control_state",
        *mut hmux2::src::options::control_state,
        []
    );
    record!(
        "src/popup.rs::control_state",
        *mut hmux2::src::popup::control_state,
        []
    );
    record!(
        "src/prompt.rs::control_state",
        *mut hmux2::src::prompt::control_state,
        []
    );
    record!(
        "src/resize.rs::control_state",
        *mut hmux2::src::resize::control_state,
        []
    );
    record!(
        "src/screen.rs::control_state",
        *mut hmux2::src::screen::control_state,
        []
    );
    record!(
        "src/screen_redraw.rs::control_state",
        *mut hmux2::src::screen_redraw::control_state,
        []
    );
    record!(
        "src/screen_write.rs::control_state",
        *mut hmux2::src::screen_write::control_state,
        []
    );
    record!(
        "src/server.rs::control_state",
        *mut hmux2::src::server::control_state,
        []
    );
    record!(
        "src/server_acl.rs::control_state",
        *mut hmux2::src::server_acl::control_state,
        []
    );
    record!(
        "src/server_client.rs::control_state",
        *mut hmux2::src::server_client::control_state,
        []
    );
    record!(
        "src/server_fn.rs::control_state",
        *mut hmux2::src::server_fn::control_state,
        []
    );
    record!(
        "src/session.rs::control_state",
        *mut hmux2::src::session::control_state,
        []
    );
    record!(
        "src/sort.rs::control_state",
        *mut hmux2::src::sort::control_state,
        []
    );
    record!(
        "src/spawn.rs::control_state",
        *mut hmux2::src::spawn::control_state,
        []
    );
    record!(
        "src/status.rs::control_state",
        *mut hmux2::src::status::control_state,
        []
    );
    record!(
        "src/style.rs::control_state",
        *mut hmux2::src::style::control_state,
        []
    );
    record!(
        "src/tty.rs::control_state",
        *mut hmux2::src::tty::control_state,
        []
    );
    record!(
        "src/tty_acs.rs::control_state",
        *mut hmux2::src::tty_acs::control_state,
        []
    );
    record!(
        "src/tty_draw.rs::control_state",
        *mut hmux2::src::tty_draw::control_state,
        []
    );
    record!(
        "src/tty_features.rs::control_state",
        *mut hmux2::src::tty_features::control_state,
        []
    );
    record!(
        "src/tty_keys.rs::control_state",
        *mut hmux2::src::tty_keys::control_state,
        []
    );
    record!(
        "src/tty_term.rs::control_state",
        *mut hmux2::src::tty_term::control_state,
        []
    );
    record!(
        "src/window.rs::control_state",
        *mut hmux2::src::window::control_state,
        []
    );
    record!(
        "src/window_border.rs::control_state",
        *mut hmux2::src::window_border::control_state,
        []
    );
    record!(
        "src/window_buffer.rs::control_state",
        *mut hmux2::src::window_buffer::control_state,
        []
    );
    record!(
        "src/window_client.rs::control_state",
        *mut hmux2::src::window_client::control_state,
        []
    );
    record!(
        "src/window_clock.rs::control_state",
        *mut hmux2::src::window_clock::control_state,
        []
    );
    record!(
        "src/window_copy.rs::control_state",
        *mut hmux2::src::window_copy::control_state,
        []
    );
    record!(
        "src/window_customize.rs::control_state",
        *mut hmux2::src::window_customize::control_state,
        []
    );
    record!(
        "src/window_panes.rs::control_state",
        *mut hmux2::src::window_panes::control_state,
        []
    );
    record!(
        "src/window_switch.rs::control_state",
        *mut hmux2::src::window_switch::control_state,
        []
    );
    record!(
        "src/window_tree.rs::control_state",
        *mut hmux2::src::window_tree::control_state,
        []
    );
    record!(
        "src/window_visible.rs::control_state",
        *mut hmux2::src::window_visible::control_state,
        []
    );
    record!(
        "src/control.rs::control_window",
        hmux2::src::control::control_window,
        [window, sx, sy, entry]
    );
    record!(
        "src/control.rs::C2RustUnnamed_44",
        hmux2::src::control::control_window_entry,
        [owner]
    );
    record!(
        "src/control.rs::control_windows",
        hmux2::src::control::control_windows,
        [storage]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-control.txt"));
}
