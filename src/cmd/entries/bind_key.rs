use crate::src::arguments::args_value;
use crate::src::arguments::{args_count, args_get, args_has, args_string, args_values};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::parse::{cmd_parse_from_arguments, cmd_parse_from_string};
use crate::src::cmd::queue::cmdq_error;
use crate::src::key_bindings::key_bindings_add;
use crate::src::key_string::key_string_parse_cstr;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_list, cmdq_item};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::key::*;

#[no_mangle]
pub static mut cmd_bind_key_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"bind-key\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"bind\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"nrN:T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(cmd_bind_key_args_parse),
        },
        usage: b"[-nr] [-T key-table] [-N note] key [command [argument ...]]\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_bind_key_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
fn cmd_bind_key_args_parse(_args: &mut args, _idx: u_int) -> args_parse_type {
    return ARGS_PARSE_COMMANDS_OR_STRING;
}
unsafe extern "C" fn cmd_bind_key_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut key: key_code = 0;
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut note: *const ::core::ffi::c_char = args_get(args, 'N' as i32 as u_char);
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    let mut repeat: ::core::ffi::c_int = 0;
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut count: u_int = args_count(args);
    key = key_string_parse_cstr(std::ffi::CStr::from_ptr(args_string(args, 0 as u_int)))
        .unwrap_or(KEYC_UNKNOWN);
    if key == KEYC_NONE as ::core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
    {
        cmdq_error(
            item,
            b"unknown key: %s\0" as *const u8 as *const ::core::ffi::c_char,
            args_string(args, 0 as u_int),
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'T' as i32 as u_char) != 0 {
        tablename = args_get(args, 'T' as i32 as u_char);
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        tablename = b"root\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        tablename = b"prefix\0" as *const u8 as *const ::core::ffi::c_char;
    }
    repeat = args_has(args, 'r' as i32 as u_char);
    if count == 1 as u_int {
        key_bindings_add(
            tablename,
            key,
            note,
            repeat,
            ::core::ptr::null_mut::<cmd_list>(),
        );
        return CMD_RETURN_NORMAL;
    }
    value = args_value(args, 1 as u_int);
    if count == 2 as u_int
        && (*value).type_0() as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        key_bindings_add(tablename, key, note, repeat, (*value).cmdlist());
        (*(*value).cmdlist()).references += 1;
        return CMD_RETURN_NORMAL;
    }
    if count == 2 as u_int {
        pr = cmd_parse_from_string(
            std::ffi::CStr::from_ptr(args_string(args, 1 as u_int)),
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
    } else {
        pr = cmd_parse_from_arguments(
            args_values(args).offset(1 as ::core::ffi::c_int as isize),
            count.wrapping_sub(1 as u_int),
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
    }
    match pr.status as ::core::ffi::c_uint {
        0 => {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                pr.error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
            );
            return CMD_RETURN_ERROR;
        }
        1 | _ => {}
    }
    key_bindings_add(tablename, key, note, repeat, pr.cmdlist);
    return CMD_RETURN_NORMAL;
}
