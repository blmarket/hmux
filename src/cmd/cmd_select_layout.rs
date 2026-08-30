use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::layout::layout_set_lookup;
use crate::layout::{layout_dump, layout_parse};
use crate::layout::{layout_root_ptr, layout_spread_out};
use crate::layout::{layout_set_next, layout_set_previous, layout_set_select};
use crate::notify::notify_window;
use crate::resize::recalculate_sizes;
use crate::server::{server_redraw_window, server_unzoom_window};
pub use crate::types::*;
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
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_TARGET_WINDOW_USAGE: &::core::ffi::CStr = c"[-t target-window]";
pub(crate) static cmd_select_layout_entry: cmd_entry = {
    cmd_entry {
        name: c"select-layout",
        alias: Some(c"selectl"),
        args: args_parse_t {
            template: c"Enopt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-Enop] [-t target-pane] [layout-name]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_select_layout_exec,
    }
};
pub(crate) static cmd_next_layout_entry: cmd_entry = {
    cmd_entry {
        name: c"next-layout",
        alias: Some(c"nextl"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE,
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
        flags: CMD_AFTERHOOK,
        exec: cmd_select_layout_exec,
    }
};
pub(crate) static cmd_previous_layout_entry: cmd_entry = {
    cmd_entry {
        name: c"previous-layout",
        alias: Some(c"prevl"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE,
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
        flags: CMD_AFTERHOOK,
        exec: cmd_select_layout_exec,
    }
};
unsafe fn cmd_select_layout_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut wl: *mut winlink = (*target).winlink();
        let mut w: *mut window = (*wl).window();
        let mut wp: *mut window_pane = (*target).pane();
        let mut layoutname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut oldlayout: Option<::std::ffi::CString> = None;
        let mut next: ::core::ffi::c_int = 0;
        let mut previous: ::core::ffi::c_int = 0;
        let mut layout: ::core::ffi::c_int = 0;
        server_unzoom_window(w);
        next =
            (::core::ptr::eq(cmd_get_entry(self_0), &cmd_next_layout_entry)) as ::core::ffi::c_int;
        if args_has(args, 'n' as i32 as u_char) != 0 {
            next = 1 as ::core::ffi::c_int;
        }
        previous = (::core::ptr::eq(cmd_get_entry(self_0), &cmd_previous_layout_entry))
            as ::core::ffi::c_int;
        if args_has(args, 'p' as i32 as u_char) != 0 {
            previous = 1 as ::core::ffi::c_int;
        }
        oldlayout = (*w).old_layout.take();
        (*w).old_layout = layout_dump(w, layout_root_ptr(&(*w).layout_root));
        if next != 0 || previous != 0 {
            if next != 0 {
                layout_set_next(w);
            } else {
                layout_set_previous(w);
            }
        } else if args_has(args, 'E' as i32 as u_char) != 0 {
            layout_spread_out(wp);
        } else {
            if args_count(args) != 0 as u_int {
                layoutname = args_string(args, 0 as u_int);
            } else if args_has(args, 'o' as i32 as u_char) != 0 {
                layoutname = cstr_ptr(&oldlayout);
            } else {
                layoutname = ::core::ptr::null::<::core::ffi::c_char>();
            }
            if args_has(args, 'o' as i32 as u_char) == 0 {
                if layoutname.is_null() {
                    layout = (*w).lastlayout;
                } else {
                    layout = layout_set_lookup(layoutname);
                }
                if layout != -(1 as ::core::ffi::c_int) {
                    layout_set_select(w, layout as u_int);
                    current_block = 16395913426687009730;
                } else {
                    current_block = 15768484401365413375;
                }
            } else {
                current_block = 15768484401365413375;
            }
            match current_block {
                16395913426687009730 => {}
                _ => {
                    if !layoutname.is_null() {
                        if let Err(cause) = layout_parse(w, layoutname) {
                            cmdq_error(
                                item,
                                c"%s: %s".as_ptr(),
                                fmt_args![cause.as_ptr(), layoutname],
                            );
                            (*w).old_layout = oldlayout;
                            return CMD_RETURN_ERROR;
                        }
                    } else {
                        return CMD_RETURN_NORMAL;
                    }
                }
            }
        }
        recalculate_sizes();
        server_redraw_window(w);
        notify_window(c"window-layout-changed".as_ptr(), w);
        CMD_RETURN_NORMAL
    }
}
