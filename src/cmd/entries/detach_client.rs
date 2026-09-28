use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_source, cmdq_get_target_client,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::compat::imsg::*;
use crate::src::server::clients;
use crate::src::server_client::{server_client_detach, server_client_exec, server_client_suspend};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_READONLY;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMD_CLIENT_TFLAG, CMD_FIND_CANFAIL, CMD_READONLY, CMD_TARGET_CLIENT_USAGE,
};
use crate::src::shared::session::session;
pub static cmd_detach_client_entry: cmd_entry = {
    cmd_entry {
        name: c"detach-client",
        alias: Some(c"detach"),
        args: args_parse {
            template: c"aE:s:t:P",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-aP] [-E shell-command] [-s target-session] [-t target-client]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_READONLY | CMD_CLIENT_TFLAG,
        exec: Some(cmd_detach_client_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_suspend_client_entry: cmd_entry = {
    cmd_entry {
        name: c"suspend-client",
        alias: Some(c"suspendc"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_CLIENT_USAGE,
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
        flags: CMD_CLIENT_TFLAG,
        exec: Some(cmd_detach_client_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_detach_client_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let c_owner = cmdq_get_client(item);
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut msgtype: msgtype = 0 as msgtype;
    let mut cmd: *const ::core::ffi::c_char = args_get(&*(args), 'E' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_suspend_client_entry) {
        server_client_suspend(tc);
        return CMD_RETURN_NORMAL;
    }
    if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
        if args_has(args, 's' as i32 as u_char) != 0
            || args_has(args, 'a' as i32 as u_char) != 0
            || c != tc
        {
            cmdq_error(item, |out| out.write_all(b"client is read-only"));
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'P' as i32 as u_char) != 0 {
        msgtype = MSG_DETACHKILL;
    } else {
        msgtype = MSG_DETACH;
    }
    if args_has(args, 's' as i32 as u_char) != 0 {
        s = (*source).s_ptr();
        if s.is_null() {
            return CMD_RETURN_NORMAL;
        }
        let mut registry_loop_0_owner = clients.first();
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            if (*loop_0).session_ptr() == s {
                if !cmd.is_null() {
                    server_client_exec(loop_0, cmd);
                } else {
                    server_client_detach(loop_0, msgtype);
                }
            }
            registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
            loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        return CMD_RETURN_STOP;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        let mut registry_loop_0_owner = clients.first();
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            if !(*loop_0).session_ptr().is_null() && loop_0 != tc {
                if !cmd.is_null() {
                    server_client_exec(loop_0, cmd);
                } else {
                    server_client_detach(loop_0, msgtype);
                }
            }
            registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
            loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        return CMD_RETURN_NORMAL;
    }
    if !cmd.is_null() {
        server_client_exec(tc, cmd);
    } else {
        server_client_detach(tc, msgtype);
    }
    return CMD_RETURN_STOP;
}
