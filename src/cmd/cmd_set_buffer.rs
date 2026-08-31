use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::ffi::strlen;
use crate::fmt_args;
use crate::paste::{paste_buffer_data, paste_free, paste_get_name, paste_get_top};
use crate::paste::{paste_rename, paste_set};
use crate::tty::tty_set_selection;
pub use crate::types::*;
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
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_BUFFER_USAGE: &::core::ffi::CStr = c"[-b buffer-name]";
pub(crate) static cmd_set_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"set-buffer",
        alias: Some(c"setb"),
        args: args_parse_t {
            template: c"ab:t:n:w",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]",
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL,
        exec: cmd_set_buffer_exec,
    }
};
pub(crate) static cmd_delete_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"delete-buffer",
        alias: Some(c"deleteb"),
        args: args_parse_t {
            template: c"b:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_BUFFER_USAGE,
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
        flags: CMD_AFTERHOOK,
        exec: cmd_set_buffer_exec,
    }
};
unsafe fn cmd_set_buffer_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
        let mut bufname: Option<CString> = None;
        let mut bufdata: Vec<u8> = Vec::new();
        let mut bufsize: size_t = 0 as size_t;
        let mut newsize: size_t = 0;
        if !args_get(args, 'b' as i32 as u_char).is_null() {
            bufname =
                Some(::std::ffi::CStr::from_ptr(args_get(args, 'b' as i32 as u_char)).to_owned());
            pb = paste_get_name(bufname.as_ref().unwrap().as_ptr());
        }
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_delete_buffer_entry) {
            if pb.is_null() {
                if bufname.is_some() {
                    cmdq_error(
                        item,
                        c"unknown buffer: %s".as_ptr(),
                        fmt_args![bufname.as_ref().unwrap().as_ptr()],
                    );
                    current_block = 8474116202651904133;
                } else {
                    pb = paste_get_top(Some(&mut bufname));
                    current_block = 3640593987805443782;
                }
            } else {
                current_block = 3640593987805443782;
            }
            match current_block {
                8474116202651904133 => {}
                _ => {
                    if pb.is_null() {
                        cmdq_error(item, c"no buffer".as_ptr(), fmt_args![]);
                    } else {
                        paste_free(pb);
                        return CMD_RETURN_NORMAL;
                    }
                }
            }
        } else if args_has(args, 'n' as i32 as u_char) != 0 {
            if pb.is_null() {
                if bufname.is_some() {
                    cmdq_error(
                        item,
                        c"unknown buffer: %s".as_ptr(),
                        fmt_args![bufname.as_ref().unwrap().as_ptr()],
                    );
                    current_block = 8474116202651904133;
                } else {
                    pb = paste_get_top(Some(&mut bufname));
                    current_block = 15904375183555213903;
                }
            } else {
                current_block = 15904375183555213903;
            }
            match current_block {
                8474116202651904133 => {}
                _ => {
                    if pb.is_null() {
                        cmdq_error(item, c"no buffer".as_ptr(), fmt_args![]);
                    } else if let Err(error) = paste_rename(
                        bufname.as_ref().unwrap().as_ptr(),
                        args_get(args, 'n' as i32 as u_char),
                    ) {
                        cmdq_error(item, c"%s".as_ptr(), fmt_args![error.as_ptr()]);
                    } else {
                        return CMD_RETURN_NORMAL;
                    }
                }
            }
        } else if args_count(args) != 1 as u_int {
            cmdq_error(item, c"no data specified".as_ptr(), fmt_args![]);
        } else {
            newsize = strlen(args_string(args, 0 as u_int));
            if newsize == 0 as size_t {
                return CMD_RETURN_NORMAL;
            }
            if args_has(args, 'a' as i32 as u_char) != 0 && !pb.is_null() {
                bufdata.extend_from_slice(paste_buffer_data(&*pb));
            }
            bufdata.extend_from_slice(::core::slice::from_raw_parts(
                args_string(args, 0 as u_int) as *const u8,
                newsize as usize,
            ));
            bufsize = bufdata.len() as size_t;
            let selection = bufdata.as_ptr() as *const ::core::ffi::c_char;
            if let Err(error) = paste_set(
                bufdata,
                bufname
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
            ) {
                cmdq_error(item, c"%s".as_ptr(), fmt_args![error.as_ptr()]);
            } else {
                if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
                    tty_set_selection(&raw mut (*tc).tty, c"".as_ptr(), selection, bufsize);
                }
                return CMD_RETURN_NORMAL;
            }
        }
        CMD_RETURN_ERROR
    }
}
