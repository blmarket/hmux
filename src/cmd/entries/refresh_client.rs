use crate::src::arguments::{args_count, args_flag_values, args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::src::compat::strtonum::strtonum;
use crate::src::control::{
    control_add_sub, control_clear_window_size, control_continue_pane, control_pause_pane,
    control_remove_sub, control_set_pane_off, control_set_pane_on, control_set_window_size,
};
use crate::src::ffi::libc::sscanf;
use crate::src::format::bytes::write_cstr;
use crate::src::log::{log_cstr, log_debug};
use crate::src::monitor::monitor_parse_owned;
use crate::src::resize::recalculate_sizes_now;
use crate::src::server_client::server_client_set_flags;
use crate::src::server_fn::{server_redraw_client, server_status_client};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_SIZECHANGED, CLIENT_STATUSFORCE, CLIENT_WINDOWSIZECHANGED,
};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_TFLAG};
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::pane::PANE_THEMECHANGED;
use crate::src::shared::tty::tty;
use crate::src::shared::window::window;
use crate::src::shared::window::{WINDOW_MAXIMUM, WINDOW_MINIMUM};
use crate::src::tty::{tty_clipboard_query, tty_set_size, tty_update_client_offset};
use crate::src::tty_keys::tty_keys_colours;
use crate::src::window::window_pane_find_by_id;
use std::ffi::{CStr, CString};
pub static cmd_refresh_client_entry: cmd_entry = {
    cmd_entry {
        name: c"refresh-client",
        alias: Some(c"refresh"),
        args: args_parse {
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
        exec: Some(
            cmd_refresh_client_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_refresh_client_update_subscription(
    mut tc: *mut client,
    mut value: *const ::core::ffi::c_char,
) {
    let Some(parsed) = monitor_parse_owned(CStr::from_ptr(value)) else {
        control_remove_sub(tc, value);
        return;
    };
    control_add_sub(
        tc,
        parsed.name.as_ptr(),
        parsed.type_0,
        parsed.id,
        parsed.format.as_ptr(),
    );
}
unsafe fn cmd_refresh_client_control_client_size(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut size: *const ::core::ffi::c_char = args_get(&*(args), 'C' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut w: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if sscanf(
        size,
        b"@%u:%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmdq_error(item, |out| out.write_all(b"size too small or too big"));
            return CMD_RETURN_ERROR;
        }
        log_debug(format_args!(
            "{}: client {} window @{}: size {}x{}",
            "cmd_refresh_client_control_client_size",
            log_cstr(
                (((*tc).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            (w) as u32,
            (x) as u32,
            (y) as u32
        ));
        control_set_window_size(tc, w, x, y);
        (*tc).flags =
            ((*tc).flags as ::core::ffi::c_ulonglong | CLIENT_WINDOWSIZECHANGED) as uint64_t;
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    if sscanf(
        size,
        b"@%u:\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut w,
    ) == 1 as ::core::ffi::c_int
    {
        log_debug(format_args!(
            "{}: client {} window @{}: no size",
            "cmd_refresh_client_control_client_size",
            log_cstr(
                (((*tc).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            (w) as u32
        ));
        control_clear_window_size(tc, w);
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    if sscanf(
        size,
        b"%u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut x,
        &raw mut y,
    ) != 2 as ::core::ffi::c_int
        && sscanf(
            size,
            b"%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut x,
            &raw mut y,
        ) != 2 as ::core::ffi::c_int
    {
        cmdq_error(item, |out| out.write_all(b"bad size argument"));
        return CMD_RETURN_ERROR;
    }
    if x < WINDOW_MINIMUM as u_int
        || x > WINDOW_MAXIMUM as u_int
        || y < WINDOW_MINIMUM as u_int
        || y > WINDOW_MAXIMUM as u_int
    {
        cmdq_error(item, |out| out.write_all(b"size too small or too big"));
        return CMD_RETURN_ERROR;
    }
    tty_set_size(&raw mut (*tc).tty, x, y, 0 as u_int, 0 as u_int);
    (*tc).flags |= CLIENT_SIZECHANGED as uint64_t;
    recalculate_sizes_now(1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}
// The pane prefix needs its own terminator for scanf. The suffix borrows the
// original argument; neither scanf nor tty_keys_colours retains its pointer.
fn cmd_refresh_parse_pane(value: &CStr) -> Option<(u_int, &CStr)> {
    let bytes = value.to_bytes();
    if bytes.first() != Some(&b'%') {
        return None;
    }
    let colon = bytes.iter().position(|&byte| byte == b':')?;
    let prefix = CString::new(&bytes[..colon]).expect("CStr prefix contains no NUL");
    let suffix = CStr::from_bytes_with_nul(&value.to_bytes_with_nul()[colon + 1..])
        .expect("CStr suffix has one trailing NUL");
    let mut pane = 0;
    // Keep scanf's existing acceptance of signs, whitespace and trailing bytes.
    let matched = unsafe { sscanf(prefix.as_ptr(), c"%%%u".as_ptr(), &mut pane) };
    (matched == 1).then_some((pane, suffix))
}

unsafe fn cmd_refresh_client_update_offset(tc: *mut client, value: *const ::core::ffi::c_char) {
    let Some((pane, action)) = cmd_refresh_parse_pane(CStr::from_ptr(value)) else {
        return;
    };
    let lookup_wp_owner = window_pane_find_by_id(pane);
    let wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    match action.to_bytes() {
        b"on" => control_set_pane_on(tc, wp),
        b"off" => control_set_pane_off(tc, wp),
        b"continue" => control_continue_pane(tc, wp),
        b"pause" => control_pause_pane(tc, wp),
        _ => {}
    }
}

unsafe fn cmd_refresh_report(tty: *mut tty, value: *const ::core::ffi::c_char) {
    let Some((pane, report)) = cmd_refresh_parse_pane(CStr::from_ptr(value)) else {
        return;
    };
    let lookup_wp_owner = window_pane_find_by_id(pane);
    let wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    let mut fg = (*wp).control_fg;
    let mut bg = (*wp).control_bg;
    let mut size: size_t = 0;
    if tty_keys_colours(
        tty,
        report.as_ptr(),
        report.to_bytes().len(),
        &mut size,
        &mut fg,
        &mut bg,
    ) == 0
    {
        if bg != (*wp).control_bg {
            (*wp).flags |= PANE_THEMECHANGED;
        }
        (*wp).control_fg = fg;
        (*wp).control_bg = bg;
    }
}

unsafe fn cmd_refresh_client_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
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
            adjust = strtonum(
                args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                cmdq_error(item, |out| {
                    out.write_all(b"adjustment ")?;
                    write_cstr(out, errstr)
                });
                return CMD_RETURN_ERROR;
            }
        }
        if args_has(args, 'c' as i32 as u_char) != 0 {
            (*tc).pan_window = std::rc::Weak::new();
        } else {
            w = (*(*(*tc).session_ptr()).curw_ptr()).window_ptr();
            if !(*tc).pan_window_is(&*w) {
                (*tc).set_pan_window(&*w);
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
        server_redraw_client(&mut *(tc));
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        tty_clipboard_query(&raw mut (*tc).tty);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'F' as i32 as u_char) != 0 {
        server_client_set_flags(tc, args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()));
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        server_client_set_flags(tc, args_get(&*(args), 'f' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()));
    }
    if args_has(args, 'r' as i32 as u_char) != 0 {
        cmd_refresh_report(tty, args_get(&*(args), 'r' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()));
    }
    if args_has(args, 'A' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            for av in args_flag_values(&*args, 'A' as i32 as u_char) {
                cmd_refresh_client_update_offset(tc, av.string_ptr());
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'B' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            for av in args_flag_values(&*args, 'B' as i32 as u_char) {
                cmd_refresh_client_update_subscription(tc, av.string_ptr());
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
            server_status_client(&mut *(tc));
        } else {
            (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
            server_redraw_client(&mut *(tc));
        }
        return CMD_RETURN_NORMAL;
    }
    cmdq_error(item, |out| out.write_all(b"not a control client"));
    return CMD_RETURN_ERROR;
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn pane_argument_keeps_scanf_and_byte_boundaries() {
        for (input, pane, suffix) in [
            (b"%42:on".as_slice(), 42, b"on".as_slice()),
            (b"% +7junk:off", 7, b"off"),
            (b"%-1:pause", u_int::MAX, b"pause"),
            (b"%0:", 0, b""),
            (b"%9:\xff:rest", 9, b"\xff:rest"),
        ] {
            let input = CString::new(input).unwrap();
            let (actual_pane, actual_suffix) = cmd_refresh_parse_pane(&input).unwrap();
            assert_eq!(actual_pane, pane);
            assert_eq!(actual_suffix.to_bytes(), suffix);
        }
        for input in [c"", c"1:on", c"%1", c"%:on", c"%x:on"] {
            assert!(cmd_refresh_parse_pane(input).is_none(), "{input:?}");
        }
        let input = CStr::from_bytes_until_nul(b"%1:on\0:off").unwrap();
        assert_eq!(cmd_refresh_parse_pane(input).unwrap(), (1, c"on"));
    }
}
