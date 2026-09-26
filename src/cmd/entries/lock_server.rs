use crate::src::cmd::cmd_get_entry;
use crate::src::cmd::queue::{cmdq_get_target, cmdq_get_target_client};
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{server_lock, server_lock_client, server_lock_session};
use crate::src::shared::arguments::args_parse;
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_CLIENT_TFLAG, CMD_TARGET_CLIENT_USAGE, CMD_TARGET_SESSION_USAGE,
};
pub static mut cmd_lock_server_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"lock-server\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"lock\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_lock_server_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_lock_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"lock-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"locks\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE.as_ptr(),
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_lock_server_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_lock_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"lock-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"lockc\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_CLIENT_USAGE.as_ptr(),
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG,
        exec: Some(cmd_lock_server_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_lock_server_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    if cmd_get_entry(self_0) == &raw const cmd_lock_server_entry {
        server_lock();
    } else if cmd_get_entry(self_0) == &raw const cmd_lock_session_entry {
        server_lock_session((*target).s);
    } else {
        server_lock_client(tc);
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}
