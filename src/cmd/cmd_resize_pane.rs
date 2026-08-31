use crate::arguments::{args_count, args_has, args_percentage, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_event, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_mouse_pane, cmd_mouse_window};
use crate::compat::strtonum;
use crate::fmt_args;
use crate::grid::grid_remove_history;
use crate::layout::{
    layout_fix_panes, layout_resize_layout, layout_resize_pane, layout_resize_pane_to,
    layout_search_by_border, layout_set_size,
};
use crate::options::options_get_number;
use crate::screen::screen_grid_ptr;
use crate::server::{server_redraw_window, server_redraw_window_borders, server_unzoom_window};
pub use crate::types::*;
use crate::window::{
    window_pane_is_floating, window_pane_show_scrollbar, window_redraw_active_switch,
    window_set_active_pane, window_unzoom, window_zoom,
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
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2;
pub const PANE_SCROLLBARS_RIGHT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub(crate) static cmd_resize_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"resize-pane",
        alias: Some(c"resizep"),
        args: args_parse_t {
            template: c"DLMRTt:Ux:y:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DLMRTUZ] [-x width] [-y height] [-t target-pane] [adjustment]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_resize_pane_exec,
    }
};
unsafe fn cmd_resize_pane_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut wp: *mut window_pane = (*target).pane();
        let mut wl: *mut winlink = (*target).winlink();
        let mut w: *mut window = (*wl).window();
        let mut cause = None;
        let mut adjust: u_int = 0;
        let mut x: ::core::ffi::c_int = 0;
        let mut y: ::core::ffi::c_int = 0;
        let mut status: ::core::ffi::c_int = 0;
        let mut gd: *mut grid = screen_grid_ptr(&raw mut (*wp).base);
        if args_has(args, 'T' as i32 as u_char) != 0 {
            if !(*wp).modes.is_empty() {
                return CMD_RETURN_NORMAL;
            }
            adjust = (*gd)
                .sy
                .wrapping_sub(1 as u_int)
                .wrapping_sub((*wp).base.cy);
            if adjust > (*gd).hsize {
                adjust = (*gd).hsize;
            }
            grid_remove_history(&mut *gd, adjust);
            (*wp).base.cy = (*wp).base.cy.wrapping_add(adjust);
            (*wp).flags |= PANE_REDRAW;
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'M' as i32 as u_char) != 0 {
            return cmd_resize_pane_mouse_update(self_0, item);
        }
        if args_has(args, 'Z' as i32 as u_char) != 0 {
            if (*w).flags & WINDOW_ZOOMED != 0 {
                window_unzoom(w, 1 as ::core::ffi::c_int);
            } else {
                window_zoom(wp);
            }
            server_redraw_window(w);
            return CMD_RETURN_NORMAL;
        }
        server_unzoom_window(w);
        if args_count(args) == 0 as u_int {
            adjust = 1 as u_int;
        } else {
            match strtonum(
                args_string(args, 0 as u_int),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
            ) {
                Ok(value) => adjust = value as u_int,
                Err(errstr) => {
                    cmdq_error(item, c"adjustment %s".as_ptr(), fmt_args![errstr.as_ptr()]);
                    return CMD_RETURN_ERROR;
                }
            }
        }
        if args_has(args, 'x' as i32 as u_char) != 0 {
            x = args_percentage(
                args,
                'x' as i32 as u_char,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                (*w).sx as ::core::ffi::c_longlong,
                &mut cause,
            ) as ::core::ffi::c_int;
            if let Some(cause) = cause.as_ref() {
                cmdq_error(item, c"width %s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
            layout_resize_pane_to(wp, LAYOUT_LEFTRIGHT, x as u_int);
        }
        if args_has(args, 'y' as i32 as u_char) != 0 {
            y = args_percentage(
                args,
                'y' as i32 as u_char,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                (*w).sy as ::core::ffi::c_longlong,
                &mut cause,
            ) as ::core::ffi::c_int;
            if let Some(cause) = cause.as_ref() {
                cmdq_error(item, c"height %s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
            status = options_get_number((*w).options_ptr(), c"pane-border-status".as_ptr())
                as ::core::ffi::c_int;
            match status {
                PANE_STATUS_TOP => {
                    if y != INT_MAX && (*wp).yoff == 1 as ::core::ffi::c_int {
                        y += 1;
                    }
                }
                PANE_STATUS_BOTTOM
                    if y != INT_MAX
                        && ((*wp).yoff as u_int).wrapping_add((*wp).sy)
                            == (*w).sy.wrapping_sub(1 as u_int) =>
                {
                    y += 1;
                }
                _ => {}
            }
            layout_resize_pane_to(wp, LAYOUT_TOPBOTTOM, y as u_int);
        }
        if args_has(args, 'L' as i32 as u_char) != 0 {
            layout_resize_pane(
                wp,
                LAYOUT_LEFTRIGHT,
                adjust.wrapping_neg() as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        } else if args_has(args, 'R' as i32 as u_char) != 0 {
            layout_resize_pane(
                wp,
                LAYOUT_LEFTRIGHT,
                adjust as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        } else if args_has(args, 'U' as i32 as u_char) != 0 {
            layout_resize_pane(
                wp,
                LAYOUT_TOPBOTTOM,
                adjust.wrapping_neg() as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        } else if args_has(args, 'D' as i32 as u_char) != 0 {
            layout_resize_pane(
                wp,
                LAYOUT_TOPBOTTOM,
                adjust as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        }
        server_redraw_window((*wl).window());
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_resize_pane_mouse_update(_self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut event: *mut key_event = cmdq_get_event(item);
        let mut wp: *mut window_pane = (*target).pane();
        let mut wl: *mut winlink = (*target).winlink();
        let mut w: *mut window = (*wl).window();
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut s: *mut session = (*target).session();
        if (*event).m.valid == 0 {
            return CMD_RETURN_NORMAL;
        }
        let Some((mouse_s, _, mouse_wp)) = cmd_mouse_pane(&raw mut (*event).m) else {
            return CMD_RETURN_NORMAL;
        };
        s = mouse_s;
        wp = mouse_wp;
        if c.is_null() || (*c).session != s {
            return CMD_RETURN_NORMAL;
        }
        if window_pane_is_floating(wp) == 0 {
            (*c).tty.mouse_drag_update = Some(cmd_resize_pane_mouse_update_tiled);
            cmd_resize_pane_mouse_update_tiled(c, &raw mut (*event).m);
            return CMD_RETURN_NORMAL;
        }
        window_redraw_active_switch(w, wp);
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
        (*c).tty.mouse_drag_update = Some(cmd_resize_pane_mouse_update_floating);
        cmd_resize_pane_mouse_update_floating(c, &raw mut (*event).m);
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_resize_pane_mouse_update_floating(mut c: *mut client, mut m: *mut mouse_event) {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut y: ::core::ffi::c_int = 0;
        let mut ly: ::core::ffi::c_int = 0;
        let mut x: ::core::ffi::c_int = 0;
        let mut lx: ::core::ffi::c_int = 0;
        let mut sx: ::core::ffi::c_int = 0;
        let mut sy: ::core::ffi::c_int = 0;
        let mut new_sx: ::core::ffi::c_int = 0;
        let mut new_sy: ::core::ffi::c_int = 0;
        let mut left: ::core::ffi::c_int = 0;
        let mut right: ::core::ffi::c_int = 0;
        let mut new_xoff: ::core::ffi::c_int = 0;
        let mut new_yoff: ::core::ffi::c_int = 0;
        let mut resizes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let Some((_, mouse_wl, mouse_wp)) = cmd_mouse_pane(m) else {
            (*c).tty.mouse_drag_update = None;
            return;
        };
        wl = mouse_wl;
        wp = mouse_wp;
        w = (*wl).window();
        lc = (*wp).layout_cell;
        sx = (*wp).sx as ::core::ffi::c_int;
        sy = (*wp).sy as ::core::ffi::c_int;
        left = (*wp).xoff - 1 as ::core::ffi::c_int;
        right = (*wp).xoff + sx;
        if window_pane_show_scrollbar(wp) != 0 && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
            left -= (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        } else if window_pane_show_scrollbar(wp) != 0 && (*w).sb_pos == PANE_SCROLLBARS_RIGHT {
            right += (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        }
        y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
        x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
        if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
            y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
                as ::core::ffi::c_int;
        } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
            y = (*m).statusat - 1 as ::core::ffi::c_int;
        }
        ly = (*m).ly.wrapping_add((*m).oy) as ::core::ffi::c_int;
        lx = (*m).lx.wrapping_add((*m).ox) as ::core::ffi::c_int;
        if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines as ::core::ffi::c_int
        {
            ly = (ly as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
                as ::core::ffi::c_int;
        } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat {
            ly = (*m).statusat - 1 as ::core::ffi::c_int;
        }
        if (lx == left || lx == left + 1 as ::core::ffi::c_int)
            && ly == (*wp).yoff - 1 as ::core::ffi::c_int
        {
            new_sx = (*lc).sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = (*lc).sy.wrapping_add((ly - y) as u_int) as ::core::ffi::c_int;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            new_xoff = x + 1 as ::core::ffi::c_int;
            new_yoff = y + 1 as ::core::ffi::c_int;
            layout_set_size(lc, new_sx as u_int, new_sy as u_int, new_xoff, new_yoff);
            resizes += 1;
        } else if (lx == right + 1 as ::core::ffi::c_int || lx == right)
            && ly == (*wp).yoff - 1 as ::core::ffi::c_int
        {
            new_sx = x - (*lc).xoff;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = (*lc).sy.wrapping_add((ly - y) as u_int) as ::core::ffi::c_int;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            new_yoff = y + 1 as ::core::ffi::c_int;
            layout_set_size(lc, new_sx as u_int, new_sy as u_int, (*lc).xoff, new_yoff);
            resizes += 1;
        } else if (lx == left || lx == left + 1 as ::core::ffi::c_int) && ly == (*wp).yoff + sy {
            new_sx = (*lc).sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = y - (*lc).yoff;
            if new_sy < PANE_MINIMUM {
                return;
            }
            new_xoff = x + 1 as ::core::ffi::c_int;
            layout_set_size(lc, new_sx as u_int, new_sy as u_int, new_xoff, (*lc).yoff);
            resizes += 1;
        } else if (lx == right + 1 as ::core::ffi::c_int || lx == right) && ly == (*wp).yoff + sy {
            new_sx = x - (*lc).xoff;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = y - (*lc).yoff;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            layout_set_size(lc, new_sx as u_int, new_sy as u_int, (*lc).xoff, (*lc).yoff);
            resizes += 1;
        } else if lx == right {
            new_sx = x - (*lc).xoff;
            if new_sx < PANE_MINIMUM {
                return;
            }
            layout_set_size(lc, new_sx as u_int, (*lc).sy, (*lc).xoff, (*lc).yoff);
            resizes += 1;
        } else if lx == left {
            new_sx = (*lc).sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                return;
            }
            new_xoff = x + 1 as ::core::ffi::c_int;
            layout_set_size(lc, new_sx as u_int, (*lc).sy, new_xoff, (*lc).yoff);
            resizes += 1;
        } else if ly == (*wp).yoff + sy {
            new_sy = y - (*lc).yoff;
            if new_sy < PANE_MINIMUM {
                return;
            }
            layout_set_size(lc, (*lc).sx, new_sy as u_int, (*lc).xoff, (*lc).yoff);
            resizes += 1;
        } else if ly == (*wp).yoff - 1 as ::core::ffi::c_int {
            new_xoff = (*lc).xoff + (x - lx);
            new_yoff = y + 1 as ::core::ffi::c_int;
            layout_set_size(lc, (*lc).sx, (*lc).sy, new_xoff, new_yoff);
            resizes += 1;
        }
        if resizes != 0 as ::core::ffi::c_int {
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            server_redraw_window(w);
            server_redraw_window_borders(w);
        }
    }
}
unsafe fn cmd_resize_pane_mouse_update_tiled(mut c: *mut client, mut m: *mut mouse_event) {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut y: u_int = 0;
        let mut ly: u_int = 0;
        let mut x: u_int = 0;
        let mut lx: u_int = 0;
        static offsets: [[::core::ffi::c_int; 2]; 5] = [
            [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
            [0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int],
            [1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
            [0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)],
            [-(1 as ::core::ffi::c_int), 0 as ::core::ffi::c_int],
        ];
        let mut cells: [*mut layout_cell; 5] = [::core::ptr::null_mut::<layout_cell>(); 5];
        let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut ncells: u_int = 0 as u_int;
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        let mut resizes: u_int = 0 as u_int;
        let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
        wl = cmd_mouse_window(m).map_or(::core::ptr::null_mut::<winlink>(), |(_, wl)| wl);
        if wl.is_null() {
            (*c).tty.mouse_drag_update = None;
            return;
        }
        w = (*wl).window();
        y = (*m).y.wrapping_add((*m).oy);
        x = (*m).x.wrapping_add((*m).ox);
        if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
            y = y.wrapping_sub((*m).statuslines);
        } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat as u_int {
            y = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
        }
        ly = (*m).ly.wrapping_add((*m).oy);
        lx = (*m).lx.wrapping_add((*m).ox);
        if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines {
            ly = ly.wrapping_sub((*m).statuslines);
        } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat as u_int {
            ly = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
        }
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[*mut layout_cell; 5]>() as usize)
                .wrapping_div(::core::mem::size_of::<*mut layout_cell>() as usize)
        {
            lc = layout_search_by_border(
                (*w).layout_root_ptr(),
                lx.wrapping_add(offsets[i as usize][0 as ::core::ffi::c_int as usize] as u_int),
                ly.wrapping_add(offsets[i as usize][1 as ::core::ffi::c_int as usize] as u_int),
            );
            if !lc.is_null() {
                j = 0 as u_int;
                while j < ncells {
                    if cells[j as usize] == lc {
                        lc = ::core::ptr::null_mut::<layout_cell>();
                        break;
                    } else {
                        j = j.wrapping_add(1);
                    }
                }
                if !lc.is_null() {
                    cells[ncells as usize] = lc;
                    ncells = ncells.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }
        if ncells == 0 as u_int {
            return;
        }
        i = 0 as u_int;
        while i < ncells {
            type_0 = (*(*cells[i as usize]).parent).type_0;
            if y != ly
                && type_0 as ::core::ffi::c_uint
                    == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                layout_resize_layout(
                    w,
                    cells[i as usize],
                    type_0,
                    y.wrapping_sub(ly) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                resizes = resizes.wrapping_add(1);
            } else if x != lx
                && type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                layout_resize_layout(
                    w,
                    cells[i as usize],
                    type_0,
                    x.wrapping_sub(lx) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                resizes = resizes.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if resizes != 0 as u_int {
            server_redraw_window(w);
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
