use crate::src::server_client::server_client_unref_owned;
use std::cell::UnsafeCell;
use std::rc::Rc;
use crate::src::shared::client::{client_retain, client_rc_ptr};
use crate::src::arguments::{
    args_count, args_has, args_make_commands, args_make_commands_now, args_make_commands_prepare,
    args_string,
};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::parse::cmd_parse_error_uppercase_first;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
};
use crate::src::ffi::libc::__ctype_toupper_loc;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::job::job_run;
use crate::src::server_client::{server_client_get_cwd};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_FIND_CANFAIL;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item, cmdq_state,
};
use crate::src::shared::environment::environ;
use crate::src::shared::job::{JobCompletion, JobExitStatus};
use crate::src::shared::rc;
use crate::src::shared::session::session;
use crate::src::status::status_message_set;

pub struct cmd_if_shell_data {
    pub cmd_if: Option<Box<args_command_state>>,
    pub cmd_else: Option<Box<args_command_state>>,
    pub client: Option<Rc<UnsafeCell<client>>>,
    pub item: *mut cmdq_item,
}
pub static cmd_if_shell_entry: cmd_entry = {
    cmd_entry {
        name: c"if-shell",
        alias: Some(c"if"),
        args: args_parse {
            template: c"bFt:",
            lower: 2 as ::core::ffi::c_int,
            upper: 3 as ::core::ffi::c_int,
            cb: Some(cmd_if_shell_args_parse),
        },
        usage: c"[-bF] [-t target-pane] shell-command command [command]",
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
        exec: Some(cmd_if_shell_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
fn cmd_if_shell_args_parse(
    _args: &mut args,
    idx: u_int,
) -> Result<args_parse_type, ArgsParseError> {
    if idx == 1 as u_int || idx == 2 as u_int {
        return Ok(ARGS_PARSE_COMMANDS_OR_STRING);
    }
    Ok(ARGS_PARSE_STRING)
}
unsafe fn cmd_if_shell_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let queue_client = cmdq_get_client(item);
    let queue_client_ptr = queue_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut s: *mut session = (*target).s_ptr();
    let mut count: u_int = args_count(args);
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let shellcmd = format_single_from_target_cstring(item, args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()));
    if args_has(args, 'F' as i32 as u_char) != 0 {
        let cmdlist = if *shellcmd.as_ptr() as ::core::ffi::c_int != '0' as i32
            && *shellcmd.as_ptr() as ::core::ffi::c_int != '\0' as i32
        {
            args_make_commands_now(self_0, item, 1 as u_int, 0 as ::core::ffi::c_int)
        } else if count == 3 as u_int {
            args_make_commands_now(self_0, item, 2 as u_int, 0 as ::core::ffi::c_int)
        } else {
            return CMD_RETURN_NORMAL;
        };
        let Some(cmdlist) = cmdlist else {
            return CMD_RETURN_ERROR;
        };
        new_item = cmdq_get_command(&cmdlist, (*item).state.as_ref());
        cmdq_insert_after(item, new_item);
        drop(cmdlist);
        return CMD_RETURN_NORMAL;
    }
    let mut cdata = Box::new(cmd_if_shell_data {
        cmd_if: None,
        cmd_else: None,
        client: None,
        item: ::core::ptr::null_mut(),
    });
    cdata.cmd_if = Some(args_make_commands_prepare(
        self_0,
        item,
        1 as u_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        wait,
        0 as ::core::ffi::c_int,
    ));
    if count == 3 as u_int {
        cdata.cmd_else = Some(args_make_commands_prepare(
            self_0,
            item,
            2 as u_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            wait,
            0 as ::core::ffi::c_int,
        ));
    }
    if wait != 0 {
        cdata.client = cmdq_get_client(item);
        cdata.item = item;
    } else {
        cdata.client = client_retain(tc);
    }
    let cwd = server_client_get_cwd(queue_client_ptr.as_ref(), s.as_ref());
    let job = job_run(
        Some(shellcmd.as_c_str()),
        &Vec::new(),
        None,
        s,
        cwd.as_deref(),
        None,
        None,
        None,
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    if job.is_null() {
        cmdq_error(item, |out| {
            out.write_all(b"failed to run command: ")?;
            write_cstr(out, shellcmd.as_ptr())
        });
        return CMD_RETURN_ERROR;
    }
    // The job takes ownership only after startup succeeds. The record also
    // drops when the completion callback is cancelled without being invoked.
    (*job).completecb = Some(Box::new(move |completion| unsafe {
        cmd_if_shell_callback(completion, &mut cdata);
    }));
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe fn cmd_if_shell_callback(completion: JobCompletion, cdata: &mut cmd_if_shell_data) {
    let mut c: *mut client = client_rc_ptr(&cdata.client);
    let mut item: *mut cmdq_item = cdata.item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let state = if completion.status == JobExitStatus::Exited(0) {
        cdata.cmd_if.as_deref_mut()
    } else {
        cdata.cmd_else.as_deref_mut()
    };
    if let Some(state) = state {
        match args_make_commands(state, &Vec::new()) {
            Err(mut error) => {
                if cdata.item.is_null() {
                    cmd_parse_error_uppercase_first(&mut error);
                }
                let error_ptr = error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr());
                if cdata.item.is_null() {
                    status_message_set(
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        |out| write_cstr(out, error_ptr),
                    );
                } else {
                    cmdq_error(cdata.item, |out| write_cstr(out, error_ptr));
                }
            }
            Ok(commands) if item.is_null() => {
                new_item =
                    cmdq_get_command(&commands, None);
                cmdq_append(c.as_ref().map(|client| client.observer.upgrade().expect("queue client is live")).as_ref(), new_item);
                drop(commands);
            }
            Ok(commands) => {
                new_item = cmdq_get_command(&commands, (*item).state.as_ref());
                cmdq_insert_after(item, new_item);
                drop(commands);
            }
        }
    }
    if !cdata.item.is_null() {
        cmdq_continue(cdata.item);
    }
}
impl Drop for cmd_if_shell_data {
    fn drop(&mut self) {
        unsafe {
            if let Some(client) = self.client.take() {
                server_client_unref_owned(client);
            }
            drop(self.cmd_else.take());
            drop(self.cmd_if.take());
        }
    }
}
