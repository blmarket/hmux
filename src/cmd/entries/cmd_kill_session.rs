//! `kill-session`: destroys sessions, or — under `-C` — destroys nothing and
//! only clears the alerts standing on the target's windows.
//!
//! Which sessions go is picked by the first switch that applies, in the order
//! the C tests them: `-C` clears the bell, activity and silence flags from
//! every window of the target and from its winlinks, and redraws the
//! session's clients; `-a` takes every session in the server but the target;
//! `-g` takes every member of the target's session group, and falls through
//! to the plain kill when the target is in no group at all; and the plain
//! kill takes the target alone. Every kill is `server_destroy_session`, which
//! moves the session's clients somewhere else or sends them away, followed by
//! `session_destroy`. The command always answers `CMD_RETURN_NORMAL`.
//!
//! Both destruction walks retain ordered session handles before mutation,
//! since destruction removes sessions from the registry and their groups.

use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_NORMAL};
use crate::session::SESSIONS_FIELD;
use crate::types::SessionRef;
#[cfg(test)]
use crate::types::session;
use ::core::ffi::c_char;

pub(crate) static cmd_kill_session_entry: RustCommandEntry = RustCommandEntry {
    name: c"kill-session",
    alias: None,
    args: args_parse_t {
        template: c"aCgt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-aCg] [-t target-session]",
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
    flags: 0,
    exec: cmd_kill_session_exec,
};

/// The group `-g` asks for: nothing unless the switch was given, and nothing
/// when the target belongs to no group, which is what makes `-g` fall through
/// to the plain kill.
fn asked_group(
    args: &RustArguments,
    session: &SessionRef,
) -> Option<impl Iterator<Item = SessionRef> + use<>> {
    (args.argument_flag_count(b'g') != 0)
        .then(|| session.group_walk_safe())
        .flatten()
}

/// Takes `s` down: its clients are moved off it or sent away, and the session
/// itself is destroyed under the name the C's `__func__` reported.
unsafe fn destroy(session: &mut SessionRef) {
    unsafe {
        session.prepare_destruction();
        session.destroy(1, c"cmd_kill_session_exec");
    }
}

/// # Safety
/// Execute on the server thread with no outstanding session/window payload
/// borrows or client flag access. Alert clearing and redraw are synchronous
/// and invoke no callbacks;
/// destruction retains its existing server lifecycle requirements.
unsafe fn cmd_kill_session_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let SESSIONS = SESSIONS_FIELD.get();

    let args = crate::Command::command_arguments(self_0).expect("the command carries arguments");
    let mut session = item.target.session().expect("kill target session");

    if args.argument_flag_count(b'C') != 0 {
        unsafe { session.clear_alert_flags() };
        unsafe { session.request_redraw() };
    } else if args.argument_flag_count(b'a') != 0 {
        for mut sloop in SESSIONS.walk_safe().filter(|sloop| !sloop.ptr_eq(&session)) {
            unsafe { destroy(&mut sloop) };
        }
    } else if let Some(members) = asked_group(args, &session) {
        for mut sloop in members {
            unsafe { destroy(&mut sloop) };
        }
    } else {
        unsafe { destroy(&mut session) };
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_kill_session.rs"]
mod tests;
