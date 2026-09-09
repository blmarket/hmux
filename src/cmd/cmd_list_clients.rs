//! `list-clients`: every attached client, one line each through the format
//! engine.
//!
//! The clients come from `sort_get_clients`, which already leaves out the
//! unattached and exiting ones and orders the rest by `-O` (activity when
//! none is given, the reverse of it under `-r`); a `-O` value the sort module
//! does not know is the command's one error. A client with no session is
//! skipped, and with `-t` so is every client on any other session. Each line
//! is the `-F` template, or the built-in one, expanded against the client's
//! defaults plus `line`, and with `-f` the filter is expanded first and the
//! line printed only when it is true.
//!
//! Quirk kept: `line` is the client's index in the sorted list, not the count
//! of lines printed so far, so a client that is skipped leaves a gap in the
//! numbering of the ones after it.

use crate::arguments::{args_get_str, args_has};
use crate::cmd::cmd_get_args;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_READONLY, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, FORMAT_NONE, SORT_END,
};
use crate::fmt_args;
use crate::format::{
    format_add, format_create_for_client, format_defaults_for_handles, format_expand, format_true,
};
use crate::sort::{RustSortCriteria, SortCriteria, sort_get_clients};
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval, cmdq_item};
pub use crate::types::{ClientRef, args_parse_t, format_tree, sort_criteria_t, u_int};
#[cfg(test)]
use crate::types::uint64_t;
use ::core::ffi::{CStr, c_char};

pub const LIST_CLIENTS_TEMPLATE: &CStr = c"#{client_name}: #{session_name} [#{client_width}x#{client_height} #{client_termname}] #{?#{!=:#{client_uid},#{uid}},[user #{?client_user,#{client_user},#{client_uid},}] ,}#{?client_flags,(,}#{client_flags}#{?client_flags,),}";
pub(crate) static cmd_list_clients_entry: RustCommandEntry = RustCommandEntry {
    name: c"list-clients",
    alias: Some(c"lsc"),
    args: args_parse_t {
        template: c"F:f:O:rt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-F format] [-f filter] [-O order][-t target-session]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_SESSION,
        flags: 0,
    },
    flags: CMD_READONLY | CMD_AFTERHOOK,
    exec: cmd_list_clients_exec,
};

/// Whether `ft` passes `filter`: always when there is no filter, and
/// otherwise when the filter expands to something the format engine counts
/// as true.
unsafe fn passes(ft: &mut format_tree, filter: Option<&CStr>) -> bool {
    unsafe {
        match filter {
            None => true,
            Some(filter) => {
                let expanded = format_expand(ft, filter);
                format_true(Some(&expanded)) != 0
            }
        }
    }
}

fn sorted_clients(sort_crit: &mut sort_criteria_t) -> Vec<ClientRef> {
    sort_get_clients(sort_crit)
}

unsafe fn cmd_list_clients_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let session = if args_has(args, b't') != 0 {
        item.target.session()
    } else {
        None
    };

    let template = args_get_str(args, b'F').unwrap_or(LIST_CLIENTS_TEMPLATE);
    let filter = args_get_str(args, b'f');

    let mut sort_crit = RustSortCriteria::new(
        RustSortCriteria::parse_order(args_get_str(args, b'O')),
        false,
    );
    if sort_crit.order() == SORT_END && args_has(args, b'O') != 0 {
        unsafe { item.error(c"invalid sort order", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    sort_crit.set_reversed(args_has(args, b'r') != 0);

    for (i, c) in sorted_clients(&mut sort_crit).iter().enumerate() {
        let Some(attached) = ({ c.attached_session() }) else {
            continue;
        };
        if session
            .as_ref()
            .is_some_and(|session| !session.ptr_eq(&attached))
        {
            continue;
        }
        let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
        format_add(&mut ft, c"line", c"%u", fmt_args![i as u_int]);
        unsafe { format_defaults_for_handles(&mut ft, Some(c), None, None, None) };
        if unsafe { passes(&mut ft, filter) } {
            let line = unsafe { format_expand(&mut ft, template) };
            unsafe { item.print(c"%s", fmt_args![line.as_c_str()]) };
        }
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../tests/test_cmd_list_clients.rs"]
mod tests;

#[cfg(test)]
pub use crate::consts::SORT_NAME;
