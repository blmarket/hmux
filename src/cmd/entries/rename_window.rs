use crate::src::arguments::args_string;
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::format::format_single_from_target_cstring;
use crate::src::options::options_set_number;
use crate::src::server_fn::{server_redraw_window_borders, server_status_window};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::window::winlink;
use crate::src::tmux::check_name;
use crate::src::window::window_set_name;
pub static mut cmd_rename_window_entry: cmd_entry = {
    cmd_entry {
        name: c"rename-window",
        alias: Some(c"renamew"),
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-t target-window] new-name",
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
        exec: Some(cmd_rename_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_rename_window_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wl: *mut winlink = (*target).wl;
    let name = format_single_from_target_cstring(item, args_string(args, 0 as u_int));
    if check_name(name.as_ptr()) == 0 {
        cmdq_error(
            item,
            b"invalid window name: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name.as_ptr(),
        );
        return CMD_RETURN_ERROR;
    }
    window_set_name((*wl).window, name.as_ptr(), 0 as ::core::ffi::c_int);
    options_set_number(
        (*(*wl).window).options,
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    server_redraw_window_borders((*wl).window);
    server_status_window((*wl).window);
    return CMD_RETURN_NORMAL;
}
