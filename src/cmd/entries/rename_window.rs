use crate::src::options::options_owner_ptr;
use crate::src::arguments::args_string;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::format::bytes::write_cstr;
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
pub static cmd_rename_window_entry: cmd_entry = {
    cmd_entry {
        name: c"rename-window",
        alias: Some(c"renamew"),
        args: args_parse {
            template: c"t:",
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
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut wl: *mut winlink = (*target).wl;
    let name = format_single_from_target_cstring(item, args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()));
    if !check_name(&name) {
        cmdq_error(item, |out| {
            out.write_all(b"invalid window name: ")?;
            write_cstr(out, name.as_ptr())
        });
        return CMD_RETURN_ERROR;
    }
    window_set_name((*wl).window_ptr(), name.as_ptr(), 0 as ::core::ffi::c_int);
    options_set_number(
        options_owner_ptr(&mut (*(*wl).window_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    server_redraw_window_borders((*wl).window_ptr());
    server_status_window((*wl).window_ptr());
    return CMD_RETURN_NORMAL;
}
