use crate::arguments::args_get_str;
use crate::arguments::{
    args_count, args_get, args_has, args_string, args_to_vector, args_value_list,
};
use crate::cmd::find::cmd_find_from_winlink_pane;
use crate::cmd::queue::cmdq_item_weak_from_ptr;
use crate::cmd::queue::{
    cmdq_error, cmdq_get_current, cmdq_get_target, cmdq_get_target_client, cmdq_insert_hook,
    cmdq_print,
};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::environ::{environ_create_box, environ_ptr, environ_put};
use crate::fmt_args;
use crate::format::format_single;
use crate::layout::{layout_close_pane, layout_get_floating_cell, layout_get_tiled_cell};
use crate::options::{options_ptr, options_set_number, options_set_string};
use crate::server::server_client_remove_pane;
use crate::server::{server_redraw_session, server_redraw_window};
use crate::spawn::spawn_pane;
pub use crate::types::*;
use crate::window::{window_pane_start_input, window_pop_zoom, window_remove_pane};
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
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_EMPTY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SPAWN_ZOOM: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const SPLIT_WINDOW_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
pub(crate) static cmd_new_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"new-pane",
        alias: Some(c"newp"),
        args: args_parse_t {
            template: c"bc:de:EfF:hIkl:Lm:p:PR:s:S:t:vx:X:y:Y:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]",
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
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_split_window_exec,
    }
};
pub(crate) static cmd_split_window_entry: cmd_entry = {
    cmd_entry {
        name: c"split-window",
        alias: Some(c"splitw"),
        args: args_parse_t {
            template: c"bc:de:EfF:hIkl:m:p:PR:s:S:t:vZ",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-t target-pane] [shell-command [argument ...]]",
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
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_split_window_exec,
    }
};
unsafe fn cmd_split_window_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut current: *mut cmd_find_state = cmdq_get_current(item);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut sc = spawn_context::default();
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut s: *mut session = (*target).session();
        let mut wl: *mut winlink = (*target).winlink();
        let mut w: *mut window = (*wl).window();
        let mut wp: *mut window_pane = (*target).pane();
        let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut fs = cmd_find_state::default();
        let mut input: ::core::ffi::c_int = 0;
        let mut empty: ::core::ffi::c_int = 0;
        let mut is_floating: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut style: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cause: Option<CString> = None;
        let mut floating_cause = None;
        let mut count: u_int = args_count(args);
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_new_pane_entry) {
            is_floating = (args_has(args, 'L' as i32 as u_char) == 0) as ::core::ffi::c_int;
        } else {
            is_floating = 0 as ::core::ffi::c_int;
        }
        flags = if is_floating != 0 {
            SPAWN_FLOATING
        } else {
            0 as ::core::ffi::c_int
        };
        if args_has(args, 'b' as i32 as u_char) != 0 {
            flags |= SPAWN_BEFORE;
        }
        if args_has(args, 'f' as i32 as u_char) != 0 {
            flags |= SPAWN_FULLSIZE;
        }
        input = args_has(args, 'I' as i32 as u_char);
        if input != 0 {
            empty = 1 as ::core::ffi::c_int;
        } else {
            empty = args_has(args, 'E' as i32 as u_char);
        }
        if empty != 0
            && count != 0 as u_int
            && (count != 1 as u_int
                || *args_string(args, 0 as u_int) as ::core::ffi::c_int != '\0' as i32)
        {
            cmdq_error(
                item,
                c"command cannot be given for empty pane".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }
        if empty != 0 {
            flags |= SPAWN_EMPTY;
        }
        if is_floating != 0 {
            lc = layout_get_floating_cell(item, args, w, wp, &mut floating_cause);
            if let Some(cause) = floating_cause.as_ref() {
                cmdq_error(
                    item,
                    c"size or position %s".as_ptr(),
                    fmt_args![cause.as_ptr()],
                );
                return CMD_RETURN_ERROR;
            }
        } else {
            let mut tiled_cause = CString::default();
            lc = layout_get_tiled_cell(item, args, w, wp, flags, &mut tiled_cause);
            if !tiled_cause.as_bytes().is_empty() {
                cmdq_error(
                    item,
                    c"size or position %s".as_ptr(),
                    fmt_args![tiled_cause.as_ptr()],
                );
                return CMD_RETURN_ERROR;
            }
        }
        sc.item = cmdq_item_weak_from_ptr(item);
        sc.s = s;
        sc.wl = wl;
        sc.wp0 = wp;
        sc.lc = lc;
        sc.argv = args_to_vector(args);
        sc.environ = Some(environ_create_box());
        for av in args_value_list(args, 'e' as i32 as u_char) {
            environ_put(
                environ_ptr(&sc.environ),
                (*av).value.string(),
                0 as ::core::ffi::c_int,
            );
        }
        sc.idx = -(1 as ::core::ffi::c_int);
        sc.cwd = args_get_str(args, 'c' as i32 as u_char);
        sc.flags = flags;
        if args_has(args, 'd' as i32 as u_char) != 0 {
            sc.flags |= SPAWN_DETACHED;
        }
        if args_has(args, 'Z' as i32 as u_char) != 0 {
            sc.flags |= SPAWN_ZOOM;
        }
        new_wp = spawn_pane(&mut sc, &mut cause);
        if new_wp.is_null() {
            let cause = cause.unwrap();
            cmdq_error(
                item,
                c"create pane failed: %s".as_ptr(),
                fmt_args![cause.as_ptr()],
            );
            drop(sc.environ.take());
            return CMD_RETURN_ERROR;
        }
        style = args_get(args, 's' as i32 as u_char);
        if !style.is_null() {
            if options_set_string(
                options_ptr(&(*new_wp).options),
                c"window-style".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![style],
            )
            .is_null()
            {
                cmdq_error(item, c"bad style: %s".as_ptr(), fmt_args![style]);
                return CMD_RETURN_ERROR;
            }
            options_set_string(
                options_ptr(&(*new_wp).options),
                c"window-active-style".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![style],
            );
            (*new_wp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
        style = args_get(args, 'S' as i32 as u_char);
        if !style.is_null()
            && options_set_string(
                options_ptr(&(*new_wp).options),
                c"pane-active-border-style".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![style],
            )
            .is_null()
        {
            cmdq_error(
                item,
                c"bad active border style: %s".as_ptr(),
                fmt_args![style],
            );
            return CMD_RETURN_ERROR;
        }
        style = args_get(args, 'R' as i32 as u_char);
        if !style.is_null()
            && options_set_string(
                options_ptr(&(*new_wp).options),
                c"pane-border-style".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![style],
            )
            .is_null()
        {
            cmdq_error(
                item,
                c"bad inactive border style: %s".as_ptr(),
                fmt_args![style],
            );
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'k' as i32 as u_char) != 0 || args_has(args, 'm' as i32 as u_char) != 0 {
            options_set_number(
                options_ptr(&(*new_wp).options),
                c"remain-on-exit".as_ptr(),
                3 as ::core::ffi::c_longlong,
            );
            if args_has(args, 'm' as i32 as u_char) != 0 {
                options_set_string(
                    options_ptr(&(*new_wp).options),
                    c"remain-on-exit-format".as_ptr(),
                    0 as ::core::ffi::c_int,
                    c"%s".as_ptr(),
                    fmt_args![args_get(args, 'm' as i32 as u_char)],
                );
            }
        }
        if input != 0 {
            match window_pane_start_input(new_wp, item) {
                Err(cause) => {
                    server_client_remove_pane(new_wp);
                    if is_floating == 0 {
                        layout_close_pane(new_wp);
                    }
                    window_remove_pane((*wp).window, new_wp);
                    cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                    drop(sc.environ.take());
                    return CMD_RETURN_ERROR;
                }
                Ok(1) => {
                    input = 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        if args_has(args, 'd' as i32 as u_char) == 0 {
            cmd_find_from_winlink_pane(&mut *current, wl, new_wp, 0 as ::core::ffi::c_int);
        }
        if is_floating == 0 {
            window_pop_zoom((*wp).window);
            server_redraw_window((*wp).window);
        }
        server_redraw_session(s);
        if args_has(args, 'P' as i32 as u_char) != 0 {
            template = args_get(args, 'F' as i32 as u_char);
            if template.is_null() {
                template = SPLIT_WINDOW_TEMPLATE.as_ptr();
            }
            let cp = format_single(
                item,
                ::core::ffi::CStr::from_ptr(template),
                tc,
                s,
                wl,
                new_wp,
            );
            cmdq_print(item, c"%s".as_ptr(), fmt_args![cp.as_ptr()]);
        }
        cmd_find_from_winlink_pane(&mut fs, wl, new_wp, 0 as ::core::ffi::c_int);
        cmdq_insert_hook(
            s,
            item,
            &raw mut fs,
            c"after-split-window".as_ptr(),
            fmt_args![],
        );
        drop(sc.environ.take());
        if input != 0 {
            return CMD_RETURN_WAIT;
        }
        CMD_RETURN_NORMAL
    }
}
