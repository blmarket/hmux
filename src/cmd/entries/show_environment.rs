use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target, cmdq_print};
use crate::src::environ::{environ_find, environ_iter};
use crate::src::format::bytes::write_cstr;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::shared::environment::{environ, environ_entry};
use crate::src::tmux::global_environ;
use std::ffi::{CStr, CString};
pub static mut cmd_show_environment_entry: cmd_entry = {
    cmd_entry {
        name: c"show-environment",
        alias: Some(c"showenv"),
        args: args_parse {
            template: c"hgst:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-hgs] [-t target-session] [variable]",
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
        exec: Some(cmd_show_environment_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
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
unsafe fn cmd_show_environment_print(
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
            cmdq_print(item, |out| {
                write_cstr(out, ((*envent).name).as_ptr().cast_mut())?;
                out.write_all(b"=")?;
                write_cstr(
                    out,
                    ((*envent).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
            });
        } else {
            cmdq_print(item, |out| {
                out.write_all(b"-")?;
                write_cstr(out, ((*envent).name).as_ptr().cast_mut())
            });
        }
        return;
    }
    if !(*envent).value.is_none() {
        let escaped = cmd_show_environment_escape(&*envent);
        cmdq_print(item, |out| {
            write_cstr(out, ((*envent).name).as_ptr().cast_mut())?;
            out.write_all(b"=\"")?;
            write_cstr(out, escaped.as_ptr())?;
            out.write_all(b"\"; export ")?;
            write_cstr(out, ((*envent).name).as_ptr().cast_mut())?;
            out.write_all(b";")
        });
    } else {
        cmdq_print(item, |out| {
            out.write_all(b"unset ")?;
            write_cstr(out, ((*envent).name).as_ptr().cast_mut())?;
            out.write_all(b";")
        });
    };
}
unsafe fn cmd_show_environment_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut tflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    tflag = args_get(args, 't' as i32 as u_char);
    if !tflag.is_null() {
        if (*target).s.is_null() {
            cmdq_error(item, |out| {
                out.write_all(b"no such session: ")?;
                write_cstr(out, tflag)
            });
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        env = global_environ;
    } else {
        if (*target).s.is_null() {
            tflag = args_get(args, 't' as i32 as u_char);
            if !tflag.is_null() {
                cmdq_error(item, |out| {
                    out.write_all(b"no such session: ")?;
                    write_cstr(out, tflag)
                });
            } else {
                cmdq_error(item, |out| out.write_all(b"no current session"));
            }
            return CMD_RETURN_ERROR;
        }
        env = (*(*target).s).environ;
    }
    if !name.is_null() {
        envent = environ_find(env, name);
        if envent.is_null() {
            cmdq_error(item, |out| {
                out.write_all(b"unknown variable: ")?;
                write_cstr(out, name)
            });
            return CMD_RETURN_ERROR;
        }
        cmd_show_environment_print(self_0, item, envent);
        return CMD_RETURN_NORMAL;
    }
    for entry in environ_iter(&*env) {
        let envent = entry.as_ptr();
        cmd_show_environment_print(self_0, item, envent);
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
        };
        let escaped = unsafe { cmd_show_environment_escape(&entry) };
        assert_eq!(escaped.as_bytes(), b"\xff\\$\\`\\\"\\\\");
    }
}
