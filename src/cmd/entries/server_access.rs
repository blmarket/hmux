use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::src::ffi::libc::{getgrnam, getpwnam, getuid};
use crate::src::format::format_single_cstring;
use crate::src::server_acl::{
    server_acl_allow, server_acl_allow_write, server_acl_deny, server_acl_deny_write,
    server_acl_display, server_acl_find,
};
use crate::src::shared::abi::id_t;
use crate::src::shared::abi::*;
use crate::src::shared::account::{group, passwd};
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_CLIENT_CANFAIL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::pane::window_pane;
use crate::src::shared::server_acl::SERVER_ACL_IS_GROUP;
use crate::src::shared::session::session;
use crate::src::shared::window::winlink;
pub static mut cmd_server_access_entry: cmd_entry =  {
    cmd_entry {
        name: b"server-access\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"adglrw\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-adglrw] [-t target-pane] [user|group]\0" as *const u8
            as *const ::core::ffi::c_char,
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
        flags: CMD_CLIENT_CANFAIL,
        exec: Some(cmd_server_access_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_server_access_deny(
    mut item: *mut cmdq_item,
    mut id: id_t,
    mut flags: ::core::ffi::c_int,
    mut type_0: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) -> cmd_retval {
    if server_acl_find(id, flags) == 0 {
        cmdq_error(
            item,
            b"%s %s not found\0" as *const u8 as *const ::core::ffi::c_char,
            type_0,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    server_acl_deny(id, flags);
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_server_access_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_target_client(item);
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut gr: *mut group = ::core::ptr::null_mut::<group>();
    let mut id: id_t = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if args_has(args, 'l' as i32 as u_char) != 0 {
        server_acl_display(item);
        return CMD_RETURN_NORMAL;
    }
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing user or group argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    let arg = format_single_cstring(
        item,
        args_string(args, 0 as u_int),
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    if args_has(args, 'g' as i32 as u_char) != 0 {
        type_0 = b"group\0" as *const u8 as *const ::core::ffi::c_char;
        gr = getgrnam(arg.as_ptr());
        if !gr.is_null() {
            id = (*gr).gr_gid as id_t;
            name = (*gr).gr_name;
            flags |= SERVER_ACL_IS_GROUP;
        }
    } else {
        type_0 = b"user\0" as *const u8 as *const ::core::ffi::c_char;
        pw = getpwnam(arg.as_ptr());
        if !pw.is_null() {
            id = (*pw).pw_uid as id_t;
            name = (*pw).pw_name;
        }
    }
    if name.is_null() {
        cmdq_error(
            item,
            b"unknown %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            type_0,
            arg.as_ptr(),
        );
        return CMD_RETURN_ERROR;
    }
    if !flags & SERVER_ACL_IS_GROUP != 0 && (id == 0 as id_t || id == getuid()) {
        cmdq_error(
            item,
            b"%s owns the server, can't change access\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 && args_has(args, 'd' as i32 as u_char) != 0 {
        cmdq_error(
            item,
            b"-a and -d cannot be used together\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'w' as i32 as u_char) != 0 && args_has(args, 'r' as i32 as u_char) != 0 {
        cmdq_error(
            item,
            b"-r and -w cannot be used together\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        return cmd_server_access_deny(item, id, flags, type_0, name);
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        if server_acl_find(id, flags) != 0 {
            cmdq_error(
                item,
                b"%s %s is already added\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        server_acl_allow(id, flags);
    } else if args_has(args, 'r' as i32 as u_char) != 0 || args_has(args, 'w' as i32 as u_char) != 0
    {
        if server_acl_find(id, flags) == 0 {
            server_acl_allow(id, flags);
        }
    }
    if args_has(args, 'w' as i32 as u_char) != 0 {
        if server_acl_find(id, flags) == 0 {
            cmdq_error(
                item,
                b"%s %s not found\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        server_acl_allow_write(id, flags);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'r' as i32 as u_char) != 0 {
        if server_acl_find(id, flags) == 0 {
            cmdq_error(
                item,
                b"%s %s not found\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        server_acl_deny_write(id, flags);
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_NORMAL;
}
