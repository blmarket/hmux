use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::environ::{environ_clear, environ_set, environ_unset};
use crate::src::ffi::libc::strchr;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::environment::environ;
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::tmux::global_environ;
pub static cmd_set_environment_entry: cmd_entry = {
    cmd_entry {
        name: c"set-environment",
        alias: Some(c"setenv"),
        args: args_parse {
            template: c"Fhgrt:u",
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
        exec: Some(cmd_set_environment_exec),
    }
};
unsafe fn cmd_set_environment_exec(mut self_0: *mut cmd, item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let env: &mut environ;
    let mut name: *const ::core::ffi::c_char = args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: Option<std::ffi::CString> = None;
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    if *name as ::core::ffi::c_int == '\0' as i32 {
        cmdq_error(item_handle, |out| out.write_all(b"empty variable name"));
        return CMD_RETURN_ERROR;
    }
    if !strchr(name, '=' as i32).is_null() {
        cmdq_error(item_handle, |out| out.write_all(b"variable name contains ="));
        return CMD_RETURN_ERROR;
    }
    if args_count(args) < 2 as u_int {
        value = ::core::ptr::null::<::core::ffi::c_char>();
    } else {
        value = args_string(&mut *(args), 1 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    }
    if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
        expanded = Some(format_single_from_target_cstring(item_handle, value));
        value = expanded.as_ref().expect("expanded value was set").as_ptr();
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        env = global_environ.as_deref_mut().expect("environment");
        current_block = 224731115979188411;
    } else if (*target).session_handle().is_none() {
        tflag = args_get(&*(args), 't' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
        if !tflag.is_null() {
            cmdq_error(item_handle, |out| {
                out.write_all(b"no such session: ")?;
                write_cstr(out, tflag)
            });
        } else {
            cmdq_error(item_handle, |out| out.write_all(b"no current session"));
        }
        return CMD_RETURN_ERROR;
    } else {
        env = (*(*target).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).environ.as_deref_mut().expect("environment");
        current_block = 224731115979188411;
    }
    match current_block {
        224731115979188411 => {
            if args_has(args, 'u' as i32 as u_char) != 0 {
                if !value.is_null() {
                    cmdq_error(item_handle, |out| out.write_all(b"can't specify a value with -u"));
                    retval = CMD_RETURN_ERROR;
                } else {
                    environ_unset(env, name);
                }
            } else if args_has(args, 'r' as i32 as u_char) != 0 {
                if !value.is_null() {
                    cmdq_error(item_handle, |out| out.write_all(b"can't specify a value with -r"));
                    retval = CMD_RETURN_ERROR;
                } else {
                    environ_clear(env, name);
                }
            } else if value.is_null() {
                cmdq_error(item_handle, |out| out.write_all(b"no value specified"));
                retval = CMD_RETURN_ERROR;
            } else if args_has(args, 'h' as i32 as u_char) != 0 {
                environ_set(env, name, ENVIRON_HIDDEN, |out| write_cstr(out, value));
            } else {
                environ_set(env, name, 0 as ::core::ffi::c_int, |out| {
                    write_cstr(out, value)
                });
            }
        }
        _ => {}
    }
    return retval;
}
