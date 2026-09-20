use crate::src::cmd::cmd_get_entry;
use crate::src::ffi::libc::{getpid, kill};
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
pub use crate::src::shared::signal::SIGTERM;
pub use crate::src::shared::command::{CMD_STARTSERVER};
use crate::src::shared::arguments::*;
use crate::src::shared::command::*;

#[no_mangle]
pub static mut cmd_kill_server_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"kill-server\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_kill_server_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_start_server_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"start-server\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"start\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_STARTSERVER,
        exec: Some(
            cmd_kill_server_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_kill_server_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    if cmd_get_entry(self_0) == &raw const cmd_kill_server_entry {
        kill(getpid(), SIGTERM);
    }
    return CMD_RETURN_NORMAL;
}
