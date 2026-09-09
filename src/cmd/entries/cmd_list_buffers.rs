//! `list-buffers`: every paste buffer, one line each through the format
//! engine.
//!
//! The buffers come from `sort_get_buffers`, which orders the whole store by
//! `-O` (activity when none is given, the reverse of it under `-r`); a `-O`
//! value the sort module does not know is the command's one error. Each line
//! is the `-F` template, or the built-in one, expanded against the buffer's
//! defaults, and with `-f` the filter is expanded first and the line printed
//! only when it is true.

use crate::args::{args_get_str, args_has};
use crate::cmd::cmd_get_args;

use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, FORMAT_NONE, SORT_END,
};
use crate::fmt_args;
use crate::format::{
    format_create_for_client, format_defaults_paste_buffer, format_expand, format_true,
};
use crate::sort::{RustSortCriteria, SortCriteria, SortedPasteBuffer, sort_get_buffers};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{args_parse_t, format_tree, sort_criteria_t};
use ::core::ffi::CStr;

pub const LIST_BUFFERS_TEMPLATE: &CStr =
    c"#{buffer_name}: #{buffer_size} bytes: \"#{buffer_sample}\"";
pub(crate) static cmd_list_buffers_entry: RustCommandEntry = RustCommandEntry {
    name: c"list-buffers",
    alias: Some(c"lsb"),
    args: args_parse_t {
        template: c"F:f:O:r",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-F format] [-f filter] [-O order]",
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
    exec: cmd_list_buffers_exec,
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

fn sorted_buffers(sort_crit: &mut sort_criteria_t) -> Vec<SortedPasteBuffer> {
    sort_get_buffers(sort_crit)
}

unsafe fn cmd_list_buffers_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);

    let template = args_get_str(args, b'F').unwrap_or(LIST_BUFFERS_TEMPLATE);
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

    for pb in sorted_buffers(&mut sort_crit) {
        let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
        format_defaults_paste_buffer(&mut ft, pb.name.as_c_str());
        if unsafe { passes(&mut ft, filter) } {
            let line = unsafe { format_expand(&mut ft, template) };
            unsafe { item.print(c"%s", fmt_args![line.as_c_str()]) };
        }
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_list_buffers.rs"]
mod tests;

#[cfg(test)]
use crate::consts::SORT_NAME;
