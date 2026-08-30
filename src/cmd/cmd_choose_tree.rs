//! `choose-tree`, `choose-client`, `choose-buffer` and `customize-mode`: the
//! four commands that put one of the mode-tree browsers on a pane.
//!
//! All four share one exec routine and differ only in which
//! [`WindowMode`](crate::types::WindowMode) they hand
//! [`window_pane_set_mode`], which is picked by comparing the command's own
//! entry against the three named statics — anything else, `choose-tree`
//! included, opens the window tree. The arguments the command was given are
//! passed straight through to the mode, which is what reads `-F`, `-K`, `-O`,
//! the `-s`/`-w` tree level and the template.
//!
//! Quirks kept:
//!
//! * `-O` is parsed here only to be thrown away. The order the parse answers
//!   is never used — the mode tree reads `-O` out of the same arguments and
//!   parses it a second time — so all this routine does with it is refuse a
//!   name no order goes by.
//! * That refusal comes first, before either "nothing to choose from" check,
//!   so `choose-buffer -O bogus` reports the bad order even when the paste
//!   store is empty and no mode would have been opened.
//! * `choose-buffer` with an empty paste store and `choose-client` with no
//!   clients answer success and open nothing at all, rather than reporting
//!   that there is nothing to choose.
//! * `customize-mode` shares the `-O` check although its own template has no
//!   `O`, so the check can never fire for it: the parser turns the flag down
//!   first and `args_has` is what guards the refusal.
//!
//! Coverage exemptions: none.

use crate::arguments::{args_get, args_has};
use crate::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_get_args_ptr, cmd_get_entry};
use crate::fmt_args;
use crate::paste::paste_is_empty;
use crate::server::server_client_how_many;
use crate::sort::sort_order_from_string;
pub use crate::types::*;
use crate::window::window_pane_set_mode;
use ::core::ffi::c_char;
use ::core::ptr::null_mut;
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
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;

pub(crate) static cmd_choose_tree_entry: cmd_entry = cmd_entry {
    name: c"choose-tree",
    alias: None,
    args: args_parse_t {
        template: c"F:f:GK:NO:rst:wyZ",
        lower: 0,
        upper: 1,
        cb: Some(
            cmd_choose_tree_args_parse,
        ),
    },
    usage: c"[-GNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

pub(crate) static cmd_choose_client_entry: cmd_entry = cmd_entry {
    name: c"choose-client",
    alias: None,
    args: args_parse_t {
        template: c"F:f:K:NO:rt:yZ",
        lower: 0,
        upper: 1,
        cb: Some(
            cmd_choose_tree_args_parse,
        ),
    },
    usage: c"[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

pub(crate) static cmd_choose_buffer_entry: cmd_entry = cmd_entry {
    name: c"choose-buffer",
    alias: None,
    args: args_parse_t {
        template: c"F:f:K:NO:rt:yZ",
        lower: 0,
        upper: 1,
        cb: Some(
            cmd_choose_tree_args_parse,
        ),
    },
    usage: c"[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

pub(crate) static cmd_customize_mode_entry: cmd_entry = cmd_entry {
    name: c"customize-mode",
    alias: None,
    args: args_parse_t {
        template: c"F:f:Nt:yZ",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-NZ] [-F format] [-f filter] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_choose_tree_exec,
};

/// How the parser is told to read the template the three tree commands take:
/// as a command list if it parses as one, and as a plain string otherwise.
unsafe fn cmd_choose_tree_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

/// The mode the entry behind `self_0` opens, or nothing when there is nothing
/// to choose from — an empty paste store for `choose-buffer`, no clients at
/// all for `choose-client` — which ends the command having opened no mode.
///
/// The comparison is against the entry statics themselves, so `choose-tree`
/// and any future entry sharing this exec fall through to the window tree.
fn cmd_choose_tree_mode(self_0: &cmd) -> Option<WindowMode> {
    let entry = cmd_get_entry(self_0);
    if ::core::ptr::eq(entry, &cmd_choose_buffer_entry) {
        match paste_is_empty() {
            0 => Some(WindowMode::Buffer),
            _ => None,
        }
    } else if ::core::ptr::eq(entry, &cmd_choose_client_entry) {
        match server_client_how_many() {
            0 => None,
            _ => Some(WindowMode::Client),
        }
    } else if ::core::ptr::eq(entry, &cmd_customize_mode_entry) {
        Some(WindowMode::Customize)
    } else {
        Some(WindowMode::Tree)
    }
}

/// Whether the `-O` the command carries names an order. A command with no `-O`
/// is fine whatever the parse answered, which is what keeps `customize-mode`,
/// whose template has no `O` at all, out of the refusal.
unsafe fn cmd_choose_tree_order_is_known(args: &args) -> bool {
    unsafe { sort_order_from_string(args_get(args, b'O')) != SORT_END || args_has(args, b'O') == 0 }
}

unsafe fn cmd_choose_tree_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        if !cmd_choose_tree_order_is_known(args) {
            cmdq_error(item, c"invalid sort order".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        if let Some(mode) = cmd_choose_tree_mode(self_0) {
            let target = cmdq_get_target(item);
            window_pane_set_mode(
                (*target).pane(),
                null_mut::<window_pane>(),
                mode,
                target,
                cmd_get_args_ptr(self_0),
            );
        }
        CMD_RETURN_NORMAL
    }
}
