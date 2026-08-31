//! `detach-client` and `suspend-client`: the two entries share one exec hook,
//! which tells them apart by the entry the command was parsed against.
//!
//! What the hook decides is who leaves and how. A read-only caller may send
//! only itself away, `-P` picks the harsher of the two exit messages, `-s`
//! takes every client of the source session and `-a` every client but the
//! target, and `-E` replaces the detach with a shell command run in the
//! client. Nothing here reaches the clients themselves: `server_client_detach`
//! records the pending exit on the client and `server_client_exec` hands the
//! command over, both of which the event loop acts on later.
//!
//! The server's client list is walked through its own links, since `client` is
//! the crate's own type and every other module reads the same list.
//!
//! Coverage exemptions: none.
use crate::arguments::{args_get, args_has};
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_source, cmdq_get_target_client};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::server::client_walk;
use crate::server::{server_client_detach, server_client_exec, server_client_suspend};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_FIND_CANFAIL: c_int = 0x40;
pub const CMD_READONLY: c_int = 0x2;
pub const CMD_CLIENT_TFLAG: c_int = 0x10;
pub const CLIENT_READONLY: c_int = 0x800;
pub const CMD_TARGET_CLIENT_USAGE: &CStr = c"[-t target-client]";
pub(crate) static cmd_detach_client_entry: cmd_entry = cmd_entry {
    name: c"detach-client",
    alias: Some(c"detach"),
    args: args_parse_t {
        template: c"aE:s:t:P",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-aP] [-E shell-command] [-s target-session] [-t target-client]",
    source: cmd_entry_flag {
        flag: b's' as ::core::ffi::c_char,
        type_0: CMD_FIND_SESSION,
        flags: CMD_FIND_CANFAIL,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_READONLY | CMD_CLIENT_TFLAG,
    exec: cmd_detach_client_exec,
};
pub(crate) static cmd_suspend_client_entry: cmd_entry = cmd_entry {
    name: c"suspend-client",
    alias: Some(c"suspendc"),
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
    flags: CMD_CLIENT_TFLAG,
    exec: cmd_detach_client_exec,
};

/// Sends `c` away: with `-E` it runs `cmd` in place of the session it was
/// attached to, and otherwise it is told to detach with `msgtype`.
unsafe fn detach_or_exec(c: *mut client, cmd: Option<&CStr>, msgtype: msgtype) {
    unsafe {
        match cmd {
            Some(cmd) => server_client_exec(c, cmd.as_ptr()),
            None => server_client_detach(c, msgtype),
        }
    }
}

unsafe fn cmd_detach_client_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let source = cmdq_get_source(item);
        let c = cmdq_get_client(&*item);
        let tc = cmdq_get_target_client(&*item);
        let cmd = args_get(args, b'E');
        let cmd = if cmd.is_null() {
            None
        } else {
            Some(CStr::from_ptr(cmd))
        };

        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_suspend_client_entry) {
            server_client_suspend(tc);
            return CMD_RETURN_NORMAL;
        }

        if (*c).flags & CLIENT_READONLY as uint64_t != 0
            && (args_has(args, b's') != 0 || args_has(args, b'a') != 0 || c != tc)
        {
            cmdq_error(item, c"client is read-only".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }

        let msgtype = if args_has(args, b'P') != 0 {
            MSG_DETACHKILL
        } else {
            MSG_DETACH
        };

        if args_has(args, b's') != 0 {
            let s = (*source).session();
            if s.is_null() {
                return CMD_RETURN_NORMAL;
            }
            for loop_0 in client_walk().filter(|loop_0| (**loop_0).session == s) {
                detach_or_exec(loop_0, cmd, msgtype);
            }
            return CMD_RETURN_STOP;
        }

        if args_has(args, b'a') != 0 {
            for loop_0 in
                client_walk().filter(|loop_0| !(**loop_0).session.is_null() && *loop_0 != tc)
            {
                detach_or_exec(loop_0, cmd, msgtype);
            }
            return CMD_RETURN_NORMAL;
        }

        detach_or_exec(tc, cmd, msgtype);
        CMD_RETURN_STOP
    }
}
