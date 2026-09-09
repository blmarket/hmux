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

use crate::cmd::cmd_get_entry;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_NORMAL,
    CMD_TARGET_CLIENT_USAGE, CMD_TARGET_SESSION_USAGE,
};
use crate::resize::recalculate_sizes;
use crate::server::server_lock;
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
pub use crate::cmd::cmdq_item;
pub use crate::types::args_parse_t;
use ::core::ffi::c_char;

pub(crate) static cmd_lock_server_entry: RustCommandEntry = RustCommandEntry {
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
pub(crate) static cmd_lock_session_entry: RustCommandEntry = RustCommandEntry {
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
pub(crate) static cmd_lock_client_entry: RustCommandEntry = RustCommandEntry {
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

unsafe fn cmd_lock_server_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let entry = cmd_get_entry(self_0);
    if core::ptr::eq(entry, &cmd_lock_server_entry) {
        server_lock();
    } else if core::ptr::eq(entry, &cmd_lock_session_entry) {
        if let Some(mut s) = item.target.session() {
            unsafe { s.lock() };
        };
    } else {
        let target_client = item.target_client();
        unsafe { target_client.expect("lock-client target").lock() };
    }
    recalculate_sizes();
    CMD_RETURN_NORMAL
}
