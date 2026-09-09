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
//! Detaching walks the live client list, as tmux does when recording pending exits.
//! The previous session is also retained until the attachment command returns.

use crate::arguments::{args_get_str, args_has};
use crate::cfg::{cfg_show_causes_for_session, configuration_finished};
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_target;

use crate::ffi::getuid;
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};
use crate::server::client_walk;
use crate::session::sessions_empty;
pub use crate::types::*;

pub use crate::consts::{
    CLIENT_READONLY, CMD_FIND_PANE, CMD_FIND_PREFER_UNATTACHED, CMD_FIND_SESSION, CMD_READONLY,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_STARTSERVER, CMDQ_STATE_REPEAT, MSG_DETACH,
    MSG_DETACHKILL,
};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

pub(crate) static cmd_attach_session_entry: RustCommandEntry = RustCommandEntry {
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

/// Sends every client of `s` other than `c` away with `msgtype`, which is what
/// `-d` and `-x` ask for. A client attached to something else, and the one
/// doing the attaching, are left alone.
unsafe fn detach_others(c: &ClientRef, s: &SessionRef, msgtype: msgtype) {
    unsafe {
        for mut c_loop in client_walk() {
            if !c_loop.ptr_eq(c)
                && c_loop
                    .attached_session()
                    .is_some_and(|session| session.ptr_eq(s))
            {
                c_loop.detach(msgtype);
            }
        }
    }
}

/// Whether `tflag` names something inside a session rather than a session,
/// which is what decides how much of the target has to be resolved. The C looks
/// for the first `:` or `.` with `strcspn` and asks whether it landed on one.
fn names_a_pane(tflag: Option<&CStr>) -> bool {
    tflag.is_some_and(|tflag| tflag.to_bytes().iter().any(|b| *b == b':' || *b == b'.'))
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn cmd_attach_session(
    item: &cmdq_item,
    tflag: Option<&CStr>,
    dflag: c_int,
    xflag: c_int,
    rflag: c_int,
    cflag: Option<&CStr>,
    Eflag: c_int,
    fflag: Option<&CStr>,
) -> cmd_retval {
    unsafe {
        if sessions_empty() {
            item.error(c"no sessions", fmt_args![]);
            return CMD_RETURN_ERROR;
        }

        let Some(mut c) = item.client() else {
            return CMD_RETURN_NORMAL;
        };
        if c.is_nested() {
            item.error(
                c"sessions should be nested with care, unset $TMUX to force",
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
        let mut session = target
            .session()
            .expect("a resolved attach target has a session");
        let link = target.winlink_ref();
        let pane = target.pane_list_ref();

        if let Some(link) = link.as_ref() {
            if let Some(pane) = pane.as_ref()
                && let Some(window) = pane.window()
            {
                window.set_active_pane(
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                    1,
                );
            }
            session.set_current(Some(link.index()));
            if link.window().is_some() {
                item.state_ref().update_current_link(
                    link,
                    pane.as_ref().filter(|pane| pane.is_alive()),
                    0,
                );
            }
        }

        if let Some(cflag) = cflag {
            let cwd = {
                let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
                format_defaults_for_handles(
                    &mut ft,
                    Some(&c),
                    Some(&session),
                    link.as_ref(),
                    pane.as_ref(),
                );
                format_expand(&mut ft, cflag)
            };
            session.set_cwd(cwd);
        }
        if let Some(fflag) = fflag {
            c.apply_flags(fflag);
        }
        if rflag != 0 {
            if c.flags() & CLIENT_READONLY as uint64_t != 0 && (c.peer_handle()).uid() != getuid() {
                item.error(c"client is read-only", fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            c.make_read_only();
        }

        let _last_session = c.remember_session();
        let fresh = c.attached_session().is_none();
        if fresh {
            let mut cause: Option<CString> = None;
            if c.open_terminal(&mut cause) != 0 {
                item.error(c"open terminal failed: %s", fmt_args![cause.as_deref()]);
                return CMD_RETURN_ERROR;
            }
        }

        if dflag != 0 || xflag != 0 {
            let msgtype = if xflag != 0 {
                MSG_DETACHKILL
            } else {
                MSG_DETACH
            };
            detach_others(&c, &session, msgtype);
        }
        if Eflag == 0 {
            session.update_environment_from(&c);
        }
        c.set_session(Some(&session));
        if fresh || item.flags() & CMDQ_STATE_REPEAT == 0 {
            c.set_key_table(None);
        }
        if fresh {
            c.finish_attachment();
        }

        if configuration_finished() {
            cfg_show_causes_for_session(Some(&session));
        }
        CMD_RETURN_NORMAL
    }
}

unsafe fn cmd_attach_session_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    unsafe {
        cmd_attach_session(
            item,
            args_get_str(args, b't'),
            args_has(args, b'd'),
            args_has(args, b'x'),
            args_has(args, b'r'),
            args_get_str(args, b'c'),
            args_has(args, b'E'),
            args_get_str(args, b'f'),
        )
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_attach_session.rs"]
mod tests;

#[cfg(test)]
pub use crate::consts::{CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_EXIT_DETACH, CLIENT_IGNORESIZE};
