use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::args::args_percentage;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_mouse_pane};
use crate::compat::strtonum;
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, INT_MAX, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, PANE_STATUS_BOTTOM, PANE_STATUS_TOP,
};
use crate::fmt_args;
use crate::types::{OptionsRef, u_char, u_int};

pub(crate) static cmd_resize_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"resize-pane",
        alias: Some(c"resizep"),
        args: args_parse_t {
            template: c"DLMRTt:Ux:y:Z",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DLMRTUZ] [-x width] [-y height] [-t target-pane] [adjustment]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_resize_pane_exec,
    }
};
unsafe fn cmd_resize_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let target = &item.target;
    let reference = target.pane_ref().expect("resize target has a pane");
    let owner = target.window().expect("resize target has a window");
    let mut cause = None;
    let adjust: u_int;
    let x: core::ffi::c_int;
    let mut y: core::ffi::c_int;
    let status: core::ffi::c_int;
    if ({
        let flag = 'T' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        unsafe { reference.trim_unused_screen() };
        return CMD_RETURN_NORMAL;
    }
    if ({
        let flag = 'M' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        return unsafe { cmd_resize_pane_mouse_update(self_0, item) };
    }
    if ({
        let flag = 'Z' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        if owner.is_zoomed() {
            unsafe { owner.unzoom(1 as core::ffi::c_int) };
        } else {
            unsafe { owner.zoom(&reference) };
        }
        unsafe { owner.redraw() };
        return CMD_RETURN_NORMAL;
    }
    unsafe { owner.unzoom_and_redraw() };
    if args.argument_count() == 0 as u_int {
        adjust = 1 as u_int;
    } else {
        match unsafe {
            strtonum(
                args.argument_string(0).expect("argument count checked"),
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
    if ({
        let flag = 'x' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        {
            x = args_percentage(
                args,
                'x' as i32 as u_char,
                0 as core::ffi::c_longlong,
                INT_MAX as core::ffi::c_longlong,
                owner.dimensions().size.width as core::ffi::c_longlong,
                &mut cause,
            ) as core::ffi::c_int
        };
        if let Some(cause) = cause.as_ref() {
            unsafe { item.error(c"width %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
        unsafe { owner.resize_pane_to(&reference, LAYOUT_LEFTRIGHT, x as u_int) };
    }
    if ({
        let flag = 'y' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        {
            y = args_percentage(
                args,
                'y' as i32 as u_char,
                0 as core::ffi::c_longlong,
                INT_MAX as core::ffi::c_longlong,
                owner.dimensions().size.height as core::ffi::c_longlong,
                &mut cause,
            ) as core::ffi::c_int
        };
        if let Some(cause) = cause.as_ref() {
            unsafe { item.error(c"height %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
        {
            status = owner.options().number(c"pane-border-status") as core::ffi::c_int
        };
        let geometry = unsafe { reference.geometry().expect("resize target is present") };
        match status {
            PANE_STATUS_TOP => {
                if y != INT_MAX && geometry.yoff == 1 as core::ffi::c_int {
                    y += 1;
                }
            }
            PANE_STATUS_BOTTOM
                if {
                    y != INT_MAX
                        && (geometry.yoff as u_int).wrapping_add(geometry.sy)
                            == owner.dimensions().size.height.wrapping_sub(1 as u_int)
                } =>
            {
                y += 1;
            }
            _ => {}
        }
        unsafe { owner.resize_pane_to(&reference, LAYOUT_TOPBOTTOM, y as u_int) };
    }
    if ({
        let flag = 'L' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        unsafe {
            owner.resize_pane(
                &reference,
                LAYOUT_LEFTRIGHT,
                adjust.wrapping_neg() as core::ffi::c_int,
                1 as core::ffi::c_int,
            )
        };
    } else if ({
        let flag = 'R' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        unsafe {
            owner.resize_pane(
                &reference,
                LAYOUT_LEFTRIGHT,
                adjust as core::ffi::c_int,
                1 as core::ffi::c_int,
            )
        };
    } else if ({
        let flag = 'U' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        unsafe {
            owner.resize_pane(
                &reference,
                LAYOUT_TOPBOTTOM,
                adjust.wrapping_neg() as core::ffi::c_int,
                1 as core::ffi::c_int,
            )
        };
    } else if ({
        let flag = 'D' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        unsafe {
            owner.resize_pane(
                &reference,
                LAYOUT_TOPBOTTOM,
                adjust as core::ffi::c_int,
                1 as core::ffi::c_int,
            )
        };
    }
    unsafe { owner.redraw() };
    CMD_RETURN_NORMAL
}
unsafe fn cmd_resize_pane_mouse_update(_self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    unsafe {
        let event = item.state_ref().event_snapshot();
        let Some(mut client) = item.client() else {
            return CMD_RETURN_NORMAL;
        };
        let Some((session, _, pane)) = cmd_mouse_pane(&event.m) else {
            return CMD_RETURN_NORMAL;
        };
        let same_session = client
            .attached_session()
            .is_some_and(|owner| owner.ptr_eq(&session));
        if !same_session {
            return CMD_RETURN_NORMAL;
        }
        let window = pane.window().expect("the mouse pane has a window");
        let floating = window.pane_is_floating(&pane);
        if !floating {
            client.start_tiled_resize_drag(&event.m);
            return CMD_RETURN_NORMAL;
        }
        let window = item.target.window().expect("resize target has a window");
        window.redraw_active_switch(&pane);
        window.set_active_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the selected pane exists"),
            1,
        );
        client.start_floating_resize_drag(&event.m);
        CMD_RETURN_NORMAL
    }
}
