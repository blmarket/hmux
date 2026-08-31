//! `kill-window` and `unlink-window`: one exec hook, told apart by which
//! entry the running command was parsed as.
//!
//! `unlink-window` takes the target's winlink out of its session and nothing
//! else, refusing without `-k` when the session is the only thing holding the
//! window — `-k` is what says the window may go with it. `kill-window` takes
//! the window out of every session that holds it. Under `-a` it takes every
//! *other* window of the target's session instead, one at a time until none is
//! left, and only then the target's own window, and only if the session holds
//! that window more than once; a session whose target window is its one and
//! only winlink is answered without touching anything.
//!
//! The `-a` walk kills one window per pass and starts the walk again, the way
//! the C's `RB_FOREACH` did with its `break`, because `server_kill_window`
//! detaches every winlink of that window from every session and so rewrites
//! the tree being walked.
//!
//! Coverage exemptions: none. The message-protocol, enumeration and
//! argument-parsing constants below are not this module's own, but
//! `test_coverage_cmd_kill_window` reads and pins them through it, so they
//! stay where the transpiler put them.
use crate::arguments::args_has;
use crate::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::resize::recalculate_sizes;
use crate::server::{server_kill_window, server_renumber_all, server_unlink_window};
use crate::session::session_is_linked;
pub use crate::types::*;
use crate::window::{winlinks_after, winlinks_before, winlinks_in};
use ::core::ffi::c_char;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
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
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub(crate) static cmd_kill_window_entry: cmd_entry = cmd_entry {
    name: c"kill-window",
    alias: Some(c"killw"),
    args: args_parse_t {
        template: c"at:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-a] [-t target-window]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    flags: 0,
    exec: cmd_kill_window_exec,
};
pub(crate) static cmd_unlink_window_entry: cmd_entry = cmd_entry {
    name: c"unlink-window",
    alias: Some(c"unlinkw"),
    args: args_parse_t {
        template: c"kt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-k] [-t target-window]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    flags: 0,
    exec: cmd_kill_window_exec,
};

/// The first winlink of `s` carrying a window other than `w`, which is the one
/// the `-a` walk gives up next. Reading only the first is what makes the walk
/// start again after every kill: `server_kill_window` detaches its window from
/// every session that holds it, so the tree behind the winlink just answered is
/// no longer the tree that was walked.
unsafe fn first_other(s: *mut session, w: *mut window) -> Option<*mut winlink> {
    unsafe { winlinks_in(s) }.find(|&wl| unsafe { (*wl).window() != w })
}

/// How many of `s`'s winlinks carry `w`.
unsafe fn times_linked(s: *mut session, w: *mut window) -> usize {
    unsafe { winlinks_in(s) }
        .filter(|&wl| unsafe { (*wl).window() == w })
        .count()
}

/// Whether `wl` is the only winlink its session holds.
unsafe fn is_alone(wl: *mut winlink) -> bool {
    unsafe { winlinks_before(wl).is_null() && winlinks_after(wl).is_null() }
}

unsafe fn cmd_kill_window_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let target = cmdq_get_target(item);
        let wl = (*target).winlink();
        let w = (*wl).window();
        let s = (*target).session();

        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_unlink_window_entry) {
            if args_has(args, b'k') == 0 && session_is_linked(s, w) == 0 {
                cmdq_error(
                    item,
                    c"window only linked to one session".as_ptr(),
                    fmt_args![],
                );
                return CMD_RETURN_ERROR;
            }
            server_unlink_window(s, wl);
            recalculate_sizes();
            return CMD_RETURN_NORMAL;
        }

        if args_has(args, b'a') != 0 {
            if is_alone(wl) {
                return CMD_RETURN_NORMAL;
            }
            while let Some(other) = first_other(s, w) {
                server_kill_window((*other).window(), 0);
            }
            if times_linked(s, w) > 1 {
                server_kill_window(w, 0);
            }
            server_renumber_all();
            return CMD_RETURN_NORMAL;
        }

        server_kill_window(w, 1);
        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_kill_window.rs"]
mod tests;
