use crate::src::arguments::{args_get, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_print};
use crate::src::cmd::{cmd_find, cmd_get_args_mut, cmd_table};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create_with_client, format_defaults, format_expand_cstring, format_free,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_STARTSERVER};
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::format::format_tree;
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
pub static cmd_list_commands_entry: cmd_entry = {
    cmd_entry {
        name: c"list-commands",
        alias: Some(c"lscm"),
        args: args_parse {
            template: c"F:",
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
        exec: Some(cmd_list_commands),
    }
};
unsafe fn cmd_list_single_command(
    entry: &cmd_entry,
    mut ft: *mut format_tree,
    mut template: *const ::core::ffi::c_char,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) {
    unsafe {
        format_add(
            ft,
            b"command_list_name\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, entry.name.as_ptr()),
        );
        format_add(
            ft,
            b"command_list_alias\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, entry.alias.unwrap_or(c"").as_ptr()),
        );
        format_add(
            ft,
            b"command_list_usage\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, entry.usage.as_ptr()),
        );
        let line = format_expand_cstring(ft, template);
        if !line.is_empty() {
            cmdq_print(item_handle, |out| write_cstr(out, line.as_ptr()));
        }
    }
}
unsafe fn cmd_list_commands(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    unsafe {
        let item = item_handle.get();
        let queue_client = cmdq_get_client((item).as_ref());
        let mut args: *mut args =
            cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
        let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
        let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        template = args_get(&*(args), 'F' as i32 as u_char)
            .map_or(std::ptr::null(), |value| value.as_ptr());
        if template.is_null() {
            template = LIST_COMMANDS_TEMPLATE.as_ptr();
        }
        let mut ft_owner = format_create_with_client(
            queue_client.as_ref(),
            Some(item_handle),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        ft = &raw mut *ft_owner;
        format_defaults(ft, None, None, (refbox::Weak::new()).clone(), None);
        command =
            args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
        if command.is_null() {
            for &entry in &cmd_table {
                cmd_list_single_command(entry, ft, template, item_handle);
            }
        } else {
            match cmd_find(CStr::from_ptr(command)) {
                Ok(found) => {
                    cmd_list_single_command(found, ft, template, item_handle);
                }
                Err(cause) => {
                    cmdq_error(item_handle, |out| write_cstr(out, cause.as_ptr()));
                    format_free(ft_owner);
                    return CMD_RETURN_ERROR;
                }
            }
        }
        format_free(ft_owner);
        return CMD_RETURN_NORMAL;
    }
}
