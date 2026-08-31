//! `kill-server` and `start-server`: the two ends of the server's life cycle,
//! behind one `exec`.
//!
//! Both entries carry the same function, which tells the two commands apart by
//! comparing the running command's entry against the `kill-server` one:
//! `kill-server` sends `SIGTERM` to the server process itself and lets the
//! signal handler take the server down, and `start-server` does nothing at all
//! — its whole effect is the `CMD_STARTSERVER` flag, which the client reads
//! before the command is ever run. Neither takes an argument or a target, and
//! both answer `CMD_RETURN_NORMAL`.
//!
//! Coverage exemptions: none. The argument-parsing, target-finding and
//! return-value constants below are not this module's own, but
//! `test_coverage_cmd_kill_server` reads and pins every one of them through
//! it, so they stay where the transpiler put them.
use crate::cmd::cmd_get_entry;
use crate::ffi::{getpid, kill};
pub use crate::types::*;
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
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub(crate) static cmd_kill_server_entry: cmd_entry = cmd_entry {
    name: c"kill-server",
    alias: None,
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
    flags: 0,
    exec: cmd_kill_server_exec,
};
pub(crate) static cmd_start_server_entry: cmd_entry = cmd_entry {
    name: c"start-server",
    alias: Some(c"start"),
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
    flags: CMD_STARTSERVER,
    exec: cmd_kill_server_exec,
};

unsafe fn cmd_kill_server_exec(self_0: &cmd, _item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_kill_server_entry) {
            kill(getpid(), SIGTERM);
        }
        CMD_RETURN_NORMAL
    }
}
