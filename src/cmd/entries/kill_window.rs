use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::format::{
    format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{server_kill_window, server_renumber_all, server_unlink_window};
use crate::src::session::session_is_linked;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{window, winlink};
use crate::src::window::{winlinks_minmax, winlinks_next, winlinks_prev};
pub static cmd_kill_window_entry: cmd_entry = {
    cmd_entry {
        name: c"kill-window",
        alias: Some(c"killw"),
        args: args_parse {
            template: c"af:t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-f filter] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_kill_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_unlink_window_entry: cmd_entry = {
    cmd_entry {
        name: c"unlink-window",
        alias: Some(c"unlinkw"),
        args: args_parse {
            template: c"kt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-k] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_kill_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_kill_window_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window_ptr();
    let mut s: *mut session = (*target).s;
    let mut filter: *const ::core::ffi::c_char = args_get(args, 'f' as i32 as u_char);
    if !filter.is_null() && args_has(args, 'a' as i32 as u_char) == 0 {
        cmdq_error(item, |out| out.write_all(b"-f only valid with -a"));
        return CMD_RETURN_ERROR;
    }
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_unlink_window_entry) {
        if args_has(args, 'k' as i32 as u_char) == 0 && session_is_linked(s, w) == 0 {
            cmdq_error(item, |out| {
                out.write_all(b"window only linked to one session")
            });
            return CMD_RETURN_ERROR;
        }
        server_unlink_window(s, wl);
        recalculate_sizes();
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        return cmd_kill_window_all(item, filter);
    }
    server_kill_window((*wl).window_ptr(), 1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_kill_window_all(
    mut item: *mut cmdq_item,
    mut filter: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut loop_0: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut found: u_int = 0;
    let mut kill_current: u_int = 0;
    if winlinks_prev(&*wl).is_null() && winlinks_next(&*wl).is_null() {
        return CMD_RETURN_NORMAL;
    }
    loop {
        found = 0 as u_int;
        loop_0 = winlinks_minmax(&(*s).windows, RB_NEGINF);
        while !loop_0.is_null() {
            if (*loop_0).window_ptr() != (*wl).window_ptr()
                && cmd_kill_window_filter(item, s, loop_0, filter) != 0
            {
                server_kill_window((*loop_0).window_ptr(), 0 as ::core::ffi::c_int);
                found = found.wrapping_add(1);
                break;
            } else {
                loop_0 = winlinks_next(&*loop_0);
            }
        }
        if !(found != 0 as u_int) {
            break;
        }
    }
    kill_current = 0 as u_int;
    found = kill_current;
    loop_0 = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !loop_0.is_null() {
        if (*loop_0).window_ptr() == (*wl).window_ptr() {
            found = found.wrapping_add(1);
            if cmd_kill_window_filter(item, s, loop_0, filter) != 0 {
                kill_current = 1 as u_int;
            }
        }
        loop_0 = winlinks_next(&*loop_0);
    }
    if kill_current != 0 && found > 1 as u_int {
        server_kill_window((*wl).window_ptr(), 0 as ::core::ffi::c_int);
    }
    server_renumber_all();
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_kill_window_filter(
    mut item: *mut cmdq_item,
    mut s: *mut session,
    mut wl: *mut winlink,
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
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    let expanded = format_expand_cstring(ft, filter);
    flag = format_true(expanded.as_ptr());
    format_free(ft);
    return flag;
}
