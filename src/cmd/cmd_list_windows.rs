//! `list-windows`: the windows of one session, or of every session under
//! `-a`, one line each through the format engine.
//!
//! Without `-a` the windows are the target session's, in the order
//! `sort_get_winlinks_session` leaves them; with `-a` they are every window
//! of every session, from `sort_get_winlinks`. Either way `-O` picks the
//! order (activity when none is given, the reverse of it under `-r`) and a
//! `-O` value the sort module does not know is the command's one error. Each
//! line is the `-F` template, or the built-in one for the walk that ran,
//! expanded against the window's defaults plus `line`, and with `-f` the
//! filter is expanded first and the line printed only when it is true.
//!
//! Quirk kept: `line` is the *number* of windows the walk found, not the
//! index of the window being printed, so every line of one run carries the
//! same value — where `list-sessions` and `list-clients` count from zero.
//!
//! Coverage exemptions: none. The enumeration and message-protocol constants
//! below are not this module's own, but the tests pin their values through
//! it, so they stay where the transpiler put them.
use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_print};
use crate::fmt_args;
use crate::format::{format_add, format_create, format_defaults, format_expand, format_true};
use crate::sort::{sort_get_winlinks, sort_get_winlinks_session, sort_order_from_string};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
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
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LIST_WINDOWS_WITH_SESSION_TEMPLATE: [::core::ffi::c_char; 127] = unsafe {
    ::core::mem::transmute::<
        [u8; 127],
        [::core::ffi::c_char; 127],
    >(
        *b"#{session_name}:#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] \0",
    )
};
/// The default template of the per-session walk. Upstream spells it as a
/// second `#define` beside the one above; the transpiler inlined it at its
/// only use, and it is a constant again here.
const LIST_WINDOWS_TEMPLATE: &CStr = c"#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] [layout #{window_layout}] #{window_id}#{?window_active, (active),}";

pub(crate) static cmd_list_windows_entry: cmd_entry = cmd_entry {
    name: c"list-windows",
    alias: Some(c"lsw"),
    args: args_parse_t {
        template: c"aF:f:O:rt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-ar] [-F format] [-f filter] [-O order][-t target-session]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_SESSION,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_list_windows_exec,
};

/// The text behind an option, as nothing when the option was not given.
unsafe fn option(args: &args, flag: u8) -> Option<&'static CStr> {
    let s = unsafe { args_get(args, flag) };
    if s.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(s) })
    }
}

/// Whether `ft` passes `filter`: always when there is no filter, and
/// otherwise when the filter expands to something the format engine counts
/// as true.
unsafe fn passes(ft: &mut format_tree, filter: Option<&CStr>) -> bool {
    unsafe {
        match filter {
            None => true,
            Some(filter) => {
                let expanded = format_expand(&mut *ft, filter);
                format_true(Some(&expanded)) != 0
            }
        }
    }
}

fn sorted_winlinks(sort_crit: &mut sort_criteria_t) -> Vec<*mut winlink> {
    unsafe { sort_get_winlinks(sort_crit) }
}

unsafe fn sorted_winlinks_session(
    s: *mut session,
    sort_crit: &mut sort_criteria_t,
) -> Vec<*mut winlink> {
    unsafe { sort_get_winlinks_session(s, sort_crit) }
}

unsafe fn cmd_list_windows_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let target = cmdq_get_target(item);

        let given = option(args, b'F');
        let filter = option(args, b'f');

        let mut sort_crit = sort_criteria_t {
            order: sort_order_from_string(args_get(args, b'O')),
            reversed: 0,
            order_seq: None,
        };
        if sort_crit.order == SORT_END && args_has(args, b'O') != 0 {
            cmdq_error(item, c"invalid sort order".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        sort_crit.reversed = args_has(args, b'r');

        let (winlinks, fallback) = if args_has(args, b'a') != 0 {
            (
                sorted_winlinks(&mut sort_crit),
                LIST_WINDOWS_WITH_SESSION_TEMPLATE.as_ptr(),
            )
        } else {
            (
                sorted_winlinks_session((*target).session(), &mut sort_crit),
                LIST_WINDOWS_TEMPLATE.as_ptr(),
            )
        };
        let template = match given {
            Some(template) => template.as_ptr(),
            None => fallback,
        };

        let n = winlinks.len() as u_int;
        for &wl in &winlinks {
            let s = (*wl).session();
            let mut ft = format_create(cmdq_get_client(&*item), item, FORMAT_NONE, 0);
            format_add(&mut ft, c"line", c"%u".as_ptr(), fmt_args![n]);
            format_defaults(&mut ft, null_mut(), s, wl, null_mut());
            if passes(&mut ft, filter) {
                let line = format_expand(&mut ft, CStr::from_ptr(template));
                cmdq_print(item, c"%s".as_ptr(), fmt_args![line.as_ptr()]);
            }
        }
        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_list_windows.rs"]
mod tests;
