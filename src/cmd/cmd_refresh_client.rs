use crate::arguments::{args_count, args_get, args_has, args_string, args_value_list};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::compat::strtonum;
use crate::control::{
    control_add_sub, control_continue_pane, control_pause_pane, control_remove_sub,
    control_set_pane_off, control_set_pane_on,
};
use crate::ffi::{sscanf, strcmp};
use crate::fmt_args;
use crate::log::log_debug;
use crate::resize::recalculate_sizes_now;
use crate::server::{client_get_pan_window, client_set_pan_window};
use crate::server::{
    server_client_add_client_window, server_client_get_client_window, server_client_set_flags,
};
use crate::server::{server_redraw_client, server_status_client};
use crate::session::session_get_curw;
use crate::tty::tty_keys_colours;
use crate::tty::{tty_clipboard_query, tty_set_size, tty_update_client_offset};
pub use crate::types::*;
use crate::window::window_pane_find_by_id;
use ::core::ffi::CStr;
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
pub const CONTROL_SUB_ALL_WINDOWS: control_sub_type = 4;
pub const CONTROL_SUB_WINDOW: control_sub_type = 3;
pub const CONTROL_SUB_ALL_PANES: control_sub_type = 2;
pub const CONTROL_SUB_PANE: control_sub_type = 1;
pub const CONTROL_SUB_SESSION: control_sub_type = 0;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const WINDOW_MINIMUM: ::core::ffi::c_int = PANE_MINIMUM;
pub const WINDOW_MAXIMUM: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_STATUSFORCE: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const CLIENT_SIZECHANGED: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const CLIENT_WINDOWSIZECHANGED: ::core::ffi::c_ulonglong =
    0x400000000 as ::core::ffi::c_ulonglong;
pub(crate) static cmd_refresh_client_entry: cmd_entry = {
    cmd_entry {
        name: c"refresh-client",
        alias: Some(c"refresh"),
        args: args_parse_t {
            template: c"A:B:cC:Df:r:F:lLRSt:U",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY] [-f flags] [-r pane:report] [-t target-client] [adjustment]",
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG,
        exec: cmd_refresh_client_exec,
    }
};
unsafe fn cmd_refresh_client_update_subscription(
    tc: *mut client,
    value: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut subtype: control_sub_type = CONTROL_SUB_SESSION;
        let mut subid: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
        let mut fields = CStr::from_ptr(value)
            .to_bytes()
            .splitn(3, |&byte| byte == b':');
        let name = CString::new(fields.next().unwrap_or_default())
            .expect("a C string has no interior NUL");
        let Some(what) = fields.next() else {
            control_remove_sub(tc, name.as_ptr());
            return;
        };
        let Some(format) = fields.next() else {
            return;
        };
        let what = CString::new(what).expect("a C string has no interior NUL");
        let format = CString::new(format).expect("a C string has no interior NUL");
        if strcmp(what.as_ptr(), c"%*".as_ptr()) == 0 as ::core::ffi::c_int {
            subtype = CONTROL_SUB_ALL_PANES;
        } else if sscanf(what.as_ptr(), c"%%%d".as_ptr(), &raw mut subid) == 1 as ::core::ffi::c_int
            && subid >= 0 as ::core::ffi::c_int
        {
            subtype = CONTROL_SUB_PANE;
        } else if strcmp(what.as_ptr(), c"@*".as_ptr()) == 0 as ::core::ffi::c_int {
            subtype = CONTROL_SUB_ALL_WINDOWS;
        } else if sscanf(what.as_ptr(), c"@%d".as_ptr(), &raw mut subid) == 1 as ::core::ffi::c_int
            && subid >= 0 as ::core::ffi::c_int
        {
            subtype = CONTROL_SUB_WINDOW;
        } else {
            subtype = CONTROL_SUB_SESSION;
        }
        control_add_sub(tc, name.as_ptr(), subtype, subid, format.as_ptr());
    }
}
unsafe fn cmd_refresh_client_control_client_size(
    mut self_0: &cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut size: *const ::core::ffi::c_char = args_get(args, 'C' as i32 as u_char);
        let mut w: u_int = 0;
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut cw: *mut client_window = ::core::ptr::null_mut::<client_window>();
        if sscanf(
            size,
            c"@%u:%ux%u".as_ptr(),
            &raw mut w,
            &raw mut x,
            &raw mut y,
        ) == 3 as ::core::ffi::c_int
        {
            if x < WINDOW_MINIMUM as u_int
                || x > WINDOW_MAXIMUM as u_int
                || y < WINDOW_MINIMUM as u_int
                || y > WINDOW_MAXIMUM as u_int
            {
                cmdq_error(item, c"size too small or too big".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            log_debug(
                c"%s: client %s window @%u: size %ux%u".as_ptr(),
                fmt_args![
                    c"cmd_refresh_client_control_client_size".as_ptr(),
                    (*tc).name.as_deref(),
                    w,
                    x,
                    y
                ],
            );
            cw = server_client_add_client_window(tc, w);
            (*cw).sx = x;
            (*cw).sy = y;
            (*tc).flags =
                ((*tc).flags as ::core::ffi::c_ulonglong | CLIENT_WINDOWSIZECHANGED) as uint64_t;
            recalculate_sizes_now(1 as ::core::ffi::c_int);
            return CMD_RETURN_NORMAL;
        }
        if sscanf(size, c"@%u:".as_ptr(), &raw mut w) == 1 as ::core::ffi::c_int {
            cw = server_client_get_client_window(tc, w);
            if !cw.is_null() {
                log_debug(
                    c"%s: client %s window @%u: no size".as_ptr(),
                    fmt_args![
                        c"cmd_refresh_client_control_client_size".as_ptr(),
                        (*tc).name.as_deref(),
                        w
                    ],
                );
                (*cw).sx = 0 as u_int;
                (*cw).sy = 0 as u_int;
                recalculate_sizes_now(1 as ::core::ffi::c_int);
            }
            return CMD_RETURN_NORMAL;
        }
        if sscanf(size, c"%u,%u".as_ptr(), &raw mut x, &raw mut y) != 2 as ::core::ffi::c_int
            && sscanf(size, c"%ux%u".as_ptr(), &raw mut x, &raw mut y) != 2 as ::core::ffi::c_int
        {
            cmdq_error(item, c"bad size argument".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        if x < WINDOW_MINIMUM as u_int
            || x > WINDOW_MAXIMUM as u_int
            || y < WINDOW_MINIMUM as u_int
            || y > WINDOW_MAXIMUM as u_int
        {
            cmdq_error(item, c"size too small or too big".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        tty_set_size(&mut (*tc).tty, x, y, 0 as u_int, 0 as u_int);
        (*tc).flags |= CLIENT_SIZECHANGED as uint64_t;
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_refresh_client_update_offset(tc: *mut client, value: *const ::core::ffi::c_char) {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut pane: u_int = 0;
        if *value as ::core::ffi::c_int != '%' as i32 {
            return;
        }
        let mut fields = CStr::from_ptr(value)
            .to_bytes()
            .splitn(2, |&byte| byte == b':');
        let pane_text = fields.next().unwrap();
        let Some(action) = fields.next() else {
            return;
        };
        let pane_text = CString::new(pane_text).expect("a C string has no interior NUL");
        let action = CString::new(action).expect("a C string has no interior NUL");
        {
            if !(sscanf(pane_text.as_ptr(), c"%%%u".as_ptr(), &raw mut pane)
                != 1 as ::core::ffi::c_int)
            {
                wp = window_pane_find_by_id(pane);
                if !wp.is_null() {
                    if strcmp(action.as_ptr(), c"on".as_ptr()) == 0 as ::core::ffi::c_int {
                        control_set_pane_on(tc, wp);
                    } else if strcmp(action.as_ptr(), c"off".as_ptr()) == 0 as ::core::ffi::c_int {
                        control_set_pane_off(tc, wp);
                    } else if strcmp(action.as_ptr(), c"continue".as_ptr())
                        == 0 as ::core::ffi::c_int
                    {
                        control_continue_pane(tc, wp);
                    } else if strcmp(action.as_ptr(), c"pause".as_ptr()) == 0 as ::core::ffi::c_int
                    {
                        control_pause_pane(tc, wp);
                    }
                }
            }
        }
    }
}
unsafe fn cmd_refresh_report(tty: &mut tty, mut value: *const ::core::ffi::c_char) {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut pane: u_int = 0;
        let mut size: size_t = 0 as size_t;
        if *value as ::core::ffi::c_int != '%' as i32 {
            return;
        }
        let mut fields = CStr::from_ptr(value)
            .to_bytes()
            .splitn(2, |&byte| byte == b':');
        let pane_text = fields.next().unwrap();
        let Some(colours) = fields.next() else {
            return;
        };
        let pane_text = CString::new(pane_text).expect("a C string has no interior NUL");
        let colours = CString::new(colours).expect("a C string has no interior NUL");
        {
            if !(sscanf(pane_text.as_ptr(), c"%%%u".as_ptr(), &raw mut pane)
                != 1 as ::core::ffi::c_int)
            {
                wp = window_pane_find_by_id(pane);
                if !wp.is_null() {
                    tty_keys_colours(
                        tty,
                        colours.to_bytes(),
                        &mut size,
                        &mut (*wp).control_fg,
                        &mut (*wp).control_bg,
                    );
                }
            }
        }
    }
}
unsafe fn cmd_refresh_client_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let tty: &mut tty = &mut (*tc).tty;
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut adjust: u_int = 0;
        if args_has(args, 'c' as i32 as u_char) != 0
            || args_has(args, 'L' as i32 as u_char) != 0
            || args_has(args, 'R' as i32 as u_char) != 0
            || args_has(args, 'U' as i32 as u_char) != 0
            || args_has(args, 'D' as i32 as u_char) != 0
        {
            if args_count(args) == 0 as u_int {
                adjust = 1 as u_int;
            } else {
                match strtonum(
                    args_string(args, 0 as u_int),
                    1 as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                ) {
                    Ok(value) => adjust = value as u_int,
                    Err(errstr) => {
                        cmdq_error(item, c"adjustment %s".as_ptr(), fmt_args![errstr.as_ptr()]);
                        return CMD_RETURN_ERROR;
                    }
                }
            }
            if args_has(args, 'c' as i32 as u_char) != 0 {
                (*tc).pan_window = None;
            } else {
                w = (*session_get_curw((*tc).session)).window();
                if client_get_pan_window(tc) != w {
                    client_set_pan_window(tc, w);
                    (*tc).pan_ox = (*tty).oox;
                    (*tc).pan_oy = (*tty).ooy;
                }
                if args_has(args, 'L' as i32 as u_char) != 0 {
                    if (*tc).pan_ox > adjust {
                        (*tc).pan_ox = (*tc).pan_ox.wrapping_sub(adjust);
                    } else {
                        (*tc).pan_ox = 0 as u_int;
                    }
                } else if args_has(args, 'R' as i32 as u_char) != 0 {
                    (*tc).pan_ox = (*tc).pan_ox.wrapping_add(adjust);
                    if (*tc).pan_ox > (*w).sx.wrapping_sub((*tty).osx) {
                        (*tc).pan_ox = (*w).sx.wrapping_sub((*tty).osx);
                    }
                } else if args_has(args, 'U' as i32 as u_char) != 0 {
                    if (*tc).pan_oy > adjust {
                        (*tc).pan_oy = (*tc).pan_oy.wrapping_sub(adjust);
                    } else {
                        (*tc).pan_oy = 0 as u_int;
                    }
                } else if args_has(args, 'D' as i32 as u_char) != 0 {
                    (*tc).pan_oy = (*tc).pan_oy.wrapping_add(adjust);
                    if (*tc).pan_oy > (*w).sy.wrapping_sub((*tty).osy) {
                        (*tc).pan_oy = (*w).sy.wrapping_sub((*tty).osy);
                    }
                }
            }
            tty_update_client_offset(tc);
            server_redraw_client(tc);
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'l' as i32 as u_char) != 0 {
            tty_clipboard_query(&mut (*tc).tty);
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'F' as i32 as u_char) != 0 {
            server_client_set_flags(tc, args_get(args, 'F' as i32 as u_char));
        }
        if args_has(args, 'f' as i32 as u_char) != 0 {
            server_client_set_flags(tc, args_get(args, 'f' as i32 as u_char));
        }
        if args_has(args, 'r' as i32 as u_char) != 0 {
            cmd_refresh_report(tty, args_get(args, 'r' as i32 as u_char));
        }
        if args_has(args, 'A' as i32 as u_char) != 0 {
            if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
                for av in args_value_list(args, 'A' as i32 as u_char) {
                    cmd_refresh_client_update_offset(tc, (*av).value.string().as_ptr());
                }
                return CMD_RETURN_NORMAL;
            }
        } else if args_has(args, 'B' as i32 as u_char) != 0 {
            if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
                for av in args_value_list(args, 'B' as i32 as u_char) {
                    cmd_refresh_client_update_subscription(tc, (*av).value.string().as_ptr());
                }
                return CMD_RETURN_NORMAL;
            }
        } else if args_has(args, 'C' as i32 as u_char) != 0 {
            if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
                return cmd_refresh_client_control_client_size(self_0, item);
            }
        } else {
            if args_has(args, 'S' as i32 as u_char) != 0 {
                (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
                server_status_client(tc);
            } else {
                (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
                server_redraw_client(tc);
            }
            return CMD_RETURN_NORMAL;
        }
        cmdq_error(item, c"not a control client".as_ptr(), fmt_args![]);
        CMD_RETURN_ERROR
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
