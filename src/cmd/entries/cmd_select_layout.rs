use crate::args::RustArguments;
use crate::args::{args_string_str};

use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::layout::layout_set_lookup;

use crate::resize::recalculate_sizes;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
use crate::types::{args_parse_t, u_char, u_int};

pub const CMD_TARGET_WINDOW_USAGE: &core::ffi::CStr = c"[-t target-window]";
pub(crate) static cmd_select_layout_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"select-layout",
        alias: Some(c"selectl"),
        args: args_parse_t {
            template: c"Enopt:",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-Enop] [-t target-pane] [layout-name]",
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
        exec: cmd_select_layout_exec,
    }
};
pub(crate) static cmd_next_layout_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"next-layout",
        alias: Some(c"nextl"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE,
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
        exec: cmd_select_layout_exec,
    }
};
pub(crate) static cmd_previous_layout_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"previous-layout",
        alias: Some(c"prevl"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE,
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
        exec: cmd_select_layout_exec,
    }
};
unsafe fn cmd_select_layout_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let current_block: u64;
    let args: &RustArguments = cmd_get_args(self_0);
    let owner = item.target.window().expect("a layout target has a window");
    let pane = item.target.pane_ref();

    let mut next: core::ffi::c_int;
    let mut previous: core::ffi::c_int;
    let layout: core::ffi::c_int;
    unsafe { owner.unzoom_and_redraw() };
    next = core::ptr::eq(cmd_get_entry(self_0), &cmd_next_layout_entry) as core::ffi::c_int;
    if ({
        let flag = 'n' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0 {
        next = 1 as core::ffi::c_int;
    }
    previous = core::ptr::eq(cmd_get_entry(self_0), &cmd_previous_layout_entry) as core::ffi::c_int;
    if ({
        let flag = 'p' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0 {
        previous = 1 as core::ffi::c_int;
    }
    let oldlayout: Option<std::ffi::CString> = owner.saved_layout();
    let new_layout = { owner.dump_layout() };
    owner.set_saved_layout(new_layout.as_deref());
    if next != 0 || previous != 0 {
        if next != 0 {
            unsafe { owner.select_next_layout() };
        } else {
            unsafe { owner.select_previous_layout() };
        }
    } else if ({
        let flag = 'E' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0 {
        if let Some(pane) = pane {
            unsafe { owner.spread_pane_layout(&pane) };
        }
    } else {
        let layoutname = if args.argument_count() != 0 as u_int {
            unsafe { args_string_str(args, 0) }
        } else if ({
            let flag = 'o' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0 {
            oldlayout.as_deref()
        } else {
            None
        };
        if ({
            let flag = 'o' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0 {
            if let Some(layoutname) = layoutname {
                layout = layout_set_lookup(layoutname);
            } else {
                layout = owner.previous_layout().unwrap_or(-1);
            }
            if layout != -(1 as core::ffi::c_int) {
                unsafe { owner.select_layout(layout as u_int) };
                current_block = 16395913426687009730;
            } else {
                current_block = 15768484401365413375;
            }
        } else {
            current_block = 15768484401365413375;
        }
        match current_block {
            16395913426687009730 => {}
            _ => {
                if let Some(layoutname) = layoutname {
                    if let Err(cause) = unsafe { owner.parse_layout(layoutname) } {
                        unsafe { item.error(c"%s: %s", fmt_args![cause.as_c_str(), layoutname]) };
                        owner.set_saved_layout(oldlayout.as_deref());
                        return CMD_RETURN_ERROR;
                    }
                } else {
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    }
    recalculate_sizes();
    unsafe { owner.redraw() };
    unsafe { owner.notify(c"window-layout-changed") };
    CMD_RETURN_NORMAL
}
