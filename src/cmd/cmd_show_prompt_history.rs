use crate::arguments::args_get;
use crate::cmd::queue::{cmdq_error, cmdq_print};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::status::status_prompt_hlist;
use crate::status::{status_prompt_type, status_prompt_type_string};
pub use crate::types::*;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
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
pub const PROMPT_NTYPES: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub(crate) static cmd_show_prompt_history_entry: cmd_entry = {
    cmd_entry {
        name: c"show-prompt-history",
        alias: Some(c"showphist"),
        args: args_parse_t {
            template: c"T:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-T prompt-type]",
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
        exec: cmd_show_prompt_history_exec,
    }
};
pub(crate) static cmd_clear_prompt_history_entry: cmd_entry = {
    cmd_entry {
        name: c"clear-prompt-history",
        alias: Some(c"clearphist"),
        args: args_parse_t {
            template: c"T:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-T prompt-type]",
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
        exec: cmd_show_prompt_history_exec,
    }
};
unsafe fn cmd_show_prompt_history_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut typestr: *const ::core::ffi::c_char = args_get(args, 'T' as i32 as u_char);
        let mut type_0: prompt_type = PROMPT_TYPE_COMMAND;
        let mut tidx: u_int = 0;
        let mut hidx: u_int = 0;
        let hlists = &mut status_prompt_hlist;
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_clear_prompt_history_entry) {
            if typestr.is_null() {
                tidx = 0 as u_int;
                while tidx < PROMPT_NTYPES as u_int {
                    hlists[tidx as usize].clear();
                    tidx = tidx.wrapping_add(1);
                }
            } else {
                type_0 = status_prompt_type(::core::ffi::CStr::from_ptr(typestr));
                if type_0 as ::core::ffi::c_uint
                    == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    cmdq_error(item, c"invalid type: %s".as_ptr(), fmt_args![typestr]);
                    return CMD_RETURN_ERROR;
                }
                hlists[type_0 as usize].clear();
            }
            return CMD_RETURN_NORMAL;
        }
        if typestr.is_null() {
            tidx = 0 as u_int;
            while tidx < PROMPT_NTYPES as u_int {
                cmdq_print(
                    item,
                    c"History for %s:\n".as_ptr(),
                    fmt_args![status_prompt_type_string(tidx).as_ptr()],
                );
                hidx = 0 as u_int;
                while (hidx as usize) < hlists[tidx as usize].len() {
                    cmdq_print(
                        item,
                        c"%d: %s".as_ptr(),
                        fmt_args![
                            hidx.wrapping_add(1 as u_int),
                            hlists[tidx as usize][hidx as usize].as_ptr()
                        ],
                    );
                    hidx = hidx.wrapping_add(1);
                }
                cmdq_print(item, c"%s".as_ptr(), fmt_args![c"".as_ptr()]);
                tidx = tidx.wrapping_add(1);
            }
        } else {
            type_0 = status_prompt_type(::core::ffi::CStr::from_ptr(typestr));
            if type_0 as ::core::ffi::c_uint
                == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                cmdq_error(item, c"invalid type: %s".as_ptr(), fmt_args![typestr]);
                return CMD_RETURN_ERROR;
            }
            cmdq_print(
                item,
                c"History for %s:\n".as_ptr(),
                fmt_args![status_prompt_type_string(type_0 as u_int).as_ptr()],
            );
            hidx = 0 as u_int;
            while (hidx as usize) < hlists[type_0 as usize].len() {
                cmdq_print(
                    item,
                    c"%d: %s".as_ptr(),
                    fmt_args![
                        hidx.wrapping_add(1 as u_int),
                        hlists[type_0 as usize][hidx as usize].as_ptr()
                    ],
                );
                hidx = hidx.wrapping_add(1);
            }
            cmdq_print(item, c"%s".as_ptr(), fmt_args![c"".as_ptr()]);
        }
        CMD_RETURN_NORMAL
    }
}
