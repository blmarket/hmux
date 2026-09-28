use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_print};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_create, format_defaults_paste_buffer, format_expand_cstring, format_free, format_true,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::sort::{sort_get_buffers, sort_order_from_string};

pub const LIST_BUFFERS_TEMPLATE: [::core::ffi::c_char; 57] = unsafe {
    ::core::mem::transmute::<[u8; 57], [::core::ffi::c_char; 57]>(
        *b"#{buffer_name}: #{buffer_size} bytes: \"#{buffer_sample}\"\0",
    )
};
pub static cmd_list_buffers_entry: cmd_entry = {
    cmd_entry {
        name: c"list-buffers",
        alias: Some(c"lsb"),
        args: args_parse {
            template: c"F:f:O:r",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-F format] [-f filter] [-O order]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_list_buffers_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_list_buffers_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let queue_client = cmdq_get_client(item);
    let queue_client_ptr = queue_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
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
    if template.is_null() {
        template = LIST_BUFFERS_TEMPLATE.as_ptr();
    }
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
    let buffers = sort_get_buffers(&sort_crit);
    for pb in buffers {
        ft = format_create(
            queue_client_ptr,
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        format_defaults_paste_buffer(&mut *ft, &pb);
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
    return CMD_RETURN_NORMAL;
}
