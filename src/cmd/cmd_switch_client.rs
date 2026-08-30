use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_get_args;
use crate::cmd::find::{cmd_find_from_session, cmd_find_target};
use crate::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_flags, cmdq_get_target_client,
};
use crate::environ::{environ_ptr, environ_update};
use crate::ffi::{getuid, strcmp, strcspn};
use crate::fmt_args;
use crate::key_bindings::key_bindings_get_table;
use crate::proc::{peer_ptr, proc_get_peer_uid};
use crate::server::client_get_last_session;
use crate::server::server_redraw_window;
use crate::server::{server_client_set_key_table, server_client_set_session};
use crate::session::{
    session_alive, session_next_session, session_previous_session, session_set_current,
};
use crate::session::{session_environ, session_options};
use crate::sort::sort_order_from_string;
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::{
    window_pop_zoom, window_push_zoom, window_redraw_active_switch, window_set_active_pane,
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
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
pub const CMD_FIND_PREFER_UNATTACHED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMDQ_STATE_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const CLIENT_IGNORESIZE: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub(crate) static cmd_switch_client_entry: cmd_entry = {
    cmd_entry {
        name: c"switch-client",
        alias: Some(c"switchc"),
        args: args_parse_t {
            template: c"c:EFlnO:pt:rT:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_READONLY | CMD_CLIENT_CFLAG,
        exec: cmd_switch_client_exec,
    }
};
unsafe fn cmd_switch_client_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut current: *mut cmd_find_state = cmdq_get_current(item);
        let mut target = cmd_find_state::default();
        let mut tflag: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
        let mut type_0: cmd_find_type = CMD_FIND_PANE;
        let mut flags: ::core::ffi::c_int = 0;
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
        let mut sort_crit = sort_criteria_t::default();
        let mut uid: uid_t = 0;
        if !tflag.is_null()
            && (*tflag.offset(strcspn(tflag, c":.%".as_ptr()) as isize) as ::core::ffi::c_int
                != '\0' as i32
                || strcmp(tflag, c"=".as_ptr()) == 0 as ::core::ffi::c_int)
        {
            type_0 = CMD_FIND_PANE;
            flags = 0 as ::core::ffi::c_int;
        } else {
            type_0 = CMD_FIND_SESSION;
            flags = CMD_FIND_PREFER_UNATTACHED;
        }
        if cmd_find_target(&mut target, item, tflag, type_0, flags) != 0 as ::core::ffi::c_int {
            return CMD_RETURN_ERROR;
        }
        s = target.session();
        wl = target.winlink();
        wp = target.pane();
        if args_has(args, 'r' as i32 as u_char) != 0 {
            if (*tc).flags & CLIENT_READONLY as uint64_t != 0 {
                uid = proc_get_peer_uid(peer_ptr(&(*c).peer));
                if uid != getuid() {
                    cmdq_error(item, c"client is read-only".as_ptr(), fmt_args![]);
                    return CMD_RETURN_ERROR;
                }
            }
            if (*tc).flags & CLIENT_READONLY as uint64_t != 0 {
                (*tc).flags &= !(CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
            } else {
                (*tc).flags |= (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
            }
        }
        tablename = args_get(args, 'T' as i32 as u_char);
        if !tablename.is_null() {
            table = key_bindings_get_table(tablename, 0 as ::core::ffi::c_int);
            if table.is_null() {
                cmdq_error(
                    item,
                    c"table %s doesn't exist".as_ptr(),
                    fmt_args![tablename],
                );
                return CMD_RETURN_ERROR;
            }
            server_client_set_key_table(tc, tablename);
            return CMD_RETURN_NORMAL;
        }
        sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
        if sort_crit.order as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
            && args_has(args, 'O' as i32 as u_char) != 0
        {
            cmdq_error(item, c"invalid sort order".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
        if args_has(args, 'n' as i32 as u_char) != 0 {
            s = session_next_session((*tc).session, &sort_crit);
            if s.is_null() {
                cmdq_error(item, c"can't find next session".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
        } else if args_has(args, 'p' as i32 as u_char) != 0 {
            s = session_previous_session((*tc).session, &sort_crit);
            if s.is_null() {
                cmdq_error(item, c"can't find previous session".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
        } else if args_has(args, 'l' as i32 as u_char) != 0 {
            let last = client_get_last_session(tc);
            if !last.is_null() && session_alive(last) != 0 {
                s = last;
            } else {
                s = ::core::ptr::null_mut::<session>();
            }
            if s.is_null() {
                cmdq_error(item, c"can't find last session".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
        } else {
            if cmdq_get_client(&*item).is_null() {
                return CMD_RETURN_NORMAL;
            }
            if !wl.is_null() && !wp.is_null() && wp != window_get_active((*wl).window()) {
                w = (*wl).window();
                if window_push_zoom(
                    w,
                    0 as ::core::ffi::c_int,
                    args_has(args, 'Z' as i32 as u_char),
                ) != 0
                {
                    server_redraw_window(w);
                }
                window_redraw_active_switch(w, wp);
                window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
                if window_pop_zoom(w) != 0 {
                    server_redraw_window(w);
                }
            }
            if !wl.is_null() {
                session_set_current(s, wl);
                cmd_find_from_session(&mut *current, s, 0 as ::core::ffi::c_int);
            }
        }
        if args_has(args, 'E' as i32 as u_char) == 0 {
            environ_update(
                session_options(s),
                environ_ptr(&(*tc).environ),
                session_environ(s),
            );
        }
        server_client_set_session(tc, s);
        if !cmdq_get_flags(&*item) & CMDQ_STATE_REPEAT != 0 {
            server_client_set_key_table(tc, ::core::ptr::null::<::core::ffi::c_char>());
        }
        CMD_RETURN_NORMAL
    }
}
