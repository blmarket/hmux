use crate::args::RustArguments;
use crate::args::args_get_str;

use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL};
use crate::fmt_args;
use crate::prompt_history::{
    PromptHistoryStore, PromptHistoryType, with_prompt_history, with_prompt_history_mut,
};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{args_parse_t, u_char, u_int};
use core::ffi::CStr;

pub(crate) static cmd_show_prompt_history_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-prompt-history",
        alias: Some(c"showphist"),
        args: args_parse_t {
            template: c"T:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-T prompt-type]",
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
        exec: cmd_show_prompt_history_exec,
    }
};
pub(crate) static cmd_clear_prompt_history_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"clear-prompt-history",
        alias: Some(c"clearphist"),
        args: args_parse_t {
            template: c"T:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-T prompt-type]",
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
        exec: cmd_show_prompt_history_exec,
    }
};
unsafe fn cmd_show_prompt_history_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let typestr = args_get_str(args, 'T' as i32 as u_char);
    let kind = match typestr.map(PromptHistoryType::parse) {
        Some(None) => {
            unsafe {
                item.error(
                    c"invalid type: %s",
                    fmt_args![typestr.expect("an invalid type was supplied")],
                );
            }
            return CMD_RETURN_ERROR;
        }
        Some(Some(kind)) => Some(kind),
        None => None,
    };
    if core::ptr::eq(cmd_get_entry(self_0), &cmd_clear_prompt_history_entry) {
        with_prompt_history_mut(|history| match kind {
            Some(kind) => history.clear(kind),
            None => history.clear_all(),
        });
        return CMD_RETURN_NORMAL;
    }
    let kinds = kind.map_or_else(|| PromptHistoryType::ALL.to_vec(), |kind| vec![kind]);
    for kind in kinds {
        unsafe {
            item.print(c"History for %s:\n", fmt_args![kind.name()]);
        }
        let entries = with_prompt_history(|history| {
            history
                .entries(kind)
                .map(CStr::to_owned)
                .collect::<Vec<_>>()
        });
        for (index, entry) in entries.iter().enumerate() {
            unsafe {
                item.print(c"%d: %s", fmt_args![(index + 1) as u_int, entry.as_c_str()]);
            }
        }
        unsafe { item.print(c"%s", fmt_args![c""]) };
    }
    CMD_RETURN_NORMAL
}
