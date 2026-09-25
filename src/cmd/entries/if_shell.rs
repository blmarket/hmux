use crate::src::arguments::{
    args_count, args_has, args_make_commands, args_make_commands_free, args_make_commands_now,
    args_make_commands_prepare, args_string,
};
use crate::src::cmd::parse::cmd_parse_error_uppercase_first;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
};
use crate::src::cmd::{cmd_get_args, cmd_list_free};
use crate::src::ffi::libc::__ctype_toupper_loc;
use crate::src::format::format_single_from_target_cstring;
use crate::src::job::{job_get_status, job_run};
use crate::src::server_client::{server_client_get_cwd, server_client_unref};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_FIND_CANFAIL;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_state,
};
use crate::src::shared::environment::environ;
use crate::src::shared::job::job;
use crate::src::shared::session::session;
use crate::src::status::status_message_set;

#[repr(C)]
pub struct cmd_if_shell_data {
    pub cmd_if: *mut args_command_state,
    pub cmd_else: *mut args_command_state,
    pub client: *mut client,
    pub item: *mut cmdq_item,
}
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
#[no_mangle]
pub static mut cmd_if_shell_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"if-shell\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"if\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bFt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 2 as ::core::ffi::c_int,
            upper: 3 as ::core::ffi::c_int,
            cb: Some(cmd_if_shell_args_parse),
        },
        usage: b"[-bF] [-t target-pane] shell-command command [command]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_if_shell_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
fn cmd_if_shell_args_parse(_args: &mut args, idx: u_int) -> args_parse_type {
    if idx == 1 as u_int || idx == 2 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    return ARGS_PARSE_STRING;
}
unsafe extern "C" fn cmd_if_shell_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut count: u_int = args_count(args);
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let shellcmd = format_single_from_target_cstring(item, args_string(args, 0 as u_int));
    if args_has(args, 'F' as i32 as u_char) != 0 {
        if *shellcmd.as_ptr() as ::core::ffi::c_int != '0' as i32
            && *shellcmd.as_ptr() as ::core::ffi::c_int != '\0' as i32
        {
            cmdlist = args_make_commands_now(self_0, item, 1 as u_int, 0 as ::core::ffi::c_int);
        } else if count == 3 as u_int {
            cmdlist = args_make_commands_now(self_0, item, 2 as u_int, 0 as ::core::ffi::c_int);
        } else {
            return CMD_RETURN_NORMAL;
        }
        if cmdlist.is_null() {
            return CMD_RETURN_ERROR;
        }
        new_item = cmdq_get_command(cmdlist, cmdq_get_state(item));
        cmdq_insert_after(item, new_item);
        cmd_list_free(cmdlist);
        return CMD_RETURN_NORMAL;
    }
    let mut cdata = Box::new(cmd_if_shell_data {
        cmd_if: ::core::ptr::null_mut(),
        cmd_else: ::core::ptr::null_mut(),
        client: ::core::ptr::null_mut(),
        item: ::core::ptr::null_mut(),
    });
    cdata.cmd_if = args_make_commands_prepare(
        self_0,
        item,
        1 as u_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        wait,
        0 as ::core::ffi::c_int,
    );
    if count == 3 as u_int {
        cdata.cmd_else = args_make_commands_prepare(
            self_0,
            item,
            2 as u_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            wait,
            0 as ::core::ffi::c_int,
        );
    }
    if wait != 0 {
        cdata.client = cmdq_get_client(item);
        cdata.item = item;
    } else {
        cdata.client = tc;
    }
    if !cdata.client.is_null() {
        (*cdata.client).references += 1;
    }
    // The job owns this pointer on success; its free callback drops the box.
    // job_run leaves data untouched on failure, so reclaim it below.
    let cdata = Box::into_raw(cdata);
    if job_run(
        Some(shellcmd.as_c_str()),
        &Vec::new(),
        ::core::ptr::null_mut::<environ>(),
        s,
        {
            let cwd = server_client_get_cwd(cmdq_get_client(item), s);
            (!cwd.is_null()).then(|| std::ffi::CStr::from_ptr(cwd))
        },
        None,
        Some(Box::new(move |job| unsafe {
            cmd_if_shell_callback(job, cdata)
        })),
        Some(Box::new(move || unsafe {
            cmd_if_shell_free(cdata)
        })),
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    )
    .is_null()
    {
        cmdq_error(
            item,
            b"failed to run command: %s\0" as *const u8 as *const ::core::ffi::c_char,
            shellcmd.as_ptr(),
        );
        cmd_if_shell_free(cdata);
        return CMD_RETURN_ERROR;
    }
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe fn cmd_if_shell_callback(mut job: *mut job, mut cdata: *mut cmd_if_shell_data) {
    let mut c: *mut client = (*cdata).client;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut status: ::core::ffi::c_int = 0;
    status = job_get_status(job);
    if !(status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int)
        || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        state = (*cdata).cmd_else;
    } else {
        state = (*cdata).cmd_if;
    }
    if !state.is_null() {
        match args_make_commands(state, &Vec::new()) {
            Err(mut error) => {
                if (*cdata).item.is_null() {
                    cmd_parse_error_uppercase_first(&mut error);
                }
                let error_ptr = error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr());
                if (*cdata).item.is_null() {
                    status_message_set(
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        error_ptr,
                    );
                } else {
                    cmdq_error(
                        (*cdata).item,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        error_ptr,
                    );
                }
            }
            Ok(commands) if item.is_null() => {
                new_item = cmdq_get_command(commands, ::core::ptr::null_mut::<cmdq_state>());
                cmdq_append(c, new_item);
                cmd_list_free(commands);
            }
            Ok(commands) => {
                new_item = cmdq_get_command(commands, cmdq_get_state(item));
                cmdq_insert_after(item, new_item);
                cmd_list_free(commands);
            }
        }
    }
    if !(*cdata).item.is_null() {
        cmdq_continue((*cdata).item);
    }
}
unsafe fn cmd_if_shell_free(data: *mut cmd_if_shell_data) {
    let cdata = Box::from_raw(data);
    if !cdata.client.is_null() {
        server_client_unref(cdata.client);
    }
    if !cdata.cmd_else.is_null() {
        args_make_commands_free(cdata.cmd_else);
    }
    args_make_commands_free(cdata.cmd_if);
}
