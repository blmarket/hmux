use crate::cmd::cmd_get_args;

use crate::fmt_args;

use crate::consts::{
    CMD_FIND_DEFAULT_MARKED, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::args::args_parse_t;

pub(crate) static cmd_swap_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"swap-pane",
        alias: Some(c"swapp"),
        args: args_parse_t {
            template: c"dDs:t:UZ",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-dDUZ] [-s src-pane] [-t dst-pane]",
        source: cmd_entry_flag {
            flag: 's' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_swap_pane_exec,
    }
};
unsafe fn cmd_swap_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let dst_owner = item
        .target
        .window()
        .expect("a swap destination has a window");
    let dst_pane = item
        .target
        .pane_ref()
        .expect("a swap destination has a pane");
    let mut src_owner = item.source.window().expect("a swap source has a window");
    let mut src_pane = item.source.pane_ref().expect("a swap source has a pane");
    if unsafe { dst_owner.push_zoom(0, args.argument_flag_count(b'Z')) != 0 } {
        unsafe { dst_owner.redraw() };
    }
    let down = args.argument_flag_count(b'D') != 0;
    if down || args.argument_flag_count(b'U') != 0 {
        src_owner = dst_owner.clone();
        let panes = dst_owner.panes();
        let index = panes
            .iter()
            .position(|pane| *pane == dst_pane)
            .expect("the destination pane belongs to its window");
        let index = if down {
            (index + 1) % panes.len()
        } else {
            index.checked_sub(1).unwrap_or(panes.len() - 1)
        };
        src_pane = panes[index].clone();
    }
    let same_window = src_owner.ptr_eq(&dst_owner);
    if unsafe { !same_window && src_owner.push_zoom(0, args.argument_flag_count(b'Z')) != 0 } {
        unsafe { src_owner.redraw() };
    }
    if src_pane != dst_pane {
        let (src_was_active, mut dst_was_active) =
            match unsafe { src_owner.exchange_pane_geometry(&src_pane, &dst_owner, &dst_pane) } {
                Ok(active) => active,
                Err(cause) => {
                    unsafe { item.error(c"%s", fmt_args![cause]) };
                    return CMD_RETURN_ERROR;
                }
            };
        if args.argument_flag_count(b'd') == 0 {
            unsafe { src_owner.set_active_pane(&dst_pane, 1) };
            if !same_window {
                unsafe { dst_owner.set_active_pane(&src_pane, 1) };
            }
        } else {
            if src_was_active {
                unsafe { src_owner.set_active_pane(&dst_pane, 1) };
                dst_was_active |= same_window;
            }
            if dst_was_active {
                unsafe { dst_owner.set_active_pane(&src_pane, 1) };
            }
        }
        if !same_window {
            unsafe { src_owner.finish_pane_exchange(&src_pane, &dst_owner, &dst_pane) };
            unsafe { src_owner.fix_layout_panes(None) };
            unsafe { src_owner.redraw() };
        }
        unsafe { dst_owner.fix_layout_panes(None) };
        unsafe { dst_owner.redraw() };
        unsafe { src_owner.notify(c"window-layout-changed") };
        if !same_window {
            unsafe { dst_owner.notify(c"window-layout-changed") };
        }
    }
    if unsafe { src_owner.pop_zoom() != 0 } {
        unsafe { src_owner.redraw() };
    }
    if unsafe { !same_window && dst_owner.pop_zoom() != 0 } {
        unsafe { dst_owner.redraw() };
    }
    CMD_RETURN_NORMAL
}
