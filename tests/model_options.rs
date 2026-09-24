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
        "src/alerts.rs::options",
        *mut hmux2::src::alerts::options,
        []
    );
    record!(
        "src/arguments.rs::options",
        *mut hmux2::src::arguments::options,
        []
    );
    record!("src/cfg.rs::options", *mut hmux2::src::cfg::options, []);
    record!(
        "src/client.rs::options",
        *mut hmux2::src::client::options,
        []
    );
    record!("src/cmd.rs::options", *mut hmux2::src::cmd::options, []);
    record!(
        "src/cmd_attach_session.rs::options",
        *mut hmux2::src::cmd_attach_session::options,
        []
    );
    record!(
        "src/cmd_bind_key.rs::options",
        *mut hmux2::src::cmd_bind_key::options,
        []
    );
    record!(
        "src/cmd_break_pane.rs::options",
        *mut hmux2::src::cmd_break_pane::options,
        []
    );
    record!(
        "src/cmd_capture_pane.rs::options",
        *mut hmux2::src::cmd_capture_pane::options,
        []
    );
    record!(
        "src/cmd_choose_tree.rs::options",
        *mut hmux2::src::cmd_choose_tree::options,
        []
    );
    record!(
        "src/cmd_command_prompt.rs::options",
        *mut hmux2::src::cmd_command_prompt::options,
        []
    );
    record!(
        "src/cmd_confirm_before.rs::options",
        *mut hmux2::src::cmd_confirm_before::options,
        []
    );
    record!(
        "src/cmd_copy_mode.rs::options",
        *mut hmux2::src::cmd_copy_mode::options,
        []
    );
    record!(
        "src/cmd_detach_client.rs::options",
        *mut hmux2::src::cmd_detach_client::options,
        []
    );
    record!(
        "src/cmd_display_menu.rs::options",
        *mut hmux2::src::cmd_display_menu::options,
        []
    );
    record!(
        "src/cmd_display_message.rs::options",
        *mut hmux2::src::cmd_display_message::options,
        []
    );
    record!(
        "src/cmd_find.rs::options",
        *mut hmux2::src::cmd_find::options,
        []
    );
    record!(
        "src/cmd_find_window.rs::options",
        *mut hmux2::src::cmd_find_window::options,
        []
    );
    record!(
        "src/cmd_if_shell.rs::options",
        *mut hmux2::src::cmd_if_shell::options,
        []
    );
    record!(
        "src/cmd_join_pane.rs::options",
        *mut hmux2::src::cmd_join_pane::options,
        []
    );
    record!(
        "src/cmd_kill_pane.rs::options",
        *mut hmux2::src::cmd_kill_pane::options,
        []
    );
    record!(
        "src/cmd_kill_session.rs::options",
        *mut hmux2::src::cmd_kill_session::options,
        []
    );
    record!(
        "src/cmd_kill_window.rs::options",
        *mut hmux2::src::cmd_kill_window::options,
        []
    );
    record!(
        "src/cmd_list_buffers.rs::options",
        *mut hmux2::src::cmd_list_buffers::options,
        []
    );
    record!(
        "src/cmd_list_clients.rs::options",
        *mut hmux2::src::cmd_list_clients::options,
        []
    );
    record!(
        "src/cmd_list_commands.rs::options",
        *mut hmux2::src::cmd_list_commands::options,
        []
    );
    record!(
        "src/cmd_list_keys.rs::options",
        *mut hmux2::src::cmd_list_keys::options,
        []
    );
    record!(
        "src/cmd_list_panes.rs::options",
        *mut hmux2::src::cmd_list_panes::options,
        []
    );
    record!(
        "src/cmd_list_sessions.rs::options",
        *mut hmux2::src::cmd_list_sessions::options,
        []
    );
    record!(
        "src/cmd_list_windows.rs::options",
        *mut hmux2::src::cmd_list_windows::options,
        []
    );
    record!(
        "src/cmd_load_buffer.rs::options",
        *mut hmux2::src::cmd_load_buffer::options,
        []
    );
    record!(
        "src/cmd_lock_server.rs::options",
        *mut hmux2::src::cmd_lock_server::options,
        []
    );
    record!(
        "src/cmd_move_window.rs::options",
        *mut hmux2::src::cmd_move_window::options,
        []
    );
    record!(
        "src/cmd_new_session.rs::options",
        *mut hmux2::src::cmd_new_session::options,
        []
    );
    record!(
        "src/cmd_new_window.rs::options",
        *mut hmux2::src::cmd_new_window::options,
        []
    );
    record!(
        "src/cmd_parse.rs::options",
        *mut hmux2::src::cmd_parse::options,
        []
    );
    record!(
        "src/cmd_paste_buffer.rs::options",
        *mut hmux2::src::cmd_paste_buffer::options,
        []
    );
    record!(
        "src/cmd_pipe_pane.rs::options",
        *mut hmux2::src::cmd_pipe_pane::options,
        []
    );
    record!(
        "src/cmd_queue.rs::options",
        *mut hmux2::src::cmd_queue::options,
        []
    );
    record!(
        "src/cmd_refresh_client.rs::options",
        *mut hmux2::src::cmd_refresh_client::options,
        []
    );
    record!(
        "src/cmd_rename_session.rs::options",
        *mut hmux2::src::cmd_rename_session::options,
        []
    );
    record!(
        "src/cmd_rename_window.rs::options",
        *mut hmux2::src::cmd_rename_window::options,
        []
    );
    record!(
        "src/cmd_resize_pane.rs::options",
        *mut hmux2::src::cmd_resize_pane::options,
        []
    );
    record!(
        "src/cmd_resize_window.rs::options",
        *mut hmux2::src::cmd_resize_window::options,
        []
    );
    record!(
        "src/cmd_respawn_pane.rs::options",
        *mut hmux2::src::cmd_respawn_pane::options,
        []
    );
    record!(
        "src/cmd_respawn_window.rs::options",
        *mut hmux2::src::cmd_respawn_window::options,
        []
    );
    record!(
        "src/cmd_rotate_window.rs::options",
        *mut hmux2::src::cmd_rotate_window::options,
        []
    );
    record!(
        "src/cmd_run_shell.rs::options",
        *mut hmux2::src::cmd_run_shell::options,
        []
    );
    record!(
        "src/cmd_save_buffer.rs::options",
        *mut hmux2::src::cmd_save_buffer::options,
        []
    );
    record!(
        "src/cmd_select_layout.rs::options",
        *mut hmux2::src::cmd_select_layout::options,
        []
    );
    record!(
        "src/cmd_select_pane.rs::options",
        *mut hmux2::src::cmd_select_pane::options,
        []
    );
    record!(
        "src/cmd_select_window.rs::options",
        *mut hmux2::src::cmd_select_window::options,
        []
    );
    record!(
        "src/cmd_send_keys.rs::options",
        *mut hmux2::src::cmd_send_keys::options,
        []
    );
    record!(
        "src/cmd_server_access.rs::options",
        *mut hmux2::src::cmd_server_access::options,
        []
    );
    record!(
        "src/cmd_set_buffer.rs::options",
        *mut hmux2::src::cmd_set_buffer::options,
        []
    );
    record!(
        "src/cmd_set_environment.rs::options",
        *mut hmux2::src::cmd_set_environment::options,
        []
    );
    record!(
        "src/cmd_set_option.rs::options",
        *mut hmux2::src::cmd_set_option::options,
        []
    );
    record!(
        "src/cmd_show_environment.rs::options",
        *mut hmux2::src::cmd_show_environment::options,
        []
    );
    record!(
        "src/cmd_show_messages.rs::options",
        *mut hmux2::src::cmd_show_messages::options,
        []
    );
    record!(
        "src/cmd_show_options.rs::options",
        *mut hmux2::src::cmd_show_options::options,
        []
    );
    record!(
        "src/cmd_source_file.rs::options",
        *mut hmux2::src::cmd_source_file::options,
        []
    );
    record!(
        "src/cmd_split_window.rs::options",
        *mut hmux2::src::cmd_split_window::options,
        []
    );
    record!(
        "src/cmd_swap_pane.rs::options",
        *mut hmux2::src::cmd_swap_pane::options,
        []
    );
    record!(
        "src/cmd_swap_window.rs::options",
        *mut hmux2::src::cmd_swap_window::options,
        []
    );
    record!(
        "src/cmd_switch_client.rs::options",
        *mut hmux2::src::cmd_switch_client::options,
        []
    );
    record!(
        "src/cmd_wait_for.rs::options",
        *mut hmux2::src::cmd_wait_for::options,
        []
    );
    record!(
        "src/colour.rs::options",
        *mut hmux2::src::colour::options,
        []
    );
    record!(
        "src/control.rs::options",
        *mut hmux2::src::control::options,
        []
    );
    record!(
        "src/control_notify.rs::options",
        *mut hmux2::src::control_notify::options,
        []
    );
    record!(
        "src/environ.rs::options",
        *mut hmux2::src::environ::options,
        []
    );
    record!(
        "src/events.rs::options",
        *mut hmux2::src::events::options,
        []
    );
    record!(
        "src/events_payload.rs::options",
        *mut hmux2::src::events_payload::options,
        []
    );
    record!("src/file.rs::options", *mut hmux2::src::file::options, []);
    record!(
        "src/format.rs::options",
        *mut hmux2::src::format::options,
        []
    );
    record!(
        "src/format_draw.rs::options",
        *mut hmux2::src::format_draw::options,
        []
    );
    record!("src/hooks.rs::options", *mut hmux2::src::hooks::options, []);
    record!("src/input.rs::options", *mut hmux2::src::input::options, []);
    record!(
        "src/input_keys.rs::options",
        *mut hmux2::src::input_keys::options,
        []
    );
    record!("src/job.rs::options", *mut hmux2::src::job::options, []);
    record!(
        "src/key_bindings.rs::options",
        *mut hmux2::src::key_bindings::options,
        []
    );
    record!(
        "src/layout.rs::options",
        *mut hmux2::src::layout::options,
        []
    );
    record!(
        "src/layout_custom.rs::options",
        *mut hmux2::src::layout_custom::options,
        []
    );
    record!(
        "src/layout_set.rs::options",
        *mut hmux2::src::layout_set::options,
        []
    );
    record!("src/menu.rs::options", *mut hmux2::src::menu::options, []);
    record!(
        "src/mode_tree.rs::options",
        *mut hmux2::src::mode_tree::options,
        []
    );
    record!(
        "src/monitor.rs::options",
        *mut hmux2::src::monitor::options,
        []
    );
    record!("src/names.rs::options", *mut hmux2::src::names::options, []);
    record!(
        "src/options.rs::options",
        hmux2::src::options::options,
        [tree, parent]
    );
    record!("src/paste.rs::options", *mut hmux2::src::paste::options, []);
    record!("src/popup.rs::options", *mut hmux2::src::popup::options, []);
    record!(
        "src/prompt.rs::options",
        *mut hmux2::src::prompt::options,
        []
    );
    record!(
        "src/prompt_history.rs::options",
        *mut hmux2::src::prompt_history::options,
        []
    );
    record!(
        "src/resize.rs::options",
        *mut hmux2::src::resize::options,
        []
    );
    record!(
        "src/screen.rs::options",
        *mut hmux2::src::screen::options,
        []
    );
    record!(
        "src/screen_redraw.rs::options",
        *mut hmux2::src::screen_redraw::options,
        []
    );
    record!(
        "src/screen_write.rs::options",
        *mut hmux2::src::screen_write::options,
        []
    );
    record!(
        "src/server.rs::options",
        *mut hmux2::src::server::options,
        []
    );
    record!(
        "src/server_acl.rs::options",
        *mut hmux2::src::server_acl::options,
        []
    );
    record!(
        "src/server_client.rs::options",
        *mut hmux2::src::server_client::options,
        []
    );
    record!(
        "src/server_fn.rs::options",
        *mut hmux2::src::server_fn::options,
        []
    );
    record!(
        "src/session.rs::options",
        *mut hmux2::src::session::options,
        []
    );
    record!("src/sort.rs::options", *mut hmux2::src::sort::options, []);
    record!("src/spawn.rs::options", *mut hmux2::src::spawn::options, []);
    record!(
        "src/status.rs::options",
        *mut hmux2::src::status::options,
        []
    );
    record!("src/style.rs::options", *mut hmux2::src::style::options, []);
    record!("src/tmux.rs::options", *mut hmux2::src::tmux::options, []);
    record!("src/tty.rs::options", *mut hmux2::src::tty::options, []);
    record!(
        "src/tty_acs.rs::options",
        *mut hmux2::src::tty_acs::options,
        []
    );
    record!(
        "src/tty_draw.rs::options",
        *mut hmux2::src::tty_draw::options,
        []
    );
    record!(
        "src/tty_features.rs::options",
        *mut hmux2::src::tty_features::options,
        []
    );
    record!(
        "src/tty_keys.rs::options",
        *mut hmux2::src::tty_keys::options,
        []
    );
    record!(
        "src/tty_term.rs::options",
        *mut hmux2::src::tty_term::options,
        []
    );
    record!("src/utf8.rs::options", *mut hmux2::src::utf8::options, []);
    record!(
        "src/window.rs::options",
        *mut hmux2::src::window::options,
        []
    );
    record!(
        "src/window_border.rs::options",
        *mut hmux2::src::window_border::options,
        []
    );
    record!(
        "src/window_buffer.rs::options",
        *mut hmux2::src::window_buffer::options,
        []
    );
    record!(
        "src/window_client.rs::options",
        *mut hmux2::src::window_client::options,
        []
    );
    record!(
        "src/window_clock.rs::options",
        *mut hmux2::src::window_clock::options,
        []
    );
    record!(
        "src/window_copy.rs::options",
        *mut hmux2::src::window_copy::options,
        []
    );
    record!(
        "src/window_customize.rs::options",
        *mut hmux2::src::window_customize::options,
        []
    );
    record!(
        "src/window_panes.rs::options",
        *mut hmux2::src::window_panes::options,
        []
    );
    record!(
        "src/window_switch.rs::options",
        *mut hmux2::src::window_switch::options,
        []
    );
    record!(
        "src/window_tree.rs::options",
        *mut hmux2::src::window_tree::options,
        []
    );
    record!(
        "src/window_visible.rs::options",
        *mut hmux2::src::window_visible::options,
        []
    );
    record!(
        "src/cmd.rs::options_array",
        hmux2::src::cmd::options_array,
        [storage]
    );
    record!(
        "src/cmd_set_option.rs::options_array",
        hmux2::src::cmd_set_option::options_array,
        [storage]
    );
    record!(
        "src/colour.rs::options_array",
        hmux2::src::colour::options_array,
        [storage]
    );
    record!(
        "src/environ.rs::options_array",
        hmux2::src::environ::options_array,
        [storage]
    );
    record!(
        "src/hooks.rs::options_array",
        hmux2::src::hooks::options_array,
        [storage]
    );
    record!(
        "src/options.rs::options_array",
        hmux2::src::options::options_array,
        [storage]
    );
    record!(
        "src/prompt.rs::options_array",
        hmux2::src::prompt::options_array,
        [storage]
    );
    record!(
        "src/status.rs::options_array",
        hmux2::src::status::options_array,
        [storage]
    );
    record!(
        "src/tty_keys.rs::options_array",
        hmux2::src::tty_keys::options_array,
        [storage]
    );
    record!(
        "src/tty_term.rs::options_array",
        hmux2::src::tty_term::options_array,
        [storage]
    );
    record!(
        "src/utf8.rs::options_array",
        hmux2::src::utf8::options_array,
        [storage]
    );
    record!(
        "src/window_customize.rs::options_array",
        hmux2::src::window_customize::options_array,
        [storage]
    );
    record!(
        "src/cmd.rs::options_array_item",
        *mut hmux2::src::cmd::options_array_item,
        []
    );
    record!(
        "src/cmd_set_option.rs::options_array_item",
        *mut hmux2::src::cmd_set_option::options_array_item,
        []
    );
    record!(
        "src/cmd_show_options.rs::options_array_item",
        *mut hmux2::src::cmd_show_options::options_array_item,
        []
    );
    record!(
        "src/colour.rs::options_array_item",
        *mut hmux2::src::colour::options_array_item,
        []
    );
    record!(
        "src/environ.rs::options_array_item",
        *mut hmux2::src::environ::options_array_item,
        []
    );
    record!(
        "src/format.rs::options_array_item",
        *mut hmux2::src::format::options_array_item,
        []
    );
    record!(
        "src/hooks.rs::options_array_item",
        *mut hmux2::src::hooks::options_array_item,
        []
    );
    record!(
        "src/options.rs::options_array_item",
        hmux2::src::options::options_array_item,
        [key, value, owner]
    );
    record!(
        "src/prompt.rs::options_array_item",
        *mut hmux2::src::prompt::options_array_item,
        []
    );
    record!(
        "src/status.rs::options_array_item",
        *mut hmux2::src::status::options_array_item,
        []
    );
    record!(
        "src/tty_keys.rs::options_array_item",
        *mut hmux2::src::tty_keys::options_array_item,
        []
    );
    record!(
        "src/tty_term.rs::options_array_item",
        *mut hmux2::src::tty_term::options_array_item,
        []
    );
    record!(
        "src/utf8.rs::options_array_item",
        *mut hmux2::src::utf8::options_array_item,
        []
    );
    record!(
        "src/window_customize.rs::options_array_item",
        *mut hmux2::src::window_customize::options_array_item,
        []
    );
    record!(
        "src/cmd.rs::options_entry",
        *mut hmux2::src::cmd::options_entry,
        []
    );
    record!(
        "src/cmd_break_pane.rs::options_entry",
        *mut hmux2::src::cmd_break_pane::options_entry,
        []
    );
    record!(
        "src/cmd_display_menu.rs::options_entry",
        *mut hmux2::src::cmd_display_menu::options_entry,
        []
    );
    record!(
        "src/cmd_new_session.rs::options_entry",
        *mut hmux2::src::cmd_new_session::options_entry,
        []
    );
    record!(
        "src/cmd_rename_window.rs::options_entry",
        *mut hmux2::src::cmd_rename_window::options_entry,
        []
    );
    record!(
        "src/cmd_resize_window.rs::options_entry",
        *mut hmux2::src::cmd_resize_window::options_entry,
        []
    );
    record!(
        "src/cmd_select_pane.rs::options_entry",
        *mut hmux2::src::cmd_select_pane::options_entry,
        []
    );
    record!(
        "src/cmd_set_option.rs::options_entry",
        *mut hmux2::src::cmd_set_option::options_entry,
        []
    );
    record!(
        "src/cmd_show_options.rs::options_entry",
        *mut hmux2::src::cmd_show_options::options_entry,
        []
    );
    record!(
        "src/cmd_split_window.rs::options_entry",
        *mut hmux2::src::cmd_split_window::options_entry,
        []
    );
    record!(
        "src/colour.rs::options_entry",
        *mut hmux2::src::colour::options_entry,
        []
    );
    record!(
        "src/environ.rs::options_entry",
        *mut hmux2::src::environ::options_entry,
        []
    );
    record!(
        "src/format.rs::options_entry",
        *mut hmux2::src::format::options_entry,
        []
    );
    record!(
        "src/hooks.rs::options_entry",
        *mut hmux2::src::hooks::options_entry,
        []
    );
    record!(
        "src/input.rs::options_entry",
        *mut hmux2::src::input::options_entry,
        []
    );
    record!(
        "src/options.rs::options_entry",
        hmux2::src::options::options_entry,
        [
            owner,
            name,
            tableentry,
            value,
            cached,
            style,
            monitor_data,
            fire_count,
            fire_time
        ]
    );
    record!(
        "src/prompt.rs::options_entry",
        *mut hmux2::src::prompt::options_entry,
        []
    );
    record!(
        "src/server.rs::options_entry",
        *mut hmux2::src::server::options_entry,
        []
    );
    record!(
        "src/server_client.rs::options_entry",
        *mut hmux2::src::server_client::options_entry,
        []
    );
    record!(
        "src/spawn.rs::options_entry",
        *mut hmux2::src::spawn::options_entry,
        []
    );
    record!(
        "src/status.rs::options_entry",
        *mut hmux2::src::status::options_entry,
        []
    );
    record!(
        "src/style.rs::options_entry",
        *mut hmux2::src::style::options_entry,
        []
    );
    record!(
        "src/tmux.rs::options_entry",
        *mut hmux2::src::tmux::options_entry,
        []
    );
    record!(
        "src/tty_keys.rs::options_entry",
        *mut hmux2::src::tty_keys::options_entry,
        []
    );
    record!(
        "src/tty_term.rs::options_entry",
        *mut hmux2::src::tty_term::options_entry,
        []
    );
    record!(
        "src/utf8.rs::options_entry",
        *mut hmux2::src::utf8::options_entry,
        []
    );
    record!(
        "src/window_customize.rs::options_entry",
        *mut hmux2::src::window_customize::options_entry,
        []
    );
    record!(
        "src/cmd_display_menu.rs::options_table_entry",
        hmux2::src::cmd_display_menu::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/cmd_set_option.rs::options_table_entry",
        hmux2::src::cmd_set_option::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/cmd_show_options.rs::options_table_entry",
        hmux2::src::cmd_show_options::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/cmd_split_window.rs::options_table_entry",
        hmux2::src::cmd_split_window::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/format.rs::options_table_entry",
        hmux2::src::format::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/hooks.rs::options_table_entry",
        hmux2::src::hooks::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/options.rs::options_table_entry",
        hmux2::src::options::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/options_table.rs::options_table_entry",
        hmux2::src::options_table::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/style.rs::options_table_entry",
        hmux2::src::style::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/tmux.rs::options_table_entry",
        hmux2::src::tmux::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/window_customize.rs::options_table_entry",
        hmux2::src::window_customize::options_table_entry,
        [
            name,
            alternative_name,
            type_0,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str,
            default_num,
            default_arr,
            separator,
            pattern,
            text,
            unit
        ]
    );
    record!(
        "src/cmd.rs::options_value",
        hmux2::src::cmd::options_value,
        []
    );
    record!(
        "src/cmd_set_option.rs::options_value",
        hmux2::src::cmd_set_option::options_value,
        []
    );
    record!(
        "src/colour.rs::options_value",
        hmux2::src::colour::options_value,
        []
    );
    record!(
        "src/environ.rs::options_value",
        hmux2::src::environ::options_value,
        []
    );
    record!(
        "src/hooks.rs::options_value",
        hmux2::src::hooks::options_value,
        []
    );
    record!(
        "src/options.rs::options_value",
        hmux2::src::options::options_value,
        []
    );
    record!(
        "src/prompt.rs::options_value",
        hmux2::src::prompt::options_value,
        []
    );
    record!(
        "src/status.rs::options_value",
        hmux2::src::status::options_value,
        []
    );
    record!(
        "src/tty_keys.rs::options_value",
        hmux2::src::tty_keys::options_value,
        []
    );
    record!(
        "src/tty_term.rs::options_value",
        hmux2::src::tty_term::options_value,
        []
    );
    record!(
        "src/utf8.rs::options_value",
        hmux2::src::utf8::options_value,
        []
    );
    record!(
        "src/window_customize.rs::options_value",
        hmux2::src::window_customize::options_value,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-options.txt"));
}
