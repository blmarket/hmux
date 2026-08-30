//! `lock-server`, `lock-session` and `lock-client`: the three lock commands,
//! behind one `exec`.
//!
//! The hook tells them apart by comparing the running command's entry against
//! the `lock-server` and `lock-session` ones, so the last arm is what
//! `lock-client` — and only `lock-client` — reaches. `lock-server` locks every
//! client the server has, `lock-session` every client of the target session,
//! and `lock-client` the one client `-t` named, which the command table
//! resolves for it through `CMD_CLIENT_TFLAG`. Each of the three ends in
//! `recalculate_sizes`, because a locked client stops counting towards the
//! size of the windows it was showing, and all three answer
//! `CMD_RETURN_NORMAL`.
//!
//! Coverage exemptions: none. The message-protocol, argument-parsing,
//! target-finding and return-value constants below are not this module's own,
//! but `test_coverage_cmd_lock_server` reads and pins them through it, so they
//! stay where the transpiler put them.
use crate::cmd::cmd_get_entry;
use crate::cmd::queue::{cmdq_get_target, cmdq_get_target_client};
use crate::resize::recalculate_sizes;
use crate::server::{server_lock, server_lock_client, server_lock_session};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_VERSION: msgtype = 12;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_AFTERHOOK: c_int = 0x4;
pub const CMD_CLIENT_TFLAG: c_int = 0x10;
pub const CMD_TARGET_SESSION_USAGE: &CStr = c"[-t target-session]";
pub const CMD_TARGET_CLIENT_USAGE: &CStr = c"[-t target-client]";
pub(crate) static cmd_lock_server_entry: cmd_entry = cmd_entry {
    name: c"lock-server",
    alias: Some(c"lock"),
    args: args_parse_t {
        template: c"",
        lower: 0,
        upper: 0,
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
    exec: cmd_lock_server_exec,
};
pub(crate) static cmd_lock_session_entry: cmd_entry = cmd_entry {
    name: c"lock-session",
    alias: Some(c"locks"),
    args: args_parse_t {
        template: c"t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: CMD_TARGET_SESSION_USAGE,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_SESSION,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_lock_server_exec,
};
pub(crate) static cmd_lock_client_entry: cmd_entry = cmd_entry {
    name: c"lock-client",
    alias: Some(c"lockc"),
    args: args_parse_t {
        template: c"t:",
        lower: 0,
        upper: 0,
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
    exec: cmd_lock_server_exec,
};

unsafe fn cmd_lock_server_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let entry = cmd_get_entry(self_0);
        if ::core::ptr::eq(entry, &cmd_lock_server_entry) {
            server_lock();
        } else if ::core::ptr::eq(entry, &cmd_lock_session_entry) {
            server_lock_session((*cmdq_get_target(item)).session());
        } else {
            server_lock_client(cmdq_get_target_client(&*item));
        }
        recalculate_sizes();
        CMD_RETURN_NORMAL
    }
}
