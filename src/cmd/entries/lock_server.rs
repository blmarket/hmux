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
pub static cmd_lock_server_entry: cmd_entry = {
    cmd_entry {
        name: c"lock-server",
        alias: Some(c"lock"),
        args: args_parse {
            template: c"",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"",
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
pub static cmd_lock_session_entry: cmd_entry = {
    cmd_entry {
        name: c"lock-session",
        alias: Some(c"locks"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
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
pub static cmd_lock_client_entry: cmd_entry = {
    cmd_entry {
        name: c"lock-client",
        alias: Some(c"lockc"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_CLIENT_USAGE,
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
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_lock_server_entry) {
        server_lock();
    } else if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_lock_session_entry) {
        server_lock_session((*target).s);
    } else {
        server_lock_client(tc);
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}
