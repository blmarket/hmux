use crate::arguments::args_has;
use crate::cmd::find::cmd_find_from_session;
use crate::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_target, cmdq_insert_hook,
};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::resize::recalculate_sizes;
use crate::server::server_redraw_session;
use crate::session::session_get_curw;
use crate::session::{session_last, session_next, session_previous, session_select};
pub use crate::types::*;
use crate::window::window_set_latest;
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
pub const CMD_TARGET_SESSION_USAGE: &::core::ffi::CStr = c"[-t target-session]";
pub(crate) static cmd_select_window_entry: cmd_entry = {
    cmd_entry {
        name: c"select-window",
        alias: Some(c"selectw"),
        args: args_parse_t {
            template: c"lnpTt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-lnpT] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
pub(crate) static cmd_next_window_entry: cmd_entry = {
    cmd_entry {
        name: c"next-window",
        alias: Some(c"next"),
        args: args_parse_t {
            template: c"at:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
pub(crate) static cmd_previous_window_entry: cmd_entry = {
    cmd_entry {
        name: c"previous-window",
        alias: Some(c"prev"),
        args: args_parse_t {
            template: c"at:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
pub(crate) static cmd_last_window_entry: cmd_entry = {
    cmd_entry {
        name: c"last-window",
        alias: Some(c"last"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
unsafe fn cmd_select_window_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut current: *mut cmd_find_state = cmdq_get_current(item);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut wl: *mut winlink = (*target).winlink();
        let mut s: *mut session = (*target).session();
        let mut next: ::core::ffi::c_int = 0;
        let mut previous: ::core::ffi::c_int = 0;
        let mut last: ::core::ffi::c_int = 0;
        let mut activity: ::core::ffi::c_int = 0;
        next =
            (::core::ptr::eq(cmd_get_entry(self_0), &cmd_next_window_entry)) as ::core::ffi::c_int;
        if args_has(args, 'n' as i32 as u_char) != 0 {
            next = 1 as ::core::ffi::c_int;
        }
        previous = (::core::ptr::eq(cmd_get_entry(self_0), &cmd_previous_window_entry))
            as ::core::ffi::c_int;
        if args_has(args, 'p' as i32 as u_char) != 0 {
            previous = 1 as ::core::ffi::c_int;
        }
        last =
            (::core::ptr::eq(cmd_get_entry(self_0), &cmd_last_window_entry)) as ::core::ffi::c_int;
        if args_has(args, 'l' as i32 as u_char) != 0 {
            last = 1 as ::core::ffi::c_int;
        }
        if next != 0 || previous != 0 || last != 0 {
            activity = args_has(args, 'a' as i32 as u_char);
            if next != 0 {
                if session_next(s, activity) != 0 as ::core::ffi::c_int {
                    cmdq_error(item, c"no next window".as_ptr(), fmt_args![]);
                    return CMD_RETURN_ERROR;
                }
            } else if previous != 0 {
                if session_previous(s, activity) != 0 as ::core::ffi::c_int {
                    cmdq_error(item, c"no previous window".as_ptr(), fmt_args![]);
                    return CMD_RETURN_ERROR;
                }
            } else if session_last(s) != 0 as ::core::ffi::c_int {
                cmdq_error(item, c"no last window".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            cmd_find_from_session(&mut *current, s, 0 as ::core::ffi::c_int);
            server_redraw_session(s);
            cmdq_insert_hook(
                s,
                item,
                current,
                c"after-select-window".as_ptr(),
                fmt_args![],
            );
        } else {
            if args_has(args, 'T' as i32 as u_char) != 0 && wl == session_get_curw(s) {
                if session_last(s) != 0 as ::core::ffi::c_int {
                    cmdq_error(item, c"no last window".as_ptr(), fmt_args![]);
                    return CMD_RETURN_ERROR;
                }
                if (*current).session() == s {
                    cmd_find_from_session(&mut *current, s, 0 as ::core::ffi::c_int);
                }
                server_redraw_session(s);
            } else if session_select(s, (*wl).idx) == 0 as ::core::ffi::c_int {
                cmd_find_from_session(&mut *current, s, 0 as ::core::ffi::c_int);
                server_redraw_session(s);
            }
            cmdq_insert_hook(
                s,
                item,
                current,
                c"after-select-window".as_ptr(),
                fmt_args![],
            );
        }
        if !c.is_null() && !(*c).session.is_null() {
            window_set_latest((*session_get_curw(s)).window(), c);
        }
        recalculate_sizes();
        CMD_RETURN_NORMAL
    }
}
