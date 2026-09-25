use crate::src::arguments::{args_get, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_print};
use crate::src::cmd::{cmd_find, cmd_get_args, cmd_table};
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
#[no_mangle]
pub static mut cmd_list_commands_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"list-commands\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"lscm\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"F:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-F format] [command]\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(
            cmd_list_commands as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_list_single_command(
    mut entry: *const cmd_entry,
    mut ft: *mut format_tree,
    mut template: *const ::core::ffi::c_char,
    mut item: *mut cmdq_item,
) {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    format_add(
        ft,
        b"command_list_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*entry).name,
    );
    if !(*entry).alias.is_null() {
        s = (*entry).alias;
    } else {
        s = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    format_add(
        ft,
        b"command_list_alias\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    if !(*entry).usage.is_null() {
        s = (*entry).usage;
    } else {
        s = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    format_add(
        ft,
        b"command_list_usage\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    let line = format_expand_cstring(ft, template);
    if !line.is_empty() {
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            line.as_ptr(),
        );
    }
}
unsafe fn cmd_list_commands(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
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
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause.as_ptr(),
                );
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
