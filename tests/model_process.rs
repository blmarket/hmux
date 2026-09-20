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
        "src/alerts.rs::tmuxpeer",
        *mut hmux2::src::alerts::tmuxpeer,
        []
    );
    record!(
        "src/arguments.rs::tmuxpeer",
        *mut hmux2::src::arguments::tmuxpeer,
        []
    );
    record!("src/cfg.rs::tmuxpeer", *mut hmux2::src::cfg::tmuxpeer, []);
    record!(
        "src/client.rs::tmuxpeer",
        *mut hmux2::src::client::tmuxpeer,
        []
    );
    record!("src/cmd.rs::tmuxpeer", *mut hmux2::src::cmd::tmuxpeer, []);
    record!(
        "src/cmd_attach_session.rs::tmuxpeer",
        *mut hmux2::src::cmd_attach_session::tmuxpeer,
        []
    );
    record!(
        "src/cmd_bind_key.rs::tmuxpeer",
        *mut hmux2::src::cmd_bind_key::tmuxpeer,
        []
    );
    record!(
        "src/cmd_break_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_break_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_capture_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::tmuxpeer",
        *mut hmux2::src::cmd_choose_tree::tmuxpeer,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::tmuxpeer",
        *mut hmux2::src::cmd_command_prompt::tmuxpeer,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::tmuxpeer",
        *mut hmux2::src::cmd_confirm_before::tmuxpeer,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::tmuxpeer",
        *mut hmux2::src::cmd_copy_mode::tmuxpeer,
        []
    );
    record!(
        "src/cmd_detach_client.rs::tmuxpeer",
        *mut hmux2::src::cmd_detach_client::tmuxpeer,
        []
    );
    record!(
        "src/cmd_display_menu.rs::tmuxpeer",
        *mut hmux2::src::cmd_display_menu::tmuxpeer,
        []
    );
    record!(
        "src/cmd_display_message.rs::tmuxpeer",
        *mut hmux2::src::cmd_display_message::tmuxpeer,
        []
    );
    record!(
        "src/cmd_find.rs::tmuxpeer",
        *mut hmux2::src::cmd_find::tmuxpeer,
        []
    );
    record!(
        "src/cmd_find_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_find_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_if_shell.rs::tmuxpeer",
        *mut hmux2::src::cmd_if_shell::tmuxpeer,
        []
    );
    record!(
        "src/cmd_join_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_join_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_kill_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_kill_session.rs::tmuxpeer",
        *mut hmux2::src::cmd_kill_session::tmuxpeer,
        []
    );
    record!(
        "src/cmd_kill_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_kill_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_buffers::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_clients.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_clients::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_commands.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_commands::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_keys.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_keys::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_panes.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_panes::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_sessions::tmuxpeer,
        []
    );
    record!(
        "src/cmd_list_windows.rs::tmuxpeer",
        *mut hmux2::src::cmd_list_windows::tmuxpeer,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::tmuxpeer",
        *mut hmux2::src::cmd_load_buffer::tmuxpeer,
        []
    );
    record!(
        "src/cmd_lock_server.rs::tmuxpeer",
        *mut hmux2::src::cmd_lock_server::tmuxpeer,
        []
    );
    record!(
        "src/cmd_move_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_move_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_new_session.rs::tmuxpeer",
        *mut hmux2::src::cmd_new_session::tmuxpeer,
        []
    );
    record!(
        "src/cmd_new_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_new_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_parse.rs::tmuxpeer",
        *mut hmux2::src::cmd_parse::tmuxpeer,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::tmuxpeer",
        *mut hmux2::src::cmd_paste_buffer::tmuxpeer,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_pipe_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_queue.rs::tmuxpeer",
        *mut hmux2::src::cmd_queue::tmuxpeer,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::tmuxpeer",
        *mut hmux2::src::cmd_refresh_client::tmuxpeer,
        []
    );
    record!(
        "src/cmd_rename_session.rs::tmuxpeer",
        *mut hmux2::src::cmd_rename_session::tmuxpeer,
        []
    );
    record!(
        "src/cmd_rename_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_rename_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_resize_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_resize_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_resize_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_respawn_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_respawn_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_rotate_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_run_shell.rs::tmuxpeer",
        *mut hmux2::src::cmd_run_shell::tmuxpeer,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::tmuxpeer",
        *mut hmux2::src::cmd_save_buffer::tmuxpeer,
        []
    );
    record!(
        "src/cmd_select_layout.rs::tmuxpeer",
        *mut hmux2::src::cmd_select_layout::tmuxpeer,
        []
    );
    record!(
        "src/cmd_select_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_select_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_select_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_select_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_send_keys.rs::tmuxpeer",
        *mut hmux2::src::cmd_send_keys::tmuxpeer,
        []
    );
    record!(
        "src/cmd_server_access.rs::tmuxpeer",
        *mut hmux2::src::cmd_server_access::tmuxpeer,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::tmuxpeer",
        *mut hmux2::src::cmd_set_buffer::tmuxpeer,
        []
    );
    record!(
        "src/cmd_set_environment.rs::tmuxpeer",
        *mut hmux2::src::cmd_set_environment::tmuxpeer,
        []
    );
    record!(
        "src/cmd_set_option.rs::tmuxpeer",
        *mut hmux2::src::cmd_set_option::tmuxpeer,
        []
    );
    record!(
        "src/cmd_show_environment.rs::tmuxpeer",
        *mut hmux2::src::cmd_show_environment::tmuxpeer,
        []
    );
    record!(
        "src/cmd_show_messages.rs::tmuxpeer",
        *mut hmux2::src::cmd_show_messages::tmuxpeer,
        []
    );
    record!(
        "src/cmd_show_options.rs::tmuxpeer",
        *mut hmux2::src::cmd_show_options::tmuxpeer,
        []
    );
    record!(
        "src/cmd_source_file.rs::tmuxpeer",
        *mut hmux2::src::cmd_source_file::tmuxpeer,
        []
    );
    record!(
        "src/cmd_split_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_split_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::tmuxpeer",
        *mut hmux2::src::cmd_swap_pane::tmuxpeer,
        []
    );
    record!(
        "src/cmd_swap_window.rs::tmuxpeer",
        *mut hmux2::src::cmd_swap_window::tmuxpeer,
        []
    );
    record!(
        "src/cmd_switch_client.rs::tmuxpeer",
        *mut hmux2::src::cmd_switch_client::tmuxpeer,
        []
    );
    record!(
        "src/cmd_wait_for.rs::tmuxpeer",
        *mut hmux2::src::cmd_wait_for::tmuxpeer,
        []
    );
    record!(
        "src/colour.rs::tmuxpeer",
        *mut hmux2::src::colour::tmuxpeer,
        []
    );
    record!(
        "src/control.rs::tmuxpeer",
        *mut hmux2::src::control::tmuxpeer,
        []
    );
    record!(
        "src/control_notify.rs::tmuxpeer",
        *mut hmux2::src::control_notify::tmuxpeer,
        []
    );
    record!(
        "src/environ.rs::tmuxpeer",
        *mut hmux2::src::environ::tmuxpeer,
        []
    );
    record!(
        "src/events.rs::tmuxpeer",
        *mut hmux2::src::events::tmuxpeer,
        []
    );
    record!(
        "src/events_payload.rs::tmuxpeer",
        *mut hmux2::src::events_payload::tmuxpeer,
        []
    );
    record!("src/file.rs::tmuxpeer", *mut hmux2::src::file::tmuxpeer, []);
    record!(
        "src/format.rs::tmuxpeer",
        *mut hmux2::src::format::tmuxpeer,
        []
    );
    record!(
        "src/format_draw.rs::tmuxpeer",
        *mut hmux2::src::format_draw::tmuxpeer,
        []
    );
    record!(
        "src/hooks.rs::tmuxpeer",
        *mut hmux2::src::hooks::tmuxpeer,
        []
    );
    record!(
        "src/input.rs::tmuxpeer",
        *mut hmux2::src::input::tmuxpeer,
        []
    );
    record!(
        "src/input_keys.rs::tmuxpeer",
        *mut hmux2::src::input_keys::tmuxpeer,
        []
    );
    record!("src/job.rs::tmuxpeer", *mut hmux2::src::job::tmuxpeer, []);
    record!(
        "src/key_bindings.rs::tmuxpeer",
        *mut hmux2::src::key_bindings::tmuxpeer,
        []
    );
    record!(
        "src/layout.rs::tmuxpeer",
        *mut hmux2::src::layout::tmuxpeer,
        []
    );
    record!(
        "src/layout_custom.rs::tmuxpeer",
        *mut hmux2::src::layout_custom::tmuxpeer,
        []
    );
    record!(
        "src/layout_set.rs::tmuxpeer",
        *mut hmux2::src::layout_set::tmuxpeer,
        []
    );
    record!("src/menu.rs::tmuxpeer", *mut hmux2::src::menu::tmuxpeer, []);
    record!(
        "src/mode_tree.rs::tmuxpeer",
        *mut hmux2::src::mode_tree::tmuxpeer,
        []
    );
    record!(
        "src/monitor.rs::tmuxpeer",
        *mut hmux2::src::monitor::tmuxpeer,
        []
    );
    record!(
        "src/names.rs::tmuxpeer",
        *mut hmux2::src::names::tmuxpeer,
        []
    );
    record!(
        "src/options.rs::tmuxpeer",
        *mut hmux2::src::options::tmuxpeer,
        []
    );
    record!(
        "src/popup.rs::tmuxpeer",
        *mut hmux2::src::popup::tmuxpeer,
        []
    );
    record!(
        "src/proc.rs::tmuxpeer",
        hmux2::src::proc::tmuxpeer,
        [parent, ibuf, event, uid, gid, flags, dispatchcb, arg, entry]
    );
    record!(
        "src/prompt.rs::tmuxpeer",
        *mut hmux2::src::prompt::tmuxpeer,
        []
    );
    record!(
        "src/resize.rs::tmuxpeer",
        *mut hmux2::src::resize::tmuxpeer,
        []
    );
    record!(
        "src/screen.rs::tmuxpeer",
        *mut hmux2::src::screen::tmuxpeer,
        []
    );
    record!(
        "src/screen_redraw.rs::tmuxpeer",
        *mut hmux2::src::screen_redraw::tmuxpeer,
        []
    );
    record!(
        "src/screen_write.rs::tmuxpeer",
        *mut hmux2::src::screen_write::tmuxpeer,
        []
    );
    record!(
        "src/server.rs::tmuxpeer",
        *mut hmux2::src::server::tmuxpeer,
        []
    );
    record!(
        "src/server_acl.rs::tmuxpeer",
        *mut hmux2::src::server_acl::tmuxpeer,
        []
    );
    record!(
        "src/server_client.rs::tmuxpeer",
        *mut hmux2::src::server_client::tmuxpeer,
        []
    );
    record!(
        "src/server_fn.rs::tmuxpeer",
        *mut hmux2::src::server_fn::tmuxpeer,
        []
    );
    record!(
        "src/session.rs::tmuxpeer",
        *mut hmux2::src::session::tmuxpeer,
        []
    );
    record!("src/sort.rs::tmuxpeer", *mut hmux2::src::sort::tmuxpeer, []);
    record!(
        "src/spawn.rs::tmuxpeer",
        *mut hmux2::src::spawn::tmuxpeer,
        []
    );
    record!(
        "src/status.rs::tmuxpeer",
        *mut hmux2::src::status::tmuxpeer,
        []
    );
    record!(
        "src/style.rs::tmuxpeer",
        *mut hmux2::src::style::tmuxpeer,
        []
    );
    record!("src/tty.rs::tmuxpeer", *mut hmux2::src::tty::tmuxpeer, []);
    record!(
        "src/tty_acs.rs::tmuxpeer",
        *mut hmux2::src::tty_acs::tmuxpeer,
        []
    );
    record!(
        "src/tty_draw.rs::tmuxpeer",
        *mut hmux2::src::tty_draw::tmuxpeer,
        []
    );
    record!(
        "src/tty_features.rs::tmuxpeer",
        *mut hmux2::src::tty_features::tmuxpeer,
        []
    );
    record!(
        "src/tty_keys.rs::tmuxpeer",
        *mut hmux2::src::tty_keys::tmuxpeer,
        []
    );
    record!(
        "src/tty_term.rs::tmuxpeer",
        *mut hmux2::src::tty_term::tmuxpeer,
        []
    );
    record!(
        "src/window.rs::tmuxpeer",
        *mut hmux2::src::window::tmuxpeer,
        []
    );
    record!(
        "src/window_border.rs::tmuxpeer",
        *mut hmux2::src::window_border::tmuxpeer,
        []
    );
    record!(
        "src/window_buffer.rs::tmuxpeer",
        *mut hmux2::src::window_buffer::tmuxpeer,
        []
    );
    record!(
        "src/window_client.rs::tmuxpeer",
        *mut hmux2::src::window_client::tmuxpeer,
        []
    );
    record!(
        "src/window_clock.rs::tmuxpeer",
        *mut hmux2::src::window_clock::tmuxpeer,
        []
    );
    record!(
        "src/window_copy.rs::tmuxpeer",
        *mut hmux2::src::window_copy::tmuxpeer,
        []
    );
    record!(
        "src/window_customize.rs::tmuxpeer",
        *mut hmux2::src::window_customize::tmuxpeer,
        []
    );
    record!(
        "src/window_panes.rs::tmuxpeer",
        *mut hmux2::src::window_panes::tmuxpeer,
        []
    );
    record!(
        "src/window_switch.rs::tmuxpeer",
        *mut hmux2::src::window_switch::tmuxpeer,
        []
    );
    record!(
        "src/window_tree.rs::tmuxpeer",
        *mut hmux2::src::window_tree::tmuxpeer,
        []
    );
    record!(
        "src/window_visible.rs::tmuxpeer",
        *mut hmux2::src::window_visible::tmuxpeer,
        []
    );
    record!(
        "src/proc.rs::C2RustUnnamed_20",
        hmux2::src::proc::tmuxpeer_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/client.rs::tmuxproc",
        *mut hmux2::src::client::tmuxproc,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::tmuxproc",
        *mut hmux2::src::cmd_pipe_pane::tmuxproc,
        []
    );
    record!("src/job.rs::tmuxproc", *mut hmux2::src::job::tmuxproc, []);
    record!(
        "src/proc.rs::tmuxproc",
        hmux2::src::proc::tmuxproc,
        [
            name,
            exit,
            signalcb,
            ev_sigint,
            ev_sighup,
            ev_sigchld,
            ev_sigcont,
            ev_sigterm,
            ev_sigusr1,
            ev_sigusr2,
            ev_sigwinch,
            peers
        ]
    );
    record!(
        "src/server.rs::tmuxproc",
        *mut hmux2::src::server::tmuxproc,
        []
    );
    record!(
        "src/server_client.rs::tmuxproc",
        *mut hmux2::src::server_client::tmuxproc,
        []
    );
    record!(
        "src/spawn.rs::tmuxproc",
        *mut hmux2::src::spawn::tmuxproc,
        []
    );
    record!(
        "src/proc.rs::C2RustUnnamed_21",
        hmux2::src::proc::tmuxproc_peers,
        [tqh_first, tqh_last]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-process.txt"));
}
