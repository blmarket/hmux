//! `detach-client` and `suspend-client`: the two entries share one exec hook,
//! which tells them apart by the entry the command was parsed against.
//!
//! What the hook decides is who leaves and how. A read-only caller may send
//! only itself away, `-P` picks the harsher of the two exit messages, `-s`
//! takes every client of the source session and `-a` every client but the
//! target, and `-E` replaces the detach with a shell command run in the
//! client. Nothing here reaches the clients themselves: `ClientRef::detach`
//! records the pending exit on the client and `ClientRef::exec_shell` hands the
//! command over, both of which the event loop acts on later.
//!
//! Multi-client operations walk the live client list while recording deferred exits.

use crate::args::args_has;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{
    CLIENT_READONLY, CMD_CLIENT_TFLAG, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_SESSION,
    CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_TARGET_CLIENT_USAGE,
    MSG_DETACH, MSG_DETACHKILL,
};
use crate::fmt_args;
use crate::server::client_walk;
use crate::types::{ClientRef, args_parse_t, msgtype, uint64_t};
use ::core::ffi::CStr;

pub(crate) static cmd_detach_client_entry: RustCommandEntry = RustCommandEntry {
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
        flag: b's' as core::ffi::c_char,
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
pub(crate) static cmd_suspend_client_entry: RustCommandEntry = RustCommandEntry {
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
unsafe fn detach_or_exec(c: &mut ClientRef, cmd: Option<&CStr>, msgtype: msgtype) {
    unsafe {
        match cmd {
            Some(cmd) => c.exec_shell(cmd),
            None => c.detach(msgtype),
        }
    }
}

unsafe fn cmd_detach_client_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let c = item.client();
    let mut tc = item.target_client();
    let source = &item.source;
    let cmd = args.argument_flag_string(b'E');

    if core::ptr::eq(cmd_get_entry(self_0), &cmd_suspend_client_entry) {
        unsafe { tc.as_mut().expect("the command has a client").suspend() };
        return CMD_RETURN_NORMAL;
    }

    if unsafe {
        c.as_ref().expect("the command has a client").flags() & CLIENT_READONLY as uint64_t != 0
            && (args_has(args, b's') != 0
                || args_has(args, b'a') != 0
                || match (&c, &tc) {
                    (Some(c), Some(tc)) => !c.ptr_eq(tc),
                    (None, None) => false,
                    _ => true,
                })
    } {
        unsafe { item.error(c"client is read-only", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }

    let msgtype = if args_has(args, b'P') != 0 {
        MSG_DETACHKILL
    } else {
        MSG_DETACH
    };

    if args_has(args, b's') != 0 {
        let Some(s_ref) = source.session() else {
            return CMD_RETURN_NORMAL;
        };
        for mut loop_0 in client_walk() {
            if loop_0
                .attached_session()
                .is_some_and(|session| session.ptr_eq(&s_ref))
            {
                unsafe { detach_or_exec(&mut loop_0, cmd, msgtype) };
            }
        }
        return CMD_RETURN_STOP;
    }

    if args_has(args, b'a') != 0 {
        for mut loop_0 in client_walk() {
            if {
                !loop_0.attached_session().is_none()
                    && tc.as_ref().is_none_or(|tc| !loop_0.ptr_eq(tc))
            } {
                unsafe { detach_or_exec(&mut loop_0, cmd, msgtype) };
            }
        }
        return CMD_RETURN_NORMAL;
    }

    unsafe { detach_or_exec(tc.as_mut().expect("the command has a client"), cmd, msgtype) };
    CMD_RETURN_STOP
}
