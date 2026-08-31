//! `list-commands`: the table of every command the server knows, printed one
//! line per command through the format engine.
//!
//! With no argument the whole table is walked in the order it is declared;
//! with one, `cmd_find` resolves it by name or alias and only that command is
//! printed, an unknown name being the command's one error. Each line is the
//! `-F` template, or the built-in one, expanded against three variables the
//! entry supplies — its name, its alias and its usage — and a line that
//! expands to nothing is not printed at all.
//!
//! The command table stays the array the rest of the crate reads, walked to
//! the null it ends with.
//!
//! Coverage exemptions: none. The enumeration and message-protocol constants
//! below are not this module's own, but the tests pin their values through
//! it, so they stay where the transpiler put them.
use crate::arguments::{args_get, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_print};
use crate::cmd::{cmd_find, cmd_get_args, cmd_table};
use crate::fmt_args;
use crate::format::{format_add, format_create, format_defaults, format_expand};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
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
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LIST_COMMANDS_TEMPLATE: [::core::ffi::c_char; 91] = unsafe {
    ::core::mem::transmute::<
        [u8; 91],
        [::core::ffi::c_char; 91],
    >(
        *b"#{command_list_name}#{?command_list_alias, (#{command_list_alias}),} #{command_list_usage}\0",
    )
};
pub(crate) static cmd_list_commands_entry: cmd_entry = cmd_entry {
    name: c"list-commands",
    alias: Some(c"lscm"),
    args: args_parse_t {
        template: c"F:",
        lower: 0,
        upper: 1,
        cb: None,
    },
    usage: c"[-F format] [command]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_STARTSERVER | CMD_AFTERHOOK,
    exec: cmd_list_commands,
};

/// The text an entry carries under its alias, as the empty string when it
/// carries none — which is what the format engine is handed for a command
/// with no alias.
fn text_or_empty(s: Option<&'static CStr>) -> &'static CStr {
    s.unwrap_or(c"")
}

/// Prints the one line `entry` gives under `template`, unless expanding it
/// left nothing to print.
unsafe fn cmd_list_single_command(
    entry: *const cmd_entry,
    ft: &mut format_tree,
    template: *const c_char,
    item: *mut cmdq_item,
) {
    unsafe {
        format_add(
            ft,
            c"command_list_name",
            c"%s".as_ptr(),
            fmt_args![(*entry).name],
        );
        format_add(
            ft,
            c"command_list_alias",
            c"%s".as_ptr(),
            fmt_args![text_or_empty((*entry).alias)],
        );
        format_add(
            ft,
            c"command_list_usage",
            c"%s".as_ptr(),
            fmt_args![(*entry).usage],
        );

        let line = format_expand(ft, CStr::from_ptr(template));
        if !line.as_bytes().is_empty() {
            cmdq_print(item, c"%s".as_ptr(), fmt_args![line.as_ptr()]);
        }
    }
}

unsafe fn cmd_list_commands(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let mut template = args_get(args, b'F');
        if template.is_null() {
            template = LIST_COMMANDS_TEMPLATE.as_ptr();
        }

        let mut ft = format_create(cmdq_get_client(&*item), item, FORMAT_NONE, 0);
        format_defaults(&mut ft, null_mut(), null_mut(), null_mut(), null_mut());

        let command = args_string(args, 0);
        if command.is_null() {
            for &entry in cmd_table {
                cmd_list_single_command(entry, &mut ft, template, item);
            }
        } else {
            let mut cause = None;
            let entry = cmd_find(command, &mut cause);
            if entry.is_null() {
                let cause = cause.unwrap();
                cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
            cmd_list_single_command(entry, &mut ft, template, item);
        }

        CMD_RETURN_NORMAL
    }
}
