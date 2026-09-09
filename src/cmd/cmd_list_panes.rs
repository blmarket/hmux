//! `list-panes`: the panes of one window, of a whole session under `-s`, or of
//! every session in the server under `-a`, one line each through the format
//! engine.
//!
//! The three walks nest: `-a` hands every session to the session walk, which
//! hands every window of it to the window walk, which is the only one that
//! prints. Which walk called decides the built-in template a line takes when
//! `-F` gives none — pane alone, window and pane, or session, window and pane
//! — and that is the whole of the [`Level`] the walks pass down. `-O` picks
//! the order the panes come in and `-r` reverses it, `-f` filters lines by a
//! format that has to expand to something true, and a `-O` value the sort
//! module does not know is the command's one error.
//!
//! Quirks kept. The sort order is read twice from the same `-O`: once in exec,
//! only to refuse an unknown one, and again in the window walk that actually
//! sorts with it. And `line` is the *number* of panes the window holds, not
//! the index of the pane being printed, so every line of one window carries
//! the same value.
//!
//! The transpiled `switch (type)` had a fourth, empty arm that would have left
//! the template null; the three walks pass one of exactly three values, which
//! [`Level`] now says outright, so that arm is gone with the conversion.

use crate::arguments::{args_get_str, args_has};
use crate::cmd::cmd_get_args;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    FORMAT_NONE, SORT_END,
};
use crate::fmt_args;
use crate::format::{
    format_add, format_create_for_client, format_defaults_for_link_pane, format_expand, format_true,
};
use crate::session::SESSIONS;
use crate::sort::{RustSortCriteria, SortCriteria};
pub use crate::types::*;
use crate::window::{WinlinkRef, winlinks_in};
use ::core::ffi::{CStr, c_char};

pub(crate) static cmd_list_panes_entry: RustCommandEntry = RustCommandEntry {
    name: c"list-panes",
    alias: Some(c"lsp"),
    args: args_parse_t {
        template: c"aF:f:O:rst:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-asr] [-F format] [-f filter] [-O order][-t target-window]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_list_panes_exec,
};

/// Which of the three walks is printing, which is all a line needs to know to
/// pick its built-in template.
///
/// The C carried this as a plain `int` and switched on it with no default arm,
/// so a value beyond these three would have left the template null and handed
/// that null to `format_expand`. Only [`cmd_list_panes_exec`],
/// [`cmd_list_panes_server`] and [`cmd_list_panes_session`] ever say which
/// walk is running, and between them they name exactly these three, which is
/// why the empty arm the transpiler wrote out is not here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Level {
    /// One window's panes, named by pane index alone.
    Window,
    /// A session's windows, so a pane needs its window's index too.
    Session,
    /// Every session, so a pane needs its session's name as well.
    Server,
}

impl Level {
    /// The template a line takes when `-F` gives none.
    fn template(self) -> &'static CStr {
        match self {
            Level::Window => {
                c"#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}"
            }
            Level::Session => {
                c"#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}"
            }
            Level::Server => {
                c"#{session_name}:#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}"
            }
        }
    }
}

/// Whether `ft` passes `filter`: always when there is no filter, and otherwise
/// when the filter expands to something the format engine counts as true.
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

unsafe fn cmd_list_panes_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);

    let order = RustSortCriteria::parse_order(args_get_str(args, b'O'));
    if order == SORT_END && args_has(args, b'O') != 0 {
        unsafe { item.error(c"invalid sort order", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }

    if args_has(args, b'a') != 0 {
        unsafe { cmd_list_panes_server(self_0, item) };
    } else if args_has(args, b's') != 0 {
        let session = item.target.session().expect("the state names a session");
        unsafe { cmd_list_panes_session(self_0, &session, item, Level::Session) };
    } else {
        let session = item.target.session().expect("the state names a session");
        let winlink = WinlinkRef::new(session, item.target.wl_idx.expect("the target has a link"))
            .expect("the target link exists");
        unsafe { cmd_list_panes_window(self_0, &winlink, item, Level::Window) };
    }
    CMD_RETURN_NORMAL
}

unsafe fn cmd_list_panes_server(self_0: &cmd, item: &cmdq_item) {
    unsafe {
        for s in SESSIONS.read().values() {
            cmd_list_panes_session(self_0, s, item, Level::Server);
        }
    }
}

unsafe fn cmd_list_panes_session(self_0: &cmd, s: &SessionRef, item: &cmdq_item, level: Level) {
    unsafe {
        for wl in winlinks_in(s) {
            cmd_list_panes_window(self_0, &wl, item, level);
        }
    }
}

unsafe fn cmd_list_panes_window(self_0: &cmd, link: &WinlinkRef, item: &cmdq_item, level: Level) {
    unsafe {
        let args = cmd_get_args(self_0);
        let template = match args_get_str(args, b'F') {
            Some(given) => given,
            None => level.template(),
        };
        let filter = args_get_str(args, b'f');

        let sort_crit = RustSortCriteria::new(
            RustSortCriteria::parse_order(args_get_str(args, b'O')),
            args_has(args, b'r') != 0,
        );

        let Some(window) = link.window() else {
            return;
        };
        let panes = window.sorted_panes(&sort_crit);
        let n = panes.len() as u_int;
        for wp in panes {
            let mut ft =
                format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
            format_add(&mut ft, c"line", c"%u", fmt_args![n]);
            if !format_defaults_for_link_pane(&mut ft, link, &wp) {
                continue;
            }
            if passes(&mut ft, filter) {
                let line = format_expand(&mut ft, template);
                item.print(c"%s", fmt_args![line.as_c_str()]);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_list_panes.rs"]
mod tests;

#[cfg(test)]
pub use crate::consts::SORT_INDEX;
