use crate::src::shared::abi::*;
use crate::src::shared::command::*;
extern "C" {
    pub type args;
    pub type cmdq_item;
    pub type cmd;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn getpid() -> __pid_t;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
}
pub type args_parse_type = ::core::ffi::c_uint;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub type args_parse_cb = Option<
    unsafe extern "C" fn(*mut args, u_int, *mut *mut ::core::ffi::c_char) -> args_parse_type,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse {
    pub template: *const ::core::ffi::c_char,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry {
    pub name: *const ::core::ffi::c_char,
    pub alias: *const ::core::ffi::c_char,
    pub args: args_parse,
    pub usage: *const ::core::ffi::c_char,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: Option<unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval>,
}
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
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
