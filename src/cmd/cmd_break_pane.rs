//! `break-pane`: takes a pane out of its window and gives it a window of its
//! own in the destination session.
//!
//! The command has two halves, told apart by how many panes the source window
//! holds. A window with nothing but the pane in it already *is* the window
//! being asked for, so `server_link_window` relinks it into the destination
//! and `server_unlink_window` takes it out of the source. Otherwise the pane
//! is unlinked from its window's `panes` and `z_index` lists, `window_create`
//! hands back an empty window, the pane becomes its only one, and after a
//! layout and a name `session_attach` puts that window into the destination
//! session. `-a`/`-b` shuffle the destination's later windows up first to free
//! an index, and `-P` prints the result through `-F`'s format or
//! [`BREAK_PANE_TEMPLATE`].
//!
//! Both pane lists are the windows' own intrusive TAILQs of the crate's
//! `window_pane`, so the relinking stays raw-pointer work behind the
//! [`remove`] and [`insert_only`] helpers, which spell out what the C's macros
//! did and keep the order they produced.
//!
//! Two transpiled branches are provably unreachable and are gone with the
//! conversion, each with its proof written where it sat: the `TAILQ_INSERT_HEAD`
//! arm that relinks whatever was at the head (see [`insert_only`]), and the
//! refusal when `winlink_find_by_window` finds no winlink for a window
//! `server_link_window` has just attached (see [`cmd_break_pane_exec`]).
//!
//! Coverage exemptions: none.
use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_from_session;
use crate::cmd::queue::{
    cmdq_error, cmdq_get_current, cmdq_get_source, cmdq_get_target, cmdq_get_target_client,
    cmdq_print,
};
use crate::fmt_args;
use crate::format::format_single;
use crate::layout::{layout_close_pane, layout_init};
use crate::names::default_window_name;
use crate::options::{
    options_get_number, options_load_pane_colours, options_set_number,
    options_set_parent,
};
use crate::server::server_client_remove_pane;
use crate::server::{
    server_link_window, server_redraw_session, server_status_session_group, server_unlink_window,
    server_unzoom_window,
};
use crate::session::{session_attach, session_select};
use crate::session::{session_get_curw, session_options};
use crate::tmux::check_name;
pub use crate::types::*;
use crate::window::window_set_active;
use crate::window::window_set_latest;
use crate::window::{
    window_count_panes, window_create, window_lost_pane, window_pane_set_window_ref,
    window_pane_zindex_insert_head, window_pane_zindex_remove, window_panes_insert_head,
    window_panes_take, window_ref_from_ptr, window_set_name, winlink_find_by_index,
    winlink_find_by_window, winlink_shuffle_up,
};
use ::core::ffi::{c_char, c_int};
use ::std::ffi::CString;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const PANE_CHANGED: c_int = 0x80;
pub const PANE_STYLECHANGED: c_int = 0x1000;
pub const PANE_THEMECHANGED: c_int = 0x2000;
pub const CMD_FIND_WINDOW_INDEX: c_int = 0x4;
pub const BREAK_PANE_TEMPLATE: [c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
pub(crate) static cmd_break_pane_entry: cmd_entry = cmd_entry {
    name: c"break-pane",
    alias: Some(c"breakp"),
    args: args_parse_t {
        template: c"abdPF:n:s:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-abdP] [-F format] [-n window-name] [-s src-pane] [-t dst-window]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: CMD_FIND_WINDOW_INDEX,
    },
    flags: 0,
    exec: cmd_break_pane_exec,
};

/// Runs `break-pane`, as the command queue calls it.
///
/// The single-pane half's look-up of the relinked window cannot fail, so the
/// C's refusal when it does is not written out. A zero from
/// `server_link_window` means its `session_attach` linked `w` into
/// `dst_s->windows`; `server_unlink_window` then takes away only `wl`, which
/// is a different winlink, because `server_link_window` refuses a destination
/// index already holding `w` ("same index") and picks a free one otherwise.
/// The `session_group_synchronize_from` inside the detach rewrites the
/// winlinks of the *other* sessions in `src_s`'s group, and `dst_s` is not one
/// of them: `server_link_window` refuses two different sessions that share a
/// group, and `src_s == dst_s` is skipped by the synchronise itself. So a
/// winlink for `w` is still in `dst_s->windows` when it is looked up.
unsafe fn cmd_break_pane_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let current = cmdq_get_current(item);
        let target = cmdq_get_target(item);
        let source = cmdq_get_source(item);
        let tc = cmdq_get_target_client(&*item);
        let src_s = (*source).session();
        let dst_s = (*target).session();
        let wp = (*source).pane();
        let mut wl = (*source).winlink();
        let mut w = (*wl).window();
        let mut w_ref = window_ref_from_ptr(w);
        let mut idx = (*target).idx;

        let name = args_get(args, b'n');
        if !name.is_null() && check_name(name) == 0 {
            cmdq_error(item, c"invalid window name: %s".as_ptr(), fmt_args![name]);
            return CMD_RETURN_ERROR;
        }

        let before = args_has(args, b'b');
        if args_has(args, b'a') != 0 || before != 0 {
            idx = if !(*target).winlink().is_null() {
                winlink_shuffle_up(dst_s, (*target).winlink(), before)
            } else {
                winlink_shuffle_up(dst_s, session_get_curw(dst_s), before)
            };
            if idx == -1 {
                return CMD_RETURN_ERROR;
            }
        }
        server_unzoom_window(w);

        if window_count_panes(w, 1) == 1 {
            let mut cause: Option<CString> = None;
            if server_link_window(
                src_s,
                wl,
                dst_s,
                idx,
                0,
                (args_has(args, b'd') == 0) as c_int,
                &mut cause,
            ) != 0
            {
                let cause = cause.unwrap();
                cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
            if !name.is_null() {
                window_set_name(w, name, 0);
                options_set_number((*w).options_ptr(), c"automatic-rename".as_ptr(), 0);
            }
            server_unlink_window(src_s, wl);
            wl = winlink_find_by_window(&mut (*dst_s).windows, w);
        } else {
            if idx != -1 && !winlink_find_by_index(&mut (*dst_s).windows, idx).is_null() {
                cmdq_error(item, c"index in use: %d".as_ptr(), fmt_args![idx]);
                return CMD_RETURN_ERROR;
            }

            server_client_remove_pane(wp);
            window_lost_pane(w, wp);
            let pane = window_panes_take(w, wp).expect("the pane is its window's");
            window_pane_zindex_remove(w, wp);
            layout_close_pane(wp);

            w_ref = Some(window_create((*w).sx, (*w).sy, (*w).xpixel, (*w).ypixel));
            w = w_ref.as_ref().unwrap().as_ptr();
            window_pane_set_window_ref(wp, w_ref.as_ref());
            options_set_parent((*wp).options_ptr(), (*w).options_ptr());
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            window_panes_insert_head(w, pane);
            window_pane_zindex_insert_head(w, wp);
            window_set_active(w, wp);
            window_set_latest(w, tc);

            if name.is_null() {
                let newname = default_window_name(w);
                window_set_name(w, newname.as_ptr(), 0);
            } else {
                window_set_name(w, name, 0);
                options_set_number((*w).options_ptr(), c"automatic-rename".as_ptr(), 0);
            }

            layout_init(w, wp);
            (*wp).flags |= PANE_CHANGED;
            options_load_pane_colours((*wp).options_ptr(), Some(&mut (*wp).palette));

            if idx == -1 {
                idx = (-1 - options_get_number(session_options(dst_s), c"base-index".as_ptr()))
                    as c_int;
            }
            let mut cause = None;
            wl = session_attach(dst_s, w, idx, &mut cause);
            if args_has(args, b'd') == 0 {
                session_select(dst_s, (*wl).idx);
                cmd_find_from_session(&mut *current, dst_s, 0);
            }

            server_redraw_session(src_s);
            if src_s != dst_s {
                server_redraw_session(dst_s);
            }
            server_status_session_group(src_s);
            if src_s != dst_s {
                server_status_session_group(dst_s);
            }
        }

        if args_has(args, b'P') != 0 {
            let mut template = args_get(args, b'F');
            if template.is_null() {
                template = BREAK_PANE_TEMPLATE.as_ptr();
            }
            let cp = format_single(
                item,
                ::core::ffi::CStr::from_ptr(template),
                tc,
                dst_s,
                wl,
                wp,
            );
            cmdq_print(item, c"%s".as_ptr(), fmt_args![cp.as_ptr()]);
        }
        CMD_RETURN_NORMAL
    }
}
#[cfg(test)]
#[path = "../tests/test_cmd_break_pane.rs"]
mod tests;
