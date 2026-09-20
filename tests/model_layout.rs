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
        "src/alerts.rs::layout_cell",
        hmux2::src::alerts::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/arguments.rs::layout_cell",
        hmux2::src::arguments::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cfg.rs::layout_cell",
        hmux2::src::cfg::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/client.rs::layout_cell",
        hmux2::src::client::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd.rs::layout_cell",
        hmux2::src::cmd::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_attach_session.rs::layout_cell",
        hmux2::src::cmd_attach_session::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_bind_key.rs::layout_cell",
        hmux2::src::cmd_bind_key::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_break_pane.rs::layout_cell",
        hmux2::src::cmd_break_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_capture_pane.rs::layout_cell",
        hmux2::src::cmd_capture_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_choose_tree.rs::layout_cell",
        hmux2::src::cmd_choose_tree::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_command_prompt.rs::layout_cell",
        hmux2::src::cmd_command_prompt::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_confirm_before.rs::layout_cell",
        hmux2::src::cmd_confirm_before::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_copy_mode.rs::layout_cell",
        hmux2::src::cmd_copy_mode::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_detach_client.rs::layout_cell",
        hmux2::src::cmd_detach_client::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_display_menu.rs::layout_cell",
        hmux2::src::cmd_display_menu::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_display_message.rs::layout_cell",
        hmux2::src::cmd_display_message::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_find.rs::layout_cell",
        hmux2::src::cmd_find::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_find_window.rs::layout_cell",
        hmux2::src::cmd_find_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_if_shell.rs::layout_cell",
        hmux2::src::cmd_if_shell::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_join_pane.rs::layout_cell",
        hmux2::src::cmd_join_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_kill_pane.rs::layout_cell",
        hmux2::src::cmd_kill_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_kill_session.rs::layout_cell",
        hmux2::src::cmd_kill_session::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_kill_window.rs::layout_cell",
        hmux2::src::cmd_kill_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_buffers.rs::layout_cell",
        hmux2::src::cmd_list_buffers::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_clients.rs::layout_cell",
        hmux2::src::cmd_list_clients::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_commands.rs::layout_cell",
        hmux2::src::cmd_list_commands::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_keys.rs::layout_cell",
        hmux2::src::cmd_list_keys::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_panes.rs::layout_cell",
        hmux2::src::cmd_list_panes::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_sessions.rs::layout_cell",
        hmux2::src::cmd_list_sessions::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_list_windows.rs::layout_cell",
        hmux2::src::cmd_list_windows::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_load_buffer.rs::layout_cell",
        hmux2::src::cmd_load_buffer::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_lock_server.rs::layout_cell",
        hmux2::src::cmd_lock_server::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_move_window.rs::layout_cell",
        hmux2::src::cmd_move_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_new_session.rs::layout_cell",
        hmux2::src::cmd_new_session::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_new_window.rs::layout_cell",
        hmux2::src::cmd_new_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_parse.rs::layout_cell",
        hmux2::src::cmd_parse::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_paste_buffer.rs::layout_cell",
        hmux2::src::cmd_paste_buffer::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_pipe_pane.rs::layout_cell",
        hmux2::src::cmd_pipe_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_queue.rs::layout_cell",
        hmux2::src::cmd_queue::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_refresh_client.rs::layout_cell",
        hmux2::src::cmd_refresh_client::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_rename_session.rs::layout_cell",
        hmux2::src::cmd_rename_session::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_rename_window.rs::layout_cell",
        hmux2::src::cmd_rename_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_resize_pane.rs::layout_cell",
        hmux2::src::cmd_resize_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_resize_window.rs::layout_cell",
        hmux2::src::cmd_resize_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_respawn_pane.rs::layout_cell",
        hmux2::src::cmd_respawn_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_respawn_window.rs::layout_cell",
        hmux2::src::cmd_respawn_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_rotate_window.rs::layout_cell",
        hmux2::src::cmd_rotate_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_run_shell.rs::layout_cell",
        hmux2::src::cmd_run_shell::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_save_buffer.rs::layout_cell",
        hmux2::src::cmd_save_buffer::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_select_layout.rs::layout_cell",
        hmux2::src::cmd_select_layout::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_select_pane.rs::layout_cell",
        hmux2::src::cmd_select_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_select_window.rs::layout_cell",
        hmux2::src::cmd_select_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_send_keys.rs::layout_cell",
        hmux2::src::cmd_send_keys::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_server_access.rs::layout_cell",
        hmux2::src::cmd_server_access::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_set_buffer.rs::layout_cell",
        hmux2::src::cmd_set_buffer::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_set_environment.rs::layout_cell",
        hmux2::src::cmd_set_environment::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_set_option.rs::layout_cell",
        hmux2::src::cmd_set_option::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_show_environment.rs::layout_cell",
        hmux2::src::cmd_show_environment::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_show_messages.rs::layout_cell",
        hmux2::src::cmd_show_messages::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_show_options.rs::layout_cell",
        hmux2::src::cmd_show_options::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_source_file.rs::layout_cell",
        hmux2::src::cmd_source_file::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_split_window.rs::layout_cell",
        hmux2::src::cmd_split_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_swap_pane.rs::layout_cell",
        hmux2::src::cmd_swap_pane::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_swap_window.rs::layout_cell",
        hmux2::src::cmd_swap_window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_switch_client.rs::layout_cell",
        hmux2::src::cmd_switch_client::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/cmd_wait_for.rs::layout_cell",
        hmux2::src::cmd_wait_for::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/colour.rs::layout_cell",
        hmux2::src::colour::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/control.rs::layout_cell",
        hmux2::src::control::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/control_notify.rs::layout_cell",
        hmux2::src::control_notify::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/environ.rs::layout_cell",
        hmux2::src::environ::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/events.rs::layout_cell",
        hmux2::src::events::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/events_payload.rs::layout_cell",
        hmux2::src::events_payload::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/file.rs::layout_cell",
        hmux2::src::file::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/format.rs::layout_cell",
        hmux2::src::format::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/format_draw.rs::layout_cell",
        hmux2::src::format_draw::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/hooks.rs::layout_cell",
        hmux2::src::hooks::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/input.rs::layout_cell",
        hmux2::src::input::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/input_keys.rs::layout_cell",
        hmux2::src::input_keys::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/job.rs::layout_cell",
        hmux2::src::job::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/key_bindings.rs::layout_cell",
        hmux2::src::key_bindings::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/layout.rs::layout_cell",
        hmux2::src::layout::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/layout_custom.rs::layout_cell",
        hmux2::src::layout_custom::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/layout_set.rs::layout_cell",
        hmux2::src::layout_set::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/menu.rs::layout_cell",
        hmux2::src::menu::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/mode_tree.rs::layout_cell",
        hmux2::src::mode_tree::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/monitor.rs::layout_cell",
        hmux2::src::monitor::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/names.rs::layout_cell",
        hmux2::src::names::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/options.rs::layout_cell",
        hmux2::src::options::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/popup.rs::layout_cell",
        hmux2::src::popup::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/prompt.rs::layout_cell",
        hmux2::src::prompt::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/resize.rs::layout_cell",
        hmux2::src::resize::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/screen.rs::layout_cell",
        hmux2::src::screen::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/screen_redraw.rs::layout_cell",
        hmux2::src::screen_redraw::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/screen_write.rs::layout_cell",
        hmux2::src::screen_write::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/server.rs::layout_cell",
        hmux2::src::server::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/server_acl.rs::layout_cell",
        hmux2::src::server_acl::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/server_client.rs::layout_cell",
        hmux2::src::server_client::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/server_fn.rs::layout_cell",
        hmux2::src::server_fn::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/session.rs::layout_cell",
        hmux2::src::session::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/sort.rs::layout_cell",
        hmux2::src::sort::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/spawn.rs::layout_cell",
        hmux2::src::spawn::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/status.rs::layout_cell",
        hmux2::src::status::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/style.rs::layout_cell",
        hmux2::src::style::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/tty.rs::layout_cell",
        hmux2::src::tty::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/tty_acs.rs::layout_cell",
        hmux2::src::tty_acs::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/tty_draw.rs::layout_cell",
        hmux2::src::tty_draw::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/tty_features.rs::layout_cell",
        hmux2::src::tty_features::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/tty_keys.rs::layout_cell",
        hmux2::src::tty_keys::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/tty_term.rs::layout_cell",
        hmux2::src::tty_term::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window.rs::layout_cell",
        hmux2::src::window::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_border.rs::layout_cell",
        hmux2::src::window_border::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_buffer.rs::layout_cell",
        hmux2::src::window_buffer::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_client.rs::layout_cell",
        hmux2::src::window_client::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_clock.rs::layout_cell",
        hmux2::src::window_clock::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_copy.rs::layout_cell",
        hmux2::src::window_copy::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_customize.rs::layout_cell",
        hmux2::src::window_customize::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_panes.rs::layout_cell",
        hmux2::src::window_panes::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_switch.rs::layout_cell",
        hmux2::src::window_switch::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_tree.rs::layout_cell",
        hmux2::src::window_tree::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/window_visible.rs::layout_cell",
        hmux2::src::window_visible::layout_cell,
        [type_0, flags, parent, g, fg, wp, cells, entry]
    );
    record!(
        "src/alerts.rs::C2RustUnnamed_22",
        hmux2::src::alerts::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/arguments.rs::C2RustUnnamed_28",
        hmux2::src::arguments::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cfg.rs::C2RustUnnamed_22",
        hmux2::src::cfg::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/client.rs::C2RustUnnamed_35",
        hmux2::src::client::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd.rs::C2RustUnnamed_22",
        hmux2::src::cmd::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_attach_session.rs::C2RustUnnamed_22",
        hmux2::src::cmd_attach_session::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_bind_key.rs::C2RustUnnamed_22",
        hmux2::src::cmd_bind_key::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_break_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_break_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_capture_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_capture_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_choose_tree.rs::C2RustUnnamed_22",
        hmux2::src::cmd_choose_tree::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_command_prompt.rs::C2RustUnnamed_22",
        hmux2::src::cmd_command_prompt::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_confirm_before.rs::C2RustUnnamed_22",
        hmux2::src::cmd_confirm_before::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_copy_mode.rs::C2RustUnnamed_22",
        hmux2::src::cmd_copy_mode::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_detach_client.rs::C2RustUnnamed_22",
        hmux2::src::cmd_detach_client::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_display_menu.rs::C2RustUnnamed_22",
        hmux2::src::cmd_display_menu::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_display_message.rs::C2RustUnnamed_22",
        hmux2::src::cmd_display_message::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_find.rs::C2RustUnnamed_22",
        hmux2::src::cmd_find::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_find_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_find_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_if_shell.rs::C2RustUnnamed_22",
        hmux2::src::cmd_if_shell::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_join_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_join_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_kill_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_kill_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_kill_session.rs::C2RustUnnamed_22",
        hmux2::src::cmd_kill_session::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_kill_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_kill_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_buffers.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_buffers::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_clients.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_clients::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_commands.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_commands::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_keys.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_keys::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_panes.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_panes::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_sessions.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_sessions::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_list_windows.rs::C2RustUnnamed_22",
        hmux2::src::cmd_list_windows::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_load_buffer.rs::C2RustUnnamed_22",
        hmux2::src::cmd_load_buffer::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_lock_server.rs::C2RustUnnamed_22",
        hmux2::src::cmd_lock_server::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_move_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_move_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_22",
        hmux2::src::cmd_new_session::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_new_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_new_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_parse.rs::C2RustUnnamed_23",
        hmux2::src::cmd_parse::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_paste_buffer.rs::C2RustUnnamed_22",
        hmux2::src::cmd_paste_buffer::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_pipe_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_pipe_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_queue.rs::C2RustUnnamed_22",
        hmux2::src::cmd_queue::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_refresh_client.rs::C2RustUnnamed_22",
        hmux2::src::cmd_refresh_client::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_rename_session.rs::C2RustUnnamed_22",
        hmux2::src::cmd_rename_session::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_rename_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_rename_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_resize_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_resize_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_resize_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_resize_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_respawn_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_respawn_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_respawn_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_respawn_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_rotate_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_rotate_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_run_shell.rs::C2RustUnnamed_22",
        hmux2::src::cmd_run_shell::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_save_buffer.rs::C2RustUnnamed_22",
        hmux2::src::cmd_save_buffer::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_select_layout.rs::C2RustUnnamed_22",
        hmux2::src::cmd_select_layout::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_select_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_select_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_select_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_select_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_send_keys.rs::C2RustUnnamed_22",
        hmux2::src::cmd_send_keys::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_server_access.rs::C2RustUnnamed_22",
        hmux2::src::cmd_server_access::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_set_buffer.rs::C2RustUnnamed_22",
        hmux2::src::cmd_set_buffer::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_set_environment.rs::C2RustUnnamed_22",
        hmux2::src::cmd_set_environment::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_set_option.rs::C2RustUnnamed_22",
        hmux2::src::cmd_set_option::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_show_environment.rs::C2RustUnnamed_22",
        hmux2::src::cmd_show_environment::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_show_messages.rs::C2RustUnnamed_22",
        hmux2::src::cmd_show_messages::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_show_options.rs::C2RustUnnamed_22",
        hmux2::src::cmd_show_options::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_source_file.rs::C2RustUnnamed_23",
        hmux2::src::cmd_source_file::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_split_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_split_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_swap_pane.rs::C2RustUnnamed_22",
        hmux2::src::cmd_swap_pane::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_swap_window.rs::C2RustUnnamed_22",
        hmux2::src::cmd_swap_window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_switch_client.rs::C2RustUnnamed_22",
        hmux2::src::cmd_switch_client::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_wait_for.rs::C2RustUnnamed_22",
        hmux2::src::cmd_wait_for::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/colour.rs::C2RustUnnamed_23",
        hmux2::src::colour::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/control.rs::C2RustUnnamed_22",
        hmux2::src::control::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/control_notify.rs::C2RustUnnamed_22",
        hmux2::src::control_notify::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/environ.rs::C2RustUnnamed_23",
        hmux2::src::environ::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/events.rs::C2RustUnnamed_22",
        hmux2::src::events::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/events_payload.rs::C2RustUnnamed_22",
        hmux2::src::events_payload::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/file.rs::C2RustUnnamed_23",
        hmux2::src::file::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/format.rs::C2RustUnnamed_23",
        hmux2::src::format::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/format_draw.rs::C2RustUnnamed_22",
        hmux2::src::format_draw::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/hooks.rs::C2RustUnnamed_22",
        hmux2::src::hooks::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/input.rs::C2RustUnnamed_22",
        hmux2::src::input::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/input_keys.rs::C2RustUnnamed_22",
        hmux2::src::input_keys::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/job.rs::C2RustUnnamed_23",
        hmux2::src::job::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/key_bindings.rs::C2RustUnnamed_22",
        hmux2::src::key_bindings::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/layout.rs::C2RustUnnamed_22",
        hmux2::src::layout::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/layout_custom.rs::C2RustUnnamed_23",
        hmux2::src::layout_custom::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/layout_set.rs::C2RustUnnamed_22",
        hmux2::src::layout_set::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/menu.rs::C2RustUnnamed_30",
        hmux2::src::menu::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/mode_tree.rs::C2RustUnnamed_22",
        hmux2::src::mode_tree::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/monitor.rs::C2RustUnnamed_22",
        hmux2::src::monitor::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/names.rs::C2RustUnnamed_23",
        hmux2::src::names::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/options.rs::C2RustUnnamed_25",
        hmux2::src::options::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/popup.rs::C2RustUnnamed_22",
        hmux2::src::popup::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/prompt.rs::C2RustUnnamed_22",
        hmux2::src::prompt::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/resize.rs::C2RustUnnamed_22",
        hmux2::src::resize::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/screen.rs::C2RustUnnamed_23",
        hmux2::src::screen::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/screen_redraw.rs::C2RustUnnamed_22",
        hmux2::src::screen_redraw::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/screen_write.rs::C2RustUnnamed_25",
        hmux2::src::screen_write::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/server.rs::C2RustUnnamed_23",
        hmux2::src::server::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/server_acl.rs::C2RustUnnamed_22",
        hmux2::src::server_acl::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/server_client.rs::C2RustUnnamed_23",
        hmux2::src::server_client::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/server_fn.rs::C2RustUnnamed_22",
        hmux2::src::server_fn::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/session.rs::C2RustUnnamed_22",
        hmux2::src::session::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/sort.rs::C2RustUnnamed_22",
        hmux2::src::sort::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/spawn.rs::C2RustUnnamed_22",
        hmux2::src::spawn::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/status.rs::C2RustUnnamed_22",
        hmux2::src::status::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/style.rs::C2RustUnnamed_22",
        hmux2::src::style::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/tty.rs::C2RustUnnamed_22",
        hmux2::src::tty::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/tty_acs.rs::C2RustUnnamed_22",
        hmux2::src::tty_acs::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/tty_draw.rs::C2RustUnnamed_22",
        hmux2::src::tty_draw::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/tty_features.rs::C2RustUnnamed_22",
        hmux2::src::tty_features::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/tty_keys.rs::C2RustUnnamed_23",
        hmux2::src::tty_keys::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/tty_term.rs::C2RustUnnamed_22",
        hmux2::src::tty_term::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window.rs::C2RustUnnamed_23",
        hmux2::src::window::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_border.rs::C2RustUnnamed_22",
        hmux2::src::window_border::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_buffer.rs::C2RustUnnamed_22",
        hmux2::src::window_buffer::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_client.rs::C2RustUnnamed_22",
        hmux2::src::window_client::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_clock.rs::C2RustUnnamed_22",
        hmux2::src::window_clock::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_copy.rs::C2RustUnnamed_22",
        hmux2::src::window_copy::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_customize.rs::C2RustUnnamed_22",
        hmux2::src::window_customize::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_panes.rs::C2RustUnnamed_22",
        hmux2::src::window_panes::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_switch.rs::C2RustUnnamed_22",
        hmux2::src::window_switch::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_22",
        hmux2::src::window_tree::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/window_visible.rs::C2RustUnnamed_22",
        hmux2::src::window_visible::layout_cell_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/alerts.rs::layout_cells",
        hmux2::src::alerts::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/arguments.rs::layout_cells",
        hmux2::src::arguments::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cfg.rs::layout_cells",
        hmux2::src::cfg::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/client.rs::layout_cells",
        hmux2::src::client::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd.rs::layout_cells",
        hmux2::src::cmd::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_attach_session.rs::layout_cells",
        hmux2::src::cmd_attach_session::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_bind_key.rs::layout_cells",
        hmux2::src::cmd_bind_key::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_break_pane.rs::layout_cells",
        hmux2::src::cmd_break_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_capture_pane.rs::layout_cells",
        hmux2::src::cmd_capture_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_choose_tree.rs::layout_cells",
        hmux2::src::cmd_choose_tree::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_command_prompt.rs::layout_cells",
        hmux2::src::cmd_command_prompt::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_confirm_before.rs::layout_cells",
        hmux2::src::cmd_confirm_before::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_copy_mode.rs::layout_cells",
        hmux2::src::cmd_copy_mode::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_detach_client.rs::layout_cells",
        hmux2::src::cmd_detach_client::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_display_menu.rs::layout_cells",
        hmux2::src::cmd_display_menu::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_display_message.rs::layout_cells",
        hmux2::src::cmd_display_message::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_find.rs::layout_cells",
        hmux2::src::cmd_find::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_find_window.rs::layout_cells",
        hmux2::src::cmd_find_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_if_shell.rs::layout_cells",
        hmux2::src::cmd_if_shell::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_join_pane.rs::layout_cells",
        hmux2::src::cmd_join_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_kill_pane.rs::layout_cells",
        hmux2::src::cmd_kill_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_kill_session.rs::layout_cells",
        hmux2::src::cmd_kill_session::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_kill_window.rs::layout_cells",
        hmux2::src::cmd_kill_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_buffers.rs::layout_cells",
        hmux2::src::cmd_list_buffers::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_clients.rs::layout_cells",
        hmux2::src::cmd_list_clients::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_commands.rs::layout_cells",
        hmux2::src::cmd_list_commands::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_keys.rs::layout_cells",
        hmux2::src::cmd_list_keys::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_panes.rs::layout_cells",
        hmux2::src::cmd_list_panes::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_sessions.rs::layout_cells",
        hmux2::src::cmd_list_sessions::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_list_windows.rs::layout_cells",
        hmux2::src::cmd_list_windows::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_load_buffer.rs::layout_cells",
        hmux2::src::cmd_load_buffer::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_lock_server.rs::layout_cells",
        hmux2::src::cmd_lock_server::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_move_window.rs::layout_cells",
        hmux2::src::cmd_move_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_new_session.rs::layout_cells",
        hmux2::src::cmd_new_session::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_new_window.rs::layout_cells",
        hmux2::src::cmd_new_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_parse.rs::layout_cells",
        hmux2::src::cmd_parse::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_paste_buffer.rs::layout_cells",
        hmux2::src::cmd_paste_buffer::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_pipe_pane.rs::layout_cells",
        hmux2::src::cmd_pipe_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_queue.rs::layout_cells",
        hmux2::src::cmd_queue::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_refresh_client.rs::layout_cells",
        hmux2::src::cmd_refresh_client::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_rename_session.rs::layout_cells",
        hmux2::src::cmd_rename_session::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_rename_window.rs::layout_cells",
        hmux2::src::cmd_rename_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_resize_pane.rs::layout_cells",
        hmux2::src::cmd_resize_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_resize_window.rs::layout_cells",
        hmux2::src::cmd_resize_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_respawn_pane.rs::layout_cells",
        hmux2::src::cmd_respawn_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_respawn_window.rs::layout_cells",
        hmux2::src::cmd_respawn_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_rotate_window.rs::layout_cells",
        hmux2::src::cmd_rotate_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_run_shell.rs::layout_cells",
        hmux2::src::cmd_run_shell::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_save_buffer.rs::layout_cells",
        hmux2::src::cmd_save_buffer::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_layout.rs::layout_cells",
        hmux2::src::cmd_select_layout::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_pane.rs::layout_cells",
        hmux2::src::cmd_select_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_window.rs::layout_cells",
        hmux2::src::cmd_select_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_send_keys.rs::layout_cells",
        hmux2::src::cmd_send_keys::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_server_access.rs::layout_cells",
        hmux2::src::cmd_server_access::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_set_buffer.rs::layout_cells",
        hmux2::src::cmd_set_buffer::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_set_environment.rs::layout_cells",
        hmux2::src::cmd_set_environment::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_set_option.rs::layout_cells",
        hmux2::src::cmd_set_option::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_show_environment.rs::layout_cells",
        hmux2::src::cmd_show_environment::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_show_messages.rs::layout_cells",
        hmux2::src::cmd_show_messages::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_show_options.rs::layout_cells",
        hmux2::src::cmd_show_options::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_source_file.rs::layout_cells",
        hmux2::src::cmd_source_file::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_split_window.rs::layout_cells",
        hmux2::src::cmd_split_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_swap_pane.rs::layout_cells",
        hmux2::src::cmd_swap_pane::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_swap_window.rs::layout_cells",
        hmux2::src::cmd_swap_window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_switch_client.rs::layout_cells",
        hmux2::src::cmd_switch_client::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_wait_for.rs::layout_cells",
        hmux2::src::cmd_wait_for::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/colour.rs::layout_cells",
        hmux2::src::colour::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/control.rs::layout_cells",
        hmux2::src::control::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/control_notify.rs::layout_cells",
        hmux2::src::control_notify::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/environ.rs::layout_cells",
        hmux2::src::environ::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/events.rs::layout_cells",
        hmux2::src::events::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/events_payload.rs::layout_cells",
        hmux2::src::events_payload::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/file.rs::layout_cells",
        hmux2::src::file::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format.rs::layout_cells",
        hmux2::src::format::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format_draw.rs::layout_cells",
        hmux2::src::format_draw::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/hooks.rs::layout_cells",
        hmux2::src::hooks::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/input.rs::layout_cells",
        hmux2::src::input::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/input_keys.rs::layout_cells",
        hmux2::src::input_keys::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/job.rs::layout_cells",
        hmux2::src::job::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/key_bindings.rs::layout_cells",
        hmux2::src::key_bindings::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/layout.rs::layout_cells",
        hmux2::src::layout::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/layout_custom.rs::layout_cells",
        hmux2::src::layout_custom::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/layout_set.rs::layout_cells",
        hmux2::src::layout_set::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/menu.rs::layout_cells",
        hmux2::src::menu::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/mode_tree.rs::layout_cells",
        hmux2::src::mode_tree::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/monitor.rs::layout_cells",
        hmux2::src::monitor::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/names.rs::layout_cells",
        hmux2::src::names::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/options.rs::layout_cells",
        hmux2::src::options::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/popup.rs::layout_cells",
        hmux2::src::popup::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/prompt.rs::layout_cells",
        hmux2::src::prompt::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/resize.rs::layout_cells",
        hmux2::src::resize::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen.rs::layout_cells",
        hmux2::src::screen::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen_redraw.rs::layout_cells",
        hmux2::src::screen_redraw::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen_write.rs::layout_cells",
        hmux2::src::screen_write::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server.rs::layout_cells",
        hmux2::src::server::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_acl.rs::layout_cells",
        hmux2::src::server_acl::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_client.rs::layout_cells",
        hmux2::src::server_client::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_fn.rs::layout_cells",
        hmux2::src::server_fn::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/session.rs::layout_cells",
        hmux2::src::session::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/sort.rs::layout_cells",
        hmux2::src::sort::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/spawn.rs::layout_cells",
        hmux2::src::spawn::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/status.rs::layout_cells",
        hmux2::src::status::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/style.rs::layout_cells",
        hmux2::src::style::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty.rs::layout_cells",
        hmux2::src::tty::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_acs.rs::layout_cells",
        hmux2::src::tty_acs::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_draw.rs::layout_cells",
        hmux2::src::tty_draw::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_features.rs::layout_cells",
        hmux2::src::tty_features::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_keys.rs::layout_cells",
        hmux2::src::tty_keys::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty_term.rs::layout_cells",
        hmux2::src::tty_term::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window.rs::layout_cells",
        hmux2::src::window::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_border.rs::layout_cells",
        hmux2::src::window_border::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_buffer.rs::layout_cells",
        hmux2::src::window_buffer::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_client.rs::layout_cells",
        hmux2::src::window_client::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_clock.rs::layout_cells",
        hmux2::src::window_clock::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_copy.rs::layout_cells",
        hmux2::src::window_copy::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_customize.rs::layout_cells",
        hmux2::src::window_customize::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_panes.rs::layout_cells",
        hmux2::src::window_panes::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_switch.rs::layout_cells",
        hmux2::src::window_switch::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_tree.rs::layout_cells",
        hmux2::src::window_tree::layout_cells,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_visible.rs::layout_cells",
        hmux2::src::window_visible::layout_cells,
        [tqh_first, tqh_last]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-layout.txt"));
}
