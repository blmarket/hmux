use crate::arguments::{args_count, args_get_str, args_has, args_string_str, args_value_list};
use crate::cmd::cmd_get_args;

use crate::compat::strtonum;
use crate::ffi::sscanf;
use crate::fmt_args;
use crate::log::log_debug;
use crate::server::ClientPanDirection;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CONTROL_SUB_ALL_PANES, CONTROL_SUB_ALL_WINDOWS, CONTROL_SUB_PANE, CONTROL_SUB_SESSION,
    CONTROL_SUB_WINDOW, INT_MAX, WINDOW_MAXIMUM, WINDOW_MINIMUM,
};
pub use crate::types::*;
use crate::window::window_pane_find_by_id;
use ::core::ffi::CStr;
use ::std::ffi::CString;

pub(crate) static cmd_refresh_client_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"refresh-client",
        alias: Some(c"refresh"),
        args: args_parse_t {
            template: c"A:B:cC:Df:r:F:lLRSt:U",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
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
unsafe fn cmd_refresh_client_update_subscription(tc: &mut ClientRef, value: &CStr) {
    unsafe {
        let mut subid: core::ffi::c_int = -(1 as core::ffi::c_int);
        let mut fields = value.to_bytes().splitn(3, |&byte| byte == b':');
        let name = CString::new(fields.next().unwrap_or_default())
            .expect("a C string has no interior NUL");
        let Some(what) = fields.next() else {
            tc.remove_control_subscription(&name);
            return;
        };
        let Some(format) = fields.next() else {
            return;
        };
        let what = CString::new(what).expect("a C string has no interior NUL");
        let format = CString::new(format).expect("a C string has no interior NUL");
        let subtype: control_sub_type = if what.as_c_str() == c"%*" {
            CONTROL_SUB_ALL_PANES
        } else if sscanf(what.as_ptr(), c"%%%d".as_ptr(), &raw mut subid) == 1 as core::ffi::c_int
            && subid >= 0 as core::ffi::c_int
        {
            CONTROL_SUB_PANE
        } else if what.as_c_str() == c"@*" {
            CONTROL_SUB_ALL_WINDOWS
        } else if sscanf(what.as_ptr(), c"@%d".as_ptr(), &raw mut subid) == 1 as core::ffi::c_int
            && subid >= 0 as core::ffi::c_int
        {
            CONTROL_SUB_WINDOW
        } else {
            CONTROL_SUB_SESSION
        };
        tc.set_control_subscription(&name, subtype, subid, &format);
    }
}
unsafe fn cmd_refresh_client_control_client_size(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut tc = item
            .target_client()
            .expect("the command has a target client");

        let size = args_get_str(args, 'C' as i32 as u_char);
        // sscanf is a C ABI boundary: it wants the bytes, not the borrow.
        let size = size.map_or(core::ptr::null(), CStr::as_ptr);
        let mut w: u_int = 0;
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        if sscanf(
            size,
            c"@%u:%ux%u".as_ptr(),
            &raw mut w,
            &raw mut x,
            &raw mut y,
        ) == 3 as core::ffi::c_int
        {
            if x < WINDOW_MINIMUM as u_int
                || x > WINDOW_MAXIMUM as u_int
                || y < WINDOW_MINIMUM as u_int
                || y > WINDOW_MAXIMUM as u_int
            {
                item.error(c"size too small or too big", fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            log_debug(
                c"%s: client %s window @%u: size %ux%u",
                fmt_args![
                    c"cmd_refresh_client_control_client_size",
                    tc.name(),
                    w,
                    x,
                    y
                ],
            );
            tc.set_control_window_size(w, x, y);
            return CMD_RETURN_NORMAL;
        }
        if sscanf(size, c"@%u:".as_ptr(), &raw mut w) == 1 as core::ffi::c_int {
            tc.clear_control_window_size(w);
            return CMD_RETURN_NORMAL;
        }
        if sscanf(size, c"%u,%u".as_ptr(), &raw mut x, &raw mut y) != 2 as core::ffi::c_int
            && sscanf(size, c"%ux%u".as_ptr(), &raw mut x, &raw mut y) != 2 as core::ffi::c_int
        {
            item.error(c"bad size argument", fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        if x < WINDOW_MINIMUM as u_int
            || x > WINDOW_MAXIMUM as u_int
            || y < WINDOW_MINIMUM as u_int
            || y > WINDOW_MAXIMUM as u_int
        {
            item.error(c"size too small or too big", fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        tc.set_control_size(x, y);
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_refresh_client_update_offset(tc: &mut ClientRef, value: &CStr) {
    unsafe {
        let mut pane: u_int = 0;
        if value.to_bytes().first() != Some(&{ b'%' }) {
            return;
        }
        let mut fields = value.to_bytes().splitn(2, |&byte| byte == b':');
        let pane_text = fields.next().unwrap();
        let Some(action) = fields.next() else {
            return;
        };
        let pane_text = CString::new(pane_text).expect("a C string has no interior NUL");
        let action = CString::new(action).expect("a C string has no interior NUL");
        {
            if !(sscanf(pane_text.as_ptr(), c"%%%u".as_ptr(), &raw mut pane)
                != 1 as core::ffi::c_int)
                && let Some(pane) = window_pane_find_by_id(pane)
            {
                if action.as_c_str() == c"on" {
                    tc.control_set_pane_on(&pane);
                } else if action.as_c_str() == c"off" {
                    tc.control_set_pane_off(&pane);
                } else if action.as_c_str() == c"continue" {
                    tc.control_continue_pane(&pane);
                } else if action.as_c_str() == c"pause" {
                    tc.control_pause_pane(&pane);
                }
            }
        }
    }
}
unsafe fn cmd_refresh_report(tc: &mut ClientRef, value: &CStr) {
    unsafe {
        let mut pane: u_int = 0;
        if value.to_bytes().first() != Some(&{ b'%' }) {
            return;
        }
        let mut fields = value.to_bytes().splitn(2, |&byte| byte == b':');
        let pane_text = fields.next().unwrap();
        let Some(colours) = fields.next() else {
            return;
        };
        let pane_text = CString::new(pane_text).expect("a C string has no interior NUL");
        let colours = CString::new(colours).expect("a C string has no interior NUL");
        {
            if !(sscanf(pane_text.as_ptr(), c"%%%u".as_ptr(), &raw mut pane)
                != 1 as core::ffi::c_int)
                && let Some(mut pane) = window_pane_find_by_id(pane)
            {
                tc.report_pane_colours(&mut pane, colours.to_bytes());
            }
        }
    }
}
unsafe fn cmd_refresh_client_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let mut tc = item
        .target_client()
        .expect("the command has a target client");

    let adjust: u_int;
    if args_has(args, 'c' as i32 as u_char) != 0
        || args_has(args, 'L' as i32 as u_char) != 0
        || args_has(args, 'R' as i32 as u_char) != 0
        || args_has(args, 'U' as i32 as u_char) != 0
        || args_has(args, 'D' as i32 as u_char) != 0
    {
        if args_count(args) == 0 as u_int {
            adjust = 1 as u_int;
        } else {
            match unsafe {
                strtonum(
                    args_string_str(args, 0).expect("argument count checked"),
                    1 as core::ffi::c_longlong,
                    INT_MAX as core::ffi::c_longlong,
                )
            } {
                Ok(value) => adjust = value as u_int,
                Err(errstr) => {
                    unsafe { item.error(c"adjustment %s", fmt_args![errstr]) };
                    return CMD_RETURN_ERROR;
                }
            }
        }
        if args_has(args, 'c' as i32 as u_char) != 0 {
            unsafe { tc.reset_window_pan() };
        } else {
            let session = { tc.attached_session() };
            let window = session
                .as_ref()
                .and_then(|session| session.curw())
                .and_then(|link| link.window());
            let Some(window) = window else {
                unsafe { item.error(c"no current window", fmt_args![]) };
                return CMD_RETURN_ERROR;
            };
            let direction = if args_has(args, 'L' as i32 as u_char) != 0 {
                ClientPanDirection::Left
            } else if args_has(args, 'R' as i32 as u_char) != 0 {
                ClientPanDirection::Right
            } else if args_has(args, 'U' as i32 as u_char) != 0 {
                ClientPanDirection::Up
            } else {
                ClientPanDirection::Down
            };
            unsafe { tc.pan_window(&window, direction, adjust) };
        }
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        unsafe { tc.query_clipboard() };
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'F' as i32 as u_char) != 0
        && let Some(flags) = args_get_str(args, 'F' as i32 as u_char)
    {
        unsafe { tc.apply_flags(flags) };
    }
    if args_has(args, 'f' as i32 as u_char) != 0
        && let Some(flags) = args_get_str(args, 'f' as i32 as u_char)
    {
        unsafe { tc.apply_flags(flags) };
    }
    if let Some(report) = args_get_str(args, 'r' as i32 as u_char) {
        unsafe { cmd_refresh_report(&mut tc, report) };
    }
    if args_has(args, 'A' as i32 as u_char) != 0 {
        if unsafe { tc.is_control() } {
            for av in args_value_list(args, 'A' as i32 as u_char) {
                unsafe { cmd_refresh_client_update_offset(&mut tc, av.value.string()) };
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'B' as i32 as u_char) != 0 {
        if unsafe { tc.is_control() } {
            for av in args_value_list(args, 'B' as i32 as u_char) {
                unsafe { cmd_refresh_client_update_subscription(&mut tc, av.value.string()) };
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'C' as i32 as u_char) != 0 {
        if unsafe { tc.is_control() } {
            return unsafe { cmd_refresh_client_control_client_size(self_0, item) };
        }
    } else {
        unsafe { tc.refresh_display(args_has(args, 'S' as i32 as u_char) != 0) };
        return CMD_RETURN_NORMAL;
    }
    unsafe { item.error(c"not a control client", fmt_args![]) };
    CMD_RETURN_ERROR
}
