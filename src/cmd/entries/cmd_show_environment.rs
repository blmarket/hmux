use crate::args::RustArguments;
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, ENVIRON_HIDDEN,
};
use crate::environ::EnvironmentStore;
use crate::environ::RustEnvironment;
use crate::environ::with_global_environment;
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::types::{args_parse_t, u_char};
use ::core::ffi::CStr;
use ::std::ffi::CString;

pub(crate) static cmd_show_environment_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-environment",
        alias: Some(c"showenv"),
        args: args_parse_t {
            template: c"hgst:",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-hgs] [-t target-session] [variable]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_show_environment_exec,
    }
};
/// A copy of `value` with the characters a double-quoted shell word takes
/// specially backslash-escaped, which is how `-s` prints it.
fn cmd_show_environment_escape(value: &CStr) -> CString {
    let mut out: Vec<u8> = Vec::new();
    for &c in value.to_bytes() {
        if matches!(c, b'$' | b'`' | b'"' | b'\\') {
            out.push(b'\\');
        }
        out.push(c);
    }
    CString::new(out).expect("environment value cannot contain NUL")
}
fn cmd_show_environment_line(
    args: &RustArguments,
    envent: &crate::environ::EnvironmentEntryRef<'_>,
) -> Option<CString> {
    let hidden = args.argument_flag_count(b'h') != 0;
    if hidden != (envent.flags & ENVIRON_HIDDEN != 0) {
        return None;
    }
    let name = envent.name;
    let value = envent.value;
    let line = if args.argument_flag_count(b's') == 0 {
        if let Some(value) = value {
            format_alloc(c"%s=%s", fmt_args![name, value])
        } else {
            format_alloc(c"-%s", fmt_args![name])
        }
    } else if let Some(value) = value {
        let escaped = cmd_show_environment_escape(value);
        format_alloc(
            c"%s=\"%s\"; export %s;",
            fmt_args![name, escaped.as_c_str(), name],
        )
    } else {
        format_alloc(c"unset %s;", fmt_args![name])
    };
    Some(line)
}

fn cmd_show_environment_lines(
    env: &RustEnvironment,
    args: &RustArguments,
    name: Option<&CStr>,
) -> Option<Vec<CString>> {
    if let Some(name) = name {
        let entry = env.find(name)?;
        Some(
            cmd_show_environment_line(args, &entry)
                .into_iter()
                .collect(),
        )
    } else {
        Some(
            env.entries()
                .filter_map(|entry| cmd_show_environment_line(args, &entry))
                .collect(),
        )
    }
}

unsafe fn cmd_show_environment_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let target = &item.target;
    let name = args.argument_string(0);
    let tflag = {
        let flag = 't' as i32 as u_char;
        args.argument_flag_string(flag)
    };
    if let Some(tflag) = tflag
        && (*target).session().is_none()
    {
        unsafe { item.error(c"no such session: %s", fmt_args![tflag]) };
        return CMD_RETURN_ERROR;
    }
    let lines = if ({
        let flag = 'g' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        with_global_environment(|env| cmd_show_environment_lines(env, args, name))
    } else {
        if (*target).session().is_none() {
            if let Some(tflag) = tflag {
                unsafe { item.error(c"no such session: %s", fmt_args![tflag]) };
            } else {
                unsafe { item.error(c"no current session", fmt_args![]) };
            }
            return CMD_RETURN_ERROR;
        }
        let session = target.session().expect("the state names a session");
        unsafe { session.with_environment(|env| cmd_show_environment_lines(env, args, name)) }
    };
    let Some(lines) = lines else {
        unsafe { item.error(c"unknown variable: %s", fmt_args![name]) };
        return CMD_RETURN_ERROR;
    };
    for line in lines {
        unsafe { item.print(c"%s", fmt_args![line.as_c_str()]) };
    }
    CMD_RETURN_NORMAL
}
