//! `attach-session`: gives a client the session a target names, either as a
//! switch away from the session it already has or as a fresh attach that has to
//! open the client's terminal first.
//!
//! [`cmd_attach_session`] is called both by this command's own exec hook and by
//! `new-session`, which parses the same flags itself, which is why the flags
//! arrive one by one rather than as an `args` to read.
//!
//! What the two arms share is the middle: `-d` and `-x` send the other clients
//! of the target away, `-E` withholds the session's `update-environment` pass,
//! and the client is then given the session. A fresh attach opens the terminal
//! before all that and, afterwards, always re-chooses the key table, tells the
//! client it is ready and marks it attached; a switch re-chooses the key table
//! only when the command is not being repeated. Neither arm reaches the clients
//! themselves: `server_client_detach` records the pending exit and
//! `server_client_set_session` schedules the redraw, both of which the event
//! loop acts on later.
//!
//! Two orderings are kept as the C has them: `-c` and `-f` have already
//! rewritten the session's working directory and the client's flags by the time
//! `-r` can refuse a read-only client, so a refused attach leaves both behind;
//! and the client's `last_session` is written before either arm can fail.
//!
//! The server's client list is walked through its own links, since `client` is
//! the crate's own type and every other module reads the same list.
//!
//! Coverage exemptions: the `MSG_READY` sent to a client that is not a control
//! client. `server_client_open` waives the terminal only for control clients,
//! so reaching that send means having taken a real terminal through `tty_open`
//! and having a live peer to send to.
use crate::arguments::{args_get, args_has};
use crate::cfg::{cfg_finished, cfg_show_causes};
use crate::cmd::cmd_get_args;
use crate::cmd::find::{cmd_find_from_winlink, cmd_find_from_winlink_pane, cmd_find_target};
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_flags};
use crate::environ::{environ_ptr, environ_update};
use crate::ffi::getuid;
use crate::fmt_args;
use crate::format::format_single;
use crate::notify::notify_client;
use crate::proc::{peer_ptr, proc_get_peer_uid, proc_send};
use crate::server::client_set_last_session;
use crate::server::client_walk;
use crate::server::{
    server_client_check_nested, server_client_detach, server_client_open, server_client_set_flags,
    server_client_set_key_table, server_client_set_session,
};
use crate::session::{
    session_environ, session_options, session_set_current, session_set_cwd, sessions,
};
pub use crate::types::*;
use crate::window::window_set_active_pane;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null;
use ::std::ffi::CString;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_READY: msgtype = 207;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_FIND_PREFER_UNATTACHED: c_int = 0x1;
pub const CMDQ_STATE_REPEAT: c_int = 0x1;
pub const CMD_STARTSERVER: c_int = 0x1;
pub const CMD_READONLY: c_int = 0x2;
pub const CLIENT_ATTACHED: c_int = 0x80;
pub const CLIENT_READONLY: c_int = 0x800;
pub const CLIENT_CONTROL: c_int = 0x2000;
pub const CLIENT_IGNORESIZE: c_int = 0x20000;
pub(crate) static cmd_attach_session_entry: cmd_entry = cmd_entry {
    name: c"attach-session",
    alias: Some(c"attach"),
    args: args_parse_t {
        template: c"c:dEf:rt:x",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-dErx] [-c working-directory] [-f flags] [-t target-session]",
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
    flags: CMD_STARTSERVER | CMD_READONLY,
    exec: cmd_attach_session_exec,
};

/// What one of the server's walks answered, as nothing once it has run out.
fn walked<T>(p: *mut T) -> Option<*mut T> {
    if p.is_null() { None } else { Some(p) }
}

/// Sends every client of `s` other than `c` away with `msgtype`, which is what
/// `-d` and `-x` ask for. A client attached to something else, and the one
/// doing the attaching, are left alone.
unsafe fn detach_others(c: *mut client, s: *mut session, msgtype: msgtype) {
    unsafe {
        for c_loop in client_walk().filter(|c_loop| (**c_loop).session == s && *c_loop != c) {
            server_client_detach(c_loop, msgtype);
        }
    }
}

/// Whether `tflag` names something inside a session rather than a session,
/// which is what decides how much of the target has to be resolved. The C looks
/// for the first `:` or `.` with `strcspn` and asks whether it landed on one.
unsafe fn names_a_pane(tflag: *const c_char) -> bool {
    !tflag.is_null()
        && unsafe { CStr::from_ptr(tflag) }
            .to_bytes()
            .iter()
            .any(|b| *b == b':' || *b == b'.')
}

pub unsafe fn cmd_attach_session(
    item: *mut cmdq_item,
    tflag: *const c_char,
    dflag: c_int,
    xflag: c_int,
    rflag: c_int,
    cflag: *const c_char,
    Eflag: c_int,
    fflag: *const c_char,
) -> cmd_retval {
    unsafe {
        if sessions.map().is_empty() {
            cmdq_error(item, c"no sessions".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }

        let c = cmdq_get_client(&*item);
        if c.is_null() {
            return CMD_RETURN_NORMAL;
        }
        if server_client_check_nested(c) != 0 {
            cmdq_error(
                item,
                c"sessions should be nested with care, unset $TMUX to force".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }

        let (type_0, flags) = if names_a_pane(tflag) {
            (CMD_FIND_PANE, 0)
        } else {
            (CMD_FIND_SESSION, CMD_FIND_PREFER_UNATTACHED)
        };
        let mut target = cmd_find_state::default();
        if cmd_find_target(&mut target, item, tflag, type_0, flags) != 0 {
            return CMD_RETURN_ERROR;
        }
        let s = target.session();
        let wl = target.winlink();
        let wp = target.pane();

        if !wl.is_null() {
            let current = cmdq_get_current(item);
            if !wp.is_null() {
                window_set_active_pane((*wp).window, wp, 1);
            }
            session_set_current(s, wl);
            match walked(wp) {
                Some(wp) => cmd_find_from_winlink_pane(&mut *current, wl, wp, 0),
                None => cmd_find_from_winlink(&mut *current, wl, 0),
            }
        }

        if !cflag.is_null() {
            let cwd = format_single(item, CStr::from_ptr(cflag), c, s, wl, wp);
            session_set_cwd(s, cwd);
        }
        if !fflag.is_null() {
            server_client_set_flags(c, fflag);
        }
        if rflag != 0 {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0
                && proc_get_peer_uid(peer_ptr(&(*c).peer)) != getuid()
            {
                cmdq_error(item, c"client is read-only".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            (*c).flags |= (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
        }

        client_set_last_session(c, (*c).session);
        let fresh = (*c).session.is_null();
        if fresh {
            let mut cause: Option<CString> = None;
            if server_client_open(c, &mut cause) != 0 {
                cmdq_error(
                    item,
                    c"open terminal failed: %s".as_ptr(),
                    fmt_args![cause.as_ref().map_or(null(), |cause| cause.as_ptr())],
                );
                return CMD_RETURN_ERROR;
            }
        }

        if dflag != 0 || xflag != 0 {
            let msgtype = if xflag != 0 {
                MSG_DETACHKILL
            } else {
                MSG_DETACH
            };
            detach_others(c, s, msgtype);
        }
        if Eflag == 0 {
            environ_update(
                session_options(s),
                environ_ptr(&(*c).environ),
                session_environ(s),
            );
        }
        server_client_set_session(c, s);
        if fresh || cmdq_get_flags(&*item) & CMDQ_STATE_REPEAT == 0 {
            server_client_set_key_table(c, null());
        }
        if fresh {
            if (*c).flags & CLIENT_CONTROL as uint64_t == 0 {
                proc_send(peer_ptr(&(*c).peer), MSG_READY, -1, null(), 0);
            }
            notify_client(c"client-attached".as_ptr(), c);
            (*c).flags |= CLIENT_ATTACHED as uint64_t;
        }

        if cfg_finished != 0 {
            cfg_show_causes(s);
        }
        CMD_RETURN_NORMAL
    }
}

unsafe fn cmd_attach_session_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        cmd_attach_session(
            item,
            args_get(args, b't'),
            args_has(args, b'd'),
            args_has(args, b'x'),
            args_has(args, b'r'),
            args_get(args, b'c'),
            args_has(args, b'E'),
            args_get(args, b'f'),
        )
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_attach_session.rs"]
mod tests;
