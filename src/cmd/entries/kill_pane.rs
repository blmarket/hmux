use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::src::format::{
    format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::layout::layout_close_pane;
use crate::src::server_client::server_client_remove_pane;
use crate::src::server_fn::{server_kill_pane, server_redraw_window, server_unzoom_window};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::winlink;
use crate::src::window::{window_pane_first, window_pane_next, window_remove_pane};
pub static cmd_kill_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"kill-pane",
        alias: Some(c"killp"),
        args: args_parse {
            template: c"af:t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-f filter] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_kill_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_kill_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut filter: *const ::core::ffi::c_char = args_get(args, 'f' as i32 as u_char);
    if !filter.is_null() && args_has(args, 'a' as i32 as u_char) == 0 {
        cmdq_error(item, |out| out.write_all(b"-f only valid with -a"));
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        return cmd_kill_pane_all(item, filter);
    }
    if wp.is_null() {
        cmdq_error(item, |out| out.write_all(b"no active pane to kill"));
        return CMD_RETURN_ERROR;
    }
    server_kill_pane(wp);
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_kill_pane_all(
    mut item: *mut cmdq_item,
    mut filter: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut loopwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut tmpwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    server_unzoom_window((*wl).window_ptr());
    loopwp = window_pane_first((*wl).window_ptr());
    while !loopwp.is_null() && {
        tmpwp = window_pane_next(loopwp);
        1 as ::core::ffi::c_int != 0
    } {
        if !(loopwp == wp) {
            if !(cmd_kill_pane_filter(item, s, wl, loopwp, filter) == 0) {
                server_client_remove_pane(loopwp);
                layout_close_pane(loopwp);
                window_remove_pane((*wl).window_ptr(), loopwp);
            }
        }
        loopwp = tmpwp;
    }
    server_redraw_window((*wl).window_ptr());
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_kill_pane_filter(
    mut item: *mut cmdq_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut flag: ::core::ffi::c_int = 0;
    if filter.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    ft = format_create(
        cmdq_get_client(item),
        item,
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    let expanded = format_expand_cstring(ft, filter);
    flag = format_true(expanded.as_ptr());
    format_free(ft);
    return flag;
}
