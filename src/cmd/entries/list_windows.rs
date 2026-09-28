use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_print};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
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
use crate::src::shared::window::winlink;
use crate::src::sort::{sort_get_winlinks, sort_get_winlinks_session, sort_order_from_string};

pub const LIST_WINDOWS_WITH_SESSION_TEMPLATE: [::core::ffi::c_char; 127] = unsafe {
    ::core::mem::transmute::<
        [u8; 127],
        [::core::ffi::c_char; 127],
    >(
        *b"#{session_name}:#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] \0",
    )
};
pub static cmd_list_windows_entry: cmd_entry = {
    cmd_entry {
        name: c"list-windows",
        alias: Some(c"lsw"),
        args: args_parse {
            template: c"aF:f:O:rt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-ar] [-F format] [-f filter] [-O order][-t target-session]",
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
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_list_windows_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_list_windows_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let queue_client = cmdq_get_client(item);
    let queue_client_ptr = queue_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let c_owner = cmdq_get_client(item);
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut filter: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    template = args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    filter = args_get(&*(args), 'f' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    sort_crit.order = sort_order_from_string(args_get(&*(args), 'O' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()));
    if sort_crit.order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    let winlinks = if args_has(args, 'a' as i32 as u_char) != 0 {
        let links = sort_get_winlinks(&raw mut sort_crit);
        if template.is_null() {
            template = LIST_WINDOWS_WITH_SESSION_TEMPLATE.as_ptr();
        }
        links
    } else {
        let links = sort_get_winlinks_session((*target).s_ptr(), &raw mut sort_crit);
        if template.is_null() {
            template = b"#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] [layout #{window_layout}] #{window_id}#{?window_active, (active),}\0"
                as *const u8 as *const ::core::ffi::c_char;
        }
        links
    };
    n = u_int::try_from(winlinks.len()).expect("too many winlinks to list");
    i = 0 as u_int;
    while i < n {
        wl = winlinks[i as usize];
        s = (*wl).session;
        ft = format_create(
            queue_client_ptr,
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        format_add(
            ft,
            b"line\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as u32),
        );
        format_defaults(ft, c, s, wl, ::core::ptr::null_mut::<window_pane>());
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
    return CMD_RETURN_NORMAL;
}
