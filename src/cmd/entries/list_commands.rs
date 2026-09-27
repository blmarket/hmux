use crate::src::arguments::{args_get, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_print};
use crate::src::cmd::{cmd_find, cmd_get_args, cmd_table};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_STARTSERVER};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::winlink;
use std::ffi::CStr;

pub const LIST_COMMANDS_TEMPLATE: [::core::ffi::c_char; 91] = unsafe {
    ::core::mem::transmute::<
        [u8; 91],
        [::core::ffi::c_char; 91],
    >(
        *b"#{command_list_name}#{?command_list_alias, (#{command_list_alias}),} #{command_list_usage}\0",
    )
};
pub static mut cmd_list_commands_entry: cmd_entry = {
    cmd_entry {
        name: c"list-commands",
        alias: Some(c"lscm"),
        args: args_parse {
            template: b"F:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-F format] [command]",
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
        flags: CMD_STARTSERVER | CMD_AFTERHOOK,
        exec: Some(cmd_list_commands as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_list_single_command(
    mut entry: *const cmd_entry,
    mut ft: *mut format_tree,
    mut template: *const ::core::ffi::c_char,
    mut item: *mut cmdq_item,
) {
    format_add(
        ft,
        b"command_list_name\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, (*entry).name.as_ptr()),
    );
    format_add(
        ft,
        b"command_list_alias\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, (*entry).alias.unwrap_or(c"").as_ptr()),
    );
    format_add(
        ft,
        b"command_list_usage\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, (*entry).usage.as_ptr()),
    );
    let line = format_expand_cstring(ft, template);
    if !line.is_empty() {
        cmdq_print(item, |out| write_cstr(out, line.as_ptr()));
    }
}
unsafe fn cmd_list_commands(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut entryp: *mut *const cmd_entry = ::core::ptr::null_mut::<*const cmd_entry>();
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    template = args_get(args, 'F' as i32 as u_char);
    if template.is_null() {
        template = LIST_COMMANDS_TEMPLATE.as_ptr();
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
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    command = args_string(args, 0 as u_int);
    if command.is_null() {
        entryp = &raw mut cmd_table as *mut *const cmd_entry;
        while !(*entryp).is_null() {
            cmd_list_single_command(*entryp, ft, template, item);
            entryp = entryp.offset(1);
        }
    } else {
        match cmd_find(CStr::from_ptr(command)) {
            Ok(found) => {
                entry = found;
            }
            Err(cause) => {
                cmdq_error(item, |out| write_cstr(out, cause.as_ptr()));
                format_free(ft);
                return CMD_RETURN_ERROR;
            }
        }
        if !entry.is_null() {
            cmd_list_single_command(entry, ft, template, item);
        }
    }
    format_free(ft);
    return CMD_RETURN_NORMAL;
}
