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
        "src/input.rs::input_cell",
        hmux2::src::input::input_cell,
        [cell, set, g0set, g1set]
    );
    record!(
        "src/alerts.rs::input_ctx",
        *mut hmux2::src::alerts::input_ctx,
        []
    );
    record!(
        "src/arguments.rs::input_ctx",
        *mut hmux2::src::arguments::input_ctx,
        []
    );
    record!("src/cfg.rs::input_ctx", *mut hmux2::src::cfg::input_ctx, []);
    record!(
        "src/client.rs::input_ctx",
        *mut hmux2::src::client::input_ctx,
        []
    );
    record!("src/cmd.rs::input_ctx", *mut hmux2::src::cmd::input_ctx, []);
    record!(
        "src/cmd_attach_session.rs::input_ctx",
        *mut hmux2::src::cmd_attach_session::input_ctx,
        []
    );
    record!(
        "src/cmd_bind_key.rs::input_ctx",
        *mut hmux2::src::cmd_bind_key::input_ctx,
        []
    );
    record!(
        "src/cmd_break_pane.rs::input_ctx",
        *mut hmux2::src::cmd_break_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::input_ctx",
        *mut hmux2::src::cmd_capture_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::input_ctx",
        *mut hmux2::src::cmd_choose_tree::input_ctx,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::input_ctx",
        *mut hmux2::src::cmd_command_prompt::input_ctx,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::input_ctx",
        *mut hmux2::src::cmd_confirm_before::input_ctx,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::input_ctx",
        *mut hmux2::src::cmd_copy_mode::input_ctx,
        []
    );
    record!(
        "src/cmd_detach_client.rs::input_ctx",
        *mut hmux2::src::cmd_detach_client::input_ctx,
        []
    );
    record!(
        "src/cmd_display_menu.rs::input_ctx",
        *mut hmux2::src::cmd_display_menu::input_ctx,
        []
    );
    record!(
        "src/cmd_display_message.rs::input_ctx",
        *mut hmux2::src::cmd_display_message::input_ctx,
        []
    );
    record!(
        "src/cmd_find.rs::input_ctx",
        *mut hmux2::src::cmd_find::input_ctx,
        []
    );
    record!(
        "src/cmd_find_window.rs::input_ctx",
        *mut hmux2::src::cmd_find_window::input_ctx,
        []
    );
    record!(
        "src/cmd_if_shell.rs::input_ctx",
        *mut hmux2::src::cmd_if_shell::input_ctx,
        []
    );
    record!(
        "src/cmd_join_pane.rs::input_ctx",
        *mut hmux2::src::cmd_join_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::input_ctx",
        *mut hmux2::src::cmd_kill_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_kill_session.rs::input_ctx",
        *mut hmux2::src::cmd_kill_session::input_ctx,
        []
    );
    record!(
        "src/cmd_kill_window.rs::input_ctx",
        *mut hmux2::src::cmd_kill_window::input_ctx,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::input_ctx",
        *mut hmux2::src::cmd_list_buffers::input_ctx,
        []
    );
    record!(
        "src/cmd_list_clients.rs::input_ctx",
        *mut hmux2::src::cmd_list_clients::input_ctx,
        []
    );
    record!(
        "src/cmd_list_commands.rs::input_ctx",
        *mut hmux2::src::cmd_list_commands::input_ctx,
        []
    );
    record!(
        "src/cmd_list_keys.rs::input_ctx",
        *mut hmux2::src::cmd_list_keys::input_ctx,
        []
    );
    record!(
        "src/cmd_list_panes.rs::input_ctx",
        *mut hmux2::src::cmd_list_panes::input_ctx,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::input_ctx",
        *mut hmux2::src::cmd_list_sessions::input_ctx,
        []
    );
    record!(
        "src/cmd_list_windows.rs::input_ctx",
        *mut hmux2::src::cmd_list_windows::input_ctx,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::input_ctx",
        *mut hmux2::src::cmd_load_buffer::input_ctx,
        []
    );
    record!(
        "src/cmd_lock_server.rs::input_ctx",
        *mut hmux2::src::cmd_lock_server::input_ctx,
        []
    );
    record!(
        "src/cmd_move_window.rs::input_ctx",
        *mut hmux2::src::cmd_move_window::input_ctx,
        []
    );
    record!(
        "src/cmd_new_session.rs::input_ctx",
        *mut hmux2::src::cmd_new_session::input_ctx,
        []
    );
    record!(
        "src/cmd_new_window.rs::input_ctx",
        *mut hmux2::src::cmd_new_window::input_ctx,
        []
    );
    record!(
        "src/cmd_parse.rs::input_ctx",
        *mut hmux2::src::cmd_parse::input_ctx,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::input_ctx",
        *mut hmux2::src::cmd_paste_buffer::input_ctx,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::input_ctx",
        *mut hmux2::src::cmd_pipe_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_queue.rs::input_ctx",
        *mut hmux2::src::cmd_queue::input_ctx,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::input_ctx",
        *mut hmux2::src::cmd_refresh_client::input_ctx,
        []
    );
    record!(
        "src/cmd_rename_session.rs::input_ctx",
        *mut hmux2::src::cmd_rename_session::input_ctx,
        []
    );
    record!(
        "src/cmd_rename_window.rs::input_ctx",
        *mut hmux2::src::cmd_rename_window::input_ctx,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::input_ctx",
        *mut hmux2::src::cmd_resize_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_resize_window.rs::input_ctx",
        *mut hmux2::src::cmd_resize_window::input_ctx,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::input_ctx",
        *mut hmux2::src::cmd_respawn_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::input_ctx",
        *mut hmux2::src::cmd_respawn_window::input_ctx,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::input_ctx",
        *mut hmux2::src::cmd_rotate_window::input_ctx,
        []
    );
    record!(
        "src/cmd_run_shell.rs::input_ctx",
        *mut hmux2::src::cmd_run_shell::input_ctx,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::input_ctx",
        *mut hmux2::src::cmd_save_buffer::input_ctx,
        []
    );
    record!(
        "src/cmd_select_layout.rs::input_ctx",
        *mut hmux2::src::cmd_select_layout::input_ctx,
        []
    );
    record!(
        "src/cmd_select_pane.rs::input_ctx",
        *mut hmux2::src::cmd_select_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_select_window.rs::input_ctx",
        *mut hmux2::src::cmd_select_window::input_ctx,
        []
    );
    record!(
        "src/cmd_send_keys.rs::input_ctx",
        *mut hmux2::src::cmd_send_keys::input_ctx,
        []
    );
    record!(
        "src/cmd_server_access.rs::input_ctx",
        *mut hmux2::src::cmd_server_access::input_ctx,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::input_ctx",
        *mut hmux2::src::cmd_set_buffer::input_ctx,
        []
    );
    record!(
        "src/cmd_set_environment.rs::input_ctx",
        *mut hmux2::src::cmd_set_environment::input_ctx,
        []
    );
    record!(
        "src/cmd_set_option.rs::input_ctx",
        *mut hmux2::src::cmd_set_option::input_ctx,
        []
    );
    record!(
        "src/cmd_show_environment.rs::input_ctx",
        *mut hmux2::src::cmd_show_environment::input_ctx,
        []
    );
    record!(
        "src/cmd_show_messages.rs::input_ctx",
        *mut hmux2::src::cmd_show_messages::input_ctx,
        []
    );
    record!(
        "src/cmd_show_options.rs::input_ctx",
        *mut hmux2::src::cmd_show_options::input_ctx,
        []
    );
    record!(
        "src/cmd_source_file.rs::input_ctx",
        *mut hmux2::src::cmd_source_file::input_ctx,
        []
    );
    record!(
        "src/cmd_split_window.rs::input_ctx",
        *mut hmux2::src::cmd_split_window::input_ctx,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::input_ctx",
        *mut hmux2::src::cmd_swap_pane::input_ctx,
        []
    );
    record!(
        "src/cmd_swap_window.rs::input_ctx",
        *mut hmux2::src::cmd_swap_window::input_ctx,
        []
    );
    record!(
        "src/cmd_switch_client.rs::input_ctx",
        *mut hmux2::src::cmd_switch_client::input_ctx,
        []
    );
    record!(
        "src/cmd_wait_for.rs::input_ctx",
        *mut hmux2::src::cmd_wait_for::input_ctx,
        []
    );
    record!(
        "src/colour.rs::input_ctx",
        *mut hmux2::src::colour::input_ctx,
        []
    );
    record!(
        "src/control.rs::input_ctx",
        *mut hmux2::src::control::input_ctx,
        []
    );
    record!(
        "src/control_notify.rs::input_ctx",
        *mut hmux2::src::control_notify::input_ctx,
        []
    );
    record!(
        "src/environ.rs::input_ctx",
        *mut hmux2::src::environ::input_ctx,
        []
    );
    record!(
        "src/events.rs::input_ctx",
        *mut hmux2::src::events::input_ctx,
        []
    );
    record!(
        "src/events_payload.rs::input_ctx",
        *mut hmux2::src::events_payload::input_ctx,
        []
    );
    record!(
        "src/file.rs::input_ctx",
        *mut hmux2::src::file::input_ctx,
        []
    );
    record!(
        "src/format.rs::input_ctx",
        *mut hmux2::src::format::input_ctx,
        []
    );
    record!(
        "src/format_draw.rs::input_ctx",
        *mut hmux2::src::format_draw::input_ctx,
        []
    );
    record!(
        "src/hooks.rs::input_ctx",
        *mut hmux2::src::hooks::input_ctx,
        []
    );
    record!(
        "src/input.rs::input_ctx",
        hmux2::src::input::input_ctx,
        [
            wp,
            event,
            ctx,
            palette,
            c,
            cell,
            old_cell,
            old_cx,
            old_cy,
            old_mode,
            interm_buf,
            interm_len,
            param_buf,
            param_len,
            input_buf,
            input_len,
            input_space,
            input_end,
            param_list,
            param_list_len,
            utf8data,
            utf8started,
            ch,
            last,
            state,
            flags,
            requests,
            request_count,
            request_timer,
            since_ground,
            ground_timer
        ]
    );
    record!(
        "src/input_keys.rs::input_ctx",
        *mut hmux2::src::input_keys::input_ctx,
        []
    );
    record!("src/job.rs::input_ctx", *mut hmux2::src::job::input_ctx, []);
    record!(
        "src/key_bindings.rs::input_ctx",
        *mut hmux2::src::key_bindings::input_ctx,
        []
    );
    record!(
        "src/layout.rs::input_ctx",
        *mut hmux2::src::layout::input_ctx,
        []
    );
    record!(
        "src/layout_custom.rs::input_ctx",
        *mut hmux2::src::layout_custom::input_ctx,
        []
    );
    record!(
        "src/layout_set.rs::input_ctx",
        *mut hmux2::src::layout_set::input_ctx,
        []
    );
    record!(
        "src/menu.rs::input_ctx",
        *mut hmux2::src::menu::input_ctx,
        []
    );
    record!(
        "src/mode_tree.rs::input_ctx",
        *mut hmux2::src::mode_tree::input_ctx,
        []
    );
    record!(
        "src/monitor.rs::input_ctx",
        *mut hmux2::src::monitor::input_ctx,
        []
    );
    record!(
        "src/names.rs::input_ctx",
        *mut hmux2::src::names::input_ctx,
        []
    );
    record!(
        "src/options.rs::input_ctx",
        *mut hmux2::src::options::input_ctx,
        []
    );
    record!(
        "src/popup.rs::input_ctx",
        *mut hmux2::src::popup::input_ctx,
        []
    );
    record!(
        "src/prompt.rs::input_ctx",
        *mut hmux2::src::prompt::input_ctx,
        []
    );
    record!(
        "src/resize.rs::input_ctx",
        *mut hmux2::src::resize::input_ctx,
        []
    );
    record!(
        "src/screen.rs::input_ctx",
        *mut hmux2::src::screen::input_ctx,
        []
    );
    record!(
        "src/screen_redraw.rs::input_ctx",
        *mut hmux2::src::screen_redraw::input_ctx,
        []
    );
    record!(
        "src/screen_write.rs::input_ctx",
        *mut hmux2::src::screen_write::input_ctx,
        []
    );
    record!(
        "src/server.rs::input_ctx",
        *mut hmux2::src::server::input_ctx,
        []
    );
    record!(
        "src/server_acl.rs::input_ctx",
        *mut hmux2::src::server_acl::input_ctx,
        []
    );
    record!(
        "src/server_client.rs::input_ctx",
        *mut hmux2::src::server_client::input_ctx,
        []
    );
    record!(
        "src/server_fn.rs::input_ctx",
        *mut hmux2::src::server_fn::input_ctx,
        []
    );
    record!(
        "src/session.rs::input_ctx",
        *mut hmux2::src::session::input_ctx,
        []
    );
    record!(
        "src/sort.rs::input_ctx",
        *mut hmux2::src::sort::input_ctx,
        []
    );
    record!(
        "src/spawn.rs::input_ctx",
        *mut hmux2::src::spawn::input_ctx,
        []
    );
    record!(
        "src/status.rs::input_ctx",
        *mut hmux2::src::status::input_ctx,
        []
    );
    record!(
        "src/style.rs::input_ctx",
        *mut hmux2::src::style::input_ctx,
        []
    );
    record!("src/tty.rs::input_ctx", *mut hmux2::src::tty::input_ctx, []);
    record!(
        "src/tty_acs.rs::input_ctx",
        *mut hmux2::src::tty_acs::input_ctx,
        []
    );
    record!(
        "src/tty_draw.rs::input_ctx",
        *mut hmux2::src::tty_draw::input_ctx,
        []
    );
    record!(
        "src/tty_features.rs::input_ctx",
        *mut hmux2::src::tty_features::input_ctx,
        []
    );
    record!(
        "src/tty_keys.rs::input_ctx",
        *mut hmux2::src::tty_keys::input_ctx,
        []
    );
    record!(
        "src/tty_term.rs::input_ctx",
        *mut hmux2::src::tty_term::input_ctx,
        []
    );
    record!(
        "src/window.rs::input_ctx",
        *mut hmux2::src::window::input_ctx,
        []
    );
    record!(
        "src/window_border.rs::input_ctx",
        *mut hmux2::src::window_border::input_ctx,
        []
    );
    record!(
        "src/window_buffer.rs::input_ctx",
        *mut hmux2::src::window_buffer::input_ctx,
        []
    );
    record!(
        "src/window_client.rs::input_ctx",
        *mut hmux2::src::window_client::input_ctx,
        []
    );
    record!(
        "src/window_clock.rs::input_ctx",
        *mut hmux2::src::window_clock::input_ctx,
        []
    );
    record!(
        "src/window_copy.rs::input_ctx",
        *mut hmux2::src::window_copy::input_ctx,
        []
    );
    record!(
        "src/window_customize.rs::input_ctx",
        *mut hmux2::src::window_customize::input_ctx,
        []
    );
    record!(
        "src/window_panes.rs::input_ctx",
        *mut hmux2::src::window_panes::input_ctx,
        []
    );
    record!(
        "src/window_switch.rs::input_ctx",
        *mut hmux2::src::window_switch::input_ctx,
        []
    );
    record!(
        "src/window_tree.rs::input_ctx",
        *mut hmux2::src::window_tree::input_ctx,
        []
    );
    record!(
        "src/window_visible.rs::input_ctx",
        *mut hmux2::src::window_visible::input_ctx,
        []
    );
    record!(
        "src/input.rs::input_end_type",
        hmux2::src::input::input_end_type,
        []
    );
    record!(
        "src/input.rs::input_param",
        hmux2::src::input::input_param,
        [type_0, c2rust_unnamed]
    );
    record!(
        "src/input.rs::C2RustUnnamed_32",
        hmux2::src::input::input_param_c2rust_unnamed,
        [num, str_0]
    );
    record!(
        "src/input.rs::C2RustUnnamed_33",
        hmux2::src::input::input_param_type_0,
        []
    );
    record!(
        "src/alerts.rs::input_request",
        *mut hmux2::src::alerts::input_request,
        []
    );
    record!(
        "src/arguments.rs::input_request",
        *mut hmux2::src::arguments::input_request,
        []
    );
    record!(
        "src/cfg.rs::input_request",
        *mut hmux2::src::cfg::input_request,
        []
    );
    record!(
        "src/client.rs::input_request",
        *mut hmux2::src::client::input_request,
        []
    );
    record!(
        "src/cmd.rs::input_request",
        *mut hmux2::src::cmd::input_request,
        []
    );
    record!(
        "src/cmd_attach_session.rs::input_request",
        *mut hmux2::src::cmd_attach_session::input_request,
        []
    );
    record!(
        "src/cmd_bind_key.rs::input_request",
        *mut hmux2::src::cmd_bind_key::input_request,
        []
    );
    record!(
        "src/cmd_break_pane.rs::input_request",
        *mut hmux2::src::cmd_break_pane::input_request,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::input_request",
        *mut hmux2::src::cmd_capture_pane::input_request,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::input_request",
        *mut hmux2::src::cmd_choose_tree::input_request,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::input_request",
        *mut hmux2::src::cmd_command_prompt::input_request,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::input_request",
        *mut hmux2::src::cmd_confirm_before::input_request,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::input_request",
        *mut hmux2::src::cmd_copy_mode::input_request,
        []
    );
    record!(
        "src/cmd_detach_client.rs::input_request",
        *mut hmux2::src::cmd_detach_client::input_request,
        []
    );
    record!(
        "src/cmd_display_menu.rs::input_request",
        *mut hmux2::src::cmd_display_menu::input_request,
        []
    );
    record!(
        "src/cmd_display_message.rs::input_request",
        *mut hmux2::src::cmd_display_message::input_request,
        []
    );
    record!(
        "src/cmd_find.rs::input_request",
        *mut hmux2::src::cmd_find::input_request,
        []
    );
    record!(
        "src/cmd_find_window.rs::input_request",
        *mut hmux2::src::cmd_find_window::input_request,
        []
    );
    record!(
        "src/cmd_if_shell.rs::input_request",
        *mut hmux2::src::cmd_if_shell::input_request,
        []
    );
    record!(
        "src/cmd_join_pane.rs::input_request",
        *mut hmux2::src::cmd_join_pane::input_request,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::input_request",
        *mut hmux2::src::cmd_kill_pane::input_request,
        []
    );
    record!(
        "src/cmd_kill_session.rs::input_request",
        *mut hmux2::src::cmd_kill_session::input_request,
        []
    );
    record!(
        "src/cmd_kill_window.rs::input_request",
        *mut hmux2::src::cmd_kill_window::input_request,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::input_request",
        *mut hmux2::src::cmd_list_buffers::input_request,
        []
    );
    record!(
        "src/cmd_list_clients.rs::input_request",
        *mut hmux2::src::cmd_list_clients::input_request,
        []
    );
    record!(
        "src/cmd_list_commands.rs::input_request",
        *mut hmux2::src::cmd_list_commands::input_request,
        []
    );
    record!(
        "src/cmd_list_keys.rs::input_request",
        *mut hmux2::src::cmd_list_keys::input_request,
        []
    );
    record!(
        "src/cmd_list_panes.rs::input_request",
        *mut hmux2::src::cmd_list_panes::input_request,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::input_request",
        *mut hmux2::src::cmd_list_sessions::input_request,
        []
    );
    record!(
        "src/cmd_list_windows.rs::input_request",
        *mut hmux2::src::cmd_list_windows::input_request,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::input_request",
        *mut hmux2::src::cmd_load_buffer::input_request,
        []
    );
    record!(
        "src/cmd_lock_server.rs::input_request",
        *mut hmux2::src::cmd_lock_server::input_request,
        []
    );
    record!(
        "src/cmd_move_window.rs::input_request",
        *mut hmux2::src::cmd_move_window::input_request,
        []
    );
    record!(
        "src/cmd_new_session.rs::input_request",
        *mut hmux2::src::cmd_new_session::input_request,
        []
    );
    record!(
        "src/cmd_new_window.rs::input_request",
        *mut hmux2::src::cmd_new_window::input_request,
        []
    );
    record!(
        "src/cmd_parse.rs::input_request",
        *mut hmux2::src::cmd_parse::input_request,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::input_request",
        *mut hmux2::src::cmd_paste_buffer::input_request,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::input_request",
        *mut hmux2::src::cmd_pipe_pane::input_request,
        []
    );
    record!(
        "src/cmd_queue.rs::input_request",
        *mut hmux2::src::cmd_queue::input_request,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::input_request",
        *mut hmux2::src::cmd_refresh_client::input_request,
        []
    );
    record!(
        "src/cmd_rename_session.rs::input_request",
        *mut hmux2::src::cmd_rename_session::input_request,
        []
    );
    record!(
        "src/cmd_rename_window.rs::input_request",
        *mut hmux2::src::cmd_rename_window::input_request,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::input_request",
        *mut hmux2::src::cmd_resize_pane::input_request,
        []
    );
    record!(
        "src/cmd_resize_window.rs::input_request",
        *mut hmux2::src::cmd_resize_window::input_request,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::input_request",
        *mut hmux2::src::cmd_respawn_pane::input_request,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::input_request",
        *mut hmux2::src::cmd_respawn_window::input_request,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::input_request",
        *mut hmux2::src::cmd_rotate_window::input_request,
        []
    );
    record!(
        "src/cmd_run_shell.rs::input_request",
        *mut hmux2::src::cmd_run_shell::input_request,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::input_request",
        *mut hmux2::src::cmd_save_buffer::input_request,
        []
    );
    record!(
        "src/cmd_select_layout.rs::input_request",
        *mut hmux2::src::cmd_select_layout::input_request,
        []
    );
    record!(
        "src/cmd_select_pane.rs::input_request",
        *mut hmux2::src::cmd_select_pane::input_request,
        []
    );
    record!(
        "src/cmd_select_window.rs::input_request",
        *mut hmux2::src::cmd_select_window::input_request,
        []
    );
    record!(
        "src/cmd_send_keys.rs::input_request",
        *mut hmux2::src::cmd_send_keys::input_request,
        []
    );
    record!(
        "src/cmd_server_access.rs::input_request",
        *mut hmux2::src::cmd_server_access::input_request,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::input_request",
        *mut hmux2::src::cmd_set_buffer::input_request,
        []
    );
    record!(
        "src/cmd_set_environment.rs::input_request",
        *mut hmux2::src::cmd_set_environment::input_request,
        []
    );
    record!(
        "src/cmd_set_option.rs::input_request",
        *mut hmux2::src::cmd_set_option::input_request,
        []
    );
    record!(
        "src/cmd_show_environment.rs::input_request",
        *mut hmux2::src::cmd_show_environment::input_request,
        []
    );
    record!(
        "src/cmd_show_messages.rs::input_request",
        *mut hmux2::src::cmd_show_messages::input_request,
        []
    );
    record!(
        "src/cmd_show_options.rs::input_request",
        *mut hmux2::src::cmd_show_options::input_request,
        []
    );
    record!(
        "src/cmd_source_file.rs::input_request",
        *mut hmux2::src::cmd_source_file::input_request,
        []
    );
    record!(
        "src/cmd_split_window.rs::input_request",
        *mut hmux2::src::cmd_split_window::input_request,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::input_request",
        *mut hmux2::src::cmd_swap_pane::input_request,
        []
    );
    record!(
        "src/cmd_swap_window.rs::input_request",
        *mut hmux2::src::cmd_swap_window::input_request,
        []
    );
    record!(
        "src/cmd_switch_client.rs::input_request",
        *mut hmux2::src::cmd_switch_client::input_request,
        []
    );
    record!(
        "src/cmd_wait_for.rs::input_request",
        *mut hmux2::src::cmd_wait_for::input_request,
        []
    );
    record!(
        "src/colour.rs::input_request",
        *mut hmux2::src::colour::input_request,
        []
    );
    record!(
        "src/control.rs::input_request",
        *mut hmux2::src::control::input_request,
        []
    );
    record!(
        "src/control_notify.rs::input_request",
        *mut hmux2::src::control_notify::input_request,
        []
    );
    record!(
        "src/environ.rs::input_request",
        *mut hmux2::src::environ::input_request,
        []
    );
    record!(
        "src/events.rs::input_request",
        *mut hmux2::src::events::input_request,
        []
    );
    record!(
        "src/events_payload.rs::input_request",
        *mut hmux2::src::events_payload::input_request,
        []
    );
    record!(
        "src/file.rs::input_request",
        *mut hmux2::src::file::input_request,
        []
    );
    record!(
        "src/format.rs::input_request",
        *mut hmux2::src::format::input_request,
        []
    );
    record!(
        "src/format_draw.rs::input_request",
        *mut hmux2::src::format_draw::input_request,
        []
    );
    record!(
        "src/hooks.rs::input_request",
        *mut hmux2::src::hooks::input_request,
        []
    );
    record!(
        "src/input.rs::input_request",
        hmux2::src::input::input_request,
        [c, ictx, type_0, t, end, idx, data, entry, centry]
    );
    record!(
        "src/input_keys.rs::input_request",
        *mut hmux2::src::input_keys::input_request,
        []
    );
    record!(
        "src/job.rs::input_request",
        *mut hmux2::src::job::input_request,
        []
    );
    record!(
        "src/key_bindings.rs::input_request",
        *mut hmux2::src::key_bindings::input_request,
        []
    );
    record!(
        "src/layout.rs::input_request",
        *mut hmux2::src::layout::input_request,
        []
    );
    record!(
        "src/layout_custom.rs::input_request",
        *mut hmux2::src::layout_custom::input_request,
        []
    );
    record!(
        "src/layout_set.rs::input_request",
        *mut hmux2::src::layout_set::input_request,
        []
    );
    record!(
        "src/menu.rs::input_request",
        *mut hmux2::src::menu::input_request,
        []
    );
    record!(
        "src/mode_tree.rs::input_request",
        *mut hmux2::src::mode_tree::input_request,
        []
    );
    record!(
        "src/monitor.rs::input_request",
        *mut hmux2::src::monitor::input_request,
        []
    );
    record!(
        "src/names.rs::input_request",
        *mut hmux2::src::names::input_request,
        []
    );
    record!(
        "src/options.rs::input_request",
        *mut hmux2::src::options::input_request,
        []
    );
    record!(
        "src/popup.rs::input_request",
        *mut hmux2::src::popup::input_request,
        []
    );
    record!(
        "src/prompt.rs::input_request",
        *mut hmux2::src::prompt::input_request,
        []
    );
    record!(
        "src/resize.rs::input_request",
        *mut hmux2::src::resize::input_request,
        []
    );
    record!(
        "src/screen.rs::input_request",
        *mut hmux2::src::screen::input_request,
        []
    );
    record!(
        "src/screen_redraw.rs::input_request",
        *mut hmux2::src::screen_redraw::input_request,
        []
    );
    record!(
        "src/screen_write.rs::input_request",
        *mut hmux2::src::screen_write::input_request,
        []
    );
    record!(
        "src/server.rs::input_request",
        *mut hmux2::src::server::input_request,
        []
    );
    record!(
        "src/server_acl.rs::input_request",
        *mut hmux2::src::server_acl::input_request,
        []
    );
    record!(
        "src/server_client.rs::input_request",
        *mut hmux2::src::server_client::input_request,
        []
    );
    record!(
        "src/server_fn.rs::input_request",
        *mut hmux2::src::server_fn::input_request,
        []
    );
    record!(
        "src/session.rs::input_request",
        *mut hmux2::src::session::input_request,
        []
    );
    record!(
        "src/sort.rs::input_request",
        *mut hmux2::src::sort::input_request,
        []
    );
    record!(
        "src/spawn.rs::input_request",
        *mut hmux2::src::spawn::input_request,
        []
    );
    record!(
        "src/status.rs::input_request",
        *mut hmux2::src::status::input_request,
        []
    );
    record!(
        "src/style.rs::input_request",
        *mut hmux2::src::style::input_request,
        []
    );
    record!(
        "src/tty.rs::input_request",
        *mut hmux2::src::tty::input_request,
        []
    );
    record!(
        "src/tty_acs.rs::input_request",
        *mut hmux2::src::tty_acs::input_request,
        []
    );
    record!(
        "src/tty_draw.rs::input_request",
        *mut hmux2::src::tty_draw::input_request,
        []
    );
    record!(
        "src/tty_features.rs::input_request",
        *mut hmux2::src::tty_features::input_request,
        []
    );
    record!(
        "src/tty_keys.rs::input_request",
        *mut hmux2::src::tty_keys::input_request,
        []
    );
    record!(
        "src/tty_term.rs::input_request",
        *mut hmux2::src::tty_term::input_request,
        []
    );
    record!(
        "src/window.rs::input_request",
        *mut hmux2::src::window::input_request,
        []
    );
    record!(
        "src/window_border.rs::input_request",
        *mut hmux2::src::window_border::input_request,
        []
    );
    record!(
        "src/window_buffer.rs::input_request",
        *mut hmux2::src::window_buffer::input_request,
        []
    );
    record!(
        "src/window_client.rs::input_request",
        *mut hmux2::src::window_client::input_request,
        []
    );
    record!(
        "src/window_clock.rs::input_request",
        *mut hmux2::src::window_clock::input_request,
        []
    );
    record!(
        "src/window_copy.rs::input_request",
        *mut hmux2::src::window_copy::input_request,
        []
    );
    record!(
        "src/window_customize.rs::input_request",
        *mut hmux2::src::window_customize::input_request,
        []
    );
    record!(
        "src/window_panes.rs::input_request",
        *mut hmux2::src::window_panes::input_request,
        []
    );
    record!(
        "src/window_switch.rs::input_request",
        *mut hmux2::src::window_switch::input_request,
        []
    );
    record!(
        "src/window_tree.rs::input_request",
        *mut hmux2::src::window_tree::input_request,
        []
    );
    record!(
        "src/window_visible.rs::input_request",
        *mut hmux2::src::window_visible::input_request,
        []
    );
    record!(
        "src/input.rs::C2RustUnnamed_30",
        hmux2::src::input::input_request_centry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/input.rs::C2RustUnnamed_31",
        hmux2::src::input::input_request_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/alerts.rs::input_requests",
        hmux2::src::alerts::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/arguments.rs::input_requests",
        hmux2::src::arguments::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cfg.rs::input_requests",
        hmux2::src::cfg::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/client.rs::input_requests",
        hmux2::src::client::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd.rs::input_requests",
        hmux2::src::cmd::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_attach_session.rs::input_requests",
        hmux2::src::cmd_attach_session::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_bind_key.rs::input_requests",
        hmux2::src::cmd_bind_key::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_break_pane.rs::input_requests",
        hmux2::src::cmd_break_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_capture_pane.rs::input_requests",
        hmux2::src::cmd_capture_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_choose_tree.rs::input_requests",
        hmux2::src::cmd_choose_tree::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_command_prompt.rs::input_requests",
        hmux2::src::cmd_command_prompt::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_confirm_before.rs::input_requests",
        hmux2::src::cmd_confirm_before::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_copy_mode.rs::input_requests",
        hmux2::src::cmd_copy_mode::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_detach_client.rs::input_requests",
        hmux2::src::cmd_detach_client::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_display_menu.rs::input_requests",
        hmux2::src::cmd_display_menu::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_display_message.rs::input_requests",
        hmux2::src::cmd_display_message::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_find.rs::input_requests",
        hmux2::src::cmd_find::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_find_window.rs::input_requests",
        hmux2::src::cmd_find_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_if_shell.rs::input_requests",
        hmux2::src::cmd_if_shell::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_join_pane.rs::input_requests",
        hmux2::src::cmd_join_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_kill_pane.rs::input_requests",
        hmux2::src::cmd_kill_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_kill_session.rs::input_requests",
        hmux2::src::cmd_kill_session::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_kill_window.rs::input_requests",
        hmux2::src::cmd_kill_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_buffers.rs::input_requests",
        hmux2::src::cmd_list_buffers::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_clients.rs::input_requests",
        hmux2::src::cmd_list_clients::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_commands.rs::input_requests",
        hmux2::src::cmd_list_commands::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_keys.rs::input_requests",
        hmux2::src::cmd_list_keys::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_panes.rs::input_requests",
        hmux2::src::cmd_list_panes::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_sessions.rs::input_requests",
        hmux2::src::cmd_list_sessions::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_windows.rs::input_requests",
        hmux2::src::cmd_list_windows::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_load_buffer.rs::input_requests",
        hmux2::src::cmd_load_buffer::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_lock_server.rs::input_requests",
        hmux2::src::cmd_lock_server::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_move_window.rs::input_requests",
        hmux2::src::cmd_move_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_new_session.rs::input_requests",
        hmux2::src::cmd_new_session::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_new_window.rs::input_requests",
        hmux2::src::cmd_new_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_parse.rs::input_requests",
        hmux2::src::cmd_parse::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_paste_buffer.rs::input_requests",
        hmux2::src::cmd_paste_buffer::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_pipe_pane.rs::input_requests",
        hmux2::src::cmd_pipe_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_queue.rs::input_requests",
        hmux2::src::cmd_queue::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_refresh_client.rs::input_requests",
        hmux2::src::cmd_refresh_client::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_rename_session.rs::input_requests",
        hmux2::src::cmd_rename_session::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_rename_window.rs::input_requests",
        hmux2::src::cmd_rename_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_resize_pane.rs::input_requests",
        hmux2::src::cmd_resize_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_resize_window.rs::input_requests",
        hmux2::src::cmd_resize_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_respawn_pane.rs::input_requests",
        hmux2::src::cmd_respawn_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_respawn_window.rs::input_requests",
        hmux2::src::cmd_respawn_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_rotate_window.rs::input_requests",
        hmux2::src::cmd_rotate_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_run_shell.rs::input_requests",
        hmux2::src::cmd_run_shell::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_save_buffer.rs::input_requests",
        hmux2::src::cmd_save_buffer::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_layout.rs::input_requests",
        hmux2::src::cmd_select_layout::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_pane.rs::input_requests",
        hmux2::src::cmd_select_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_window.rs::input_requests",
        hmux2::src::cmd_select_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_send_keys.rs::input_requests",
        hmux2::src::cmd_send_keys::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_server_access.rs::input_requests",
        hmux2::src::cmd_server_access::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_set_buffer.rs::input_requests",
        hmux2::src::cmd_set_buffer::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_set_environment.rs::input_requests",
        hmux2::src::cmd_set_environment::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_set_option.rs::input_requests",
        hmux2::src::cmd_set_option::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_show_environment.rs::input_requests",
        hmux2::src::cmd_show_environment::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_show_messages.rs::input_requests",
        hmux2::src::cmd_show_messages::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_show_options.rs::input_requests",
        hmux2::src::cmd_show_options::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_source_file.rs::input_requests",
        hmux2::src::cmd_source_file::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_split_window.rs::input_requests",
        hmux2::src::cmd_split_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_swap_pane.rs::input_requests",
        hmux2::src::cmd_swap_pane::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_swap_window.rs::input_requests",
        hmux2::src::cmd_swap_window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_switch_client.rs::input_requests",
        hmux2::src::cmd_switch_client::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_wait_for.rs::input_requests",
        hmux2::src::cmd_wait_for::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/colour.rs::input_requests",
        hmux2::src::colour::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/control.rs::input_requests",
        hmux2::src::control::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/control_notify.rs::input_requests",
        hmux2::src::control_notify::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/environ.rs::input_requests",
        hmux2::src::environ::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/events.rs::input_requests",
        hmux2::src::events::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/events_payload.rs::input_requests",
        hmux2::src::events_payload::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/file.rs::input_requests",
        hmux2::src::file::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format.rs::input_requests",
        hmux2::src::format::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format_draw.rs::input_requests",
        hmux2::src::format_draw::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/hooks.rs::input_requests",
        hmux2::src::hooks::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/input.rs::input_requests",
        hmux2::src::input::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/input_keys.rs::input_requests",
        hmux2::src::input_keys::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/job.rs::input_requests",
        hmux2::src::job::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/key_bindings.rs::input_requests",
        hmux2::src::key_bindings::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/layout.rs::input_requests",
        hmux2::src::layout::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/layout_custom.rs::input_requests",
        hmux2::src::layout_custom::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/layout_set.rs::input_requests",
        hmux2::src::layout_set::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/menu.rs::input_requests",
        hmux2::src::menu::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/mode_tree.rs::input_requests",
        hmux2::src::mode_tree::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/monitor.rs::input_requests",
        hmux2::src::monitor::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/names.rs::input_requests",
        hmux2::src::names::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/options.rs::input_requests",
        hmux2::src::options::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/popup.rs::input_requests",
        hmux2::src::popup::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/prompt.rs::input_requests",
        hmux2::src::prompt::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/resize.rs::input_requests",
        hmux2::src::resize::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen.rs::input_requests",
        hmux2::src::screen::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen_redraw.rs::input_requests",
        hmux2::src::screen_redraw::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen_write.rs::input_requests",
        hmux2::src::screen_write::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server.rs::input_requests",
        hmux2::src::server::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_acl.rs::input_requests",
        hmux2::src::server_acl::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_client.rs::input_requests",
        hmux2::src::server_client::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_fn.rs::input_requests",
        hmux2::src::server_fn::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/session.rs::input_requests",
        hmux2::src::session::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/sort.rs::input_requests",
        hmux2::src::sort::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/spawn.rs::input_requests",
        hmux2::src::spawn::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/status.rs::input_requests",
        hmux2::src::status::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/style.rs::input_requests",
        hmux2::src::style::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty.rs::input_requests",
        hmux2::src::tty::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_acs.rs::input_requests",
        hmux2::src::tty_acs::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_draw.rs::input_requests",
        hmux2::src::tty_draw::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_features.rs::input_requests",
        hmux2::src::tty_features::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_keys.rs::input_requests",
        hmux2::src::tty_keys::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_term.rs::input_requests",
        hmux2::src::tty_term::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window.rs::input_requests",
        hmux2::src::window::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_border.rs::input_requests",
        hmux2::src::window_border::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_buffer.rs::input_requests",
        hmux2::src::window_buffer::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_client.rs::input_requests",
        hmux2::src::window_client::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_clock.rs::input_requests",
        hmux2::src::window_clock::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_copy.rs::input_requests",
        hmux2::src::window_copy::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_customize.rs::input_requests",
        hmux2::src::window_customize::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_panes.rs::input_requests",
        hmux2::src::window_panes::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_switch.rs::input_requests",
        hmux2::src::window_switch::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_tree.rs::input_requests",
        hmux2::src::window_tree::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_visible.rs::input_requests",
        hmux2::src::window_visible::input_requests,
        [tqh_first, tqh_last]
    );
    record!(
        "src/input.rs::input_state",
        hmux2::src::input::input_state,
        [name, enter, exit, transitions]
    );
    record!(
        "src/input.rs::input_transition",
        hmux2::src::input::input_transition,
        [first, last, handler, state]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-input.txt"));
}
