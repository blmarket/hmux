use crate::src::arguments::args_get;
use crate::src::cmd::queue::{cmdq_error, cmdq_print};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::prompt::{prompt_type, prompt_type_string};
use crate::src::prompt_history::{prompt_history_clear, prompt_history_get, prompt_history_size};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::prompt::PROMPT_NTYPES;
use crate::src::shared::prompt::*;
pub static mut cmd_show_prompt_history_entry: cmd_entry =  {
    cmd_entry {
        name: b"show-prompt-history\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showphist\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-T prompt-type]\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(
            cmd_show_prompt_history_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
pub static mut cmd_clear_prompt_history_entry: cmd_entry =  {
    cmd_entry {
        name: b"clear-prompt-history\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"clearphist\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-T prompt-type]\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(
            cmd_show_prompt_history_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_show_prompt_history_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut typestr: *const ::core::ffi::c_char = args_get(args, 'T' as i32 as u_char);
    let mut type_0: prompt_type = PROMPT_TYPE_COMMAND;
    let mut t: u_int = 0;
    let mut h: u_int = 0;
    let mut v: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if cmd_get_entry(self_0) == &raw const cmd_clear_prompt_history_entry {
        if typestr.is_null() {
            t = 0 as u_int;
            while t < PROMPT_NTYPES as u_int {
                prompt_history_clear(t as prompt_type);
                t = t.wrapping_add(1);
            }
        } else {
            type_0 = prompt_type(typestr);
            if type_0 as ::core::ffi::c_uint
                == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                cmdq_error(
                    item,
                    b"invalid type: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    typestr,
                );
                return CMD_RETURN_ERROR;
            }
            prompt_history_clear(type_0);
        }
        return CMD_RETURN_NORMAL;
    }
    if typestr.is_null() {
        t = 0 as u_int;
        while t < PROMPT_NTYPES as u_int {
            typestr = prompt_type_string(t as prompt_type);
            cmdq_print(
                item,
                b"History for %s:\n\0" as *const u8 as *const ::core::ffi::c_char,
                typestr,
            );
            h = 0 as u_int;
            while h < prompt_history_size(t as prompt_type) {
                v = prompt_history_get(t as prompt_type, h);
                cmdq_print(
                    item,
                    b"%d: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    h.wrapping_add(1 as u_int),
                    v,
                );
                h = h.wrapping_add(1);
            }
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
            t = t.wrapping_add(1);
        }
    } else {
        type_0 = prompt_type(typestr);
        if type_0 as ::core::ffi::c_uint
            == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cmdq_error(
                item,
                b"invalid type: %s\0" as *const u8 as *const ::core::ffi::c_char,
                typestr,
            );
            return CMD_RETURN_ERROR;
        }
        cmdq_print(
            item,
            b"History for %s:\n\0" as *const u8 as *const ::core::ffi::c_char,
            prompt_type_string(type_0),
        );
        h = 0 as u_int;
        while h < prompt_history_size(type_0) {
            v = prompt_history_get(type_0, h);
            cmdq_print(
                item,
                b"%d: %s\0" as *const u8 as *const ::core::ffi::c_char,
                h.wrapping_add(1 as u_int),
                v,
            );
            h = h.wrapping_add(1);
        }
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return CMD_RETURN_NORMAL;
}
