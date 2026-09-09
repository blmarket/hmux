//! `kill-pane`: takes a pane out of its window, or under `-a` takes every
//! *other* pane of the target's window out instead.
//!
//! The `-a` half unzooms the window and then walks its panes, skipping the
//! target and giving each of the rest the three steps that let a pane go: off
//! every client that was reading it, out of the layout tree, and out of the
//! window — the last of which frees it. The plain half hands the target to
//! `server_kill_pane`, which does the same three steps for a window with
//! another pane behind it and kills the whole window when the target was its
//! last one. A target carrying no active pane at all is the command's one
//! refusal, and it can only come up without `-a`: the `-a` half never reads
//! the pane except to compare it against the ones it is removing, so a null
//! one simply matches nothing and every pane of the window goes.
//!
//! The `-a` walk snapshots pane identities before removing any of them, so
//! no pane borrow survives the operation that frees its storage.

use crate::arguments::args_has;
use crate::cmd::cmd_get_args;

use crate::fmt_args;

use crate::server::server_kill_pane;
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval, cmdq_item};
pub use crate::types::{RustWindowPaneWeak, WindowRef, args_parse_t};

pub use crate::consts::{CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL};
use ::core::ffi::c_char;

pub(crate) static cmd_kill_pane_entry: RustCommandEntry = RustCommandEntry {
    name: c"kill-pane",
    alias: Some(c"killp"),
    args: args_parse_t {
        template: c"at:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-a] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_kill_pane_exec,
};

/// Releases a pane from its clients, layout, and owning window.
unsafe fn remove(owner: &WindowRef, pane: &RustWindowPaneWeak) {
    unsafe { owner.discard_pane(pane, true) };
}

unsafe fn cmd_kill_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let owner = item.target.window().expect("a kill target has a window");
    let target = item.target.pane_ref();
    if args_has(args, b'a') != 0 {
        unsafe { owner.unzoom_and_redraw() };
        let ids = owner.panes();
        for pane in ids.into_iter().filter(|pane| Some(pane) != target.as_ref()) {
            unsafe { remove(&owner, &pane) };
        }
        unsafe { owner.redraw() };
        return CMD_RETURN_NORMAL;
    }

    let Some(target) = target else {
        unsafe { item.error(c"no active pane to kill", fmt_args![]) };
        return CMD_RETURN_ERROR;
    };
    unsafe { server_kill_pane(&target) };
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../tests/test_cmd_kill_pane.rs"]
mod tests;
