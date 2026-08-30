use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::ffi::{getpwnam, getuid};
use crate::fmt_args;
use crate::format::format_single;
use crate::proc::{peer_ptr, proc_get_peer_uid};
use crate::server::client_walk;
use crate::server::{
    server_acl_display, server_acl_user_allow, server_acl_user_allow_write, server_acl_user_deny,
    server_acl_user_deny_write,
};
use crate::server::{server_acl_get_uid, server_acl_user_find};
pub use crate::types::*;
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
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub(crate) static cmd_server_access_entry: cmd_entry = {
    cmd_entry {
        name: c"server-access",
        alias: None,
        args: args_parse_t {
            template: c"adlrw",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-adlrw] [-t target-pane] [user]",
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
        flags: CMD_CLIENT_CANFAIL,
        exec: cmd_server_access_exec,
    }
};
unsafe fn cmd_server_access_deny(mut item: *mut cmdq_item, mut pw: *mut passwd) -> cmd_retval {
    unsafe {
        let mut user: *mut server_acl_user = ::core::ptr::null_mut::<server_acl_user>();
        let mut uid: uid_t = 0;
        user = server_acl_user_find((*pw).pw_uid as uid_t);
        if user.is_null() {
            cmdq_error(
                item,
                c"user %s not found".as_ptr(),
                fmt_args![(*pw).pw_name],
            );
            return CMD_RETURN_ERROR;
        }
        for loop_0 in client_walk() {
            uid = proc_get_peer_uid(peer_ptr(&(*loop_0).peer));
            if uid == server_acl_get_uid(user) {
                (*loop_0).exit_message = Some(c"access not allowed".to_owned());
                (*loop_0).flags |= CLIENT_EXIT as uint64_t;
            }
        }
        server_acl_user_deny((*pw).pw_uid as uid_t);
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_server_access_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut c: *mut client = cmdq_get_target_client(&*item);
        let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
        if args_has(args, 'l' as i32 as u_char) != 0 {
            server_acl_display(item);
            return CMD_RETURN_NORMAL;
        }
        if args_count(args) == 0 as u_int {
            cmdq_error(item, c"missing user argument".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        let name = format_single(
            item,
            ::core::ffi::CStr::from_ptr(args_string(args, 0 as u_int)),
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if !name.as_bytes().is_empty() {
            pw = getpwnam(name.as_ptr());
        }
        if pw.is_null() {
            cmdq_error(item, c"unknown user: %s".as_ptr(), fmt_args![name.as_ptr()]);
            return CMD_RETURN_ERROR;
        }
        if (*pw).pw_uid == 0 as __uid_t || (*pw).pw_uid == getuid() {
            cmdq_error(
                item,
                c"%s owns the server, can't change access".as_ptr(),
                fmt_args![(*pw).pw_name],
            );
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'a' as i32 as u_char) != 0 && args_has(args, 'd' as i32 as u_char) != 0 {
            cmdq_error(
                item,
                c"-a and -d cannot be used together".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'w' as i32 as u_char) != 0 && args_has(args, 'r' as i32 as u_char) != 0 {
            cmdq_error(
                item,
                c"-r and -w cannot be used together".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'd' as i32 as u_char) != 0 {
            return cmd_server_access_deny(item, pw);
        }
        if args_has(args, 'a' as i32 as u_char) != 0 {
            if !server_acl_user_find((*pw).pw_uid as uid_t).is_null() {
                cmdq_error(
                    item,
                    c"user %s is already added".as_ptr(),
                    fmt_args![(*pw).pw_name],
                );
                return CMD_RETURN_ERROR;
            }
            server_acl_user_allow((*pw).pw_uid as uid_t);
        } else if (args_has(args, 'r' as i32 as u_char) != 0
            || args_has(args, 'w' as i32 as u_char) != 0)
            && server_acl_user_find((*pw).pw_uid as uid_t).is_null()
        {
            server_acl_user_allow((*pw).pw_uid as uid_t);
        }
        if args_has(args, 'w' as i32 as u_char) != 0 {
            if server_acl_user_find((*pw).pw_uid as uid_t).is_null() {
                cmdq_error(
                    item,
                    c"user %s not found".as_ptr(),
                    fmt_args![(*pw).pw_name],
                );
                return CMD_RETURN_ERROR;
            }
            server_acl_user_allow_write((*pw).pw_uid as uid_t);
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'r' as i32 as u_char) != 0 {
            if server_acl_user_find((*pw).pw_uid as uid_t).is_null() {
                cmdq_error(
                    item,
                    c"user %s not found".as_ptr(),
                    fmt_args![(*pw).pw_name],
                );
                return CMD_RETURN_ERROR;
            }
            server_acl_user_deny_write((*pw).pw_uid as uid_t);
            return CMD_RETURN_NORMAL;
        }
        CMD_RETURN_NORMAL
    }
}
