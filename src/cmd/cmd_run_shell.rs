use crate::arguments::{
    args_count, args_get_str, args_has, args_make_commands, args_make_commands_prepare,
    args_string_str,
};
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_from_nothing;
use crate::cmdq::{CmdqItemWeak, cmdq_append, cmdq_item_weak_of};
use crate::compat::toupper;
pub use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_STRING, CMD_FIND_CANFAIL, CMD_FIND_PANE,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT, JOB_NOWAIT, JOB_SHOWSTDERR,
};
use crate::fmt_args;
use crate::format::{format_add, format_create_from_target, format_expand};
use crate::job::job_run_for_session;
use crate::reactor::Timer;
use crate::server::client_working_directory;
use crate::status::status_message_for_client;
pub use crate::cmd::{CmdListRef, RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
pub use crate::cmdq::cmdq_item;
pub use crate::types::{
    __suseconds_t, __time_t, ClientRef, JobEvent, SessionRef, Stream, TimerHandle, WindowPane,
    args, args_command_state, args_parse_t, args_parse_type, cmd_find_state, time_t, timeval,
    u_char, u_int,
};
#[cfg(test)]
use crate::types::WindowMode;
use crate::window::window_pane_find_by_id;
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::cell::RefCell;
use ::std::ffi::CString;
use ::std::rc::Rc;

#[repr(C)]
pub struct cmd_run_shell_data {
    pub(crate) client_ref: Option<ClientRef>,
    pub cmd: Option<CString>,
    pub state: Option<Box<args_command_state>>,
    pub cwd: Option<CString>,
    pub(crate) item: Option<CmdqItemWeak>,
    pub(crate) session_ref: Option<SessionRef>,
    pub wp_id: core::ffi::c_int,
    pub timer: TimerHandle,
    pub flags: core::ffi::c_int,
}

impl Drop for cmd_run_shell_data {
    fn drop(&mut self) {
        self.timer.disarm();
        drop(self.state.take());
    }
}

pub(crate) static cmd_run_shell_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"run-shell",
        alias: Some(c"run"),
        args: args_parse_t {
            template: c"bd:Ct:Es:c:",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: Some(
                cmd_run_shell_args_parse,
            ),
        },
        usage: c"[-bCE] [-c start-directory] [-d delay] [-t target-pane] [shell-command [argument ...]]",
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
        exec: cmd_run_shell_exec,
    }
};
fn cmd_run_shell_args_parse(
    args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    if args_has(args, 'C' as i32 as u_char) != 0 {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    ARGS_PARSE_STRING
}
unsafe fn cmd_run_shell_print(cdata: &cmd_run_shell_data, msg: &CStr) {
    unsafe {
        let mut pane = if cdata.wp_id != -1 {
            window_pane_find_by_id(cdata.wp_id as u_int)
        } else {
            None
        };
        if pane.is_none() {
            if let Some(asked) = cdata.item.as_ref().and_then(CmdqItemWeak::upgrade) {
                (asked.read()).print(c"%s", fmt_args![msg]);
                return;
            }
            let mut fs = cmd_find_state::default();
            if cmd_find_from_nothing(&mut fs, 0) == 0 {
                pane = fs.pane_ref();
            }
        }
        let Some(pane) = pane else { return };
        pane.append_view_line(msg);
    }
}
unsafe fn cmd_run_shell_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let mut cdata = Box::new(cmd_run_shell_data {
        client_ref: None,
        cmd: None,
        state: None,
        cwd: None,
        item: None,
        session_ref: None,
        wp_id: -1,
        timer: TimerHandle(0),
        flags: 0,
    });
    let cmdq_client = item.client();
    let c = cmdq_client.clone();
    let target_client = item.target_client();
    let target_session = item.target.session();
    let target_pane_id = item.target.pane_ref().map(|pane| pane.id());
    let mut d: core::ffi::c_double = 0.;
    let mut tv = timeval::default();
    let mut i: u_int;
    let wait: core::ffi::c_int = (args_has(args, 'b' as i32 as u_char) == 0) as core::ffi::c_int;
    let delay = args_get_str(args, 'd' as i32 as u_char);
    if let Some(delay) = delay {
        let Some(parsed) = crate::compat::strtod_complete(delay) else {
            unsafe { item.error(c"invalid delay time: %s", fmt_args![delay]) };
            return CMD_RETURN_ERROR;
        };
        d = parsed;
    } else if args_count(args) == 0 as u_int {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'C' as i32 as u_char) == 0 {
        let cmd = unsafe { args_string_str(args, 0) };
        if let Some(cmd) = cmd {
            let mut ft = unsafe { format_create_from_target(item) };
            i = 1 as u_int;
            while i < args_count(args) {
                let key = xasprintf(c"%u", fmt_args![i]);
                unsafe {
                    format_add(
                        &mut ft,
                        &key,
                        c"%s",
                        fmt_args![args_string_str(args, i).expect("argument index checked")],
                    )
                };
                i = i.wrapping_add(1);
            }
            unsafe { cdata.cmd = Some(format_expand(&mut ft, cmd)) };
        }
    } else {
        cdata.state = Some(args_make_commands_prepare(
            self_0,
            item,
            0 as u_int,
            None,
            wait,
            |cmd| unsafe { crate::format::format_single_from_target(item, cmd) },
        ));
    }
    if args_has(args, 't' as i32 as u_char) != 0 {
        cdata.wp_id = target_pane_id.map_or(-1, |id| id as core::ffi::c_int);
    } else {
        cdata.wp_id = -(1 as core::ffi::c_int);
    }
    if wait != 0 {
        cdata.client_ref = cmdq_client;
        cdata.item = cmdq_item_weak_of(item);
    } else {
        cdata.client_ref = target_client;
        cdata.flags |= JOB_NOWAIT;
    }
    if let Some(cflag) = args_get_str(args, 'c' as i32 as u_char) {
        cdata.cwd = Some(cflag.to_owned());
    } else {
        unsafe {
            cdata.cwd = Some(client_working_directory(
                c.as_ref(),
                target_session.as_ref(),
            ))
        };
    }
    if args_has(args, 'E' as i32 as u_char) != 0 {
        cdata.flags |= JOB_SHOWSTDERR;
    }
    cdata.session_ref = target_session;
    let cdata = Rc::new(RefCell::new(Some(cdata)));
    let callback_data = Rc::clone(&cdata);
    cdata
        .borrow_mut()
        .as_mut()
        .expect("run-shell data is present")
        .timer
        .set_callback(move || {
            let data = callback_data.borrow_mut().take();
            if let Some(data) = data {
                unsafe { cmd_run_shell_timer(data) };
            }
        });
    if delay.is_some() {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = d as time_t as __time_t;
        tv.tv_usec = ((d - tv.tv_sec as core::ffi::c_double)
            * 1000000 as core::ffi::c_uint as core::ffi::c_double)
            as __suseconds_t;
        cdata
            .borrow_mut()
            .as_mut()
            .expect("run-shell data is present")
            .timer
            .arm(tv);
    } else {
        cdata
            .borrow_mut()
            .as_mut()
            .expect("run-shell data is present")
            .timer
            .arm(timeval::from_secs(0));
    }
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    CMD_RETURN_WAIT
}
unsafe fn cmd_run_shell_timer(mut cdata: Box<cmd_run_shell_data>) {
    unsafe {
        let mut c_opt = cdata.client_ref.clone();
        let cmd = cdata.cmd.clone();
        let cmd_for_error = cmd.clone();
        let item = cdata.item.as_ref().and_then(CmdqItemWeak::upgrade);

        let mut error = None;
        if cdata.state.is_none() {
            if cmd.is_none() {
                if let Some(item) = &item {
                    item.resume();
                }
                return;
            }
            let state_opt = cdata.session_ref.clone();
            let cwd = cdata.cwd.clone();
            let flags = cdata.flags;
            if job_run_for_session(
                cmd.as_deref(),
                &[],
                None,
                state_opt.as_ref(),
                cwd.as_deref(),
                None,
                Some(Box::new(move |job| cmd_run_shell_callback(job, &mut cdata))),
                flags,
                -(1 as core::ffi::c_int),
                -(1 as core::ffi::c_int),
            )
            .is_none()
            {
                if let Some(item) = &item {
                    (item.read()).error(
                        c"failed to run command: %s",
                        fmt_args![cmd_for_error.as_deref()],
                    );
                    item.resume();
                } else {
                    status_message_for_client(
                        c_opt.as_mut(),
                        -(1 as core::ffi::c_int),
                        1 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        c"failed to run command: %s",
                        fmt_args![cmd_for_error.as_deref()],
                    );
                }
            }
            return;
        }
        let cmdlist: Option<CmdListRef> =
            args_make_commands(cdata.state.as_deref_mut().unwrap(), &[], &mut error);
        if error.is_some() {
            if let Some(item) = &item
                && let Some(error) = error.as_ref()
            {
                (item.read()).error(c"%s", fmt_args![error.as_c_str()]);
            } else if let Some(error) = error.as_mut() {
                uppercase_first_byte(error);
                status_message_for_client(
                    c_opt.as_mut(),
                    -(1 as core::ffi::c_int),
                    1 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    c"%s",
                    fmt_args![error.as_c_str()],
                );
            }
        } else if let Some(item) = &item {
            let state = item.state_ref();
            item.insert_after((cmdlist.as_ref().unwrap()).queue_items(Some(&state)));
        } else {
            cmdq_append(
                c_opt.as_ref(),
                (cmdlist.as_ref().unwrap()).queue_items(None),
            );
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
unsafe fn cmd_run_shell_callback(job: JobEvent, cdata: &mut cmd_run_shell_data) {
    unsafe {
        let event: Stream = job.event;
        let item = cdata.item.as_ref().and_then(CmdqItemWeak::upgrade);
        let mut msg: Option<CString> = None;
        let mut retcode: core::ffi::c_int;

        loop {
            let Some(line) = event.with_input(|buffer| buffer.read_line()).flatten() else {
                break;
            };
            let mut line_data = line.to_vec();
            line_data.push(0);
            let line =
                CStr::from_bytes_until_nul(&line_data).expect("a NUL was pushed on the line above");
            cmd_run_shell_print(cdata, line);
        }
        let remainder = event
            .with_input(|buffer| buffer.as_slice().to_vec())
            .unwrap_or_default();
        if !remainder.is_empty() {
            let mut line = remainder;
            line.push(0);
            let line =
                CStr::from_bytes_until_nul(&line).expect("a NUL was pushed on the line above");
            cmd_run_shell_print(cdata, line);
        }
        let cmd = cdata.cmd.as_deref();
        let status: core::ffi::c_int = job.status;
        if status & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int {
            retcode = (status & 0xff00 as core::ffi::c_int) >> 8 as core::ffi::c_int;
            if retcode != 0 as core::ffi::c_int {
                msg = Some(xasprintf(c"'%s' returned %d", fmt_args![cmd, retcode]));
            }
        } else if ((status & 0x7f as core::ffi::c_int) + 1 as core::ffi::c_int)
            as core::ffi::c_schar as core::ffi::c_int
            >> 1 as core::ffi::c_int
            > 0 as core::ffi::c_int
        {
            retcode = status & 0x7f as core::ffi::c_int;
            msg = Some(xasprintf(
                c"'%s' terminated by signal %d",
                fmt_args![cmd, retcode],
            ));
            retcode += 128 as core::ffi::c_int;
        } else {
            retcode = 0 as core::ffi::c_int;
        }
        if let Some(msg) = &msg {
            cmd_run_shell_print(cdata, msg);
        }
        if let Some(item) = &item {
            if let Some(asked) = item.client()
                && asked.attached_session().is_none()
            {
                asked.set_return_code(retcode);
            }
            item.resume();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::format_grid_line;
    use crate::pane_identity::PaneIdentity;
    use crate::tests::test_fixtures::{Target, globals};

    fn output_state(pane_id: core::ffi::c_int) -> cmd_run_shell_data {
        cmd_run_shell_data {
            client_ref: None,
            cmd: None,
            state: None,
            cwd: None,
            item: None,
            session_ref: None,
            wp_id: pane_id,
            timer: TimerHandle(0),
            flags: 0,
        }
    }

    #[test]
    fn output_reuses_view_mode_and_resolves_a_missing_target_again() {
        let _guard = globals();
        let mut target = Target::new(20, 5);
        unsafe {
            let pane = window_pane_find_by_id((*target.pane(0)).pane_id()).unwrap();
            let mut data = output_state(pane.id() as core::ffi::c_int);
            cmd_run_shell_print(&data, c"first");
            cmd_run_shell_print(&data, c"second");
            data.wp_id = 9999;
            cmd_run_shell_print(&data, c"fallback");
            let wp = pane.get().unwrap();
            assert_eq!(wp.modes().len(), 1);
            let mode = wp.modes().first().unwrap();
            assert_eq!(mode.mode(), WindowMode::View);
            let data = mode.state.copy_mode_data_ref().unwrap();
            let grid = crate::screen::RustScreen::grid(data.backing.as_deref().unwrap());
            for (row, expected) in [c"first", c"second", c"fallback"].into_iter().enumerate() {
                assert_eq!(format_grid_line(grid, row as u_int), expected);
            }
        }
    }

    #[test]
    fn output_without_a_live_target_or_queue_item_is_ignored() {
        let _guard = globals();
        unsafe { cmd_run_shell_print(&output_state(9999), c"unused") };
    }
}
