use crate::arguments::{args_count, args_get_str, args_has, args_string_str};
use crate::cmd::cmd_get_args;

use crate::environ::EnvironmentStore;
use crate::environ::RustEnvironment;
use crate::environ::with_global_environment_mut;
use crate::fmt_args;
use crate::format::format_single_from_target;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, ENVIRON_HIDDEN,
};
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval, cmdq_item};
pub use crate::types::args_parse_t;
use ::core::ffi::CStr;

pub(crate) static cmd_set_environment_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"set-environment",
        alias: Some(c"setenv"),
        args: args_parse_t {
            template: c"Fhgrt:u",
            lower: 1 as core::ffi::c_int,
            upper: 2 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-Fhgru] [-t target-session] variable [value]",
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
        exec: cmd_set_environment_exec,
    }
};
enum EnvironmentChange<'a> {
    Unset,
    Clear,
    Set {
        value: &'a CStr,
        flags: core::ffi::c_int,
    },
}

impl EnvironmentChange<'_> {
    fn apply(self, env: &mut RustEnvironment, name: &CStr) {
        match self {
            Self::Unset => env.unset(name),
            Self::Clear => env.clear(name),
            Self::Set { value, flags } => env.set(name, flags, value),
        }
    }
}

unsafe fn cmd_set_environment_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let mut target_session = item.target.session();
    let name = unsafe { args_string_str(args, 0) }.expect("argument count checked");
    if name.is_empty() {
        unsafe { item.error(c"empty variable name", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if name.to_bytes().contains(&b'=') {
        unsafe { item.error(c"variable name contains =", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    let value = if args_count(args) < 2 {
        None
    } else {
        Some(unsafe { args_string_str(args, 1) }.expect("argument count checked"))
    };
    let expanded = if args_has(args, b'F') != 0 {
        value.map(|value| unsafe { format_single_from_target(item, value) })
    } else {
        None
    };
    let value = expanded.as_deref().or(value);
    let global = args_has(args, b'g') != 0;
    if !global && target_session.is_none() {
        if let Some(target) = args_get_str(args, b't') {
            unsafe { item.error(c"no such session: %s", fmt_args![target]) };
        } else {
            unsafe { item.error(c"no current session", fmt_args![]) };
        }
        return CMD_RETURN_ERROR;
    }
    let change = if args_has(args, b'u') != 0 {
        if value.is_some() {
            unsafe { item.error(c"can't specify a value with -u", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        EnvironmentChange::Unset
    } else if args_has(args, b'r') != 0 {
        if value.is_some() {
            unsafe { item.error(c"can't specify a value with -r", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        EnvironmentChange::Clear
    } else if let Some(value) = value {
        let flags = if args_has(args, b'h') != 0 {
            ENVIRON_HIDDEN
        } else {
            0
        };
        EnvironmentChange::Set { value, flags }
    } else {
        unsafe { item.error(c"no value specified", fmt_args![]) };
        return CMD_RETURN_ERROR;
    };
    if global {
        with_global_environment_mut(|env| change.apply(env, name));
    } else {
        let session = target_session.as_mut().expect("the target names a session");
        change.apply(unsafe { session.environ() }, name);
    }
    CMD_RETURN_NORMAL
}
