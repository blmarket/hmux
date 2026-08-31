//! `join-pane` and `move-pane`: two entries over one exec hook, which takes a
//! pane out of its window and hands it to a cell split off another window's
//! pane.
//!
//! The move is three steps. `layout_get_tiled_cell` splits the destination
//! pane and answers the cell the moved pane is about to fill, or the reason it
//! cannot be filled; the pane is unlinked from its old window's `panes` and
//! `z_index` lists and linked into the destination window's; and
//! `layout_assign_pane` drops it into the cell. Both lists are the windows'
//! own intrusive TAILQs of the crate's `window_pane`, so the relinking stays
//! raw-pointer work behind the [`remove`] and [`insert_after`] helpers, which
//! spell out what the C's macros did and keep the order they produced.
//!
//! Quirk kept: the hook's `flags` is zero from its declaration to its last
//! read. `-b` does reach the split, through `layout_get_tiled_cell`, which
//! adds `SPAWN_BEFORE` to its **own** copy of the flags — the hook's copy
//! never hears about it, so the C's `if (flags & SPAWN_BEFORE)` arm, a
//! `TAILQ_INSERT_BEFORE` of the moved pane, cannot run and is gone with the
//! conversion. `join-pane -b` therefore gives the joined pane the first half
//! of the split while still landing it *behind* the target pane in both of the
//! destination window's lists.
//!
//! Coverage exemptions: none.
use crate::arguments::args_has;
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_from_session;
use crate::cmd::queue::{cmdq_error, cmdq_get_current, cmdq_get_source, cmdq_get_target};
use crate::fmt_args;
use crate::layout::{layout_assign_pane, layout_close_pane, layout_get_tiled_cell};
use crate::notify::notify_window;
use crate::options::{options_load_pane_colours, options_set_parent};
use crate::resize::recalculate_sizes;
use crate::server::server_client_remove_pane;
use crate::server::{
    server_kill_window, server_redraw_session, server_redraw_window, server_status_session,
    server_unzoom_window,
};
use crate::session::session_select;
pub use crate::types::*;
use crate::window::{
    window_count_panes, window_lost_pane, window_pane_set_window, window_pane_zindex_insert_after,
    window_pane_zindex_remove, window_panes_insert_after, window_panes_take,
    window_set_active_pane,
};
use ::core::ffi::{c_char, c_int};
use ::std::ffi::CString;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const PANE_STYLECHANGED: c_int = 0x1000;
pub const PANE_THEMECHANGED: c_int = 0x2000;
pub const CMD_FIND_DEFAULT_MARKED: c_int = 0x8;
pub const SPAWN_BEFORE: c_int = 0x8;
pub(crate) static cmd_join_pane_entry: cmd_entry = cmd_entry {
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
pub(crate) static cmd_move_pane_entry: cmd_entry = cmd_entry {
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

unsafe fn cmd_join_pane_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let current = cmdq_get_current(item);
        let target = cmdq_get_target(item);
        let source = cmdq_get_source(item);

        let dst_s = (*target).session();
        let dst_wl = (*target).winlink();
        let dst_wp = (*target).pane();
        let dst_w = (*dst_wl).window();
        let dst_idx = (*dst_wl).idx;
        server_unzoom_window(dst_w);

        let src_wl = (*source).winlink();
        let src_wp = (*source).pane();
        let src_w = (*src_wl).window();
        server_unzoom_window(src_w);

        if src_wp == dst_wp {
            cmdq_error(
                item,
                c"source and target panes must be different".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }

        let mut cause = CString::default();
        let lc = layout_get_tiled_cell(item, args, dst_w, dst_wp, 0, &mut cause);
        if !cause.as_bytes().is_empty() {
            cmdq_error(
                item,
                c"size or position %s".as_ptr(),
                fmt_args![cause.as_ptr()],
            );
            return CMD_RETURN_ERROR;
        }

        layout_close_pane(src_wp);

        server_client_remove_pane(src_wp);
        window_lost_pane(src_w, src_wp);
        let pane = window_panes_take(src_w, src_wp).expect("the pane is its window's");
        window_pane_zindex_remove(src_w, src_wp);

        window_pane_set_window(src_wp, dst_w);
        options_set_parent(
            (*src_wp).options_ptr(),
            (*dst_w).options_ptr(),
        );
        (*src_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        window_panes_insert_after(dst_w, dst_wp, pane);
        window_pane_zindex_insert_after(dst_w, dst_wp, src_wp);
        layout_assign_pane(lc, src_wp, 0);
        options_load_pane_colours((*src_wp).options_ptr(), &raw mut (*src_wp).palette);

        recalculate_sizes();

        server_redraw_window(src_w);
        server_redraw_window(dst_w);

        if args_has(args, b'd') == 0 {
            window_set_active_pane(dst_w, src_wp, 1);
            session_select(dst_s, dst_idx);
            cmd_find_from_session(&mut *current, dst_s, 0);
            server_redraw_session(dst_s);
        } else {
            server_status_session(dst_s);
        }

        if window_count_panes(src_w, 1) == 0 {
            server_kill_window(src_w, 1);
        } else {
            notify_window(c"window-layout-changed".as_ptr(), src_w);
        }
        notify_window(c"window-layout-changed".as_ptr(), dst_w);

        CMD_RETURN_NORMAL
    }
}
#[cfg(test)]
#[path = "../tests/test_cmd_join_pane.rs"]
mod tests;
