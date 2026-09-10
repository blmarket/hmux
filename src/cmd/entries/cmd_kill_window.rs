//! `kill-window` and `unlink-window`: one exec hook, told apart by which
//! entry the running command was parsed as.
//!
//! `unlink-window` takes the target's winlink out of its session and nothing
//! else, refusing without `-k` when the session is the only thing holding the
//! window — `-k` is what says the window may go with it. `kill-window` takes
//! the window out of every session that holds it. Under `-a` it takes every
//! *other* window of the target's session instead, one at a time until none is
//! left, and only then the target's own window, and only if the session holds
//! that window more than once; a session whose target window is its one and
//! only winlink is answered without touching anything.
//!
//! The `-a` walk kills one window per pass and starts the walk again, the way
//! the C's `RB_FOREACH` did with its `break`, because `server_kill_window`
//! detaches every winlink of that window from every session and so rewrites
//! the tree being walked.

use crate::args::args_parse_t;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL};
use crate::fmt_args;
use crate::resize::recalculate_sizes;
use crate::server::server_renumber_all;
#[cfg(test)]
use crate::types::cmd_find_state;
use ::core::ffi::c_char;

pub(crate) static cmd_kill_window_entry: RustCommandEntry = RustCommandEntry {
    name: c"kill-window",
    alias: Some(c"killw"),
    args: args_parse_t {
        template: c"at:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-a] [-t target-window]",
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
    flags: 0,
    exec: cmd_kill_window_exec,
};
pub(crate) static cmd_unlink_window_entry: RustCommandEntry = RustCommandEntry {
    name: c"unlink-window",
    alias: Some(c"unlinkw"),
    args: args_parse_t {
        template: c"kt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-k] [-t target-window]",
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
    flags: 0,
    exec: cmd_kill_window_exec,
};

unsafe fn cmd_kill_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let mut session = item
        .target
        .session()
        .expect("a kill-window target has a session");
    let window = item
        .target
        .window()
        .expect("a kill-window target has a window");
    let index = item.target.wl.expect("a kill-window target has a link");

    if cmd_get_entry(self_0).name == cmd_unlink_window_entry.name {
        if args.argument_flag_count(b'k') == 0 && !session.is_linked(&window) {
            unsafe { item.error(c"window only linked to one session", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        unsafe { session.unlink_window(index) };
        recalculate_sizes();
        return CMD_RETURN_NORMAL;
    }

    if args.argument_flag_count(b'a') != 0 {
        if unsafe { session.window_count() == 1 } {
            return CMD_RETURN_NORMAL;
        }
        while let Some(other) = unsafe { session.first_other_window(&window) } {
            unsafe { (other.clone()).kill(0) };
        }
        if unsafe { session.links_to(&window) > 1 } {
            unsafe { (window.clone()).kill(0) };
        }
        server_renumber_all();
        return CMD_RETURN_NORMAL;
    }

    unsafe { (window.clone()).kill(1) };
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_kill_window.rs"]
mod tests;
