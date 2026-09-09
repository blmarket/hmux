//! `join-pane` and `move-pane`: two entries over one exec hook, which takes a
//! pane out of its window and hands it to a cell split off another window's
//! pane.
//!
//! The window operation owns splitting, layout tracking, pane ownership and
//! inherited appearance. The command chooses unzoom, selection, redraw and
//! empty-source-window policy around that transition.

use crate::arguments::args_has;
use crate::cmd::cmd_get_args;

use crate::fmt_args;

use crate::resize::recalculate_sizes;

pub use crate::consts::{
    CMD_FIND_DEFAULT_MARKED, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
pub use crate::types::*;
use ::core::ffi::c_char;

pub(crate) static cmd_join_pane_entry: RustCommandEntry = RustCommandEntry {
    name: c"join-pane",
    alias: Some(c"joinp"),
    args: args_parse_t {
        template: c"bdfhvp:l:s:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_PANE,
        flags: CMD_FIND_DEFAULT_MARKED,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_join_pane_exec,
};
pub(crate) static cmd_move_pane_entry: RustCommandEntry = RustCommandEntry {
    name: c"move-pane",
    alias: Some(c"movep"),
    args: args_parse_t {
        template: c"bdfhvp:l:s:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_PANE,
        flags: CMD_FIND_DEFAULT_MARKED,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_join_pane_exec,
};

unsafe fn cmd_join_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let current_state_ref = item.state_ref();
    let dst_session = item
        .target
        .session()
        .expect("a join destination has a session");
    let dst_owner = item
        .target
        .window()
        .expect("a join destination has a window");
    let destination_pane = item
        .target
        .pane_ref()
        .expect("a join destination has a pane");
    let dst_idx = item
        .target
        .wl_idx
        .expect("a join destination has a window link");
    unsafe { dst_owner.unzoom_and_redraw() };
    let src_owner = item.source.window().expect("a join source has a window");
    let source_pane = item.source.pane_ref().expect("a join source has a pane");
    unsafe { src_owner.unzoom_and_redraw() };

    if source_pane == destination_pane {
        unsafe { item.error(c"source and target panes must be different", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }

    if let Err(cause) =
        unsafe { dst_owner.join_pane(&src_owner, &source_pane, &destination_pane, item, args) }
    {
        unsafe { item.error(c"size or position %s", fmt_args![cause.as_c_str()]) };
        return CMD_RETURN_ERROR;
    }

    recalculate_sizes();
    unsafe { src_owner.redraw() };
    unsafe { dst_owner.redraw() };
    if args_has(args, b'd') == 0 {
        unsafe { dst_owner.set_active_pane(&source_pane, 1) };
        unsafe { dst_session.select(dst_idx) };
        unsafe { current_state_ref.update_current_session(&dst_session, 0) };
        unsafe { dst_session.request_redraw() };
    } else {
        unsafe { dst_session.request_status() };
    }

    if src_owner.pane_count() == 0 {
        unsafe { (src_owner.clone()).kill(1) };
    } else {
        unsafe { src_owner.notify(c"window-layout-changed") };
    }
    unsafe { dst_owner.notify(c"window-layout-changed") };
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../tests/test_cmd_join_pane.rs"]
mod tests;
