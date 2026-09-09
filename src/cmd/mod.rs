//! The commands: the table every tmux command is looked up in, the parsing
//! and printing of a command list, and one private module per command.
//!
//! A command module is reached only through the table below, so the modules
//! are private. Command behavior is tested through the conformance suites.
//! What else the rest of the crate may use is re-exported here.

use crate::options::{OptionsEngine, RustOptionsEngine};
use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
mod cmd_attach_session;
mod cmd_bind_key;
mod cmd_break_pane;
mod cmd_capture_pane;
mod cmd_choose_tree;
mod cmd_command_prompt;
mod cmd_confirm_before;
mod cmd_copy_mode;
mod cmd_detach_client;
mod cmd_display_menu;
mod cmd_display_message;
mod cmd_display_panes;
mod cmd_find_window;
mod cmd_if_shell;
mod cmd_join_pane;
mod cmd_kill_pane;
mod cmd_kill_server;
mod cmd_kill_session;
mod cmd_kill_window;
mod cmd_list_buffers;
mod cmd_list_clients;
mod cmd_list_commands;
mod cmd_list_keys;
mod cmd_list_panes;
mod cmd_list_sessions;
mod cmd_list_windows;
mod cmd_load_buffer;
mod cmd_lock_server;
mod cmd_move_window;
mod cmd_new_session;
mod cmd_new_window;
mod cmd_paste_buffer;
mod cmd_pipe_pane;
mod cmd_refresh_client;
mod cmd_rename_session;
mod cmd_rename_window;
mod cmd_resize_pane;
mod cmd_resize_window;
mod cmd_respawn_pane;
mod cmd_respawn_window;
mod cmd_rotate_window;
mod cmd_run_shell;
mod cmd_save_buffer;
mod cmd_select_layout;
mod cmd_select_pane;
mod cmd_select_window;
mod cmd_send_keys;
mod cmd_server_access;
mod cmd_set_buffer;
mod cmd_set_environment;
mod cmd_set_option;
mod cmd_show_environment;
mod cmd_show_messages;
mod cmd_show_options;
mod cmd_show_prompt_history;
mod cmd_source_file;
mod cmd_split_window;
mod cmd_swap_pane;
mod cmd_swap_window;
mod cmd_switch_client;
mod cmd_unbind_key;
mod cmd_wait_for;

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
    CMD_PARSE_COMMANDS, CMD_PARSE_STRING, CMD_PARSE_SUCCESS, cmd_parse_argument, cmd_parse_command,
    cmd_parse_from_arguments, cmd_parse_from_buffer, cmd_parse_from_file, cmd_parse_from_string,
    cmd_parse_state,
};
#[cfg(test)]
pub(crate) use queue::CmdqListOps;
pub use queue::{
    CmdqItemWeak, cmdq_append, cmdq_item, cmdq_items, cmdq_list, cmdq_next, cmdq_running,
};
pub use queue::{CmdqListRef, CmdqListWeak};

#[cfg(test)]
pub(crate) use find::{cmd_find_best_client, cmd_find_target};
pub(crate) use find::{cmd_find_best_session, cmd_find_log_state_with_window};
pub(crate) use parse::cmd_parse_and_append;
#[cfg(test)]
pub(crate) use parse::{
    CMD_PARSE_ERROR, CMD_PARSE_MAX_ENVIRON_LEN, CMD_PARSE_NOALIAS, CMD_PARSE_ONEGROUP,
    CMD_PARSE_PARSED_COMMANDS, CMD_PARSE_PARSEONLY, CMD_PARSE_VERBOSE, DOUBLE_QUOTES, NONE,
    SINGLE_QUOTES, START,
};
pub(crate) use parse::{
    cmd_parse_from_arguments_impl, cmd_parse_from_buffer_impl, cmd_parse_from_file_impl,
    cmd_parse_from_string_impl,
};
pub use queue::CmdqItemRef;
#[cfg(test)]
pub(crate) use queue::{CMD_AFTERHOOK, CMDQ_STATE_NOHOOKS, CMDQ_WAITING, CmdqType, KEYC_NONE};
pub(crate) use queue::{CmdqStateRef, cmdq_item_ref_of};

pub use cmd_command_prompt::cmd_command_prompt_cdata;
pub(crate) use cmd_command_prompt::{cmd_command_prompt_callback, cmd_command_prompt_free};
pub(crate) use cmd_confirm_before::cmd_confirm_before_callback;
pub use cmd_confirm_before::cmd_confirm_before_data;
pub use cmd_display_panes::{DisplayPanesRef, cmd_display_panes_data};
pub use cmd_if_shell::cmd_if_shell_data;
pub use cmd_load_buffer::cmd_load_buffer_data;
pub use cmd_run_shell::cmd_run_shell_data;
pub use cmd_source_file::cmd_source_file_data;
pub use cmd_wait_for::cmd_wait_for_flush;

use crate::arguments::{args_copy, args_parse, args_print};
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
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::log::log_debug;

pub use crate::consts::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_PANE, CMD_FIND_SESSION,
    CMD_FIND_WINDOW, CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM,
    LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED,
    MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE,
    MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH,
    PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK,
    SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE,
    STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT,
    STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH, STYLE_DEFAULT_SET, STYLE_LIST_FOCUS,
    STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE, STYLE_RANGE_RIGHT,
    STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW, THEME_DARK, THEME_LIGHT,
    THEME_UNKNOWN,
};
use crate::tmux::global_options;
pub use crate::types::{
    ArgsValue, OptionsRef, RustWindowPaneWeak, SessionRef, WindowPane, WindowRef, args,
    args_value_t, mouse_event, u_int,
};
use crate::window::WinlinkRef;
use crate::xmalloc::xasprintf;
use crate::{ArgumentTextCodec, RustArgumentTextCodec};
use crate::{CommandCatalog, CommandEntry, RustCommandCatalog};
use ::core::ffi::{CStr, c_int, c_uint};
use ::std::ffi::CString;

/// The commands of one command list, in the order they run. Each command
/// belongs to the list it sits in.
pub type cmds = Vec<Box<cmd>>;

#[repr(C)]
pub struct cmd {
    pub entry: &'static RustCommandEntry,
    pub args: Option<Box<args>>,
    pub group: u_int,
    pub file: Option<CString>,
    pub line: u_int,
    pub parse_flags: c_int,
}

impl crate::CommandGroupState for cmd {
    fn command_group(&self) -> u_int {
        self.group
    }

    fn set_command_group(&mut self, group: u_int) {
        self.group = group;
    }
}

impl crate::CommandSourceState for cmd {
    fn command_source(&self) -> crate::CommandSource<'_> {
        crate::CommandSource {
            file: self.file.as_deref(),
            line: self.line,
        }
    }

    fn set_command_source(&mut self, source: crate::CommandSource<'_>) {
        self.file = source.file.map(CStr::to_owned);
        self.line = source.line;
    }
}

impl crate::CommandParseFlagsState for cmd {
    fn command_parse_flags(&self) -> c_int {
        self.parse_flags
    }

    fn set_command_parse_flags(&mut self, flags: c_int) {
        self.parse_flags = flags;
    }
}

impl crate::Command for cmd {
    type Arguments = args;
    type Entry = RustCommandEntry;

    fn command_entry(&self) -> &'static RustCommandEntry {
        self.entry
    }

    fn command_arguments(&self) -> Option<&Self::Arguments> {
        self.args.as_deref()
    }

    fn command_arguments_mut(&mut self) -> Option<&mut Self::Arguments> {
        self.args.as_deref_mut()
    }
}

/// Every command the server knows.
pub static cmd_table: &[&RustCommandEntry] = &[
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
static CMD_LIST_NEXT_GROUP: AtomicU32 = AtomicU32::new(1);

/// The number the next command list, or the next move onto one, is stamped
/// with. Every command in a list carries its list's number, and a step
/// between two of them is what `;;` prints as.
fn next_group() -> u_int {
    CMD_LIST_NEXT_GROUP.fetch_add(1, Ordering::Relaxed)
}

/// Moves everything `from` holds onto the end of `list`, leaving `from` empty.
fn list_concat(list: &mut cmds, from: &mut cmds) {
    let moved = core::mem::take(from);
    list.extend(moved);
}

fn cmd_list_commands(cmdlist: &cmd_list) -> &cmds {
    cmdlist
        .list
        .as_deref()
        .expect("command list has been dropped")
}

fn cmd_list_commands_mut(cmdlist: &mut cmd_list) -> &mut cmds {
    cmdlist
        .list
        .as_deref_mut()
        .expect("command list has been dropped")
}

/// The commands in `list`, in the order they run, walked the way the C's
/// `TAILQ_FOREACH` walked them.
fn list_commands(list: &cmds) -> impl Iterator<Item = &cmd> + '_ {
    list.iter().map(Box::as_ref)
}

fn list_commands_mut(list: &mut cmds) -> impl Iterator<Item = &mut cmd> + '_ {
    list.iter_mut().map(Box::as_mut)
}

pub unsafe fn cmd_log_argv(argv: &[CString], fmt: &CStr, args: &[FmtArg]) {
    unsafe {
        let prefix = format_alloc(fmt, args);
        for (i, arg) in argv.iter().enumerate() {
            log_debug(
                c"%s: argv[%d]=%s",
                fmt_args![prefix.as_c_str(), i as c_int, arg.as_c_str()],
            );
        }
    }
}

pub(crate) fn cmd_pack_argv_impl(argv: &[CString], out: &mut [u8]) -> c_int {
    if argv.is_empty() {
        return 0;
    }
    unsafe { cmd_log_argv(argv, c"%s", fmt_args![c"cmd_pack_argv"]) };
    let Some(first) = out.first_mut() else {
        return -1;
    };
    *first = 0;
    let mut rest = out;
    for arg in argv {
        let bytes = arg.as_bytes_with_nul();
        if bytes.len() > rest.len() {
            if let Some((last, prefix)) = rest.split_last_mut() {
                let copied = prefix.len().min(arg.as_bytes().len());
                prefix[..copied].copy_from_slice(&arg.as_bytes()[..copied]);
                *last = 0;
            }
            return -1;
        }
        let (packed, remaining) = rest.split_at_mut(bytes.len());
        packed.copy_from_slice(bytes);
        rest = remaining;
    }
    0
}

pub(crate) fn cmd_unpack_argv_impl(packed: &mut [u8], argc: c_int) -> Option<Vec<CString>> {
    if argc == 0 {
        return Some(Vec::new());
    }
    if !(0..=1000).contains(&argc) {
        return None;
    }
    if packed.is_empty() {
        return None;
    }
    *packed.last_mut().unwrap() = 0;
    let mut at = 0;
    let mut argv = Vec::with_capacity(argc as usize);
    for _ in 0..argc {
        if at == packed.len() {
            return None;
        }
        let end = packed[at..].iter().position(|&byte| byte == 0)?;
        argv.push(CString::new(&packed[at..at + end]).unwrap());
        at += end + 1;
    }
    unsafe { cmd_log_argv(&argv, c"%s", fmt_args![c"cmd_unpack_argv"]) };
    Some(argv)
}

pub(crate) fn cmd_stringify_argv_impl(argv: &[CString]) -> CString {
    if argv.is_empty() {
        return CString::default();
    }
    let mut out = Vec::<u8>::new();
    for (i, arg) in argv.iter().enumerate() {
        let escaped = RustArgumentTextCodec.escape(arg.as_c_str());
        unsafe {
            log_debug(
                c"%s: %u %s = %s",
                fmt_args![
                    c"cmd_stringify_argv",
                    i as c_uint,
                    arg.as_c_str(),
                    escaped.as_c_str()
                ],
            );
        }
        if i != 0 {
            out.push(b' ');
        }
        out.extend_from_slice(escaped.as_bytes());
    }
    CString::new(out).expect("escaped arguments contain no NUL bytes")
}

pub fn cmd_get_entry(cmd: &cmd) -> &'static RustCommandEntry {
    crate::Command::command_entry(cmd)
}

pub fn cmd_get_args(cmd: &cmd) -> &args {
    crate::Command::command_arguments(cmd).expect("the command carries arguments")
}

/// The same, for a caller that means to change what the command carries.
pub fn cmd_get_args_mut(cmd: &mut cmd) -> &mut args {
    crate::Command::command_arguments_mut(cmd).expect("the command carries arguments")
}

pub fn cmd_get_group(cmd: &cmd) -> u_int {
    crate::CommandGroupState::command_group(cmd)
}

/// Where the command was parsed from: the file, if it came from one, and
/// the line in it.
pub fn cmd_get_source(cmd: &cmd) -> (Option<&CStr>, u_int) {
    let source = crate::CommandSourceState::command_source(cmd);
    (source.file, source.line)
}

pub fn cmd_get_parse_flags(cmd: &cmd) -> c_int {
    crate::CommandParseFlagsState::command_parse_flags(cmd)
}

pub unsafe fn cmd_get_alias(name: &CStr) -> Option<CString> {
    unsafe {
        global_options
            .as_ref()
            .expect("global options are initialized")
            .with_entry(c"command-alias", true, |entry| {
                let entry = entry?;
                let wanted = name.to_bytes();
                for index in RustOptionsEngine.array_indices(entry) {
                    let value = RustOptionsEngine
                        .value_string(RustOptionsEngine.array_get(entry, index).unwrap())
                        .to_bytes();
                    if let Some(n) = value.iter().position(|&byte| byte == b'=')
                        && &value[..n] == wanted
                    {
                        return Some(
                            CString::new(&value[n + 1..]).expect("option text has no NUL"),
                        );
                    }
                }
                None
            })
    }
}

pub fn cmd_find(name: &CStr) -> Result<&'static RustCommandEntry, CString> {
    RustCommandCatalog.find(name)
}

pub unsafe fn cmd_parse(
    values: &[args_value_t],
    file: Option<&CStr>,
    line: u_int,
    parse_flags: c_int,
) -> Result<Box<cmd>, CString> {
    unsafe {
        if values.is_empty() || !matches!(&values[0].value, ArgsValue::String(_)) {
            return Err(xasprintf(c"no command", fmt_args![]));
        }
        let entry = cmd_find(values[0].value.string())?;
        let mut error: Option<CString> = None;
        let Some(args) = args_parse(&entry.argument_parse(), values, &mut error) else {
            return Err(match error {
                Some(error) => {
                    xasprintf(c"command %s: %s", fmt_args![entry.name(), error.as_c_str()])
                }
                None => xasprintf(c"usage: %s %s", fmt_args![entry.name(), entry.usage()]),
            });
        };
        Ok(Box::new(cmd {
            entry,
            args: Some(args),
            group: 0,
            file: file.map(CStr::to_owned),
            line,
            parse_flags,
        }))
    }
}

pub unsafe fn cmd_copy(from: &cmd, argv: &[CString]) -> Box<cmd> {
    unsafe {
        Box::new(cmd {
            entry: from.entry,
            args: Some(args_copy(cmd_get_args(from), argv)),
            group: 0,
            file: from.file.clone(),
            line: from.line,
            parse_flags: 0,
        })
    }
}

pub unsafe fn cmd_print(cmd: &cmd) -> CString {
    unsafe {
        let s = args_print(cmd_get_args(cmd));

        if !s.as_bytes().is_empty() {
            xasprintf(c"%s %s", fmt_args![cmd.entry.name(), s.as_c_str()])
        } else {
            cmd.entry.name().to_owned()
        }
    }
}

/// Where in `wp` the mouse event happened, or nothing when it fell outside
/// the pane.
pub fn cmd_mouse_at(
    wp: &impl crate::WindowPane,
    m: &mouse_event,
    last: c_int,
) -> Option<(u_int, u_int)> {
    let (x, mut y) = if last != 0 {
        (m.lx.wrapping_add(m.ox), m.ly.wrapping_add(m.oy))
    } else {
        (m.x.wrapping_add(m.ox), m.y.wrapping_add(m.oy))
    };
    unsafe {
        log_debug(
            c"%s: x=%u, y=%u%s",
            fmt_args![
                c"cmd_mouse_at",
                x,
                y,
                if last != 0 { c" (last)" } else { c"" }
            ],
        );
    }
    if m.statusat == 0 && y >= m.statuslines {
        y = y.wrapping_sub(m.statuslines);
    }
    if (x as c_int) < wp.geometry().x
        || x as c_int >= wp.geometry().x.wrapping_add(wp.geometry().width as c_int)
    {
        return None;
    }
    if (y as c_int) < wp.geometry().y
        || y as c_int >= wp.geometry().y.wrapping_add(wp.geometry().height as c_int)
    {
        return None;
    }
    Some((
        x.wrapping_sub(wp.geometry().x as u_int),
        y.wrapping_sub(wp.geometry().y as u_int),
    ))
}

/// The session a mouse event names, paired with its matching link if present.
pub(crate) unsafe fn cmd_mouse_window(m: &mouse_event) -> Option<(SessionRef, Option<WinlinkRef>)> {
    unsafe {
        if m.valid == 0 || m.s == -1 {
            return None;
        }
        let owner = SessionRef::find_by_id(m.s as u_int)?;
        let s = owner.as_session();
        let index = if m.w == -1 {
            s.curw_idx
        } else {
            let window = WindowRef::find_by_id(m.w as u_int)?;
            s.windows.iter().find_map(|(&index, link)| {
                link.window_handle()
                    .is_some_and(|linked| linked.ptr_eq(&window))
                    .then_some(index)
            })
        };
        let link = index.and_then(|index| WinlinkRef::new(owner.clone(), index));
        Some((owner, link))
    }
}

/// The pane a mouse event names, with the session and window link it sits in.
pub(crate) unsafe fn cmd_mouse_pane(
    m: &mouse_event,
) -> Option<(SessionRef, WinlinkRef, RustWindowPaneWeak)> {
    unsafe {
        let (session, link) = cmd_mouse_window(m)?;
        let link = link?;
        let window = link.get()?.window_handle()?.clone();
        let id = if m.wp == -1 {
            window.active_pane_id()?
        } else {
            m.wp as u_int
        };
        let pane = window.pane_by_id(id)?;
        Some((session, link, pane))
    }
}

/// The characters a quoted replacement puts a backslash in front of.
const QUOTE: &[u8] = b"\"\\$;~";

pub fn cmd_template_replace(template: &CStr, s: &CStr, idx: c_int) -> CString {
    if !template.to_bytes().contains(&b'%') {
        return template.to_owned();
    }
    let template = template.to_bytes();
    let argument = s.to_bytes();
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
    CString::new(buf).expect("template replacement contains no NUL bytes")
}

#[cfg(test)]
pub(crate) use crate::tests::test_coverage_cmd::{
    cmd_pack_argv, cmd_stringify_argv, cmd_unpack_argv,
};

impl CmdListRef {
    pub(crate) fn empty() -> CmdListRef {
        let list = Box::new(cmds::new());
        CmdListRef::new(cmd_list {
            group: next_group(),
            list: Some(list),
        })
    }
    pub fn append(&self, cmd: Box<cmd>) {
        let cmdlist = self;

        cmdlist.with_mut(|list| {
            let mut cmd = cmd;
            cmd.group = list.group;
            cmd_list_commands_mut(list).push(cmd);
        });
    }
    pub fn append_all(&self, from: &CmdListRef) {
        let cmdlist = self;

        let group = cmdlist.with(|list| list.group);
        from.with_mut(|from| {
            for cmd in list_commands_mut(cmd_list_commands_mut(from)) {
                cmd.group = group;
            }
        });
        cmdlist.with_mut(|cmdlist| {
            from.with_mut(|from| {
                list_concat(cmd_list_commands_mut(cmdlist), cmd_list_commands_mut(from));
            });
        });
    }
    pub fn move_from(&self, from: &CmdListRef) {
        let cmdlist = self;

        cmdlist.with_mut(|cmdlist| {
            from.with_mut(|from| {
                list_concat(cmd_list_commands_mut(cmdlist), cmd_list_commands_mut(from));
            });
            cmdlist.group = next_group();
        });
    }
    pub(crate) unsafe fn copy_with_arguments(&self, argv: &[CString]) -> CmdListRef {
        let cmdlist = self;

        unsafe {
            let mut group = cmdlist.with(|list| list.group);
            let s = cmdlist.print(0);
            log_debug(c"%s: %s", fmt_args![c"cmd_list_copy", s.as_c_str()]);
            let new_cmdlist = CmdListRef::empty();
            cmdlist.with(|cmdlist| {
                for cmd in list_commands(cmd_list_commands(cmdlist)) {
                    if cmd.group != group {
                        new_cmdlist.with_mut(|new_cmdlist| new_cmdlist.group = next_group());
                        group = cmd.group;
                    }
                    new_cmdlist.append(cmd_copy(cmd, argv));
                }
            });
            let s = new_cmdlist.print(0);
            log_debug(c"%s: %s", fmt_args![c"cmd_list_copy", s.as_c_str()]);
            new_cmdlist
        }
    }
    pub unsafe fn print(&self, flags: c_int) -> CString {
        let cmdlist = self;

        unsafe {
            cmdlist.with(|cmdlist| {
                let escaped = flags & CMD_LIST_PRINT_ESCAPED != 0;
                let no_groups = flags & CMD_LIST_PRINT_NO_GROUPS != 0;
                let single_separator: &[u8] = if escaped { b" \\; " } else { b" ; " };
                let double_separator: &[u8] = if escaped { b" \\;\\; " } else { b" ;; " };
                let mut buf = Vec::<u8>::new();
                let mut commands = list_commands(cmd_list_commands(cmdlist)).peekable();
                while let Some(cmd) = commands.next() {
                    let this = cmd_print(cmd);
                    buf.extend_from_slice(this.as_bytes());
                    if let Some(next) = commands.peek() {
                        if !no_groups && cmd.group != next.group {
                            buf.extend_from_slice(double_separator);
                        } else {
                            buf.extend_from_slice(single_separator);
                        }
                    }
                }
                CString::new(buf).unwrap_or_default()
            })
        }
    }
    /// Maps commands in list order while their list remains immutably borrowed.
    pub fn map<T>(&self, map: impl FnMut(&cmd) -> T) -> Vec<T> {
        let cmdlist = self;

        cmdlist.with(|cmdlist| list_commands(cmd_list_commands(cmdlist)).map(map).collect())
    }
    pub fn all_have(&self, flag: c_int) -> c_int {
        let cmdlist = self;

        cmdlist.with(|cmdlist| {
            for cmd in list_commands(cmd_list_commands(cmdlist)) {
                if !cmd.entry.flags() & flag != 0 {
                    return 0;
                }
            }
            1
        })
    }
    pub fn any_have(&self, flag: c_int) -> c_int {
        let cmdlist = self;

        cmdlist.with(|cmdlist| {
            for cmd in list_commands(cmd_list_commands(cmdlist)) {
                if cmd.entry.flags() & flag != 0 {
                    return 1;
                }
            }
            0
        })
    }
}

pub type cmd_find_type = core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: core::ffi::c_int,
}
impl cmd_entry_flag {
    /// Builds a source or target lookup descriptor for a command entry.
    pub const fn new(
        flag: core::ffi::c_char,
        type_0: cmd_find_type,
        flags: core::ffi::c_int,
    ) -> Self {
        Self {
            flag,
            type_0,
            flags,
        }
    }
}
pub type cmd_retval = core::ffi::c_int;
pub use crate::command_entry::RustCommandEntry;
pub type cmd_entry = RustCommandEntry;

#[repr(C)]
pub struct cmd_list {
    pub group: u_int,
    pub list: Option<Box<cmds>>,
}

impl crate::CommandListGroupState for cmd_list {
    fn command_list_group(&self) -> u_int {
        self.group
    }

    fn set_command_list_group(&mut self, group: u_int) {
        self.group = group;
    }
}

impl crate::CommandList for cmd_list {
    type Command = cmd;

    fn command_count(&self) -> usize {
        self.list.as_deref().map_or(0, Vec::len)
    }

    fn command_at(&self, index: usize) -> Option<&Self::Command> {
        self.list.as_deref()?.get(index).map(Box::as_ref)
    }

    fn command_at_mut(&mut self, index: usize) -> Option<&mut Self::Command> {
        self.list.as_deref_mut()?.get_mut(index).map(Box::as_mut)
    }
}

/// A strong owner of a parsed command list.
///
/// A shared owner of a command list. Commands remain borrowed through the
/// list's guard, and callers can clone the owner to retain a command while
/// mutating the queue item that selected it.
#[derive(Clone)]
pub struct CmdListRef(Rc<RefCell<cmd_list>>);

impl CmdListRef {
    pub(crate) fn new(value: cmd_list) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn with<R>(&self, operation: impl FnOnce(&cmd_list) -> R) -> R {
        operation(&self.0.borrow())
    }

    pub(crate) fn with_mut<R>(&self, operation: impl FnOnce(&mut cmd_list) -> R) -> R {
        operation(&mut self.0.borrow_mut())
    }

    /// Borrows one command while retaining the list's shared borrow guard.
    pub(crate) fn command(&self, index: usize) -> Option<std::cell::Ref<'_, cmd>> {
        std::cell::Ref::filter_map(self.0.borrow(), |list| {
            crate::CommandList::command_at(list, index)
        })
        .ok()
    }

    /// Borrows one command exclusively through the list's mutable guard.
    #[cfg(test)]
    pub(crate) fn command_mut(&self, index: usize) -> Option<std::cell::RefMut<'_, cmd>> {
        std::cell::RefMut::filter_map(self.0.borrow_mut(), |list| {
            crate::CommandList::command_at_mut(list, index)
        })
        .ok()
    }
}

/// The state a `source-file` run carries between the reads it starts. Several
/// client files may be reading for one run, so they share it and the last one
/// gone takes it with them.
#[derive(Clone)]
pub struct SourceFileRef(Rc<RefCell<cmd_source_file_data>>);

impl SourceFileRef {
    pub(crate) fn new(value: cmd_source_file_data) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn with<R>(&self, operation: impl FnOnce(&cmd_source_file_data) -> R) -> R {
        operation(&self.0.borrow())
    }

    pub(crate) fn with_mut<R>(&self, operation: impl FnOnce(&mut cmd_source_file_data) -> R) -> R {
        operation(&mut self.0.borrow_mut())
    }
}

impl fmt::Debug for SourceFileRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SourceFileRef")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}

impl PartialEq for CmdListRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CmdListRef {}

impl fmt::Debug for CmdListRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CmdListRef")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}
