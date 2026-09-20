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
        "src/screen_redraw.rs::redraw_line",
        hmux2::src::screen_redraw::redraw_line,
        [spans]
    );
    record!(
        "src/alerts.rs::redraw_scene",
        *mut hmux2::src::alerts::redraw_scene,
        []
    );
    record!(
        "src/arguments.rs::redraw_scene",
        *mut hmux2::src::arguments::redraw_scene,
        []
    );
    record!(
        "src/cfg.rs::redraw_scene",
        *mut hmux2::src::cfg::redraw_scene,
        []
    );
    record!(
        "src/client.rs::redraw_scene",
        *mut hmux2::src::client::redraw_scene,
        []
    );
    record!(
        "src/cmd.rs::redraw_scene",
        *mut hmux2::src::cmd::redraw_scene,
        []
    );
    record!(
        "src/cmd_attach_session.rs::redraw_scene",
        *mut hmux2::src::cmd_attach_session::redraw_scene,
        []
    );
    record!(
        "src/cmd_bind_key.rs::redraw_scene",
        *mut hmux2::src::cmd_bind_key::redraw_scene,
        []
    );
    record!(
        "src/cmd_break_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_break_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_capture_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::redraw_scene",
        *mut hmux2::src::cmd_choose_tree::redraw_scene,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::redraw_scene",
        *mut hmux2::src::cmd_command_prompt::redraw_scene,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::redraw_scene",
        *mut hmux2::src::cmd_confirm_before::redraw_scene,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::redraw_scene",
        *mut hmux2::src::cmd_copy_mode::redraw_scene,
        []
    );
    record!(
        "src/cmd_detach_client.rs::redraw_scene",
        *mut hmux2::src::cmd_detach_client::redraw_scene,
        []
    );
    record!(
        "src/cmd_display_menu.rs::redraw_scene",
        *mut hmux2::src::cmd_display_menu::redraw_scene,
        []
    );
    record!(
        "src/cmd_display_message.rs::redraw_scene",
        *mut hmux2::src::cmd_display_message::redraw_scene,
        []
    );
    record!(
        "src/cmd_find.rs::redraw_scene",
        *mut hmux2::src::cmd_find::redraw_scene,
        []
    );
    record!(
        "src/cmd_find_window.rs::redraw_scene",
        *mut hmux2::src::cmd_find_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_if_shell.rs::redraw_scene",
        *mut hmux2::src::cmd_if_shell::redraw_scene,
        []
    );
    record!(
        "src/cmd_join_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_join_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_kill_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_kill_session.rs::redraw_scene",
        *mut hmux2::src::cmd_kill_session::redraw_scene,
        []
    );
    record!(
        "src/cmd_kill_window.rs::redraw_scene",
        *mut hmux2::src::cmd_kill_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::redraw_scene",
        *mut hmux2::src::cmd_list_buffers::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_clients.rs::redraw_scene",
        *mut hmux2::src::cmd_list_clients::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_commands.rs::redraw_scene",
        *mut hmux2::src::cmd_list_commands::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_keys.rs::redraw_scene",
        *mut hmux2::src::cmd_list_keys::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_panes.rs::redraw_scene",
        *mut hmux2::src::cmd_list_panes::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::redraw_scene",
        *mut hmux2::src::cmd_list_sessions::redraw_scene,
        []
    );
    record!(
        "src/cmd_list_windows.rs::redraw_scene",
        *mut hmux2::src::cmd_list_windows::redraw_scene,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::redraw_scene",
        *mut hmux2::src::cmd_load_buffer::redraw_scene,
        []
    );
    record!(
        "src/cmd_lock_server.rs::redraw_scene",
        *mut hmux2::src::cmd_lock_server::redraw_scene,
        []
    );
    record!(
        "src/cmd_move_window.rs::redraw_scene",
        *mut hmux2::src::cmd_move_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_new_session.rs::redraw_scene",
        *mut hmux2::src::cmd_new_session::redraw_scene,
        []
    );
    record!(
        "src/cmd_new_window.rs::redraw_scene",
        *mut hmux2::src::cmd_new_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_parse.rs::redraw_scene",
        *mut hmux2::src::cmd_parse::redraw_scene,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::redraw_scene",
        *mut hmux2::src::cmd_paste_buffer::redraw_scene,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_pipe_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_queue.rs::redraw_scene",
        *mut hmux2::src::cmd_queue::redraw_scene,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::redraw_scene",
        *mut hmux2::src::cmd_refresh_client::redraw_scene,
        []
    );
    record!(
        "src/cmd_rename_session.rs::redraw_scene",
        *mut hmux2::src::cmd_rename_session::redraw_scene,
        []
    );
    record!(
        "src/cmd_rename_window.rs::redraw_scene",
        *mut hmux2::src::cmd_rename_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_resize_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_resize_window.rs::redraw_scene",
        *mut hmux2::src::cmd_resize_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_respawn_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::redraw_scene",
        *mut hmux2::src::cmd_respawn_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::redraw_scene",
        *mut hmux2::src::cmd_rotate_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_run_shell.rs::redraw_scene",
        *mut hmux2::src::cmd_run_shell::redraw_scene,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::redraw_scene",
        *mut hmux2::src::cmd_save_buffer::redraw_scene,
        []
    );
    record!(
        "src/cmd_select_layout.rs::redraw_scene",
        *mut hmux2::src::cmd_select_layout::redraw_scene,
        []
    );
    record!(
        "src/cmd_select_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_select_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_select_window.rs::redraw_scene",
        *mut hmux2::src::cmd_select_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_send_keys.rs::redraw_scene",
        *mut hmux2::src::cmd_send_keys::redraw_scene,
        []
    );
    record!(
        "src/cmd_server_access.rs::redraw_scene",
        *mut hmux2::src::cmd_server_access::redraw_scene,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::redraw_scene",
        *mut hmux2::src::cmd_set_buffer::redraw_scene,
        []
    );
    record!(
        "src/cmd_set_environment.rs::redraw_scene",
        *mut hmux2::src::cmd_set_environment::redraw_scene,
        []
    );
    record!(
        "src/cmd_set_option.rs::redraw_scene",
        *mut hmux2::src::cmd_set_option::redraw_scene,
        []
    );
    record!(
        "src/cmd_show_environment.rs::redraw_scene",
        *mut hmux2::src::cmd_show_environment::redraw_scene,
        []
    );
    record!(
        "src/cmd_show_messages.rs::redraw_scene",
        *mut hmux2::src::cmd_show_messages::redraw_scene,
        []
    );
    record!(
        "src/cmd_show_options.rs::redraw_scene",
        *mut hmux2::src::cmd_show_options::redraw_scene,
        []
    );
    record!(
        "src/cmd_source_file.rs::redraw_scene",
        *mut hmux2::src::cmd_source_file::redraw_scene,
        []
    );
    record!(
        "src/cmd_split_window.rs::redraw_scene",
        *mut hmux2::src::cmd_split_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::redraw_scene",
        *mut hmux2::src::cmd_swap_pane::redraw_scene,
        []
    );
    record!(
        "src/cmd_swap_window.rs::redraw_scene",
        *mut hmux2::src::cmd_swap_window::redraw_scene,
        []
    );
    record!(
        "src/cmd_switch_client.rs::redraw_scene",
        *mut hmux2::src::cmd_switch_client::redraw_scene,
        []
    );
    record!(
        "src/cmd_wait_for.rs::redraw_scene",
        *mut hmux2::src::cmd_wait_for::redraw_scene,
        []
    );
    record!(
        "src/colour.rs::redraw_scene",
        *mut hmux2::src::colour::redraw_scene,
        []
    );
    record!(
        "src/control.rs::redraw_scene",
        *mut hmux2::src::control::redraw_scene,
        []
    );
    record!(
        "src/control_notify.rs::redraw_scene",
        *mut hmux2::src::control_notify::redraw_scene,
        []
    );
    record!(
        "src/environ.rs::redraw_scene",
        *mut hmux2::src::environ::redraw_scene,
        []
    );
    record!(
        "src/events.rs::redraw_scene",
        *mut hmux2::src::events::redraw_scene,
        []
    );
    record!(
        "src/events_payload.rs::redraw_scene",
        *mut hmux2::src::events_payload::redraw_scene,
        []
    );
    record!(
        "src/file.rs::redraw_scene",
        *mut hmux2::src::file::redraw_scene,
        []
    );
    record!(
        "src/format.rs::redraw_scene",
        *mut hmux2::src::format::redraw_scene,
        []
    );
    record!(
        "src/format_draw.rs::redraw_scene",
        *mut hmux2::src::format_draw::redraw_scene,
        []
    );
    record!(
        "src/hooks.rs::redraw_scene",
        *mut hmux2::src::hooks::redraw_scene,
        []
    );
    record!(
        "src/input.rs::redraw_scene",
        *mut hmux2::src::input::redraw_scene,
        []
    );
    record!(
        "src/input_keys.rs::redraw_scene",
        *mut hmux2::src::input_keys::redraw_scene,
        []
    );
    record!(
        "src/job.rs::redraw_scene",
        *mut hmux2::src::job::redraw_scene,
        []
    );
    record!(
        "src/key_bindings.rs::redraw_scene",
        *mut hmux2::src::key_bindings::redraw_scene,
        []
    );
    record!(
        "src/layout.rs::redraw_scene",
        *mut hmux2::src::layout::redraw_scene,
        []
    );
    record!(
        "src/layout_custom.rs::redraw_scene",
        *mut hmux2::src::layout_custom::redraw_scene,
        []
    );
    record!(
        "src/layout_set.rs::redraw_scene",
        *mut hmux2::src::layout_set::redraw_scene,
        []
    );
    record!(
        "src/menu.rs::redraw_scene",
        *mut hmux2::src::menu::redraw_scene,
        []
    );
    record!(
        "src/mode_tree.rs::redraw_scene",
        *mut hmux2::src::mode_tree::redraw_scene,
        []
    );
    record!(
        "src/monitor.rs::redraw_scene",
        *mut hmux2::src::monitor::redraw_scene,
        []
    );
    record!(
        "src/names.rs::redraw_scene",
        *mut hmux2::src::names::redraw_scene,
        []
    );
    record!(
        "src/options.rs::redraw_scene",
        *mut hmux2::src::options::redraw_scene,
        []
    );
    record!(
        "src/popup.rs::redraw_scene",
        *mut hmux2::src::popup::redraw_scene,
        []
    );
    record!(
        "src/prompt.rs::redraw_scene",
        *mut hmux2::src::prompt::redraw_scene,
        []
    );
    record!(
        "src/resize.rs::redraw_scene",
        *mut hmux2::src::resize::redraw_scene,
        []
    );
    record!(
        "src/screen.rs::redraw_scene",
        *mut hmux2::src::screen::redraw_scene,
        []
    );
    record!(
        "src/screen_redraw.rs::redraw_scene",
        hmux2::src::screen_redraw::redraw_scene,
        [c, w, lines, generation, sx, sy, ox, oy]
    );
    record!(
        "src/screen_write.rs::redraw_scene",
        *mut hmux2::src::screen_write::redraw_scene,
        []
    );
    record!(
        "src/server.rs::redraw_scene",
        *mut hmux2::src::server::redraw_scene,
        []
    );
    record!(
        "src/server_acl.rs::redraw_scene",
        *mut hmux2::src::server_acl::redraw_scene,
        []
    );
    record!(
        "src/server_client.rs::redraw_scene",
        *mut hmux2::src::server_client::redraw_scene,
        []
    );
    record!(
        "src/server_fn.rs::redraw_scene",
        *mut hmux2::src::server_fn::redraw_scene,
        []
    );
    record!(
        "src/session.rs::redraw_scene",
        *mut hmux2::src::session::redraw_scene,
        []
    );
    record!(
        "src/sort.rs::redraw_scene",
        *mut hmux2::src::sort::redraw_scene,
        []
    );
    record!(
        "src/spawn.rs::redraw_scene",
        *mut hmux2::src::spawn::redraw_scene,
        []
    );
    record!(
        "src/status.rs::redraw_scene",
        *mut hmux2::src::status::redraw_scene,
        []
    );
    record!(
        "src/style.rs::redraw_scene",
        *mut hmux2::src::style::redraw_scene,
        []
    );
    record!(
        "src/tty.rs::redraw_scene",
        *mut hmux2::src::tty::redraw_scene,
        []
    );
    record!(
        "src/tty_acs.rs::redraw_scene",
        *mut hmux2::src::tty_acs::redraw_scene,
        []
    );
    record!(
        "src/tty_draw.rs::redraw_scene",
        *mut hmux2::src::tty_draw::redraw_scene,
        []
    );
    record!(
        "src/tty_features.rs::redraw_scene",
        *mut hmux2::src::tty_features::redraw_scene,
        []
    );
    record!(
        "src/tty_keys.rs::redraw_scene",
        *mut hmux2::src::tty_keys::redraw_scene,
        []
    );
    record!(
        "src/tty_term.rs::redraw_scene",
        *mut hmux2::src::tty_term::redraw_scene,
        []
    );
    record!(
        "src/window.rs::redraw_scene",
        *mut hmux2::src::window::redraw_scene,
        []
    );
    record!(
        "src/window_border.rs::redraw_scene",
        *mut hmux2::src::window_border::redraw_scene,
        []
    );
    record!(
        "src/window_buffer.rs::redraw_scene",
        *mut hmux2::src::window_buffer::redraw_scene,
        []
    );
    record!(
        "src/window_client.rs::redraw_scene",
        *mut hmux2::src::window_client::redraw_scene,
        []
    );
    record!(
        "src/window_clock.rs::redraw_scene",
        *mut hmux2::src::window_clock::redraw_scene,
        []
    );
    record!(
        "src/window_copy.rs::redraw_scene",
        *mut hmux2::src::window_copy::redraw_scene,
        []
    );
    record!(
        "src/window_customize.rs::redraw_scene",
        *mut hmux2::src::window_customize::redraw_scene,
        []
    );
    record!(
        "src/window_panes.rs::redraw_scene",
        *mut hmux2::src::window_panes::redraw_scene,
        []
    );
    record!(
        "src/window_switch.rs::redraw_scene",
        *mut hmux2::src::window_switch::redraw_scene,
        []
    );
    record!(
        "src/window_tree.rs::redraw_scene",
        *mut hmux2::src::window_tree::redraw_scene,
        []
    );
    record!(
        "src/window_visible.rs::redraw_scene",
        *mut hmux2::src::window_visible::redraw_scene,
        []
    );
    record!(
        "src/screen_redraw.rs::redraw_span",
        hmux2::src::screen_redraw::redraw_span,
        [x, width, data, entry]
    );
    record!(
        "src/window_border.rs::redraw_span",
        *mut hmux2::src::window_border::redraw_span,
        []
    );
    record!(
        "src/screen_redraw.rs::redraw_span_data",
        hmux2::src::screen_redraw::redraw_span_data,
        [type_0, c2rust_unnamed]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_35",
        hmux2::src::screen_redraw::redraw_span_data_c2rust_unnamed,
        [p, b, st, sb, m]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_39",
        hmux2::src::screen_redraw::redraw_span_data_c2rust_unnamed_b,
        [
            top_wp,
            bottom_wp,
            left_wp,
            right_wp,
            style_wp,
            cell_type,
            cell_mask,
            top_lines,
            bottom_lines,
            left_lines,
            right_lines,
            flags
        ]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_36",
        hmux2::src::screen_redraw::redraw_span_data_c2rust_unnamed_m,
        [md, px, py]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_40",
        hmux2::src::screen_redraw::redraw_span_data_c2rust_unnamed_p,
        [wp, px, py]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_37",
        hmux2::src::screen_redraw::redraw_span_data_c2rust_unnamed_sb,
        [wp, y, height, flags]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_38",
        hmux2::src::screen_redraw::redraw_span_data_c2rust_unnamed_st,
        [wp, offset, cell_type]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_34",
        hmux2::src::screen_redraw::redraw_span_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/screen_redraw.rs::redraw_span_type",
        hmux2::src::screen_redraw::redraw_span_type,
        []
    );
    record!(
        "src/screen_redraw.rs::redraw_spans",
        hmux2::src::screen_redraw::redraw_spans,
        [tqh_first, tqh_last]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-redraw.txt"));
}
