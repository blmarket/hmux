use crate::args::args_parse_t;
use crate::args::RustArguments;
use crate::args::args_make_commands;
use crate::cmd::{cmd_make_commands_now, cmd_make_commands_prepare};
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item;
use crate::cmd::{CmdListRef, RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{CmdqItemWeak, cmdq_append, cmdq_item_ref_of, cmdq_item_weak_of};
use crate::compat::toupper;
use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_STRING, CMD_FIND_CANFAIL, CMD_FIND_PANE,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT,
};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::job::job_run_for_session;
use crate::server::client_working_directory;
use crate::status::status_message_for_client;
use crate::types::{
    ClientRef, JobEvent, args, args_command_state, args_parse_type, u_char, u_int,
};
use ::std::ffi::CString;

#[derive(Default)]
#[repr(C)]
pub struct cmd_if_shell_data {
    pub cmd_if: Option<Box<args_command_state>>,
    pub cmd_else: Option<Box<args_command_state>>,
    pub(crate) client_ref: Option<ClientRef>,
    pub(crate) item: Option<CmdqItemWeak>,
}

impl Drop for cmd_if_shell_data {
    fn drop(&mut self) {
        drop(self.cmd_else.take());
        drop(self.cmd_if.take());
    }
}

pub(crate) static cmd_if_shell_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"if-shell",
        alias: Some(c"if"),
        args: args_parse_t {
            template: c"bFt:",
            lower: 2 as core::ffi::c_int,
            upper: 3 as core::ffi::c_int,
            cb: Some(cmd_if_shell_args_parse),
        },
        usage: c"[-bF] [-t target-pane] shell-command command [command]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_if_shell_exec,
    }
};
fn cmd_if_shell_args_parse(
    _args: &args,
    idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    if idx == 1 as u_int || idx == 2 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    ARGS_PARSE_STRING
}
unsafe fn cmd_if_shell_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let mut cdata = Box::<cmd_if_shell_data>::default();
    let target_client = item.target_client();
    let target_session = item.target.session();
    let cmdlist: Option<CmdListRef>;
    let count: u_int = args.argument_count();
    let wait: core::ffi::c_int = (({
        let flag = 'b' as i32 as u_char;
        args.argument_flag_count(flag)
    }) == 0) as core::ffi::c_int;
    let shellcmd = unsafe {
        format_single_from_target(
            item,
            args.argument_string(0).expect("argument count checked"),
        )
    };
    if ({
        let flag = 'F' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0 {
        if shellcmd
            .as_bytes()
            .first()
            .is_some_and(|&byte| byte != b'0')
        {
            unsafe {
                cmdlist = cmd_make_commands_now(self_0, item, 1 as u_int, 0 as core::ffi::c_int)
            };
        } else if count == 3 as u_int {
            unsafe {
                cmdlist = cmd_make_commands_now(self_0, item, 2 as u_int, 0 as core::ffi::c_int)
            };
        } else {
            return CMD_RETURN_NORMAL;
        }
        let Some(cmdlist) = cmdlist.as_ref() else {
            return CMD_RETURN_ERROR;
        };
        let item = cmdq_item_ref_of(item).expect("a running command item is on its queue");
        let state = item.state_ref();
        unsafe { item.insert_after(cmdlist.queue_items(Some(&state))) };
        return CMD_RETURN_NORMAL;
    }
    cdata.cmd_if = Some(cmd_make_commands_prepare(
        self_0,
        item,
        1 as u_int,
        None,
        wait,
        ::core::ffi::CStr::to_owned,
    ));
    if count == 3 as u_int {
        cdata.cmd_else = Some(cmd_make_commands_prepare(
            self_0,
            item,
            2 as u_int,
            None,
            wait,
            ::core::ffi::CStr::to_owned,
        ));
    }
    if wait != 0 {
        cdata.client_ref = item.client();
        cdata.item = cmdq_item_weak_of(item);
    } else {
        cdata.client_ref = target_client;
    }
    let mut cdata = cdata;
    let session = target_session.as_ref();
    let cwd = unsafe { client_working_directory(item.client().as_ref(), session) };
    if unsafe {
        job_run_for_session(
            Some(&shellcmd),
            &[],
            None,
            session,
            Some(cwd.as_c_str()),
            None,
            Some(Box::new(move |job| cmd_if_shell_callback(job, &mut cdata))),
            0 as core::ffi::c_int,
            -(1 as core::ffi::c_int),
            -(1 as core::ffi::c_int),
        )
        .is_none()
    } {
        unsafe { item.error(c"failed to run command: %s", fmt_args![shellcmd.as_c_str()]) };
        return CMD_RETURN_ERROR;
    }
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    CMD_RETURN_WAIT
}
pub unsafe fn cmd_if_shell_callback(job: JobEvent, cdata: &mut cmd_if_shell_data) {
    unsafe {
        let c = cdata.client_ref.as_mut();
        let item = cdata.item.as_ref().and_then(CmdqItemWeak::upgrade);

        let cmdlist: Option<CmdListRef>;
        let mut error = None;

        let status: core::ffi::c_int = job.status;
        let state = if !(status & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int)
            || (status & 0xff00 as core::ffi::c_int) >> 8 as core::ffi::c_int
                != 0 as core::ffi::c_int
        {
            cdata.cmd_else.as_deref_mut()
        } else {
            cdata.cmd_if.as_deref_mut()
        };
        if let Some(state) = state {
            cmdlist = args_make_commands(state, &[], &mut error);
            if error.is_some() {
                if item.is_none() {
                    if let Some(error) = error.as_mut() {
                        uppercase_first_byte(error);
                        status_message_for_client(
                            c,
                            -(1 as core::ffi::c_int),
                            1 as core::ffi::c_int,
                            0 as core::ffi::c_int,
                            0 as core::ffi::c_int,
                            c"%s",
                            fmt_args![error.as_c_str()],
                        );
                    }
                } else if let Some(item) = &item
                    && let Some(error) = error.as_ref()
                {
                    (item.read()).error(c"%s", fmt_args![error.as_c_str()]);
                }
            } else if let Some(item) = &item {
                let state = item.state_ref();
                item.insert_after((cmdlist.as_ref().unwrap()).queue_items(Some(&state)));
            } else {
                cmdq_append(c.as_deref(), (cmdlist.as_ref().unwrap()).queue_items(None));
            }
        }
        if let Some(item) = &item {
            item.resume();
        }
    }
}

fn uppercase_first_byte(error: &mut CString) {
    let mut bytes = error.as_bytes().to_vec();
    if let Some(first) = bytes.first_mut() {
        *first = toupper(*first);
    }
    *error = CString::new(bytes).expect("parser errors contain no NUL");
}
