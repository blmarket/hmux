use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_print};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::session::sessions;
use crate::src::session::{sessions_minmax, sessions_next};
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
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::winlink;
use crate::src::sort::{sort_get_panes_window, sort_order_from_string};
use crate::src::window::{winlinks_minmax, winlinks_next};
pub static cmd_list_panes_entry: cmd_entry = {
    cmd_entry {
        name: c"list-panes",
        alias: Some(c"lsp"),
        args: args_parse {
            template: c"aF:f:O:rst:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-asr] [-F format] [-f filter] [-O order][-t target-window]",
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
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_list_panes_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_list_panes_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut order: sort_order = SORT_ACTIVITY;
    order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if order as ::core::ffi::c_uint == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        cmd_list_panes_server(self_0, item);
    } else if args_has(args, 's' as i32 as u_char) != 0 {
        cmd_list_panes_session(self_0, s, item, 1 as ::core::ffi::c_int);
    } else {
        cmd_list_panes_window(self_0, s, wl, item, 0 as ::core::ffi::c_int);
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_list_panes_server(mut self_0: *mut cmd, mut item: *mut cmdq_item) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        cmd_list_panes_session(self_0, s, item, 2 as ::core::ffi::c_int);
        s_owner = sessions_next(&*s);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
}
unsafe fn cmd_list_panes_session(
    mut self_0: *mut cmd,
    mut s: *mut session,
    mut item: *mut cmdq_item,
    mut type_0: ::core::ffi::c_int,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        cmd_list_panes_window(self_0, s, wl, item, type_0);
        wl = winlinks_next(&*wl);
    }
}
unsafe fn cmd_list_panes_window(
    mut self_0: *mut cmd,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut item: *mut cmdq_item,
    mut type_0: ::core::ffi::c_int,
) {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut c: *mut client = cmdq_get_client(item);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut filter: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    template = args_get(args, 'F' as i32 as u_char);
    if template.is_null() {
        match type_0 {
            0 => {
                template = b"#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            1 => {
                template = b"#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            2 => {
                template = b"#{session_name}:#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            _ => {}
        }
    }
    filter = args_get(args, 'f' as i32 as u_char);
    sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    let l = sort_get_panes_window((*wl).window_ptr(), &raw mut sort_crit);
    let n = u_int::try_from(l.len()).expect("too many panes to list");
    i = 0 as u_int;
    while i < n {
        wp = l[i as usize];
        ft = format_create(
            cmdq_get_client(item),
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        format_add(
            ft,
            b"line\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as u32),
        );
        format_defaults(ft, c, s, wl, wp);
        if !filter.is_null() {
            let expanded = format_expand_cstring(ft, filter);
            flag = format_true(expanded.as_ptr());
        } else {
            flag = 1 as ::core::ffi::c_int;
        }
        if flag != 0 {
            let line = format_expand_cstring(ft, template);
            cmdq_print(item, |out| write_cstr(out, line.as_ptr()));
        }
        format_free(ft);
        i = i.wrapping_add(1);
    }
}
