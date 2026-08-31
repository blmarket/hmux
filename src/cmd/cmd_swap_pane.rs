use crate::arguments::args_has;
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_source, cmdq_get_target};
use crate::fmt_args;
use crate::layout::layout_cell_set_pane;
use crate::layout::layout_fix_panes;
use crate::notify::notify_window;
use crate::options::{options_load_pane_colours, options_set_parent};
use crate::server::server_client_remove_pane;
use crate::server::server_redraw_window;
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::window_get_active;
use crate::window::{
    window_pane_is_floating, window_pane_resize, window_pane_set_window, window_pane_stack_remove,
    window_panes_first, window_panes_last, window_panes_next, window_panes_prev, window_panes_swap,
    window_pop_zoom, window_push_zoom, window_set_active_pane,
};
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
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_FIND_DEFAULT_MARKED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub(crate) static cmd_swap_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"swap-pane",
        alias: Some(c"swapp"),
        args: args_parse_t {
            template: c"dDs:t:UZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-dDUZ] [-s src-pane] [-t dst-pane]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_swap_pane_exec,
    }
};
unsafe fn cmd_swap_pane_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut source: *mut cmd_find_state = cmdq_get_source(item);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut src_w: *mut window = ::core::ptr::null_mut::<window>();
        let mut dst_w: *mut window = ::core::ptr::null_mut::<window>();
        let mut tmp_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut src_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut dst_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut src_lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut dst_lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xoff: u_int = 0;
        let mut yoff: u_int = 0;
        dst_w = (*(*target).winlink()).window();
        dst_wp = (*target).pane();
        src_w = (*(*source).winlink()).window();
        src_wp = (*source).pane();
        if window_push_zoom(
            dst_w,
            0 as ::core::ffi::c_int,
            args_has(args, 'Z' as i32 as u_char),
        ) != 0
        {
            server_redraw_window(dst_w);
        }
        if args_has(args, 'D' as i32 as u_char) != 0 {
            src_w = dst_w;
            src_wp = window_panes_next(dst_w, dst_wp);
            if src_wp.is_null() {
                src_wp = window_panes_first(dst_w);
            }
        } else if args_has(args, 'U' as i32 as u_char) != 0 {
            src_w = dst_w;
            src_wp = window_panes_prev(dst_w, dst_wp);
            if src_wp.is_null() {
                src_wp = window_panes_last(dst_w);
            }
        }
        if src_w != dst_w
            && window_push_zoom(
                src_w,
                0 as ::core::ffi::c_int,
                args_has(args, 'Z' as i32 as u_char),
            ) != 0
        {
            server_redraw_window(src_w);
        }
        if !(src_wp == dst_wp) {
            if window_pane_is_floating(src_wp) != 0 || window_pane_is_floating(dst_wp) != 0 {
                cmdq_error(item, c"cannot swap floating panes".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            let src_was_active = window_get_active(src_w) == src_wp;
            let mut dst_was_active = window_get_active(dst_w) == dst_wp;
            server_client_remove_pane(src_wp);
            server_client_remove_pane(dst_wp);
            window_panes_swap(src_w, src_wp, dst_w, dst_wp);
            let src_at = (*src_w)
                .z_index
                .iter()
                .position(|entry| *entry == (*src_wp).id)
                .unwrap();
            let dst_at = (*dst_w)
                .z_index
                .iter()
                .position(|entry| *entry == (*dst_wp).id)
                .unwrap();
            (*src_w).z_index[src_at] = (*dst_wp).id;
            (*dst_w).z_index[dst_at] = (*src_wp).id;
            src_lc = (*src_wp).layout_cell;
            dst_lc = (*dst_wp).layout_cell;
            layout_cell_set_pane(src_lc, dst_wp);
            (*dst_wp).layout_cell = src_lc;
            layout_cell_set_pane(dst_lc, src_wp);
            (*src_wp).layout_cell = dst_lc;
            if window_pane_is_floating(src_wp) != window_pane_is_floating(dst_wp) {
                (*(*src_wp).layout_cell).flags ^= LAYOUT_CELL_FLOATING;
                (*(*dst_wp).layout_cell).flags ^= LAYOUT_CELL_FLOATING;
            }
            window_pane_set_window(src_wp, dst_w);
            options_set_parent(
                (*src_wp).options_ptr(),
                (*dst_w).options_ptr(),
            );
            (*src_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            window_pane_set_window(dst_wp, src_w);
            options_set_parent(
                (*dst_wp).options_ptr(),
                (*src_w).options_ptr(),
            );
            (*dst_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            sx = (*src_wp).sx;
            sy = (*src_wp).sy;
            xoff = (*src_wp).xoff as u_int;
            yoff = (*src_wp).yoff as u_int;
            (*src_wp).xoff = (*dst_wp).xoff;
            (*src_wp).yoff = (*dst_wp).yoff;
            window_pane_resize(src_wp, (*dst_wp).sx, (*dst_wp).sy);
            (*dst_wp).xoff = xoff as ::core::ffi::c_int;
            (*dst_wp).yoff = yoff as ::core::ffi::c_int;
            window_pane_resize(dst_wp, sx, sy);
            if args_has(args, 'd' as i32 as u_char) == 0 {
                if src_w != dst_w {
                    window_set_active_pane(src_w, dst_wp, 1 as ::core::ffi::c_int);
                    window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
                } else {
                    tmp_wp = dst_wp;
                    window_set_active_pane(src_w, tmp_wp, 1 as ::core::ffi::c_int);
                }
            } else {
                if src_was_active {
                    window_set_active_pane(src_w, dst_wp, 1 as ::core::ffi::c_int);
                    dst_was_active |= src_w == dst_w;
                }
                if dst_was_active {
                    window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
                }
            }
            if src_w != dst_w {
                window_pane_stack_remove(src_w, PaneStack::LastUsed, src_wp);
                window_pane_stack_remove(dst_w, PaneStack::LastUsed, dst_wp);
                options_load_pane_colours((*src_wp).options_ptr(), Some(&mut (*src_wp).palette));
                options_load_pane_colours((*dst_wp).options_ptr(), Some(&mut (*dst_wp).palette));
                layout_fix_panes(src_w, ::core::ptr::null_mut::<window_pane>());
                server_redraw_window(src_w);
            }
            layout_fix_panes(dst_w, ::core::ptr::null_mut::<window_pane>());
            server_redraw_window(dst_w);
            notify_window(c"window-layout-changed".as_ptr(), src_w);
            if src_w != dst_w {
                notify_window(c"window-layout-changed".as_ptr(), dst_w);
            }
        }
        if window_pop_zoom(src_w) != 0 {
            server_redraw_window(src_w);
        }
        if src_w != dst_w && window_pop_zoom(dst_w) != 0 {
            server_redraw_window(dst_w);
        }
        CMD_RETURN_NORMAL
    }
}
