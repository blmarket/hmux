use crate::args::args_parse_t;
use crate::args::RustArguments;
use crate::cmd::cmd_get_args;

use crate::fmt_args;
use crate::format::format_single_from_target;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
use crate::tmux::{check_name, clean_name};
use crate::types::{SessionRef};

pub(crate) static cmd_rename_session_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"rename-session",
        alias: Some(c"rename"),
        args: args_parse_t {
            template: c"t:",
            lower: 1 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-t target-session] new-name",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_rename_session_exec,
    }
};
unsafe fn cmd_rename_session_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let target = &item.target;
    let session = target.session().expect("the target retains its session");
    let tmp = unsafe {
        format_single_from_target(
            item,
            args.argument_string(0).expect("argument count checked"),
        )
    };
    if check_name(Some(&tmp)) == 0 {
        unsafe { item.error(c"invalid session name: %s", fmt_args![tmp.as_c_str()]) };
        return CMD_RETURN_ERROR;
    }
    let Some(newname) = (unsafe { clean_name(&tmp, 0 as core::ffi::c_int) }) else {
        return CMD_RETURN_NORMAL;
    };
    if session.name().as_deref() == Some(newname.as_c_str()) {
        return CMD_RETURN_NORMAL;
    }
    if SessionRef::find(&newname).is_some() {
        unsafe { item.error(c"duplicate session: %s", fmt_args![newname.as_c_str()]) };
        return CMD_RETURN_ERROR;
    }
    unsafe { session.rename(newname) };
    unsafe { session.request_status() };
    unsafe { session.notify(c"session-renamed") };
    CMD_RETURN_NORMAL
}
