use crate::arguments::args_string_str;
use crate::cmd::cmd_get_args;

use crate::fmt_args;
use crate::format::format_single_from_target;

use crate::tmux::check_name;
pub use crate::types::{
    OptionsRef, RustCommandEntry, args, args_parse_t, cmd, cmd_entry_flag, cmd_retval, cmdq_item,
};

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};

pub(crate) static cmd_rename_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"rename-window",
        alias: Some(c"renamew"),
        args: args_parse_t {
            template: c"t:",
            lower: 1 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-t target-window] new-name",
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
        exec: cmd_rename_window_exec,
    }
};
unsafe fn cmd_rename_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let window = item
        .target
        .winlink_ref()
        .and_then(|link| link.window())
        .expect("the command target has a window");
    let name = unsafe {
        format_single_from_target(
            item,
            args_string_str(args, 0).expect("argument count checked"),
        )
    };
    if unsafe { check_name(Some(&name)) == 0 } {
        unsafe { item.error(c"invalid window name: %s", fmt_args![name.as_c_str()]) };
        return CMD_RETURN_ERROR;
    }
    unsafe { window.set_name(&name, 0 as core::ffi::c_int) };
    unsafe {
        window
            .options()
            .set_number(c"automatic-rename", 0 as core::ffi::c_longlong)
    };
    unsafe { window.redraw_borders() };
    unsafe { window.redraw_status() };
    CMD_RETURN_NORMAL
}
