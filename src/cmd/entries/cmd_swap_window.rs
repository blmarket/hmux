use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::cmd::cmd_get_args;

use crate::fmt_args;
use crate::resize::recalculate_sizes;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_FIND_DEFAULT_MARKED, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
use crate::window::WinlinkRef;

pub(crate) static cmd_swap_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"swap-window",
        alias: Some(c"swapw"),
        args: args_parse_t {
            template: c"ds:t:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-d] [-s src-window] [-t dst-window]",
        source: cmd_entry_flag {
            flag: 's' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_swap_window_exec,
    }
};
unsafe fn cmd_swap_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let src = item.source.session().expect("swap source session");
    let dst = item.target.session().expect("swap target session");
    let same_session = src.ptr_eq(&dst);
    let source = WinlinkRef::new(src.clone(), item.source.wl_idx.expect("swap source link"))
        .expect("swap source link is present");
    let target = WinlinkRef::new(dst.clone(), item.target.wl_idx.expect("swap target link"))
        .expect("swap target link is present");
    if !same_session && src.shares_group(&dst) {
        unsafe { item.error(c"can't move window, sessions are grouped", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    let source_window = source.window().expect("swap source window");
    let target_window = target.window().expect("swap target window");
    if source_window.ptr_eq(&target_window) {
        return CMD_RETURN_NORMAL;
    }
    unsafe { source.exchange_linked_windows(&target) };
    if args.argument_flag_count(b'd') != 0 {
        unsafe { dst.select(target.index()) };
        if !same_session {
            unsafe { src.select(source.index()) };
        }
    }
    unsafe { src.synchronize_group_from() };
    unsafe { src.redraw_group() };
    if !same_session {
        unsafe { dst.synchronize_group_from() };
        unsafe { dst.redraw_group() };
    }
    recalculate_sizes();
    CMD_RETURN_NORMAL
}
