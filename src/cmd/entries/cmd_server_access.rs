use crate::args::{args_count, args_has, args_string_str};
use crate::cmd::cmd_get_args;

use crate::ffi::getuid;
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};

pub use crate::consts::{CMD_CLIENT_CANFAIL, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL};
use crate::server::client_walk;
use crate::server::{
    ServerAclAccess, ServerAclStore, server_acl_display, server_acl_update_clients,
    with_server_acl, with_server_acl_mut,
};
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
pub use crate::cmd::cmdq_item;
pub use crate::types::{__uid_t, args, args_parse_t, u_char, u_int, uid_t};
use crate::{UserAccount, UserAccountRecord};

pub(crate) static cmd_server_access_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"server-access",
        alias: None,
        args: args_parse_t {
            template: c"adlrw",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-adlrw] [-t target-pane] [user]",
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
        exec: cmd_server_access_exec,
    }
};
unsafe fn cmd_server_access_deny(item: &cmdq_item, pw: &UserAccountRecord) -> cmd_retval {
    unsafe {
        let uid = pw.account_user_id() as uid_t;
        if with_server_acl(|acl| acl.find(uid)).is_none() {
            item.error(c"user %s not found", fmt_args![pw.account_name()]);
            return CMD_RETURN_ERROR;
        }
        for mut owner in client_walk() {
            let peer_uid = owner.peer_handle().uid();
            if peer_uid == uid {
                owner.request_exit(c"access not allowed");
            }
        }
        with_server_acl_mut(|acl| acl.deny(uid));
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_server_access_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let c = item.target_client();
    if args_has(args, 'l' as i32 as u_char) != 0 {
        unsafe { server_acl_display(item) };
        return CMD_RETURN_NORMAL;
    }
    if args_count(args) == 0 as u_int {
        unsafe { item.error(c"missing user argument", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    let name = unsafe {
        let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
        format_defaults_for_handles(&mut ft, c.as_ref(), None, None, None);
        format_expand(
            &mut ft,
            args_string_str(args, 0).expect("argument count checked"),
        )
    };
    let pw = if name.as_bytes().is_empty() {
        None
    } else {
        UserAccountRecord::lookup_name(&name)
    };
    let Some(pw) = pw else {
        unsafe { item.error(c"unknown user: %s", fmt_args![name.as_c_str()]) };
        return CMD_RETURN_ERROR;
    };
    if unsafe { pw.account_user_id() == 0 as __uid_t || pw.account_user_id() == getuid() } {
        unsafe {
            item.error(
                c"%s owns the server, can't change access",
                fmt_args![pw.account_name()],
            )
        };
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 && args_has(args, 'd' as i32 as u_char) != 0 {
        unsafe { item.error(c"-a and -d cannot be used together", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'w' as i32 as u_char) != 0 && args_has(args, 'r' as i32 as u_char) != 0 {
        unsafe { item.error(c"-r and -w cannot be used together", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        return unsafe { cmd_server_access_deny(item, &pw) };
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        if with_server_acl(|acl| acl.find(pw.account_user_id() as uid_t)).is_some() {
            unsafe { item.error(c"user %s is already added", fmt_args![pw.account_name()]) };
            return CMD_RETURN_ERROR;
        }
        with_server_acl_mut(|acl| acl.allow(pw.account_user_id() as uid_t));
    } else if (args_has(args, 'r' as i32 as u_char) != 0
        || args_has(args, 'w' as i32 as u_char) != 0)
        && with_server_acl(|acl| acl.find(pw.account_user_id() as uid_t)).is_none()
    {
        with_server_acl_mut(|acl| acl.allow(pw.account_user_id() as uid_t));
    }
    if args_has(args, 'w' as i32 as u_char) != 0 {
        let uid = pw.account_user_id() as uid_t;
        if with_server_acl(|acl| acl.find(uid)).is_none() {
            unsafe { item.error(c"user %s not found", fmt_args![pw.account_name()]) };
            return CMD_RETURN_ERROR;
        }
        with_server_acl_mut(|acl| acl.set_access(uid, ServerAclAccess::ReadWrite));
        server_acl_update_clients(uid, ServerAclAccess::ReadWrite);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'r' as i32 as u_char) != 0 {
        let uid = pw.account_user_id() as uid_t;
        if with_server_acl(|acl| acl.find(uid)).is_none() {
            unsafe { item.error(c"user %s not found", fmt_args![pw.account_name()]) };
            return CMD_RETURN_ERROR;
        }
        with_server_acl_mut(|acl| acl.set_access(uid, ServerAclAccess::ReadOnly));
        server_acl_update_clients(uid, ServerAclAccess::ReadOnly);
        return CMD_RETURN_NORMAL;
    }
    CMD_RETURN_NORMAL
}
