//! The commands: the table every tmux command is looked up in, the parsing
//! and printing of a command list, and one private module per command.
//!
//! A command module is reached only through the table below, so the modules
//! are private; a test build opens them so the per-command suites can reach
//! in. What else the rest of the crate may use is re-exported here.

macro_rules! command_modules {
    ($($name:ident),+ $(,)?) => {
        $(
            #[cfg(not(test))]
            mod $name;
            #[cfg(test)]
            pub(crate) mod $name;
        )+
    };
}

command_modules!(
    cmd_attach_session,
    cmd_bind_key,
    cmd_break_pane,
    cmd_capture_pane,
    cmd_choose_tree,
    cmd_command_prompt,
    cmd_confirm_before,
    cmd_copy_mode,
    cmd_detach_client,
    cmd_display_menu,
    cmd_display_message,
    cmd_display_panes,
    cmd_find_window,
    cmd_if_shell,
    cmd_join_pane,
    cmd_kill_pane,
    cmd_kill_server,
    cmd_kill_session,
    cmd_kill_window,
    cmd_list_buffers,
    cmd_list_clients,
    cmd_list_commands,
    cmd_list_keys,
    cmd_list_panes,
    cmd_list_sessions,
    cmd_list_windows,
    cmd_load_buffer,
    cmd_lock_server,
    cmd_move_window,
    cmd_new_session,
    cmd_new_window,
    cmd_paste_buffer,
    cmd_pipe_pane,
    cmd_refresh_client,
    cmd_rename_session,
    cmd_rename_window,
    cmd_resize_pane,
    cmd_resize_window,
    cmd_respawn_pane,
    cmd_respawn_window,
    cmd_rotate_window,
    cmd_run_shell,
    cmd_save_buffer,
    cmd_select_layout,
    cmd_select_pane,
    cmd_select_window,
    cmd_send_keys,
    cmd_server_access,
    cmd_set_buffer,
    cmd_set_environment,
    cmd_set_option,
    cmd_show_environment,
    cmd_show_messages,
    cmd_show_options,
    cmd_show_prompt_history,
    cmd_source_file,
    cmd_split_window,
    cmd_swap_pane,
    cmd_swap_window,
    cmd_switch_client,
    cmd_unbind_key,
    cmd_wait_for
);

mod find;
mod parse;
mod queue;

pub use find::{
    cmd_find_clear_state, cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_client,
    cmd_find_from_mouse, cmd_find_from_nothing, cmd_find_from_pane, cmd_find_from_session,
    cmd_find_from_session_window, cmd_find_from_window, cmd_find_from_winlink,
    cmd_find_from_winlink_pane, cmd_find_valid_state,
};
pub use parse::{
    CMD_PARSE_SUCCESS, cmd_parse_from_arguments, cmd_parse_from_buffer, cmd_parse_from_file,
    cmd_parse_from_string,
};
pub use queue::{
    CmdqItemWeak, cmdq_add_format, cmdq_add_formats, cmdq_append, cmdq_continue, cmdq_error,
    cmdq_get_callback1, cmdq_get_client, cmdq_get_error, cmdq_get_event, cmdq_get_flags,
    cmdq_get_name, cmdq_get_target, cmdq_get_target_client, cmdq_guard, cmdq_insert_after,
    cmdq_item, cmdq_items, cmdq_list, cmdq_merge_formats, cmdq_new, cmdq_next, cmdq_print,
    cmdq_running,
};

#[cfg(test)]
pub(crate) use find::{cmd_find_best_client, cmd_find_target};
pub(crate) use parse::cmd_parse_and_append;
#[cfg(test)]
pub(crate) use parse::{
    CMD_PARSE_COMMANDS, CMD_PARSE_ERROR, CMD_PARSE_MAX_ENVIRON_LEN, CMD_PARSE_NOALIAS,
    CMD_PARSE_ONEGROUP, CMD_PARSE_PARSED_COMMANDS, CMD_PARSE_PARSEONLY, CMD_PARSE_STRING,
    CMD_PARSE_VERBOSE, DOUBLE_QUOTES, NONE, SINGLE_QUOTES, START, cmd_parse_state,
};
#[cfg(test)]
pub(crate) use queue::{
    CMD_AFTERHOOK, CMDQ_STATE_NOHOOKS, CMDQ_WAITING, CmdqItemRef, CmdqType, KEYC_NONE, cmdq_free,
    cmdq_get_current, cmdq_get_state, cmdq_item_new, cmdq_set_target_client,
};
pub(crate) use queue::{
    CmdqStateRef, cmdq_copy_state, cmdq_get_command, cmdq_get_state_ref, cmdq_item_weak_from_ptr,
    cmdq_new_state,
};

pub use cmd_command_prompt::cmd_command_prompt_cdata;
pub(crate) use cmd_command_prompt::{cmd_command_prompt_callback, cmd_command_prompt_free};
pub(crate) use cmd_confirm_before::cmd_confirm_before_callback;
pub use cmd_confirm_before::cmd_confirm_before_data;
pub use cmd_display_panes::cmd_display_panes_data;
pub(crate) use cmd_display_panes::{
    cmd_display_panes_draw, cmd_display_panes_free_box, cmd_display_panes_key,
};
pub use cmd_if_shell::cmd_if_shell_data;
pub use cmd_load_buffer::cmd_load_buffer_data;
pub use cmd_run_shell::cmd_run_shell_data;
pub use cmd_source_file::cmd_source_file_data;
pub use cmd_wait_for::cmd_wait_for_flush;

use crate::arguments::{args_copy, args_escape, args_parse, args_print, args_ptr};
use crate::cmd::cmd_attach_session::cmd_attach_session_entry;
use crate::cmd::cmd_bind_key::cmd_bind_key_entry;
use crate::cmd::cmd_break_pane::cmd_break_pane_entry;
use crate::cmd::cmd_capture_pane::{cmd_capture_pane_entry, cmd_clear_history_entry};
use crate::cmd::cmd_choose_tree::{
    cmd_choose_buffer_entry, cmd_choose_client_entry, cmd_choose_tree_entry,
    cmd_customize_mode_entry,
};
use crate::cmd::cmd_command_prompt::cmd_command_prompt_entry;
use crate::cmd::cmd_confirm_before::cmd_confirm_before_entry;
use crate::cmd::cmd_copy_mode::{cmd_clock_mode_entry, cmd_copy_mode_entry};
use crate::cmd::cmd_detach_client::{cmd_detach_client_entry, cmd_suspend_client_entry};
use crate::cmd::cmd_display_menu::{cmd_display_menu_entry, cmd_display_popup_entry};
use crate::cmd::cmd_display_message::cmd_display_message_entry;
use crate::cmd::cmd_display_panes::cmd_display_panes_entry;
use crate::cmd::cmd_find_window::cmd_find_window_entry;
use crate::cmd::cmd_if_shell::cmd_if_shell_entry;
use crate::cmd::cmd_join_pane::{cmd_join_pane_entry, cmd_move_pane_entry};
use crate::cmd::cmd_kill_pane::cmd_kill_pane_entry;
use crate::cmd::cmd_kill_server::{cmd_kill_server_entry, cmd_start_server_entry};
use crate::cmd::cmd_kill_session::cmd_kill_session_entry;
use crate::cmd::cmd_kill_window::{cmd_kill_window_entry, cmd_unlink_window_entry};
use crate::cmd::cmd_list_buffers::cmd_list_buffers_entry;
use crate::cmd::cmd_list_clients::cmd_list_clients_entry;
use crate::cmd::cmd_list_commands::cmd_list_commands_entry;
use crate::cmd::cmd_list_keys::cmd_list_keys_entry;
use crate::cmd::cmd_list_panes::cmd_list_panes_entry;
use crate::cmd::cmd_list_sessions::cmd_list_sessions_entry;
use crate::cmd::cmd_list_windows::cmd_list_windows_entry;
use crate::cmd::cmd_load_buffer::cmd_load_buffer_entry;
use crate::cmd::cmd_lock_server::{
    cmd_lock_client_entry, cmd_lock_server_entry, cmd_lock_session_entry,
};
use crate::cmd::cmd_move_window::{cmd_link_window_entry, cmd_move_window_entry};
use crate::cmd::cmd_new_session::{cmd_has_session_entry, cmd_new_session_entry};
use crate::cmd::cmd_new_window::cmd_new_window_entry;
use crate::cmd::cmd_paste_buffer::cmd_paste_buffer_entry;
use crate::cmd::cmd_pipe_pane::cmd_pipe_pane_entry;
use crate::cmd::cmd_refresh_client::cmd_refresh_client_entry;
use crate::cmd::cmd_rename_session::cmd_rename_session_entry;
use crate::cmd::cmd_rename_window::cmd_rename_window_entry;
use crate::cmd::cmd_resize_pane::cmd_resize_pane_entry;
use crate::cmd::cmd_resize_window::cmd_resize_window_entry;
use crate::cmd::cmd_respawn_pane::cmd_respawn_pane_entry;
use crate::cmd::cmd_respawn_window::cmd_respawn_window_entry;
use crate::cmd::cmd_rotate_window::cmd_rotate_window_entry;
use crate::cmd::cmd_run_shell::cmd_run_shell_entry;
use crate::cmd::cmd_save_buffer::{cmd_save_buffer_entry, cmd_show_buffer_entry};
use crate::cmd::cmd_select_layout::{
    cmd_next_layout_entry, cmd_previous_layout_entry, cmd_select_layout_entry,
};
use crate::cmd::cmd_select_pane::{cmd_last_pane_entry, cmd_select_pane_entry};
use crate::cmd::cmd_select_window::{
    cmd_last_window_entry, cmd_next_window_entry, cmd_previous_window_entry,
    cmd_select_window_entry,
};
use crate::cmd::cmd_send_keys::{cmd_send_keys_entry, cmd_send_prefix_entry};
use crate::cmd::cmd_server_access::cmd_server_access_entry;
use crate::cmd::cmd_set_buffer::{cmd_delete_buffer_entry, cmd_set_buffer_entry};
use crate::cmd::cmd_set_environment::cmd_set_environment_entry;
use crate::cmd::cmd_set_option::{
    cmd_set_hook_entry, cmd_set_option_entry, cmd_set_window_option_entry,
};
use crate::cmd::cmd_show_environment::cmd_show_environment_entry;
use crate::cmd::cmd_show_messages::cmd_show_messages_entry;
use crate::cmd::cmd_show_options::{
    cmd_show_hooks_entry, cmd_show_options_entry, cmd_show_window_options_entry,
};
use crate::cmd::cmd_show_prompt_history::{
    cmd_clear_prompt_history_entry, cmd_show_prompt_history_entry,
};
use crate::cmd::cmd_source_file::cmd_source_file_entry;
use crate::cmd::cmd_split_window::{cmd_new_pane_entry, cmd_split_window_entry};
use crate::cmd::cmd_swap_pane::cmd_swap_pane_entry;
use crate::cmd::cmd_swap_window::cmd_swap_window_entry;
use crate::cmd::cmd_switch_client::cmd_switch_client_entry;
use crate::cmd::cmd_unbind_key::cmd_unbind_key_entry;
use crate::cmd::cmd_wait_for::cmd_wait_for_entry;
use crate::ffi::{strchr, strlcpy, strlen};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::list::foreach_owned;
use crate::log::log_debug;
use crate::options::options_get_only_ptr;
use crate::options::{options_array_first, options_array_item_value, options_array_next};
use crate::session::session_find_by_id;
use crate::session::session_get_curw;
use crate::tmux::global_options;
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::{
    window_find_by_id, window_has_pane, window_pane_find_by_id, winlink_find_by_window,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_char, c_int, c_uint};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
/// The commands of one command list, in the order they run. Each command
/// belongs to the list it sits in.
pub type cmds = ::std::vec::Vec<::std::boxed::Box<cmd>>;

#[repr(C)]
pub struct cmd {
    pub entry: &'static cmd_entry,
    pub args: Option<Box<args>>,
    pub group: u_int,
    pub file: Option<::std::ffi::CString>,
    pub line: u_int,
    pub parse_flags: ::core::ffi::c_int,
}
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const CMD_LIST_PRINT_ESCAPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_LIST_PRINT_NO_GROUPS: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
/// Every command the server knows.
pub static cmd_table: &[&cmd_entry] = &[
    &cmd_attach_session_entry,
    &cmd_bind_key_entry,
    &cmd_break_pane_entry,
    &cmd_capture_pane_entry,
    &cmd_choose_buffer_entry,
    &cmd_choose_client_entry,
    &cmd_choose_tree_entry,
    &cmd_clear_history_entry,
    &cmd_clear_prompt_history_entry,
    &cmd_clock_mode_entry,
    &cmd_command_prompt_entry,
    &cmd_confirm_before_entry,
    &cmd_copy_mode_entry,
    &cmd_customize_mode_entry,
    &cmd_delete_buffer_entry,
    &cmd_detach_client_entry,
    &cmd_display_menu_entry,
    &cmd_display_message_entry,
    &cmd_display_popup_entry,
    &cmd_display_panes_entry,
    &cmd_find_window_entry,
    &cmd_has_session_entry,
    &cmd_if_shell_entry,
    &cmd_join_pane_entry,
    &cmd_kill_pane_entry,
    &cmd_kill_server_entry,
    &cmd_kill_session_entry,
    &cmd_kill_window_entry,
    &cmd_last_pane_entry,
    &cmd_last_window_entry,
    &cmd_link_window_entry,
    &cmd_list_buffers_entry,
    &cmd_list_clients_entry,
    &cmd_list_commands_entry,
    &cmd_list_keys_entry,
    &cmd_list_panes_entry,
    &cmd_list_sessions_entry,
    &cmd_list_windows_entry,
    &cmd_load_buffer_entry,
    &cmd_lock_client_entry,
    &cmd_lock_server_entry,
    &cmd_lock_session_entry,
    &cmd_move_pane_entry,
    &cmd_move_window_entry,
    &cmd_new_pane_entry,
    &cmd_new_session_entry,
    &cmd_new_window_entry,
    &cmd_next_layout_entry,
    &cmd_next_window_entry,
    &cmd_paste_buffer_entry,
    &cmd_pipe_pane_entry,
    &cmd_previous_layout_entry,
    &cmd_previous_window_entry,
    &cmd_refresh_client_entry,
    &cmd_rename_session_entry,
    &cmd_rename_window_entry,
    &cmd_resize_pane_entry,
    &cmd_resize_window_entry,
    &cmd_respawn_pane_entry,
    &cmd_respawn_window_entry,
    &cmd_rotate_window_entry,
    &cmd_run_shell_entry,
    &cmd_save_buffer_entry,
    &cmd_select_layout_entry,
    &cmd_select_pane_entry,
    &cmd_select_window_entry,
    &cmd_send_keys_entry,
    &cmd_send_prefix_entry,
    &cmd_server_access_entry,
    &cmd_set_buffer_entry,
    &cmd_set_environment_entry,
    &cmd_set_hook_entry,
    &cmd_set_option_entry,
    &cmd_set_window_option_entry,
    &cmd_show_buffer_entry,
    &cmd_show_environment_entry,
    &cmd_show_hooks_entry,
    &cmd_show_messages_entry,
    &cmd_show_options_entry,
    &cmd_show_prompt_history_entry,
    &cmd_show_window_options_entry,
    &cmd_source_file_entry,
    &cmd_split_window_entry,
    &cmd_start_server_entry,
    &cmd_suspend_client_entry,
    &cmd_swap_pane_entry,
    &cmd_swap_window_entry,
    &cmd_switch_client_entry,
    &cmd_unbind_key_entry,
    &cmd_unlink_window_entry,
    &cmd_wait_for_entry,
];
static mut cmd_list_next_group: u_int = 1 as u_int;

/// A command entry's name.
fn entry_name(entry: &cmd_entry) -> &'static [u8] {
    entry.name.to_bytes()
}

/// A command entry's alias, where it has one.
fn entry_alias(entry: &cmd_entry) -> Option<&'static [u8]> {
    entry.alias.map(CStr::to_bytes)
}

/// The number the next command list, or the next move onto one, is stamped
/// with. Every command in a list carries its list's number, and a step
/// between two of them is what `;;` prints as.
fn next_group() -> u_int {
    unsafe {
        let group = cmd_list_next_group;
        cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
        group
    }
}

/// Moves everything `from` holds onto the end of `list`, leaving `from` empty.
unsafe fn list_concat(list: *mut cmds, from: *mut cmds) {
    unsafe {
        let moved = ::core::mem::take(&mut *from);
        (*list).extend(moved);
    }
}

unsafe fn cmd_list_raw(cmdlist: *const cmd_list) -> *mut cmds {
    unsafe {
        (*cmdlist)
            .list
            .as_ref()
            .map(|list| list.as_ref() as *const cmds as *mut cmds)
            .expect("command list has been dropped")
    }
}

/// The commands in `list`, in the order they run, walked the way the C's
/// `TAILQ_FOREACH` walked them.
unsafe fn list_commands(list: *mut cmds) -> impl Iterator<Item = *mut cmd> {
    unsafe { foreach_owned(list) }
}

pub unsafe fn cmd_log_argv(argv: &[CString], fmt: *const c_char, args: &[FmtArg]) {
    unsafe {
        let prefix = format_alloc(fmt, args);
        for (i, arg) in argv.iter().enumerate() {
            log_debug(
                c"%s: argv[%d]=%s".as_ptr(),
                fmt_args![prefix.as_ptr(), i as c_int, arg.as_ptr()],
            );
        }
    }
}

pub fn cmd_prepend_argv(argv: &mut Vec<CString>, arg: &CStr) {
    argv.insert(0, arg.to_owned());
}

pub fn cmd_append_argv(argv: &mut Vec<CString>, arg: &CStr) {
    argv.push(arg.to_owned());
}

pub unsafe fn cmd_pack_argv(argv: &[CString], mut buf: *mut c_char, mut len: size_t) -> c_int {
    unsafe {
        if argv.is_empty() {
            return 0;
        }
        cmd_log_argv(argv, c"%s".as_ptr(), fmt_args![c"cmd_pack_argv".as_ptr()]);
        *buf = b'\0' as c_char;
        for arg in argv {
            if strlcpy(buf, arg.as_ptr(), len) as size_t >= len {
                return -1;
            }
            let arglen = strlen(arg.as_ptr()).wrapping_add(1 as size_t);
            buf = buf.add(arglen);
            len = len.wrapping_sub(arglen);
        }
        0
    }
}

pub unsafe fn cmd_unpack_argv(
    mut buf: *mut c_char,
    mut len: size_t,
    argc: c_int,
) -> Option<Vec<CString>> {
    unsafe {
        if argc == 0 {
            return Some(Vec::new());
        }
        if !(0..=1000).contains(&argc) {
            return None;
        }
        if len == 0 as size_t {
            return None;
        }
        *buf.add(len.wrapping_sub(1 as size_t)) = b'\0' as c_char;
        let mut argv = Vec::with_capacity(argc as usize);
        for _ in 0..argc {
            if len == 0 as size_t {
                return None;
            }
            let s = CStr::from_ptr(buf).to_owned();
            let arglen = strlen(buf).wrapping_add(1 as size_t);
            argv.push(s);
            buf = buf.add(arglen);
            len = len.wrapping_sub(arglen);
        }
        cmd_log_argv(
            &argv,
            c"%s".as_ptr(),
            fmt_args![c"cmd_unpack_argv".as_ptr()],
        );
        Some(argv)
    }
}

pub fn cmd_stringify_argv(argv: &[CString]) -> CString {
    if argv.is_empty() {
        return CString::default();
    }
    let mut out = Vec::<u8>::new();
    for (i, arg) in argv.iter().enumerate() {
        let escaped = unsafe { args_escape(arg.as_ptr()) };
        unsafe {
            log_debug(
                c"%s: %u %s = %s".as_ptr(),
                fmt_args![
                    c"cmd_stringify_argv".as_ptr(),
                    i as c_uint,
                    arg.as_ptr(),
                    escaped.as_ptr()
                ],
            );
        }
        if i != 0 {
            out.push(b' ');
        }
        out.extend_from_slice(escaped.as_bytes());
    }
    unsafe { CString::from_vec_unchecked(out) }
}

pub fn cmd_get_entry(cmd: &cmd) -> &'static cmd_entry {
    cmd.entry
}

pub fn cmd_get_args(cmd: &cmd) -> &args {
    unsafe { &*args_ptr(&cmd.args) }
}

pub fn cmd_get_args_ptr(cmd: &cmd) -> *mut args {
    args_ptr(&cmd.args)
}

pub fn cmd_get_group(cmd: &cmd) -> u_int {
    cmd.group
}

/// Where the command was parsed from: the file, if it came from one, and
/// the line in it.
pub fn cmd_get_source(cmd: &cmd) -> (*const c_char, u_int) {
    (cstr_ptr(&cmd.file), cmd.line)
}

pub fn cmd_get_parse_flags(cmd: &cmd) -> c_int {
    cmd.parse_flags
}

pub unsafe fn cmd_get_alias(name: *const c_char) -> Option<CString> {
    unsafe {
        let o = options_get_only_ptr(global_options, c"command-alias".as_ptr());
        if o.is_null() {
            return None;
        }
        let wanted = CStr::from_ptr(name).to_bytes();
        let mut a = options_array_first(o);
        while !a.is_null() {
            let string = (*options_array_item_value(a)).string();
            let value = CStr::from_ptr(string).to_bytes();
            if let Some(n) = value.iter().position(|&byte| byte == b'=')
                && &value[..n] == wanted
            {
                return Some(CStr::from_ptr(string.add(n + 1)).to_owned());
            }
            a = options_array_next(o, a);
        }
        None
    }
}

pub unsafe fn cmd_find(name: *const c_char, cause: &mut Option<CString>) -> *const cmd_entry {
    unsafe {
        let wanted = CStr::from_ptr(name).to_bytes();
        let mut found: Option<&'static cmd_entry> = None;
        let mut ambiguous = false;
        for &entry in cmd_table {
            if entry_alias(entry) == Some(wanted) {
                ambiguous = false;
                found = Some(entry);
                break;
            }
            let this = entry_name(entry);
            if this.starts_with(wanted) {
                if found.is_some() {
                    ambiguous = true;
                }
                found = Some(entry);
                if this == wanted {
                    break;
                }
            }
        }
        if ambiguous {
            // The names, never the aliases: an alias is only ever matched whole,
            // and one that matched would have ended the walk above.
            //
            // The C built this list with `strlcat` into an 8192-byte buffer and
            // gave up on the first truncation. Neither guard could fire: the whole
            // table, which an empty name matches every entry of, lists as 1264
            // bytes counting the ", " after each name, so the two breaks are gone
            // with the rewrite.
            let mut s = Vec::<u8>::new();
            for &entry in cmd_table {
                let this = entry_name(entry);
                if this.starts_with(wanted) {
                    s.extend_from_slice(this);
                    s.extend_from_slice(b", ");
                }
            }
            // Two or more names is what made it ambiguous, so there is always a
            // trailing ", " to cut back off.
            s.truncate(s.len() - 2);
            s.push(b'\0');
            *cause = Some(xasprintf(
                c"ambiguous command: %s, could be: %s".as_ptr(),
                fmt_args![name, s.as_ptr() as *const c_char],
            ));
            return null::<cmd_entry>();
        }
        let Some(found) = found else {
            *cause = Some(xasprintf(c"unknown command: %s".as_ptr(), fmt_args![name]));
            return null::<cmd_entry>();
        };
        found
    }
}

pub unsafe fn cmd_parse(
    values: *mut args_value_t,
    count: u_int,
    file: *const c_char,
    line: u_int,
    parse_flags: c_int,
) -> Result<Box<cmd>, CString> {
    unsafe {
        if count == 0 || !matches!(&(*values).value, ArgsValue::String(_)) {
            return Err(xasprintf(c"no command".as_ptr(), fmt_args![]));
        }
        let mut cause: Option<CString> = None;
        let entry = cmd_find((*values).value.string(), &mut cause);
        if entry.is_null() {
            return Err(cause.expect("cmd_find gives a cause when it finds nothing"));
        }
        let mut error: Option<CString> = None;
        let Some(args) = args_parse(&raw const (*entry).args, values, count, &mut error) else {
            return Err(match error {
                Some(error) => xasprintf(
                    c"command %s: %s".as_ptr(),
                    fmt_args![(*entry).name, error.as_ptr()],
                ),
                None => xasprintf(
                    c"usage: %s %s".as_ptr(),
                    fmt_args![(*entry).name, (*entry).usage],
                ),
            });
        };
        Ok(Box::new(cmd {
            entry: &*entry,
            args: Some(args),
            group: 0,
            file: if !file.is_null() {
                Some(CStr::from_ptr(file).to_owned())
            } else {
                None
            },
            line,
            parse_flags,
        }))
    }
}

pub unsafe fn cmd_free(mut cmd: Box<cmd>) {
    unsafe {
        let cmd_ptr = &raw mut *cmd;
        (*cmd_ptr).file = None;
        drop((*cmd_ptr).args.take());
        drop(cmd);
    }
}

pub unsafe fn cmd_copy(from: &cmd, argv: &[CString]) -> Box<cmd> {
    unsafe {
        Box::new(cmd {
            entry: from.entry,
            args: Some(args_copy(args_ptr(&from.args), argv)),
            group: 0,
            file: from.file.clone(),
            line: from.line,
            parse_flags: 0,
        })
    }
}

pub unsafe fn cmd_print(cmd: *mut cmd) -> CString {
    unsafe {
        let s = args_print(args_ptr(&(*cmd).args));

        if !s.as_bytes().is_empty() {
            xasprintf(c"%s %s".as_ptr(), fmt_args![(*cmd).entry.name, s.as_ptr()])
        } else {
            (*cmd).entry.name.to_owned()
        }
    }
}

pub(crate) fn cmd_list_new() -> CmdListRef {
    let list = Box::new(cmds::new());
    CmdListRef::new(cmd_list {
        group: next_group(),
        list: Some(list),
    })
}

impl Drop for cmd_list {
    fn drop(&mut self) {
        unsafe {
            let Some(mut list) = self.list.take() else {
                return;
            };
            while let Some(cmd) = list.pop() {
                cmd_free(cmd);
            }
        }
    }
}

pub unsafe fn cmd_list_append(cmdlist: *mut cmd_list, cmd: Box<cmd>) {
    unsafe {
        let mut cmd = cmd;
        cmd.group = (*cmdlist).group;
        (*cmd_list_raw(cmdlist)).push(cmd);
    }
}

pub unsafe fn cmd_list_append_all(cmdlist: *mut cmd_list, from: *mut cmd_list) {
    unsafe {
        for cmd in list_commands(cmd_list_raw(from)) {
            (*cmd).group = (*cmdlist).group;
        }
        list_concat(cmd_list_raw(cmdlist), cmd_list_raw(from));
    }
}

pub unsafe fn cmd_list_move(cmdlist: *mut cmd_list, from: *mut cmd_list) {
    unsafe {
        list_concat(cmd_list_raw(cmdlist), cmd_list_raw(from));
        (*cmdlist).group = next_group();
    }
}

pub(crate) unsafe fn cmd_list_copy(cmdlist: *const cmd_list, argv: &[CString]) -> CmdListRef {
    unsafe {
        let mut group = (*cmdlist).group;
        let s = cmd_list_print(cmdlist, 0);
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_list_copy".as_ptr(), s.as_ptr()],
        );
        let new_cmdlist = cmd_list_new();
        for cmd in list_commands(cmd_list_raw(cmdlist)) {
            if (*cmd).group != group {
                (*new_cmdlist.as_ptr()).group = next_group();
                group = (*cmd).group;
            }
            cmd_list_append(new_cmdlist.as_ptr(), cmd_copy(&*cmd, argv));
        }
        let s = cmd_list_print(new_cmdlist.as_ptr(), 0);
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_list_copy".as_ptr(), s.as_ptr()],
        );
        new_cmdlist
    }
}

pub unsafe fn cmd_list_print(cmdlist: *const cmd_list, flags: c_int) -> CString {
    unsafe {
        let escaped = flags & CMD_LIST_PRINT_ESCAPED != 0;
        let no_groups = flags & CMD_LIST_PRINT_NO_GROUPS != 0;
        let single_separator: &[u8] = if escaped { b" \\; " } else { b" ; " };
        let double_separator: &[u8] = if escaped { b" \\;\\; " } else { b" ;; " };
        let mut buf = Vec::<u8>::new();
        let mut commands = list_commands(cmd_list_raw(cmdlist)).peekable();
        while let Some(cmd) = commands.next() {
            let this = cmd_print(cmd);
            buf.extend_from_slice(this.as_bytes());
            if let Some(&next) = commands.peek() {
                if !no_groups && (*cmd).group != (*next).group {
                    buf.extend_from_slice(double_separator);
                } else {
                    buf.extend_from_slice(single_separator);
                }
            }
        }
        CString::new(buf).unwrap_or_default()
    }
}

pub unsafe fn cmd_list_first(cmdlist: *mut cmd_list) -> *mut cmd {
    unsafe {
        (*cmd_list_raw(cmdlist))
            .first()
            .map(|cmd| &raw const **cmd as *mut cmd)
            .unwrap_or(null_mut::<cmd>())
    }
}

/// Every command of `cmdlist`, in the order they run.
/// The `at`th command of `cmdlist`, or null when it holds no such command.
pub unsafe fn cmd_list_at(cmdlist: *mut cmd_list, at: usize) -> *mut cmd {
    unsafe {
        (*cmd_list_raw(cmdlist))
            .get(at)
            .map(|cmd| &raw const **cmd as *mut cmd)
            .unwrap_or(::core::ptr::null_mut())
    }
}

pub unsafe fn cmd_list_all(cmdlist: *mut cmd_list) -> Vec<*mut cmd> {
    unsafe { list_commands(cmd_list_raw(cmdlist)).collect() }
}

pub unsafe fn cmd_list_all_have(cmdlist: *mut cmd_list, flag: c_int) -> c_int {
    unsafe {
        for cmd in list_commands(cmd_list_raw(cmdlist)) {
            if !(*cmd).entry.flags & flag != 0 {
                return 0;
            }
        }
        1
    }
}

pub unsafe fn cmd_list_any_have(cmdlist: *mut cmd_list, flag: c_int) -> c_int {
    unsafe {
        for cmd in list_commands(cmd_list_raw(cmdlist)) {
            if (*cmd).entry.flags & flag != 0 {
                return 1;
            }
        }
        0
    }
}

/// Where in `wp` the mouse event happened, or nothing when it fell outside
/// the pane.
pub unsafe fn cmd_mouse_at(
    wp: *mut window_pane,
    m: *mut mouse_event,
    last: c_int,
) -> Option<(u_int, u_int)> {
    unsafe {
        let m = &*m;
        let wp = &*wp;
        let (mut x, mut y) = if last != 0 {
            (m.lx.wrapping_add(m.ox), m.ly.wrapping_add(m.oy))
        } else {
            (m.x.wrapping_add(m.ox), m.y.wrapping_add(m.oy))
        };
        log_debug(
            c"%s: x=%u, y=%u%s".as_ptr(),
            fmt_args![
                c"cmd_mouse_at".as_ptr(),
                x,
                y,
                if last != 0 {
                    c" (last)".as_ptr()
                } else {
                    c"".as_ptr()
                }
            ],
        );
        if m.statusat == 0 && y >= m.statuslines {
            y = y.wrapping_sub(m.statuslines);
        }
        if (x as c_int) < wp.xoff || x as c_int >= wp.xoff.wrapping_add(wp.sx as c_int) {
            return None;
        }
        if (y as c_int) < wp.yoff || y as c_int >= wp.yoff.wrapping_add(wp.sy as c_int) {
            return None;
        }
        Some((
            x.wrapping_sub(wp.xoff as u_int),
            y.wrapping_sub(wp.yoff as u_int),
        ))
    }
}

pub unsafe fn cmd_mouse_window(m: *mut mouse_event, sp: *mut *mut session) -> *mut winlink {
    unsafe {
        let m = &*m;
        if m.valid == 0 || m.s == -1 {
            return null_mut::<winlink>();
        }
        let s = session_find_by_id(m.s as u_int);
        if s.is_null() {
            return null_mut::<winlink>();
        }
        let wl = if m.w == -1 {
            session_get_curw(s)
        } else {
            let w = window_find_by_id(m.w as u_int);
            if w.is_null() {
                return null_mut::<winlink>();
            }
            winlink_find_by_window(&raw mut (*s).windows, w)
        };
        if !sp.is_null() {
            *sp = s;
        }
        wl
    }
}

pub unsafe fn cmd_mouse_pane(
    m: *mut mouse_event,
    sp: *mut *mut session,
    wlp: *mut *mut winlink,
) -> *mut window_pane {
    unsafe {
        let wl = cmd_mouse_window(m, sp);
        if wl.is_null() {
            return null_mut::<window_pane>();
        }
        let wp = if (*m).wp == -1 {
            window_get_active((*wl).window())
        } else {
            let wp = window_pane_find_by_id((*m).wp as u_int);
            if wp.is_null() || window_has_pane((*wl).window(), wp) == 0 {
                return null_mut::<window_pane>();
            }
            wp
        };
        if !wlp.is_null() {
            *wlp = wl;
        }
        wp
    }
}

/// The characters a quoted replacement puts a backslash in front of.
const QUOTE: &[u8] = b"\"\\$;~";

pub unsafe fn cmd_template_replace(
    template: *const c_char,
    s: *const c_char,
    idx: c_int,
) -> CString {
    unsafe {
        if strchr(template, '%' as c_int).is_null() {
            return CStr::from_ptr(template).to_owned();
        }
        let template = CStr::from_ptr(template).to_bytes();
        let argument = CStr::from_ptr(s).to_bytes();
        let mut buf = Vec::<u8>::new();
        let mut replaced = false;
        let mut i = 0;
        while i < template.len() {
            let ch = template[i];
            i += 1;
            if ch == b'%' {
                let next = template.get(i).copied().unwrap_or(b'\0');
                if !(next.is_ascii_digit() && next != b'0' && (next - b'0') as c_int == idx) {
                    // A bare `%%` stands for the argument too, but only the first
                    // one in a template does: the flag is never put back.
                    if next != b'%' || replaced {
                        buf.push(ch);
                        continue;
                    }
                    replaced = true;
                }
                i += 1;
                let quoted = template.get(i).copied().unwrap_or(b'\0') == b'%';
                if quoted {
                    i += 1;
                }
                for &byte in argument {
                    if quoted && QUOTE.contains(&byte) {
                        buf.push(b'\\');
                    }
                    buf.push(byte);
                }
                continue;
            }
            buf.push(ch);
        }
        CString::from_vec_unchecked(buf)
    }
}
