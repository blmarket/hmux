use crate::src::options::options_owner_ptr;
use crate::src::arguments::{args_count, args_has, args_string, args_strtonum_result};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::compat::strtonum::strtonum;
use crate::src::format::bytes::write_cstr;
use crate::src::options::options_set_number;
use crate::src::resize::{default_window_size, recalculate_size};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::shared::window::{
    WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_SIZE_LARGEST, WINDOW_SIZE_MANUAL,
};

pub const WINDOW_SIZE_SMALLEST: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub static cmd_resize_window_entry: cmd_entry = {
    cmd_entry {
        name: c"resize-window",
        alias: Some(c"resizew"),
        args: args_parse {
            template: c"aADLRt:Ux:y:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-aADLRU] [-x width] [-y height] [-t target-window] [adjustment]",
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
        exec: Some(cmd_resize_window_exec),
    }
};
unsafe fn cmd_resize_window_exec(mut self_0: refbox::Weak<cmd>, item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args = cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let mut w: *mut window = wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut s: *mut session = (*target).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut adjust: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0 as u_int;
    let mut ypixel: u_int = 0 as u_int;
    if args_count(args) == 0 as u_int {
        adjust = 1 as u_int;
    } else {
        adjust = strtonum(
            args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()),
            1 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as u_int;
        if !errstr.is_null() {
            cmdq_error(item_handle, |out| {
                out.write_all(b"adjustment ")?;
                write_cstr(out, errstr)
            });
            return CMD_RETURN_ERROR;
        }
    }
    sx = (*w).sx;
    sy = (*w).sy;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        sx = match args_strtonum_result(
            args,
            'x' as i32 as u_char,
            WINDOW_MINIMUM as ::core::ffi::c_longlong,
            WINDOW_MAXIMUM as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(error) => {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"width ")?;
                    write_cstr(out, error.message().as_ptr())
                });
                return CMD_RETURN_ERROR;
            }
        };
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        sy = match args_strtonum_result(
            args,
            'y' as i32 as u_char,
            WINDOW_MINIMUM as ::core::ffi::c_longlong,
            WINDOW_MAXIMUM as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(error) => {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"height ")?;
                    write_cstr(out, error.message().as_ptr())
                });
                return CMD_RETURN_ERROR;
            }
        };
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
        default_window_size(
            None,
            &(*(s)).observer.upgrade().expect("live session"),
            (w).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
            WINDOW_SIZE_LARGEST,
        );
    } else if args_has(args, 'a' as i32 as u_char) != 0 {
        default_window_size(
            None,
            &(*(s)).observer.upgrade().expect("live session"),
            (w).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
            WINDOW_SIZE_SMALLEST,
        );
    }
    options_set_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
        WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong,
    );
    (*w).manual_sx = sx;
    (*w).manual_sy = sy;
    recalculate_size(&(*(w)).observer.upgrade().expect("live window"), 1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}
