use crate::args::{args_count, args_has, args_string_str, args_strtonum};
use crate::cmd::cmd_get_args;

use crate::compat::strtonum;
use crate::fmt_args;

use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, INT_MAX,
    WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_SIZE_LARGEST, WINDOW_SIZE_MANUAL,
};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{OptionsRef, args, args_parse_t, u_char, u_int};

pub const WINDOW_SIZE_SMALLEST: core::ffi::c_int = 1 as core::ffi::c_int;

pub(crate) static cmd_resize_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"resize-window",
        alias: Some(c"resizew"),
        args: args_parse_t {
            template: c"aADLRt:Ux:y:",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-aADLRU] [-x width] [-y height] [-t target-window] [adjustment]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_resize_window_exec,
    }
};
unsafe fn cmd_resize_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let target = item
        .target
        .winlink_ref()
        .expect("the command target has a window link");
    let window = target.window().expect("the command target has a window");
    let mut cause = None;
    let adjust: u_int;
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
    let size = { window.dimensions().size };
    let mut sx = size.width;
    let mut sy = size.height;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        sx = args_strtonum(
            args,
            'x' as i32 as u_char,
            WINDOW_MINIMUM as core::ffi::c_longlong,
            WINDOW_MAXIMUM as core::ffi::c_longlong,
            &mut cause,
        ) as u_int;
        if let Some(cause) = cause.as_ref() {
            unsafe { item.error(c"width %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        sy = args_strtonum(
            args,
            'y' as i32 as u_char,
            WINDOW_MINIMUM as core::ffi::c_longlong,
            WINDOW_MAXIMUM as core::ffi::c_longlong,
            &mut cause,
        ) as u_int;
        if let Some(cause) = cause.as_ref() {
            unsafe { item.error(c"height %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'L' as i32 as u_char) != 0 {
        if sx >= adjust {
            sx = sx.wrapping_sub(adjust);
        }
    } else if args_has(args, 'R' as i32 as u_char) != 0 {
        sx = sx.wrapping_add(adjust);
    } else if args_has(args, 'U' as i32 as u_char) != 0 {
        if sy >= adjust {
            sy = sy.wrapping_sub(adjust);
        }
    } else if args_has(args, 'D' as i32 as u_char) != 0 {
        sy = sy.wrapping_add(adjust);
    }
    if args_has(args, 'A' as i32 as u_char) != 0 {
        unsafe { (sx, sy, _, _) = window.default_size(target.session(), WINDOW_SIZE_LARGEST) };
    } else if args_has(args, 'a' as i32 as u_char) != 0 {
        unsafe { (sx, sy, _, _) = window.default_size(target.session(), WINDOW_SIZE_SMALLEST) };
    }
    unsafe {
        window
            .options()
            .set_number(c"window-size", WINDOW_SIZE_MANUAL as core::ffi::c_longlong)
    };
    window.set_manual_size(crate::pane_resize::PaneSize {
        width: sx,
        height: sy,
    });
    unsafe { window.recalculate_size(1 as core::ffi::c_int) };
    CMD_RETURN_NORMAL
}
