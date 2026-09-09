//! `list-windows`: the windows of one session, or of every session under
//! `-a`, one line each through the format engine.
//!
//! Without `-a` the windows are the target session's, in the order
//! `sort_get_winlinks_session` leaves them; with `-a` they are every window
//! of every session, from `sort_get_winlinks`. Either way `-O` picks the
//! order (activity when none is given, the reverse of it under `-r`) and a
//! `-O` value the sort module does not know is the command's one error. Each
//! line is the `-F` template, or the built-in one for the walk that ran,
//! expanded against the window's defaults plus `line`, and with `-f` the
//! filter is expanded first and the line printed only when it is true.
//!
//! Quirk kept: `line` is the *number* of windows the walk found, not the
//! index of the window being printed, so every line of one run carries the
//! same value — where `list-sessions` and `list-clients` count from zero.

use crate::arguments::{args_get_str, args_has};
use crate::cmd::cmd_get_args;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    FORMAT_NONE, SORT_END,
};
use crate::fmt_args;
use crate::format::{
    format_add, format_create_for_client, format_defaults_for_link, format_expand, format_true,
};
use crate::sort::{RustSortCriteria, SortCriteria, sort_get_winlinks};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmdq::cmdq_item;
use crate::types::{args_parse_t, format_tree, u_int};
use ::core::ffi::{CStr, c_char};

pub const LIST_WINDOWS_WITH_SESSION_TEMPLATE: &CStr = c"#{session_name}:#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] ";
/// The default template of the per-session walk. Upstream spells it as a
/// second `#define` beside the one above; the transpiler inlined it at its
/// only use, and it is a constant again here.
const LIST_WINDOWS_TEMPLATE: &CStr = c"#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] [layout #{window_layout}] #{window_id}#{?window_active, (active),}";

pub(crate) static cmd_list_windows_entry: RustCommandEntry = RustCommandEntry {
    name: c"list-windows",
    alias: Some(c"lsw"),
    args: args_parse_t {
        template: c"aF:f:O:rt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-ar] [-F format] [-f filter] [-O order][-t target-session]",
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
    flags: CMD_AFTERHOOK,
    exec: cmd_list_windows_exec,
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

unsafe fn cmd_list_windows_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let session = item.target.session();

    let given = args_get_str(args, b'F');
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

    let (winlinks, fallback) = if args_has(args, b'a') != 0 {
        (
            sort_get_winlinks(&sort_crit),
            LIST_WINDOWS_WITH_SESSION_TEMPLATE,
        )
    } else {
        (
            (session.as_ref().expect("the target has a session")).sorted_winlinks(&sort_crit),
            LIST_WINDOWS_TEMPLATE,
        )
    };
    let template = given.unwrap_or(fallback);

    let n = winlinks.len() as u_int;
    for link in winlinks {
        let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
        format_add(&mut ft, c"line", c"%u", fmt_args![n]);
        if !unsafe { format_defaults_for_link(&mut ft, &link) } {
            continue;
        }
        if unsafe { passes(&mut ft, filter) } {
            let line = unsafe { format_expand(&mut ft, template) };
            unsafe { item.print(c"%s", fmt_args![line.as_c_str()]) };
        }
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../tests/test_cmd_list_windows.rs"]
mod tests;

#[cfg(test)]
pub use crate::consts::SORT_NAME;
