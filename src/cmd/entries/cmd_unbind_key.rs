use crate::args::RustArguments;
use crate::args::{args_has, args_string_str};
use crate::cmd::cmd_get_args;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, KEYC_NONE, KEYC_UNKNOWN,
};
use crate::fmt_args;
use crate::key_bindings::key_bindings_get_table;
use crate::key_bindings::{key_bindings_remove, key_bindings_remove_table};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::types::{args_parse_t, key_code, u_char};

pub(crate) static cmd_unbind_key_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"unbind-key",
        alias: Some(c"unbind"),
        args: args_parse_t {
            template: c"anqT:",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-anq] [-T key-table] key",
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
        exec: cmd_unbind_key_exec,
    }
};
unsafe fn cmd_unbind_key_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);

    let keystr = unsafe { args_string_str(args, 0) };
    let quiet: core::ffi::c_int = args_has(args, 'q' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 {
        if keystr.is_some() {
            if quiet == 0 {
                unsafe { item.error(c"key given with -a", fmt_args![]) };
            }
            return CMD_RETURN_ERROR;
        }
        let tablename = match {
            let flag = 'T' as i32 as u_char;
            args.argument_flag_string(flag)
        } {
            Some(given) => given,
            None if args_has(args, 'n' as i32 as u_char) != 0 => c"root",
            None => c"prefix",
        };
        if key_bindings_get_table(tablename, 0 as core::ffi::c_int).is_none() {
            if quiet == 0 {
                unsafe { item.error(c"table %s doesn't exist", fmt_args![tablename]) };
            }
            return CMD_RETURN_ERROR;
        }
        unsafe { key_bindings_remove_table(tablename) };
        return CMD_RETURN_NORMAL;
    }
    if keystr.is_none() {
        if quiet == 0 {
            unsafe { item.error(c"missing key", fmt_args![]) };
        }
        return CMD_RETURN_ERROR;
    }
    let key: key_code = RustKeyStringCodec.parse_key(keystr.expect("checked above"));
    if key == KEYC_NONE as core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as core::ffi::c_ulong as key_code
    {
        if quiet == 0 {
            unsafe { item.error(c"unknown key: %s", fmt_args![keystr.unwrap()]) };
        }
        return CMD_RETURN_ERROR;
    }
    let tablename = if let Some(given) = {
        let flag = 'T' as i32 as u_char;
        args.argument_flag_string(flag)
    } {
        if key_bindings_get_table(given, 0 as core::ffi::c_int).is_none() {
            if quiet == 0 {
                unsafe { item.error(c"table %s doesn't exist", fmt_args![given]) };
            }
            return CMD_RETURN_ERROR;
        }
        given
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        c"root"
    } else {
        c"prefix"
    };
    unsafe { key_bindings_remove(tablename, key) };
    CMD_RETURN_NORMAL
}
