//! `move-window` and `link-window`: one exec hook, told apart by which entry
//! the running command was parsed as — the link keeps the window where it was,
//! the move unlinks it afterwards.
//!
//! `-t` is special here: the command resolves it itself rather than letting the
//! queue do it, because it wants `CMD_FIND_WINDOW_INDEX`, which answers an
//! index no window holds with the index alone and a null winlink. `-a` and `-b`
//! then make room with [`winlink_shuffle_up`] — around the target window when
//! one was found, and around the destination session's *current* window when
//! `-t` named a free index — and refuse when there is no free index left above
//! the one they were given. `-r` is a different command altogether: it
//! renumbers a session's windows and answers, without moving anything.
//!
//! `server_link_window` is what actually links the window into the destination
//! and reports why it would not; `-k` lets it take an index already in use and
//! `-d` keeps the destination from selecting the newcomer. Afterwards the
//! *source* session is renumbered when its `renumber-windows` is on, unless
//! `-s` named the source, since the destination is already where the caller
//! asked for.
//!
//! Coverage exemptions: none. The message-protocol, enumeration and
//! argument-parsing constants below are not this module's own, but
//! `test_coverage_cmd_move_window` reads and pins them through it, so they stay
//! where the transpiler put them.
use crate::arguments::{args_get, args_has};
use crate::cmd::find::cmd_find_target;
use crate::cmd::queue::{cmdq_error, cmdq_get_source};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::options::options_get_number;
use crate::resize::recalculate_sizes;
use crate::server::{server_link_window, server_status_session, server_unlink_window};
use crate::session::session_renumber_windows;
use crate::session::{session_get_curw, session_options};
pub use crate::types::*;
use crate::window::winlink_shuffle_up;
use ::core::ffi::{c_char, c_int};
use ::std::ffi::CString;
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
pub const CMD_FIND_QUIET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_FIND_WINDOW_INDEX: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub(crate) static cmd_move_window_entry: cmd_entry = cmd_entry {
    name: c"move-window",
    alias: Some(c"movew"),
    args: args_parse_t {
        template: c"abdkrs:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-abdkr] [-s src-window] [-t dst-window]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_move_window_exec,
};
pub(crate) static cmd_link_window_entry: cmd_entry = cmd_entry {
    name: c"link-window",
    alias: Some(c"linkw"),
    args: args_parse_t {
        template: c"abdks:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-abdk] [-s src-window] [-t dst-window]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_move_window_exec,
};

/// What `-t` resolves to, or nothing when it resolves to nothing at all. The
/// entries leave their own target unset, so this hook does the resolution the
/// command queue would otherwise have done, and picks the flags itself.
unsafe fn resolve(
    item: *mut cmdq_item,
    tflag: *const c_char,
    type_0: cmd_find_type,
    flags: c_int,
) -> Option<cmd_find_state> {
    unsafe {
        let mut target = cmd_find_state::default();
        (cmd_find_target(&mut target, item, tflag, type_0, flags) == 0).then_some(target)
    }
}

/// The index `-a` or `-b` frees up in `dst`: around the target window when
/// `-t` found one, and around the destination's current window when
/// `CMD_FIND_WINDOW_INDEX` answered a `-t` index no window holds. Nothing when
/// `winlink_shuffle_up` finds no free index left above the one it was given.
unsafe fn room_for(dst: *mut session, target: &cmd_find_state, before: c_int) -> Option<c_int> {
    unsafe {
        let around = if !target.winlink().is_null() {
            target.winlink()
        } else {
            session_get_curw(dst)
        };
        match winlink_shuffle_up(dst, around, before) {
            -1 => None,
            idx => Some(idx),
        }
    }
}

unsafe fn cmd_move_window_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let source = cmdq_get_source(item);
        let tflag = args_get(args, b't');
        let src = (*source).session();
        let wl = (*source).winlink();

        if args_has(args, b'r') != 0 {
            let Some(target) = resolve(item, tflag, CMD_FIND_SESSION, CMD_FIND_QUIET) else {
                return CMD_RETURN_ERROR;
            };
            session_renumber_windows(target.session());
            recalculate_sizes();
            server_status_session(target.session());
            return CMD_RETURN_NORMAL;
        }

        let Some(target) = resolve(item, tflag, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX) else {
            return CMD_RETURN_ERROR;
        };
        let dst = target.session();
        let mut idx = target.idx;

        let kflag = args_has(args, b'k');
        let dflag = args_has(args, b'd');
        let sflag = args_has(args, b's');

        let before = args_has(args, b'b');
        if args_has(args, b'a') != 0 || before != 0 {
            match room_for(dst, &target, before) {
                Some(freed) => idx = freed,
                None => return CMD_RETURN_ERROR,
            }
        }

        let mut cause: Option<CString> = None;
        if server_link_window(src, wl, dst, idx, kflag, (dflag == 0) as c_int, &mut cause) != 0 {
            let cause = cause.unwrap();
            cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
            return CMD_RETURN_ERROR;
        }
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_move_window_entry) {
            server_unlink_window(src, wl);
        }

        if sflag == 0 && options_get_number(session_options(src), c"renumber-windows".as_ptr()) != 0
        {
            session_renumber_windows(src);
        }
        recalculate_sizes();
        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_move_window.rs"]
mod tests;
