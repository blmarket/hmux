use crate::arguments::{
    args_count, args_get, args_has, args_make_commands, args_make_commands_prepare, args_string,
};
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_from_nothing;
use crate::cmd::queue::{
    CmdqItemWeak, cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command,
    cmdq_get_state_ref, cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
    cmdq_item_weak_from_ptr, cmdq_print,
};
use crate::environ::environ_t;
use crate::ffi::{__ctype_toupper_loc, strtod};
use crate::fmt_args;
use crate::format::{format_add, format_create_from_target, format_expand};
use crate::job::{job_get_data, job_get_event, job_get_status, job_run};
use crate::modes::window_copy_add;
use crate::reactor::Timer;
use crate::server::{client_ref_from_ptr, server_client_get_cwd, server_client_get_pane};
use crate::session::session_ref_from_ptr;
use crate::status::status_message_set;
pub use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::{window_pane_find_by_id, window_pane_set_mode};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
pub const BUFFER_EOL_NUL: ::core::ffi::c_uint = 4;
pub const BUFFER_EOL_LF: ::core::ffi::c_uint = 3;
pub const BUFFER_EOL_CRLF_STRICT: ::core::ffi::c_uint = 2;
pub const BUFFER_EOL_CRLF: ::core::ffi::c_uint = 1;
pub const BUFFER_EOL_ANY: ::core::ffi::c_uint = 0;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
#[repr(C)]
pub struct cmd_run_shell_data {
    pub(crate) client_ref: Option<ClientRef>,
    pub cmd: Option<::std::ffi::CString>,
    pub state: Option<Box<args_command_state>>,
    pub cwd: Option<::std::ffi::CString>,
    pub(crate) item: Option<CmdqItemWeak>,
    pub(crate) session_ref: Option<SessionRef>,
    pub wp_id: ::core::ffi::c_int,
    pub timer: TimerHandle,
    pub flags: ::core::ffi::c_int,
}

impl cmd_run_shell_data {
    /// The client the shell command runs for, or null when there is none.
    pub(crate) fn client(&self) -> *mut client {
        self.client_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), ClientRef::as_ptr)
    }

    /// The session the shell command runs in, or null when there is none.
    pub(crate) fn session(&self) -> *mut session {
        self.session_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), SessionRef::as_ptr)
    }
}

impl Drop for cmd_run_shell_data {
    fn drop(&mut self) {
        self.timer.disarm();
        drop(self.state.take());
    }
}
#[inline]
fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
            *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
        } else {
            __c
        }
    }
}
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const JOB_SHOWSTDERR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub(crate) static cmd_run_shell_entry: cmd_entry = {
    cmd_entry {
        name: c"run-shell",
        alias: Some(c"run"),
        args: args_parse_t {
            template: c"bd:Ct:Es:c:",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
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
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_run_shell_exec,
    }
};
unsafe fn cmd_run_shell_args_parse(
    args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    unsafe {
        if args_has(args, 'C' as i32 as u_char) != 0 {
            return ARGS_PARSE_COMMANDS_OR_STRING;
        }
        ARGS_PARSE_STRING
    }
}
unsafe fn cmd_run_shell_print(mut job: *mut job, mut msg: *const ::core::ffi::c_char) {
    unsafe {
        let cdata: *mut cmd_run_shell_data = match job_get_data(job) {
            JobData::RunShell(data) => {
                data.as_ref() as *const cmd_run_shell_data as *mut cmd_run_shell_data
            }
            _ => panic!("run-shell job data is not run-shell data"),
        };
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut fs = cmd_find_state::default();
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        if (*cdata).wp_id != -(1 as ::core::ffi::c_int) {
            wp = window_pane_find_by_id((*cdata).wp_id as u_int);
        }
        if wp.is_null() {
            let asked = (*cdata).item.as_ref().and_then(CmdqItemWeak::upgrade);
            if let Some(asked) = &asked {
                cmdq_print(asked.as_ptr(), c"%s".as_ptr(), fmt_args![msg]);
                return;
            }
            if asked.is_some() && !(*cdata).client().is_null() {
                wp = server_client_get_pane((*cdata).client());
            }
            if wp.is_null()
                && cmd_find_from_nothing(&mut fs, 0 as ::core::ffi::c_int)
                    == 0 as ::core::ffi::c_int
            {
                wp = fs.pane();
            }
            if wp.is_null() {
                return;
            }
        }
        wme = window_pane_current_mode(wp);
        if wme.is_null() || (*wme).mode() != WindowMode::View {
            window_pane_set_mode(
                wp,
                ::core::ptr::null_mut::<window_pane>(),
                WindowMode::View,
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<args>(),
            );
        }
        window_copy_add(wp, 1 as ::core::ffi::c_int, c"%s".as_ptr(), fmt_args![msg]);
    }
}
unsafe fn cmd_run_shell_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
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
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut s: *mut session = (*target).session();
        let mut wp: *mut window_pane = (*target).pane();
        let mut delay: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut d: ::core::ffi::c_double = 0.;
        let mut tv = timeval::default();
        let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: u_int = 0;
        let mut wait: ::core::ffi::c_int =
            (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
        delay = args_get(args, 'd' as i32 as u_char);
        if !delay.is_null() {
            d = strtod(delay, &raw mut end);
            if *end as ::core::ffi::c_int != '\0' as i32 {
                cmdq_error(item, c"invalid delay time: %s".as_ptr(), fmt_args![delay]);
                return CMD_RETURN_ERROR;
            }
        } else if args_count(args) == 0 as u_int {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'C' as i32 as u_char) == 0 {
            cmd = args_string(args, 0 as u_int);
            if !cmd.is_null() {
                let mut ft = format_create_from_target(item);
                i = 1 as u_int;
                while i < args_count(args) {
                    let key = xasprintf(c"%u".as_ptr(), fmt_args![i]);
                    format_add(
                        &mut ft,
                        &key,
                        c"%s".as_ptr(),
                        fmt_args![args_string(args, i)],
                    );
                    i = i.wrapping_add(1);
                }
                cdata.cmd = Some(format_expand(&mut ft, CStr::from_ptr(cmd)));
            }
        } else {
            cdata.state = Some(args_make_commands_prepare(
                self_0,
                item,
                0 as u_int,
                ::core::ptr::null::<::core::ffi::c_char>(),
                wait,
                1 as ::core::ffi::c_int,
            ));
        }
        if args_has(args, 't' as i32 as u_char) != 0 && !wp.is_null() {
            cdata.wp_id = (*wp).id as ::core::ffi::c_int;
        } else {
            cdata.wp_id = -(1 as ::core::ffi::c_int);
        }
        if wait != 0 {
            cdata.client_ref = client_ref_from_ptr(c);
            cdata.item = cmdq_item_weak_from_ptr(item);
        } else {
            cdata.client_ref = client_ref_from_ptr(tc);
            cdata.flags |= JOB_NOWAIT;
        }
        if args_has(args, 'c' as i32 as u_char) != 0 {
            cdata.cwd = Some(CStr::from_ptr(args_get(args, 'c' as i32 as u_char)).to_owned());
        } else {
            cdata.cwd = Some(CStr::from_ptr(server_client_get_cwd(c, s)).to_owned());
        }
        if args_has(args, 'E' as i32 as u_char) != 0 {
            cdata.flags |= JOB_SHOWSTDERR;
        }
        cdata.session_ref = session_ref_from_ptr(s);
        let cdata = Box::into_raw(cdata);
        (*cdata)
            .timer
            .set_callback(move || cmd_run_shell_timer(cdata));
        if !delay.is_null() {
            tv.tv_usec = 0 as __suseconds_t;
            tv.tv_sec = tv.tv_usec as __time_t;
            tv.tv_sec = d as time_t as __time_t;
            tv.tv_usec = ((d - tv.tv_sec as ::core::ffi::c_double)
                * 1000000 as ::core::ffi::c_uint as ::core::ffi::c_double)
                as __suseconds_t;
            (*cdata).timer.arm(tv);
        } else {
            (*cdata).timer.arm(timeval::from_secs(0));
        }
        if wait == 0 {
            return CMD_RETURN_NORMAL;
        }
        CMD_RETURN_WAIT
    }
}
unsafe fn cmd_run_shell_timer(data: *mut cmd_run_shell_data) {
    unsafe {
        let mut cdata = Box::from_raw(data);
        let c: *mut client = cdata.client();
        let cmd: *const ::core::ffi::c_char = cstr_ptr(&cdata.cmd);
        let cmd_for_error = cdata.cmd.clone();
        let item = cdata.item.as_ref().and_then(CmdqItemWeak::upgrade);
        let mut cmdlist: Option<CmdListRef> = None;
        let mut error = None;
        if cdata.state.is_none() {
            if cmd.is_null() {
                if let Some(item) = &item {
                    cmdq_continue(item.as_ptr());
                }
                return;
            }
            let state = cdata.session();
            let cwd = cstr_ptr(&cdata.cwd);
            let flags = cdata.flags;
            if job_run(
                cmd,
                &[],
                ::core::ptr::null_mut::<environ_t>(),
                state,
                cwd,
                None,
                Some(cmd_run_shell_callback),
                Some(cmd_run_shell_free),
                JobData::RunShell(cdata),
                flags,
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            )
            .is_null()
            {
                if let Some(item) = &item {
                    cmdq_error(
                        item.as_ptr(),
                        c"failed to run command: %s".as_ptr(),
                        fmt_args![cstr_ptr(&cmd_for_error)],
                    );
                    cmdq_continue(item.as_ptr());
                } else {
                    status_message_set(
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        c"failed to run command: %s".as_ptr(),
                        fmt_args![cstr_ptr(&cmd_for_error)],
                    );
                }
            }
            return;
        }
        cmdlist = args_make_commands(cdata.state.as_deref_mut().unwrap(), &[], &mut error);
        if error.is_some() {
            if let Some(item) = &item {
                cmdq_error(
                    item.as_ptr(),
                    c"%s".as_ptr(),
                    fmt_args![error.as_ref().unwrap().as_ptr()],
                );
            } else if let Some(error) = error.as_mut() {
                uppercase_first_byte(error);
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    c"%s".as_ptr(),
                    fmt_args![error.as_ptr()],
                );
            }
        } else if let Some(item) = &item {
            cmdq_insert_after(
                item.as_ptr(),
                cmdq_get_command(
                    cmdlist.as_ref().unwrap(),
                    Some(cmdq_get_state_ref(item.as_ptr())),
                ),
            );
        } else {
            cmdq_append(c, cmdq_get_command(cmdlist.as_ref().unwrap(), None));
        }
        if let Some(item) = &item {
            cmdq_continue(item.as_ptr());
        }
    }
}

fn uppercase_first_byte(error: &mut CString) {
    let mut bytes = error.as_bytes().to_vec();
    if let Some(first) = bytes.first_mut() {
        *first = toupper(*first as u_char as ::core::ffi::c_int) as u8;
    }
    *error = CString::new(bytes).expect("parser errors contain no NUL");
}
unsafe fn cmd_run_shell_callback(mut job: *mut job) {
    unsafe {
        let cdata: *mut cmd_run_shell_data = match job_get_data(job) {
            JobData::RunShell(data) => {
                data.as_ref() as *const cmd_run_shell_data as *mut cmd_run_shell_data
            }
            _ => panic!("run-shell job data is not run-shell data"),
        };
        let mut event: Stream = job_get_event(job);
        let item = (*cdata).item.as_ref().and_then(CmdqItemWeak::upgrade);
        let mut cmd: *mut ::core::ffi::c_char = cstr_ptr(&(*cdata).cmd);
        let mut msg: Option<::std::ffi::CString> = None;
        let mut retcode: ::core::ffi::c_int = 0;
        let mut status: ::core::ffi::c_int = 0;
        loop {
            let Some(line) = event.with_input(|buffer| buffer.read_line()).flatten() else {
                break;
            };
            let mut line_data = line.to_vec();
            line_data.push(0);
            cmd_run_shell_print(job, line_data.as_ptr() as *const ::core::ffi::c_char);
        }
        let remainder = event
            .with_input(|buffer| buffer.as_slice().to_vec())
            .unwrap_or_default();
        if !remainder.is_empty() {
            let mut line = remainder;
            line.push(0);
            cmd_run_shell_print(job, line.as_ptr() as *const ::core::ffi::c_char);
        }
        status = job_get_status(job);
        if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            retcode = (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
            if retcode != 0 as ::core::ffi::c_int {
                msg = Some(xasprintf(
                    c"'%s' returned %d".as_ptr(),
                    fmt_args![cmd, retcode],
                ));
            }
        } else if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_schar as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            retcode = status & 0x7f as ::core::ffi::c_int;
            msg = Some(xasprintf(
                c"'%s' terminated by signal %d".as_ptr(),
                fmt_args![cmd, retcode],
            ));
            retcode += 128 as ::core::ffi::c_int;
        } else {
            retcode = 0 as ::core::ffi::c_int;
        }
        if let Some(msg) = &msg {
            cmd_run_shell_print(job, msg.as_ptr());
        }
        if let Some(item) = &item {
            let asked = cmdq_get_client(&*item.as_ptr());
            if !asked.is_null() && (*asked).session.is_null() {
                (*asked).retval = retcode;
            }
            cmdq_continue(item.as_ptr());
        }
    }
}
unsafe fn cmd_run_shell_free(data: JobData) {
    let JobData::RunShell(mut cdata) = data else {
        panic!("run-shell job data is not run-shell data");
    };
    cdata.timer.disarm();
    let _ = cdata.session_ref.take();
    let _ = cdata.client_ref.take();
}
