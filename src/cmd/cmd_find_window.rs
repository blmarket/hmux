//! `find-window`: turns a search into a tree-mode filter and opens the tree on
//! a pane.
//!
//! The command's whole job is translation. `-C`, `-N` and `-T` say whether to
//! look in a pane's content, a window's name and a pane's title — none of them
//! means all three — and `-r` and `-i` say whether the match is a regular
//! expression and whether case matters. What comes out is one format string
//! testing whichever of the three were asked for, joined by `#{||:}`, which is
//! handed to a fresh set of arguments as `-f` and from there to
//! [`WindowMode::Tree`](crate::types::WindowMode). `-Z` is passed on as a bare
//! flag for the mode's own
//! zoom.
//!
//! Quirks kept:
//!
//! * `-r` does two things at once: it selects the `/r` search modifier *and*
//!   drops the `*`s that would otherwise wrap the string, since a regular
//!   expression anchors itself. `-i` alone selects `/i` and keeps the stars.
//! * The search string is interpolated into the format as it stands, so a
//!   string carrying `,`, `}` or `#{` is read as part of the format rather
//!   than as text to look for.
//! * There is no error branch at all: the routine always answers
//!   [`CMD_RETURN_NORMAL`], and it builds the filter and the arguments even
//!   when [`window_pane_set_mode`] is going to refuse them because the pane is
//!   already in that mode.
//!
//! Coverage exemptions: none.

use crate::arguments::{args_create, args_has, args_set, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::cmdq_get_target;
pub use crate::types::*;
use crate::window::window_pane_set_mode;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
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

pub(crate) static cmd_find_window_entry: cmd_entry = cmd_entry {
    name: c"find-window",
    alias: Some(c"findw"),
    args: args_parse_t {
        template: c"CiNrt:TZ",
        lower: 1,
        upper: 1,
        cb: None,
    },
    usage: c"[-CiNrTZ] [-t target-pane] match-string",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_find_window_exec,
};

/// The filter format the tree mode is given, built from the search string and
/// the flags.
///
/// The seven shapes the C spells out as seven `xasprintf` calls are one list
/// and one fold here. Each of `-C`, `-N` and `-T` contributes the format that
/// tests its own place — the pane's content, the window's name, the pane's
/// title — and the list is folded from the right into nested `#{||:a,b}`,
/// which is exactly what those seven strings say: one part on its own, two
/// joined once, three joined as `#{||:C,#{||:N,T}}`.
///
/// The list is never empty, so the fold always has something to start from:
/// a command naming none of the three is read as naming all three.
///
/// The search string is the command's one argument, which the entry's own
/// template makes mandatory — `lower` and `upper` are both one — so the
/// parser has refused the command before exec runs if it is missing, and
/// there is no null to guard here.
unsafe fn cmd_find_window_filter(args: &args) -> Vec<u8> {
    unsafe {
        let s = CStr::from_ptr(args_string(args, 0 as u_int)).to_bytes();
        let regex = args_has(args, b'r') != 0;
        let ignore_case = args_has(args, b'i') != 0;
        let star: &[u8] = match regex {
            true => b"",
            false => b"*",
        };
        let suffix: &[u8] = match (regex, ignore_case) {
            (true, true) => b"/ri",
            (true, false) => b"/r",
            (false, true) => b"/i",
            (false, false) => b"",
        };

        let mut content = args_has(args, b'C') != 0;
        let mut name = args_has(args, b'N') != 0;
        let mut title = args_has(args, b'T') != 0;
        if !content && !name && !title {
            content = true;
            name = true;
            title = true;
        }

        let mut parts: Vec<Vec<u8>> = Vec::new();
        if content {
            parts.push([b"#{C", suffix, b":", s, b"}"].concat());
        }
        for (wanted, place) in [
            (name, &b"#{window_name}"[..]),
            (title, &b"#{pane_title}"[..]),
        ] {
            if wanted {
                parts.push([b"#{m", suffix, b":", star, s, star, b",", place, b"}"].concat());
            }
        }

        let mut filter = parts.pop().expect("no place to search in");
        while let Some(part) = parts.pop() {
            filter = [b"#{||:", &part[..], b",", &filter[..], b"}"].concat();
        }
        filter
    }
}

unsafe fn cmd_find_window_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let target = cmdq_get_target(item);

        let text = cmd_find_window_filter(args);
        let mut filter = Box::new(args_value_t::default());
        filter.value = ArgsValue::String(CString::from_vec_unchecked(text));

        let mut new_args = args_create();
        let new_args_ptr = &raw mut *new_args;
        if args_has(args, b'Z') != 0 {
            args_set(new_args_ptr, b'Z', None, 0);
        }
        args_set(new_args_ptr, b'f', Some(filter), 0);

        window_pane_set_mode(
            (*target).pane(),
            null_mut::<window_pane>(),
            WindowMode::Tree,
            target,
            new_args_ptr,
        );
        CMD_RETURN_NORMAL
    }
}
