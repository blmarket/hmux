use crate::arguments::{
    args_count, args_get, args_has, args_percentage, args_string, args_strtonum, args_to_vector,
    args_value_list,
};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_event, cmdq_get_target, cmdq_get_target_client};
use crate::environ::{environ_create_box, environ_ptr, environ_put, environ_t};
use crate::ffi::{strcmp, strtol};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::format::{format_add, format_create_from_target, format_expand};
use crate::log::log_debug;
use crate::options::options_find_choice;
use crate::options::options_get_ptr;
use crate::options::{options_get_number, options_get_string, options_table_entry};
use crate::overlay::menu_create;
use crate::overlay::{menu_add_item, menu_display};
use crate::overlay::{popup_display, popup_modify, popup_present};
use crate::server::{server_client_clear_overlay, server_client_get_cwd};
use crate::session::{session_get_curw, session_options};
use crate::status::{status_at_line, status_line_size};
use crate::text::key_string_lookup_string;
use crate::tmux::checkshell;
use crate::tty::tty_window_offset;
pub use crate::types::*;
use ::std::ffi::{CStr, CString};
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
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_PADDED: box_lines = 5;
pub const BOX_LINES_ROUNDED: box_lines = 4;
pub const BOX_LINES_SIMPLE: box_lines = 3;
pub const BOX_LINES_HEAVY: box_lines = 2;
pub const BOX_LINES_DOUBLE: box_lines = 1;
pub const BOX_LINES_SINGLE: box_lines = 0;
pub const BOX_LINES_DEFAULT: box_lines = -1;
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
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_STRING: options_table_type = 0;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const _PATH_BSHELL: &CStr = c"/bin/sh";
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MENU_NOMOUSE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MENU_STAYOPEN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const POPUP_CLOSEEXIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const POPUP_CLOSEEXITZERO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const POPUP_CLOSEANYKEY: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub(crate) static cmd_display_menu_entry: cmd_entry = {
    cmd_entry {
        name: c"display-menu",
        alias: Some(c"menu"),
        args: args_parse_t {
            template: c"b:c:C:H:s:S:MOt:T:x:y:",
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(
                cmd_display_menu_args_parse,
            ),
        },
        usage: c"[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...",
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: cmd_display_menu_exec,
    }
};
pub(crate) static cmd_display_popup_entry: cmd_entry = {
    cmd_entry {
        name: c"display-popup",
        alias: Some(c"popup"),
        args: args_parse_t {
            template: c"Bb:Cc:d:e:Eh:kNs:S:t:T:w:x:y:",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]",
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: cmd_display_popup_exec,
    }
};
unsafe fn cmd_display_menu_args_parse(
    args: &args,
    mut idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    unsafe {
        let mut i: u_int = 0 as u_int;
        let mut type_0: args_parse_type = ARGS_PARSE_STRING;
        loop {
            type_0 = ARGS_PARSE_STRING;
            if i == idx {
                break;
            }
            let fresh0 = i;
            i = i.wrapping_add(1);
            if *args_string(args, fresh0) as ::core::ffi::c_int == '\0' as i32 {
                continue;
            }
            type_0 = ARGS_PARSE_STRING;
            let fresh1 = i;
            i = i.wrapping_add(1);
            if fresh1 == idx {
                break;
            }
            type_0 = ARGS_PARSE_COMMANDS_OR_STRING;
            let fresh2 = i;
            i = i.wrapping_add(1);
            if fresh2 == idx {
                break;
            }
        }
        type_0
    }
}
unsafe fn cmd_display_menu_get_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    args: &args,
    mut w: u_int,
    mut h: u_int,
) -> Option<(u_int, u_int)> {
    unsafe {
        let tty: &mut tty = &mut (*tc).tty;
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut event: *mut key_event = cmdq_get_event(item);
        let mut s: *mut session = (*tc).session;
        let mut wl: *mut winlink = (*target).winlink();
        let mut wp: *mut window_pane = (*target).pane();
        let mut start: Option<u_int> = None;
        let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut top: ::core::ffi::c_int = 0;
        let mut line: u_int = 0;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut lines: u_int = 0;
        let mut position: u_int = 0;
        let mut n: ::core::ffi::c_long = 0;
        if w > (*tty).sx || h > (*tty).sy {
            return None;
        }
        let mut ft = format_create_from_target(item);
        if (*event).m.valid != 0 {
            format_add(
                &mut ft,
                c"popup_mouse_x",
                c"%u".as_ptr(),
                fmt_args![(*event).m.x],
            );
            format_add(
                &mut ft,
                c"popup_mouse_y",
                c"%u".as_ptr(),
                fmt_args![(*event).m.y],
            );
        }
        top = status_at_line(tc);
        if top != -(1 as ::core::ffi::c_int) {
            lines = status_line_size(tc);
            if top == 0 as ::core::ffi::c_int {
                top = lines as ::core::ffi::c_int;
            } else {
                top = 0 as ::core::ffi::c_int;
            }
            position = options_get_number(session_options(s), c"status-position".as_ptr()) as u_int;
            line = 0 as u_int;
            while line < lines {
                start = (*tc).status.entries[line as usize]
                    .ranges
                    .iter()
                    .find(|candidate| {
                        candidate.type_0 as ::core::ffi::c_uint
                            == STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
                            && candidate.argument == (*wl).idx as u_int
                    })
                    .map(|candidate| candidate.start);
                if start.is_some() {
                    break;
                }
                line = line.wrapping_add(1);
            }
            if let Some(start) = start {
                format_add(
                    &mut ft,
                    c"popup_window_status_line_x",
                    c"%u".as_ptr(),
                    fmt_args![start],
                );
                if position == 0 as u_int {
                    format_add(
                        &mut ft,
                        c"popup_window_status_line_y",
                        c"%u".as_ptr(),
                        fmt_args![line.wrapping_add(1 as u_int).wrapping_add(h)],
                    );
                } else {
                    format_add(
                        &mut ft,
                        c"popup_window_status_line_y",
                        c"%u".as_ptr(),
                        fmt_args![(*tty).sy.wrapping_sub(lines).wrapping_add(line)],
                    );
                }
            }
            if position == 0 as u_int {
                format_add(
                    &mut ft,
                    c"popup_status_line_y",
                    c"%u".as_ptr(),
                    fmt_args![lines.wrapping_add(h)],
                );
            } else {
                format_add(
                    &mut ft,
                    c"popup_status_line_y",
                    c"%u".as_ptr(),
                    fmt_args![(*tty).sy.wrapping_sub(lines)],
                );
            }
        } else {
            top = 0 as ::core::ffi::c_int;
        }
        format_add(&mut ft, c"popup_width", c"%u".as_ptr(), fmt_args![w]);
        format_add(&mut ft, c"popup_height", c"%u".as_ptr(), fmt_args![h]);
        n = (*tty).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_long / 2 as ::core::ffi::c_long
            - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_centre_x",
                c"%u".as_ptr(),
                fmt_args![0 as ::core::ffi::c_int],
            );
        } else {
            format_add(&mut ft, c"popup_centre_x", c"%ld".as_ptr(), fmt_args![n]);
        }
        n = (*tty)
            .sy
            .wrapping_sub(1 as u_int)
            .wrapping_div(2 as u_int)
            .wrapping_add(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
        if n >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_centre_y",
                c"%u".as_ptr(),
                fmt_args![(*tty).sy.wrapping_sub(h)],
            );
        } else {
            format_add(&mut ft, c"popup_centre_y", c"%ld".as_ptr(), fmt_args![n]);
        }
        if (*event).m.valid != 0 {
            n = (*event).m.x as ::core::ffi::c_long
                - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
            if n < 0 as ::core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_centre_x",
                    c"%u".as_ptr(),
                    fmt_args![0 as ::core::ffi::c_int],
                );
            } else {
                format_add(
                    &mut ft,
                    c"popup_mouse_centre_x",
                    c"%ld".as_ptr(),
                    fmt_args![n],
                );
            }
            n = (*event).m.y.wrapping_sub(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
            if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_centre_y",
                    c"%u".as_ptr(),
                    fmt_args![(*tty).sy.wrapping_sub(h)],
                );
            } else {
                format_add(
                    &mut ft,
                    c"popup_mouse_centre_y",
                    c"%ld".as_ptr(),
                    fmt_args![n],
                );
            }
            n = (*event).m.y as ::core::ffi::c_long + h as ::core::ffi::c_long;
            if n >= (*tty).sy as ::core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_top",
                    c"%u".as_ptr(),
                    fmt_args![(*tty).sy.wrapping_sub(1 as u_int)],
                );
            } else {
                format_add(&mut ft, c"popup_mouse_top", c"%ld".as_ptr(), fmt_args![n]);
            }
            n = (*event).m.y.wrapping_sub(h) as ::core::ffi::c_long;
            if n < 0 as ::core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_bottom",
                    c"%u".as_ptr(),
                    fmt_args![0 as ::core::ffi::c_int],
                );
            } else {
                format_add(
                    &mut ft,
                    c"popup_mouse_bottom",
                    c"%ld".as_ptr(),
                    fmt_args![n],
                );
            }
        }
        {
            let (_bigger, off_x, off_y, _off_sx, _off_sy) = tty_window_offset(&(*tc).tty);
            (ox, oy) = (off_x, off_y);
        }
        n = ((top + (*wp).yoff) as u_int)
            .wrapping_sub(oy)
            .wrapping_add(h) as ::core::ffi::c_long;
        if n >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_pane_top",
                c"%u".as_ptr(),
                fmt_args![(*tty).sy.wrapping_sub(h)],
            );
        } else {
            format_add(&mut ft, c"popup_pane_top", c"%ld".as_ptr(), fmt_args![n]);
        }
        format_add(
            &mut ft,
            c"popup_pane_bottom",
            c"%u".as_ptr(),
            fmt_args![
                ((top + (*wp).yoff) as u_int)
                    .wrapping_add((*wp).sy)
                    .wrapping_sub(oy)
            ],
        );
        format_add(
            &mut ft,
            c"popup_pane_left",
            c"%u".as_ptr(),
            fmt_args![((*wp).xoff as u_int).wrapping_sub(ox)],
        );
        n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
            - ox as ::core::ffi::c_long
            - w as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_pane_right",
                c"%u".as_ptr(),
                fmt_args![0 as ::core::ffi::c_int],
            );
        } else {
            format_add(&mut ft, c"popup_pane_right", c"%ld".as_ptr(), fmt_args![n]);
        }
        xp = args_get(args, 'x' as i32 as u_char);
        if xp.is_null() || strcmp(xp, c"C".as_ptr()) == 0 as ::core::ffi::c_int {
            xp = c"#{popup_centre_x}".as_ptr();
        } else if strcmp(xp, c"R".as_ptr()) == 0 as ::core::ffi::c_int {
            xp = c"#{popup_pane_right}".as_ptr();
        } else if strcmp(xp, c"P".as_ptr()) == 0 as ::core::ffi::c_int {
            xp = c"#{popup_pane_left}".as_ptr();
        } else if strcmp(xp, c"M".as_ptr()) == 0 as ::core::ffi::c_int {
            xp = c"#{popup_mouse_centre_x}".as_ptr();
        } else if strcmp(xp, c"W".as_ptr()) == 0 as ::core::ffi::c_int {
            xp = c"#{popup_window_status_line_x}".as_ptr();
        }
        let p = format_expand(&mut ft, CStr::from_ptr(xp));
        n = strtol(
            p.as_ptr(),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            10 as ::core::ffi::c_int,
        );
        if n + w as ::core::ffi::c_long >= (*tty).sx as ::core::ffi::c_long {
            n = (*tty).sx.wrapping_sub(w) as ::core::ffi::c_long;
        } else if n < 0 as ::core::ffi::c_long {
            n = 0 as ::core::ffi::c_long;
        }
        let px: u_int = n as u_int;
        log_debug(
            c"%s: -x: %s = %s = %u (-w %u)".as_ptr(),
            fmt_args![c"cmd_display_menu_get_pos".as_ptr(), xp, p.as_ptr(), px, w],
        );
        yp = args_get(args, 'y' as i32 as u_char);
        if yp.is_null() || strcmp(yp, c"C".as_ptr()) == 0 as ::core::ffi::c_int {
            yp = c"#{popup_centre_y}".as_ptr();
        } else if strcmp(yp, c"P".as_ptr()) == 0 as ::core::ffi::c_int {
            yp = c"#{popup_pane_bottom}".as_ptr();
        } else if strcmp(yp, c"M".as_ptr()) == 0 as ::core::ffi::c_int {
            yp = c"#{popup_mouse_top}".as_ptr();
        } else if strcmp(yp, c"S".as_ptr()) == 0 as ::core::ffi::c_int {
            yp = c"#{popup_status_line_y}".as_ptr();
        } else if strcmp(yp, c"W".as_ptr()) == 0 as ::core::ffi::c_int {
            yp = c"#{popup_window_status_line_y}".as_ptr();
        }
        let p = format_expand(&mut ft, CStr::from_ptr(yp));
        n = strtol(
            p.as_ptr(),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            10 as ::core::ffi::c_int,
        );
        if n < h as ::core::ffi::c_long {
            n = 0 as ::core::ffi::c_long;
        } else {
            n -= h as ::core::ffi::c_long;
        }
        if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
            n = (*tty).sy.wrapping_sub(h) as ::core::ffi::c_long;
        } else if n < 0 as ::core::ffi::c_long {
            n = 0 as ::core::ffi::c_long;
        }
        let py: u_int = n as u_int;
        log_debug(
            c"%s: -y: %s = %s = %u (-h %u)".as_ptr(),
            fmt_args![c"cmd_display_menu_get_pos".as_ptr(), yp, p.as_ptr(), py, h],
        );
        Some((px, py))
    }
}
unsafe fn cmd_display_menu_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut event: *mut key_event = cmdq_get_event(item);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut menu_item = menu_item::default();
        let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut style: *const ::core::ffi::c_char = args_get(args, 's' as i32 as u_char);
        let mut border_style: *const ::core::ffi::c_char = args_get(args, 'S' as i32 as u_char);
        let mut selected_style: *const ::core::ffi::c_char = args_get(args, 'H' as i32 as u_char);
        let mut lines: box_lines = BOX_LINES_DEFAULT;
        let mut cause = None;
        let mut number_cause = None;
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut starting_choice: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut i: u_int = 0;
        let mut count: u_int = args_count(args);
        let mut o: *mut options =
            (*(*session_get_curw((*target).session())).window()).options_ptr();
        let mut oe: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        if (*tc).overlay().is_some() {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'C' as i32 as u_char) != 0 {
            if strcmp(args_get(args, 'C' as i32 as u_char), c"-".as_ptr())
                == 0 as ::core::ffi::c_int
            {
                starting_choice = -(1 as ::core::ffi::c_int);
                current_block = 4166486009154926805;
            } else {
                starting_choice = args_strtonum(
                    args,
                    'C' as i32 as u_char,
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &mut number_cause,
                ) as ::core::ffi::c_int;
                if let Some(cause) = number_cause.as_ref() {
                    cmdq_error(
                        item,
                        c"starting choice %s".as_ptr(),
                        fmt_args![cause.as_ptr()],
                    );
                    current_block = 17781493482496987363;
                } else {
                    current_block = 4166486009154926805;
                }
            }
        } else {
            current_block = 4166486009154926805;
        }
        if current_block == 4166486009154926805 {
            let title = if args_has(args, 'T' as i32 as u_char) != 0 {
                format_single_from_target(
                    item,
                    CStr::from_ptr(args_get(args, 'T' as i32 as u_char)),
                )
            } else {
                c"".to_owned()
            };
            let mut menu = menu_create(title.as_ptr());
            i = 0 as u_int;
            loop {
                if !(i != count) {
                    current_block = 15925075030174552612;
                    break;
                }
                let fresh3 = i;
                i = i.wrapping_add(1);
                name = args_string(args, fresh3);
                if *name as ::core::ffi::c_int == '\0' as i32 {
                    menu_add_item(&raw mut *menu, None, item, tc, target);
                } else if count.wrapping_sub(i) < 2 as u_int {
                    cmdq_error(item, c"not enough arguments".as_ptr(), fmt_args![]);
                    current_block = 17781493482496987363;
                    break;
                } else {
                    let fresh4 = i;
                    i = i.wrapping_add(1);
                    key = args_string(args, fresh4);
                    menu_item.name = (!name.is_null()).then(|| CStr::from_ptr(name));
                    menu_item.key = key_string_lookup_string(key);
                    let fresh5 = i;
                    i = i.wrapping_add(1);
                    let cmd = args_string(args, fresh5);
                    menu_item.command = (!cmd.is_null()).then(|| CStr::from_ptr(cmd));
                    menu_add_item(&raw mut *menu, Some(&menu_item), item, tc, target);
                }
            }
            match current_block {
                17781493482496987363 => {}
                _ => {
                    if menu.items.is_empty() {
                        current_block = 11305506228944373502;
                    } else if match cmd_display_menu_get_pos(
                        tc,
                        item,
                        args,
                        menu.width.wrapping_add(4 as u_int),
                        (menu.items.len() as u_int).wrapping_add(2 as u_int),
                    ) {
                        Some((at_px, at_py)) => {
                            (px, py) = (at_px, at_py);
                            false
                        }
                        None => true,
                    } {
                        current_block = 11305506228944373502;
                    } else {
                        value = args_get(args, 'b' as i32 as u_char);
                        if !value.is_null() {
                            oe = options_get_ptr(o, c"menu-border-lines".as_ptr());
                            lines = options_find_choice(
                                options_table_entry(oe).unwrap(),
                                ::core::ffi::CStr::from_ptr(value),
                                &mut cause,
                            ) as box_lines;
                            if let Some(cause) = cause.as_ref() {
                                cmdq_error(
                                    item,
                                    c"menu-border-lines %s".as_ptr(),
                                    fmt_args![cause.as_ptr()],
                                );
                                current_block = 17781493482496987363;
                            } else {
                                current_block = 11048769245176032998;
                            }
                        } else {
                            current_block = 11048769245176032998;
                        }
                        match current_block {
                            17781493482496987363 => {}
                            _ => {
                                if args_has(args, 'O' as i32 as u_char) != 0 {
                                    flags |= MENU_STAYOPEN;
                                }
                                if (*event).m.valid == 0
                                    && args_has(args, 'M' as i32 as u_char) == 0
                                {
                                    flags |= MENU_NOMOUSE;
                                }
                                if menu_display(
                                    menu,
                                    flags,
                                    starting_choice,
                                    item,
                                    px,
                                    py,
                                    tc,
                                    lines,
                                    style,
                                    selected_style,
                                    border_style,
                                    target,
                                    None,
                                    MenuCallbackData::None,
                                ) != 0 as ::core::ffi::c_int
                                {
                                    current_block = 11305506228944373502;
                                } else {
                                    return CMD_RETURN_WAIT;
                                }
                            }
                        }
                    }
                    match current_block {
                        17781493482496987363 => {}
                        _ => {
                            return CMD_RETURN_NORMAL;
                        }
                    }
                }
            }
        }
        CMD_RETURN_ERROR
    }
}
unsafe fn cmd_display_popup_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut s: *mut session = (*target).session();
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let tty: &mut tty = &mut (*tc).tty;
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut shellcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut style: *const ::core::ffi::c_char = args_get(args, 's' as i32 as u_char);
        let mut border_style: *const ::core::ffi::c_char = args_get(args, 'S' as i32 as u_char);
        let mut cwd: Option<CString> = None;
        let mut cause = None;
        let mut percentage_cause = None;
        let mut argv: Vec<CString> = Vec::new();
        let mut title: Option<CString> = None;
        let mut modify: ::core::ffi::c_int = popup_present(tc);
        let mut flags: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
        let mut lines: box_lines = BOX_LINES_DEFAULT;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut w: u_int = 0;
        let mut h: u_int = 0;
        let mut count: u_int = args_count(args);
        let mut env: Option<Box<environ_t>> = None;
        let mut o: *mut options = (*(*session_get_curw(s)).window()).options_ptr();
        let mut oe: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        if args_has(args, 'C' as i32 as u_char) != 0 {
            server_client_clear_overlay(tc);
            return CMD_RETURN_NORMAL;
        }
        if modify == 0 && (*tc).overlay().is_some() {
            return CMD_RETURN_NORMAL;
        }
        if modify == 0 {
            h = (*tty).sy.wrapping_div(2 as u_int);
            if args_has(args, 'h' as i32 as u_char) != 0 {
                h = args_percentage(
                    args,
                    'h' as i32 as u_char,
                    1 as ::core::ffi::c_longlong,
                    (*tty).sy as ::core::ffi::c_longlong,
                    (*tty).sy as ::core::ffi::c_longlong,
                    &mut percentage_cause,
                ) as u_int;
                if let Some(cause) = percentage_cause.as_ref() {
                    cmdq_error(item, c"height %s".as_ptr(), fmt_args![cause.as_ptr()]);
                    current_block = 5914453722638945560;
                } else {
                    current_block = 8831408221741692167;
                }
            } else {
                current_block = 8831408221741692167;
            }
            match current_block {
                5914453722638945560 => {}
                _ => {
                    w = (*tty).sx.wrapping_div(2 as u_int);
                    if args_has(args, 'w' as i32 as u_char) != 0 {
                        w = args_percentage(
                            args,
                            'w' as i32 as u_char,
                            1 as ::core::ffi::c_longlong,
                            (*tty).sx as ::core::ffi::c_longlong,
                            (*tty).sx as ::core::ffi::c_longlong,
                            &mut percentage_cause,
                        ) as u_int;
                        if let Some(cause) = percentage_cause.as_ref() {
                            cmdq_error(item, c"width %s".as_ptr(), fmt_args![cause.as_ptr()]);
                            current_block = 5914453722638945560;
                        } else {
                            current_block = 12147880666119273379;
                        }
                    } else {
                        current_block = 12147880666119273379;
                    }
                    match current_block {
                        5914453722638945560 => {}
                        _ => {
                            if w > (*tty).sx {
                                w = (*tty).sx;
                            }
                            if h > (*tty).sy {
                                h = (*tty).sy;
                            }
                            if match cmd_display_menu_get_pos(tc, item, args, w, h) {
                                Some((at_px, at_py)) => {
                                    (px, py) = (at_px, at_py);
                                    false
                                }
                                None => true,
                            } {
                                current_block = 4318653505921602179;
                            } else {
                                value = args_get(args, 'd' as i32 as u_char);
                                if !value.is_null() {
                                    cwd = Some(format_single_from_target(
                                        item,
                                        CStr::from_ptr(value),
                                    ));
                                } else {
                                    cwd = Some(
                                        CStr::from_ptr(server_client_get_cwd(tc, s)).to_owned(),
                                    );
                                }
                                if count == 0 as u_int {
                                    shellcmd = options_get_string(
                                        session_options(s),
                                        c"default-command".as_ptr(),
                                    );
                                } else if count == 1 as u_int {
                                    shellcmd = args_string(args, 0 as u_int);
                                }
                                if count <= 1 as u_int
                                    && (shellcmd.is_null()
                                        || *shellcmd as ::core::ffi::c_int == '\0' as i32)
                                {
                                    shellcmd = ::core::ptr::null::<::core::ffi::c_char>();
                                    shell = options_get_string(
                                        session_options(s),
                                        c"default-shell".as_ptr(),
                                    );
                                    if checkshell(shell) == 0 {
                                        shell = _PATH_BSHELL.as_ptr();
                                    }
                                    argv.push(CStr::from_ptr(shell).to_owned());
                                } else {
                                    argv = args_to_vector(args);
                                }
                                if args_has(args, 'e' as i32 as u_char) >= 1 as ::core::ffi::c_int {
                                    let mut e = environ_create_box();
                                    for av in args_value_list(args, 'e' as i32 as u_char) {
                                        environ_put(
                                            &mut *e,
                                            (*av).value.string().as_ptr(),
                                            0 as ::core::ffi::c_int,
                                        );
                                    }
                                    env = Some(e);
                                }
                                current_block = 14447253356787937536;
                            }
                        }
                    }
                }
            }
        } else {
            current_block = 14447253356787937536;
        }
        if current_block == 14447253356787937536 {
            value = args_get(args, 'b' as i32 as u_char);
            if args_has(args, 'B' as i32 as u_char) != 0 {
                lines = BOX_LINES_NONE;
                current_block = 12556861819962772176;
            } else if !value.is_null() {
                oe = options_get_ptr(o, c"popup-border-lines".as_ptr());
                lines = options_find_choice(
                    options_table_entry(oe).unwrap(),
                    ::core::ffi::CStr::from_ptr(value),
                    &mut cause,
                ) as box_lines;
                if let Some(cause) = cause.as_ref() {
                    cmdq_error(
                        item,
                        c"popup-border-lines %s".as_ptr(),
                        fmt_args![cause.as_ptr()],
                    );
                    current_block = 5914453722638945560;
                } else {
                    current_block = 12556861819962772176;
                }
            } else {
                current_block = 12556861819962772176;
            }
            match current_block {
                5914453722638945560 => {}
                _ => {
                    if args_has(args, 'T' as i32 as u_char) != 0 {
                        title = Some(format_single_from_target(
                            item,
                            CStr::from_ptr(args_get(args, 'T' as i32 as u_char)),
                        ));
                    } else {
                        title = Some(c"".to_owned());
                    }
                    if args_has(args, 'N' as i32 as u_char) != 0 || modify == 0 {
                        flags = 0 as ::core::ffi::c_int;
                    }
                    if args_has(args, 'E' as i32 as u_char) > 1 as ::core::ffi::c_int {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEEXITZERO;
                    } else if args_has(args, 'E' as i32 as u_char) != 0 {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEEXIT;
                    }
                    if args_has(args, 'k' as i32 as u_char) != 0 {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEANYKEY;
                    }
                    if modify != 0 {
                        popup_modify(tc, cstr_ptr(&title), style, border_style, lines, flags);
                    } else if !(popup_display(
                        flags,
                        lines,
                        item,
                        px,
                        py,
                        w,
                        h,
                        environ_ptr(&env),
                        shellcmd,
                        &argv,
                        cstr_ptr(&cwd),
                        cstr_ptr(&title),
                        tc,
                        s,
                        style,
                        border_style,
                        None,
                    ) != 0 as ::core::ffi::c_int)
                    {
                        return CMD_RETURN_WAIT;
                    }
                    current_block = 4318653505921602179;
                }
            }
        }
        match current_block {
            5914453722638945560 => CMD_RETURN_ERROR,
            _ => CMD_RETURN_NORMAL,
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
