use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::environ::{environ_clear, environ_set, environ_unset};
use crate::src::ffi::libc::strchr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::environment::environ;
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::tmux::global_environ;
pub static mut cmd_set_environment_entry: cmd_entry =  {
    cmd_entry {
        name: c"set-environment",
        alias: Some(c"setenv"),
        args: args_parse {
            template: b"Fhgrt:u\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-Fhgru] [-t target-session] variable [value]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_set_environment_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_set_environment_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut name: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: Option<std::ffi::CString> = None;
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    if *name as ::core::ffi::c_int == '\0' as i32 {
        cmdq_error(
            item,
            b"empty variable name\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if !strchr(name, '=' as i32).is_null() {
        cmdq_error(
            item,
            b"variable name contains =\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_count(args) < 2 as u_int {
        value = ::core::ptr::null::<::core::ffi::c_char>();
    } else {
        value = args_string(args, 1 as u_int);
    }
    if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
        expanded = Some(format_single_from_target_cstring(item, value));
        value = expanded.as_ref().expect("expanded value was set").as_ptr();
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        env = global_environ;
        current_block = 224731115979188411;
    } else if (*target).s.is_null() {
        tflag = args_get(args, 't' as i32 as u_char);
        if !tflag.is_null() {
            cmdq_error(
                item,
                b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                tflag,
            );
        } else {
            cmdq_error(
                item,
                b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        retval = CMD_RETURN_ERROR;
        current_block = 2189724439242469808;
    } else {
        env = (*(*target).s).environ;
        current_block = 224731115979188411;
    }
    match current_block {
        224731115979188411 => {
            if args_has(args, 'u' as i32 as u_char) != 0 {
                if !value.is_null() {
                    cmdq_error(
                        item,
                        b"can't specify a value with -u\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    retval = CMD_RETURN_ERROR;
                } else {
                    environ_unset(env, name);
                }
            } else if args_has(args, 'r' as i32 as u_char) != 0 {
                if !value.is_null() {
                    cmdq_error(
                        item,
                        b"can't specify a value with -r\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    retval = CMD_RETURN_ERROR;
                } else {
                    environ_clear(env, name);
                }
            } else if value.is_null() {
                cmdq_error(
                    item,
                    b"no value specified\0" as *const u8 as *const ::core::ffi::c_char,
                );
                retval = CMD_RETURN_ERROR;
            } else if args_has(args, 'h' as i32 as u_char) != 0 {
                environ_set(
                    env,
                    name,
                    ENVIRON_HIDDEN,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
            } else {
                environ_set(
                    env,
                    name,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
            }
        }
        _ => {}
    }
    return retval;
}
