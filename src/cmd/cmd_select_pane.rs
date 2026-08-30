use crate::arguments::{args_get, args_has};
use crate::cmd::find::{cmd_find_from_winlink, cmd_find_from_winlink_pane};
use crate::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_target, cmdq_insert_hook, cmdq_print,
};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::notify::notify_pane;
use crate::options::{options_get_string, options_ptr, options_set_string};
use crate::screen::screen_set_title;
use crate::server::client_walk;
use crate::server::{marked_pane, server_is_marked, server_set_marked};
use crate::server::{server_check_marked, server_clear_marked};
use crate::server::{server_client_get_pane, server_client_set_pane};
use crate::server::{
    server_redraw_client, server_redraw_window, server_redraw_window_borders, server_status_window,
};
use crate::session::session_get_curw;
use crate::session::session_has;
use crate::tty::tty_window_bigger;
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::window_get_active;
use crate::window::{
    window_count_panes, window_pane_find_down, window_pane_find_left, window_pane_find_right,
    window_pane_find_up, window_pane_is_floating, window_pane_stack_first, window_pane_visible,
    window_panes_next, window_panes_prev, window_pop_zoom, window_push_zoom,
    window_redraw_active_switch, window_set_active_pane,
};
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
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_INPUTOFF: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_ACTIVEPANE: ::core::ffi::c_ulonglong = 0x80000000 as ::core::ffi::c_ulonglong;
pub(crate) static cmd_select_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"select-pane",
        alias: Some(c"selectp"),
        args: args_parse_t {
            template: c"DdegLlMmP:RT:t:UZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DdeLlMmRUZ] [-T title] [-t target-pane]",
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
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_select_pane_exec,
    }
};
pub(crate) static cmd_last_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"last-pane",
        alias: Some(c"lastp"),
        args: args_parse_t {
            template: c"det:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-deZ] [-t target-window]",
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
        exec: cmd_select_pane_exec,
    }
};
unsafe fn cmd_select_pane_redraw(mut w: *mut window) {
    unsafe {
        for c in client_walk() {
            if !((*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0) {
                if (*session_get_curw((*c).session)).window() == w
                    && tty_window_bigger(&raw mut (*c).tty) != 0
                {
                    server_redraw_client(c);
                } else {
                    if (*session_get_curw((*c).session)).window() == w {
                        (*c).flags |= CLIENT_REDRAWBORDERS as uint64_t;
                    }
                    if session_has((*c).session, w) != 0 {
                        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
                    }
                }
            }
        }
    }
}
unsafe fn cmd_select_pane_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut entry: *const cmd_entry = cmd_get_entry(self_0);
        let mut current: *mut cmd_find_state = cmdq_get_current(item);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut wl: *mut winlink = (*target).winlink();
        let mut w: *mut window = (*wl).window();
        let mut s: *mut session = (*target).session();
        let mut wp: *mut window_pane = (*target).pane();
        let mut activewp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut markedwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut oo: *mut options = options_ptr(&(*wp).options);
        let mut style: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        if entry == &raw const cmd_last_pane_entry || args_has(args, 'l' as i32 as u_char) != 0 {
            lastwp = window_pane_stack_first(w, PaneStack::LastUsed);
            if lastwp.is_null() && window_count_panes(w, 1 as ::core::ffi::c_int) == 2 as u_int {
                lastwp = window_panes_prev(w, window_get_active(w));
                if lastwp.is_null() {
                    lastwp = window_panes_next(w, window_get_active(w));
                }
            }
            if lastwp.is_null() {
                cmdq_error(item, c"no last pane".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            if args_has(args, 'e' as i32 as u_char) != 0 {
                (*lastwp).flags &= !PANE_INPUTOFF;
                server_redraw_window_borders((*lastwp).window);
                server_status_window((*lastwp).window);
            } else if args_has(args, 'd' as i32 as u_char) != 0 {
                (*lastwp).flags |= PANE_INPUTOFF;
                server_redraw_window_borders((*lastwp).window);
                server_status_window((*lastwp).window);
            } else {
                if window_push_zoom(
                    w,
                    0 as ::core::ffi::c_int,
                    args_has(args, 'Z' as i32 as u_char),
                ) != 0
                {
                    server_redraw_window(w);
                }
                window_redraw_active_switch(w, lastwp);
                if window_set_active_pane(w, lastwp, 1 as ::core::ffi::c_int) != 0 {
                    cmd_find_from_winlink(&mut *current, wl, 0 as ::core::ffi::c_int);
                    cmd_select_pane_redraw(w);
                }
                if window_pop_zoom(w) != 0 {
                    server_redraw_window(w);
                }
            }
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'm' as i32 as u_char) != 0 || args_has(args, 'M' as i32 as u_char) != 0 {
            if args_has(args, 'm' as i32 as u_char) != 0 && window_pane_visible(wp) == 0 {
                return CMD_RETURN_NORMAL;
            }
            if server_check_marked() != 0 {
                lastwp = marked_pane.pane();
            } else {
                lastwp = ::core::ptr::null_mut::<window_pane>();
            }
            if args_has(args, 'M' as i32 as u_char) != 0 || server_is_marked(s, wl, wp) != 0 {
                server_clear_marked();
            } else {
                server_set_marked(s, wl, wp);
            }
            markedwp = marked_pane.pane();
            if !lastwp.is_null() {
                (*lastwp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
                server_redraw_window_borders((*lastwp).window);
                server_status_window((*lastwp).window);
            }
            if !markedwp.is_null() {
                (*markedwp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
                server_redraw_window_borders((*markedwp).window);
                server_status_window((*markedwp).window);
            }
            if window_pane_is_floating(wp) != 0 {
                window_redraw_active_switch(w, wp);
                window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
            }
            return CMD_RETURN_NORMAL;
        }
        style = args_get(args, 'P' as i32 as u_char);
        if !style.is_null() {
            o = options_set_string(
                oo,
                c"window-style".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![style],
            );
            if o.is_null() {
                cmdq_error(item, c"bad style: %s".as_ptr(), fmt_args![style]);
                return CMD_RETURN_ERROR;
            }
            options_set_string(
                oo,
                c"window-active-style".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![style],
            );
            (*wp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
        if args_has(args, 'g' as i32 as u_char) != 0 {
            cmdq_print(
                item,
                c"%s".as_ptr(),
                fmt_args![options_get_string(oo, c"window-style".as_ptr(),)],
            );
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'L' as i32 as u_char) != 0 {
            window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            wp = window_pane_find_left(wp);
            window_pop_zoom(w);
        } else if args_has(args, 'R' as i32 as u_char) != 0 {
            window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            wp = window_pane_find_right(wp);
            window_pop_zoom(w);
        } else if args_has(args, 'U' as i32 as u_char) != 0 {
            window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            wp = window_pane_find_up(wp);
            window_pop_zoom(w);
        } else if args_has(args, 'D' as i32 as u_char) != 0 {
            window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            wp = window_pane_find_down(wp);
            window_pop_zoom(w);
        }
        if wp.is_null() {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'e' as i32 as u_char) != 0 {
            (*wp).flags &= !PANE_INPUTOFF;
            server_redraw_window_borders((*wp).window);
            server_status_window((*wp).window);
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'd' as i32 as u_char) != 0 {
            (*wp).flags |= PANE_INPUTOFF;
            server_redraw_window_borders((*wp).window);
            server_status_window((*wp).window);
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'T' as i32 as u_char) != 0 {
            let title = format_single_from_target(
                item,
                ::core::ffi::CStr::from_ptr(args_get(args, 'T' as i32 as u_char)),
            );
            if screen_set_title(&raw mut (*wp).base, title.as_ptr(), 0 as ::core::ffi::c_int) != 0 {
                notify_pane(c"pane-title-changed".as_ptr(), wp);
                server_redraw_window_borders((*wp).window);
                server_status_window((*wp).window);
            }
            return CMD_RETURN_NORMAL;
        }
        if !c.is_null()
            && !(*c).session.is_null()
            && (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ACTIVEPANE != 0
        {
            activewp = server_client_get_pane(c);
        } else {
            activewp = window_get_active(w);
        }
        if wp == activewp {
            return CMD_RETURN_NORMAL;
        }
        if window_push_zoom(
            w,
            0 as ::core::ffi::c_int,
            args_has(args, 'Z' as i32 as u_char),
        ) != 0
        {
            server_redraw_window(w);
        }
        window_redraw_active_switch(w, wp);
        if !c.is_null()
            && !(*c).session.is_null()
            && (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ACTIVEPANE != 0
        {
            server_client_set_pane(c, wp);
        } else if window_set_active_pane(w, wp, 1 as ::core::ffi::c_int) != 0 {
            cmd_find_from_winlink_pane(&mut *current, wl, wp, 0 as ::core::ffi::c_int);
        }
        cmdq_insert_hook(s, item, current, c"after-select-pane".as_ptr(), fmt_args![]);
        cmd_select_pane_redraw(w);
        if window_pop_zoom(w) != 0 {
            server_redraw_window(w);
        }
        CMD_RETURN_NORMAL
    }
}
