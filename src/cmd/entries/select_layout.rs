use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target, cmdq_get_target_client};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::events::events_fire_window;
use crate::src::layout::custom::{layout_dump_owned, layout_parse};
use crate::src::layout::layout_spread_out;
use crate::src::layout::set::{
    layout_set_lookup, layout_set_next, layout_set_previous, layout_set_select,
};
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{server_redraw_window, server_unzoom_window};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_TARGET_WINDOW_USAGE};
use crate::src::shared::layout::*;
use crate::src::shared::pane::window_pane;
use crate::src::shared::window::{window, winlink};
use crate::src::window::window_replace_old_layout;
use std::ffi::CString;
pub static mut cmd_select_layout_entry: cmd_entry =  {
    cmd_entry {
        name: b"select-layout\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"selectl\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Enopt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-Enop] [-t target-pane] [layout-name]\0" as *const u8
            as *const ::core::ffi::c_char,
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
        exec: Some(cmd_select_layout_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_next_layout_entry: cmd_entry =  {
    cmd_entry {
        name: b"next-layout\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"nextl\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE.as_ptr(),
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
        exec: Some(cmd_select_layout_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_previous_layout_entry: cmd_entry =  {
    cmd_entry {
        name: b"previous-layout\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"prevl\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE.as_ptr(),
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
        exec: Some(cmd_select_layout_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_select_layout_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut c: *mut client = cmdq_get_target_client(item);
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = (*target).wp;
    let mut layoutname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: Option<CString> = None;
    let mut next: ::core::ffi::c_int = 0;
    let mut previous: ::core::ffi::c_int = 0;
    let mut layout: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    server_unzoom_window(w);
    next = (cmd_get_entry(self_0) == &raw const cmd_next_layout_entry) as ::core::ffi::c_int;
    if args_has(args, 'n' as i32 as u_char) != 0 {
        next = 1 as ::core::ffi::c_int;
    }
    previous =
        (cmd_get_entry(self_0) == &raw const cmd_previous_layout_entry) as ::core::ffi::c_int;
    if args_has(args, 'p' as i32 as u_char) != 0 {
        previous = 1 as ::core::ffi::c_int;
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    let new_layout = layout_dump_owned(w, (*w).layout_root, flags);
    let mut oldlayout = window_replace_old_layout(w, new_layout);
    let oldlayout_ptr = oldlayout
        .as_ref()
        .map_or(::core::ptr::null(), |value| value.as_ptr());
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
            layoutname = oldlayout_ptr;
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
                current_block = 16863505586472967431;
            } else {
                current_block = 15125582407903384992;
            }
        } else {
            current_block = 15125582407903384992;
        }
        match current_block {
            16863505586472967431 => {}
            _ => {
                if !layoutname.is_null() {
                    if layout_parse(w, layoutname, &raw mut cause) == -(1 as ::core::ffi::c_int) {
                        cmdq_error(
                            item,
                            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            cause
                                .as_ref()
                                .map_or(::core::ptr::null(), |message| message.as_ptr()),
                            layoutname,
                        );
                        drop(window_replace_old_layout(w, oldlayout.take()));
                        return CMD_RETURN_ERROR;
                    }
                } else {
                    drop(oldlayout);
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    }
    drop(oldlayout);
    recalculate_sizes();
    server_redraw_window(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    return CMD_RETURN_NORMAL;
}
