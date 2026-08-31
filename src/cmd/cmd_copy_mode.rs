//! `copy-mode` and `clock-mode`: put a pane into one of the window modes.
//! Both are the same exec routine, told apart by the entry the command
//! carries — `clock-mode` opens the clock and stops there, everything else in
//! the routine belongs to `copy-mode`.
//!
//! Which pane the mode lands on is the resolved target, or, with `-M`, the
//! pane the mouse report names; `-s` names a second pane whose screen the copy
//! mode reads from instead of the target's own. `-q` is the way back out and
//! takes every mode off the pane. Once the mode is open the remaining flags
//! move the view: `-u` a page back, `-d` a page on (with `-e` to leave the
//! mode at the bottom) and `-S` to wherever the scrollbar slider was dragged.
//!
//! Line numbers are on unless the key that ran the command was a mouse key,
//! which is what [`key_is_mouse`] decides, and they are applied whether the
//! mode was opened by this call or was already there.
//!
//! Quirks kept: `-M` reads the event's mouse report before the null check the
//! line-number test makes of the same pointer, and `-S` reads the client's
//! terminal without checking that there is a client, so either would follow a
//! null pointer on a command queue that had neither; `-q` is tested first, so
//! `copy-mode -q` ignores every other flag; `clock-mode` is tested before `-s`
//! and the flags below it, so a clock item never reaches them; and the mode's
//! line numbers are set on both halves of the "was it already open" branch,
//! only the drag start being conditional.
//!
//! Coverage exemptions: none.

use crate::arguments::args_has;
use crate::cmd::queue::{cmdq_get_client, cmdq_get_event, cmdq_get_source, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_get_args_ptr, cmd_get_entry, cmd_mouse_pane};
use crate::modes::{
    window_copy_pagedown, window_copy_pageup, window_copy_scroll, window_copy_set_line_numbers,
    window_copy_start_drag,
};
use crate::tty::tty_window_offset;
pub use crate::types::*;
use crate::window::{window_pane_reset_mode_all, window_pane_set_mode};
use ::core::ptr::null_mut;
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
pub const KEYC_TYPE_NOTYPE: key_code_type = 13;
pub const KEYC_TYPE_TRIPLECLICK: key_code_type = 12;
pub const KEYC_TYPE_DOUBLECLICK: key_code_type = 11;
pub const KEYC_TYPE_SECONDCLICK: key_code_type = 10;
pub const KEYC_TYPE_MOUSEDRAGEND: key_code_type = 7;
pub const KEYC_TYPE_MOUSEDRAG: key_code_type = 6;
pub const KEYC_TYPE_MOUSEUP: key_code_type = 5;
pub const KEYC_TYPE_MOUSEDOWN: key_code_type = 4;
pub const KEYC_TYPE_MOUSEMOVE: key_code_type = 3;
pub type keyc = ::core::ffi::c_ulong;
pub const KEYC_DRAGGING: keyc = 8589934642;
pub const KEYC_MOUSE: keyc = 8589934641;
pub const KEYC_ANY: keyc = 8589934596;
pub const KEYC_UNKNOWN: keyc = 8589934593;
pub const KEYC_NONE: keyc = 8589934592;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const KEYC_MASK_TYPE: ::core::ffi::c_ulonglong = 0xff00000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_TARGET_PANE_USAGE: &::core::ffi::CStr = c"[-t target-pane]";
pub(crate) static cmd_copy_mode_entry: cmd_entry = cmd_entry {
    name: c"copy-mode",
    alias: None,
    args: args_parse_t {
        template: c"deHMqSs:t:u",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-deHMqSu] [-s src-pane] [-t target-pane]",
    source: cmd_entry_flag {
        flag: b's' as ::core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as ::core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK | CMD_READONLY,
    exec: cmd_copy_mode_exec,
};
pub(crate) static cmd_clock_mode_entry: cmd_entry = cmd_entry {
    name: c"clock-mode",
    alias: None,
    args: args_parse_t {
        template: c"t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: CMD_TARGET_PANE_USAGE,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as ::core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_copy_mode_exec,
};

/// tmux's `KEYC_IS_MOUSE`: a key is a mouse key when it is the mouse key
/// itself, or when its type is one of the mouse types, which the enum keeps in
/// one run from a move to a triple click.
fn key_is_mouse(key: key_code) -> bool {
    let type_0 = key & KEYC_MASK_TYPE;
    key & KEYC_MASK_KEY == KEYC_MOUSE as key_code
        || (type_0 >= (KEYC_TYPE_MOUSEMOVE as key_code) << 32
            && type_0 <= (KEYC_TYPE_TRIPLECLICK as key_code) << 32)
}

unsafe fn cmd_copy_mode_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let event = cmdq_get_event(item);
        let c = cmdq_get_client(&*item);
        let mut wp = (*cmdq_get_target(item)).pane();

        if args_has(args, b'q') != 0 {
            window_pane_reset_mode_all(wp);
            return CMD_RETURN_NORMAL;
        }

        if args_has(args, b'M') != 0 {
            let Some((s, _, mouse_wp)) = cmd_mouse_pane(&raw mut (*event).m) else {
                return CMD_RETURN_NORMAL;
            };
            wp = mouse_wp;
            if c.is_null() || (*c).session != s {
                return CMD_RETURN_NORMAL;
            }
        }

        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_clock_mode_entry) {
            window_pane_set_mode(wp, null_mut(), WindowMode::Clock, null_mut(), null_mut());
            return CMD_RETURN_NORMAL;
        }

        let swp = if args_has(args, b's') != 0 {
            (*cmdq_get_source(item)).pane()
        } else {
            wp
        };

        let line_numbers = if !event.is_null() && key_is_mouse((*event).key) {
            0
        } else {
            1
        };
        let opened = window_pane_set_mode(
            wp,
            swp,
            WindowMode::Copy,
            null_mut(),
            cmd_get_args_ptr(self_0),
        ) == 0;
        window_copy_set_line_numbers(wp, line_numbers);
        if opened && args_has(args, b'M') != 0 {
            window_copy_start_drag(c, &raw mut (*event).m);
        }

        if args_has(args, b'u') != 0 {
            window_copy_pageup(wp, 0);
        }
        if args_has(args, b'd') != 0 {
            window_copy_pagedown(wp, 0, args_has(args, b'e'));
        }
        if args_has(args, b'S') != 0 {
            let (_bigger, _tty_ox, tty_oy, _tty_sx, _tty_sy) = tty_window_offset(&raw mut (*c).tty);
            window_copy_scroll(
                wp,
                (*c).tty.mouse_slider_mpos,
                (*event).m.y,
                tty_oy,
                args_has(args, b'e'),
            );
            return CMD_RETURN_NORMAL;
        }

        CMD_RETURN_NORMAL
    }
}
