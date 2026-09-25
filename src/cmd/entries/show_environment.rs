use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target, cmdq_print};
use crate::src::environ::{environ_find, environ_first, environ_next};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::shared::environment::{environ, environ_entry};
use crate::src::tmux::global_environ;
use std::ffi::{CStr, CString};

#[no_mangle]
pub static mut cmd_show_environment_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-environment\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showenv\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"hgst:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-hgs] [-t target-session] [variable]\0" as *const u8
            as *const ::core::ffi::c_char,
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
        exec: Some(
            cmd_show_environment_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_show_environment_escape(envent: &environ_entry) -> CString {
    // The entry value is a C string: only bytes before its first NUL are visible.
    let value = CStr::from_ptr(
        envent
            .value
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    )
    .to_bytes();
    let mut escaped = Vec::with_capacity(value.len().saturating_mul(2));
    for &byte in value {
        if matches!(byte, b'$' | b'`' | b'"' | b'\\') {
            escaped.push(b'\\');
        }
        escaped.push(byte);
    }
    CString::new(escaped).expect("environment value was truncated at its first NUL")
}
unsafe extern "C" fn cmd_show_environment_print(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut envent: *mut environ_entry,
) {
    let mut args: *mut args = cmd_get_args(self_0);
    if args_has(args, 'h' as i32 as u_char) == 0 && (*envent).flags & ENVIRON_HIDDEN != 0 {
        return;
    }
    if args_has(args, 'h' as i32 as u_char) != 0 && !(*envent).flags & ENVIRON_HIDDEN != 0 {
        return;
    }
    if args_has(args, 's' as i32 as u_char) == 0 {
        if !(*envent).value.is_none() {
            cmdq_print(
                item,
                b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*envent).name).as_ptr().cast_mut(),
                ((*envent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
        } else {
            cmdq_print(
                item,
                b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*envent).name).as_ptr().cast_mut(),
            );
        }
        return;
    }
    if !(*envent).value.is_none() {
        let escaped = cmd_show_environment_escape(&*envent);
        cmdq_print(
            item,
            b"%s=\"%s\"; export %s;\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).name).as_ptr().cast_mut(),
            escaped.as_ptr(),
            ((*envent).name).as_ptr().cast_mut(),
        );
    } else {
        cmdq_print(
            item,
            b"unset %s;\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).name).as_ptr().cast_mut(),
        );
    };
}
unsafe fn cmd_show_environment_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut tflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    tflag = args_get(args, 't' as i32 as u_char);
    if !tflag.is_null() {
        if (*target).s.is_null() {
            cmdq_error(
                item,
                b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                tflag,
            );
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        env = global_environ;
    } else {
        if (*target).s.is_null() {
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
            return CMD_RETURN_ERROR;
        }
        env = (*(*target).s).environ;
    }
    if !name.is_null() {
        envent = environ_find(env, name);
        if envent.is_null() {
            cmdq_error(
                item,
                b"unknown variable: %s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        cmd_show_environment_print(self_0, item, envent);
        return CMD_RETURN_NORMAL;
    }
    envent = environ_first(env);
    while !envent.is_null() {
        cmd_show_environment_print(self_0, item, envent);
        envent = environ_next(envent);
    }
    return CMD_RETURN_NORMAL;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_preserves_non_utf8_bytes_and_stops_at_first_nul() {
        let mut value = b"\xff$`\"\\\0ignored".to_vec();
        let entry = environ_entry {
            name: Default::default(),
            value: Some(
                std::ffi::CStr::from_bytes_until_nul(&value)
                    .unwrap()
                    .to_owned(),
            ),
            flags: 0,
            owner: None,
        };
        let escaped = unsafe { cmd_show_environment_escape(&entry) };
        assert_eq!(escaped.as_bytes(), b"\xff\\$\\`\\\"\\\\");
    }
}
