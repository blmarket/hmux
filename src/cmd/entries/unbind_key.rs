use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::cmdq_error;
use crate::src::key_bindings::{
    key_bindings_get_table, key_bindings_remove, key_bindings_remove_table,
};
use crate::src::key_string::key_string_parse_cstr;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::key::*;
pub static mut cmd_unbind_key_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"unbind-key\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"unbind\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"anqT:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-anq] [-T key-table] key\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(cmd_unbind_key_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_unbind_key_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut key: key_code = 0;
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut keystr: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    let mut quiet: ::core::ffi::c_int = args_has(args, 'q' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 {
        if !keystr.is_null() {
            if quiet == 0 {
                cmdq_error(
                    item,
                    b"key given with -a\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return CMD_RETURN_ERROR;
        }
        tablename = args_get(args, 'T' as i32 as u_char);
        if tablename.is_null() {
            if args_has(args, 'n' as i32 as u_char) != 0 {
                tablename = b"root\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                tablename = b"prefix\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        if key_bindings_get_table(tablename, 0 as ::core::ffi::c_int).is_null() {
            if quiet == 0 {
                cmdq_error(
                    item,
                    b"table %s doesn't exist\0" as *const u8 as *const ::core::ffi::c_char,
                    tablename,
                );
            }
            return CMD_RETURN_ERROR;
        }
        key_bindings_remove_table(tablename);
        return CMD_RETURN_NORMAL;
    }
    if keystr.is_null() {
        if quiet == 0 {
            cmdq_error(
                item,
                b"missing key\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return CMD_RETURN_ERROR;
    }
    key = key_string_parse_cstr(std::ffi::CStr::from_ptr(keystr)).unwrap_or(KEYC_UNKNOWN);
    if key == KEYC_NONE as ::core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
    {
        if quiet == 0 {
            cmdq_error(
                item,
                b"unknown key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                keystr,
            );
        }
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'T' as i32 as u_char) != 0 {
        tablename = args_get(args, 'T' as i32 as u_char);
        if key_bindings_get_table(tablename, 0 as ::core::ffi::c_int).is_null() {
            if quiet == 0 {
                cmdq_error(
                    item,
                    b"table %s doesn't exist\0" as *const u8 as *const ::core::ffi::c_char,
                    tablename,
                );
            }
            return CMD_RETURN_ERROR;
        }
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        tablename = b"root\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        tablename = b"prefix\0" as *const u8 as *const ::core::ffi::c_char;
    }
    key_bindings_remove(tablename, key);
    return CMD_RETURN_NORMAL;
}
