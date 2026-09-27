use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_print};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_READONLY};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::window::winlink;
use crate::src::sort::{sort_get_clients, sort_order_from_string};

pub const LIST_CLIENTS_TEMPLATE: [::core::ffi::c_char; 225] = unsafe {
    ::core::mem::transmute::<
        [u8; 225],
        [::core::ffi::c_char; 225],
    >(
        *b"#{client_name}: #{session_name} [#{client_width}x#{client_height} #{client_termname}] #{?#{!=:#{client_uid},#{uid}},[user #{?client_user,#{client_user},#{client_uid},}] ,}#{?client_flags,(,}#{client_flags}#{?client_flags,),}\0",
    )
};
pub static cmd_list_clients_entry: cmd_entry = {
    cmd_entry {
        name: c"list-clients",
        alias: Some(c"lsc"),
        args: args_parse {
            template: c"F:f:O:rt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-F format] [-f filter] [-O order][-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_READONLY | CMD_AFTERHOOK,
        exec: Some(cmd_list_clients_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_list_clients_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut filter: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    if args_has(args, 't' as i32 as u_char) != 0 {
        s = (*target).s;
    } else {
        s = ::core::ptr::null_mut::<session>();
    }
    template = args_get(args, 'F' as i32 as u_char);
    if template.is_null() {
        template = LIST_CLIENTS_TEMPLATE.as_ptr();
    }
    filter = args_get(args, 'f' as i32 as u_char);
    sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if sort_crit.order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    let clients_sorted = sort_get_clients(&raw mut sort_crit);
    i = 0 as u_int;
    while (i as usize) < clients_sorted.len() {
        let c = clients_sorted[i as usize];
        if !((*c).session.is_null() || !s.is_null() && s != (*c).session) {
            ft = format_create(
                cmdq_get_client(item),
                item,
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            );
            format_add(
                ft,
                b"line\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (i) as u32),
            );
            format_defaults(
                ft,
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
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
        }
        i = i.wrapping_add(1);
    }
    return CMD_RETURN_NORMAL;
}
