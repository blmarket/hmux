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
//! The two walks that destroy as they go read the successor before the body
//! has the session, the way the C's `RB_FOREACH_SAFE` and
//! `TAILQ_FOREACH_SAFE` did, since `session_destroy` takes the session out of
//! both the server's tree and its group's list.
//!
//! Coverage exemptions: none. The message-protocol, enumeration and
//! argument-parsing constants below are not this module's own, but
//! `test_coverage_cmd_kill_session` reads and pins them through it, so they
//! stay where the transpiler put them.
use crate::arguments::args_has;
use crate::cmd::cmd_get_args;
use crate::cmd::queue::cmdq_get_target;
use crate::server::{server_destroy_session, server_redraw_session};
use crate::session::group_walk;
use crate::session::{session_destroy, session_group_contains, session_owners};
pub use crate::types::*;
use crate::window::winlinks_in;
use ::core::ffi::{c_char, c_int};
pub const MSG_EXEC: msgtype = 217;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
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
pub const RB_NEGINF: c_int = -1;
pub const WINDOW_BELL: c_int = 0x1;
pub const WINDOW_ACTIVITY: c_int = 0x2;
pub const WINDOW_SILENCE: c_int = 0x4;
pub const WINDOW_ALERTFLAGS: c_int = WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE;
pub const WINLINK_BELL: c_int = 0x1;
pub const WINLINK_ACTIVITY: c_int = 0x2;
pub const WINLINK_SILENCE: c_int = 0x4;
pub const WINLINK_ALERTFLAGS: c_int = WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub(crate) static cmd_kill_session_entry: cmd_entry = cmd_entry {
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

/// Every session the server knows, in name order, walked by the handles that
/// own them, so that destroying one neither loses the rest of the tree nor
/// leaves the walk holding a session that has been given up.
fn each_session() -> impl Iterator<Item = SessionRef> {
    session_owners().into_iter()
}

/// The sessions of `sg`, in the order its list holds them, walked the way
/// the C's `TAILQ_FOREACH_SAFE` walked them for the same reason.
fn members_of(sg: *mut session_group) -> impl Iterator<Item = *mut session> {
    unsafe { group_walk(sg) }
}

/// The group `-g` asks for: nothing unless the switch was given, and nothing
/// when the target belongs to no group, which is what makes `-g` fall through
/// to the plain kill.
unsafe fn asked_group(args: &args, s: *mut session) -> Option<*mut session_group> {
    unsafe {
        (args_has(args, b'g') != 0)
            .then(|| session_group_contains(s))
            .filter(|sg| !sg.is_null())
    }
}

/// Takes `s` down: its clients are moved off it or sent away, and the session
/// itself is destroyed under the name the C's `__func__` reported.
unsafe fn destroy(s: *mut session) {
    unsafe {
        server_destroy_session(s);
        session_destroy(s, 1, c"cmd_kill_session_exec".as_ptr());
    }
}

unsafe fn cmd_kill_session_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let s = (*cmdq_get_target(item)).session();

        if args_has(args, b'C') != 0 {
            for wl in winlinks_in(s) {
                (*(*wl).window()).flags &= !WINDOW_ALERTFLAGS;
                (*wl).flags &= !WINLINK_ALERTFLAGS;
            }
            server_redraw_session(s);
        } else if args_has(args, b'a') != 0 {
            for sloop in each_session().filter(|sloop| sloop.as_ptr() != s) {
                destroy(sloop.as_ptr());
            }
        } else if let Some(sg) = asked_group(args, s) {
            for sloop in members_of(sg) {
                destroy(sloop);
            }
        } else {
            destroy(s);
        }
        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_kill_session.rs"]
mod tests;
