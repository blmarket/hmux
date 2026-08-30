use crate::arguments::args_make_commands_now;
use crate::arguments::{
    args_count, args_has, args_make_commands, args_make_commands_prepare, args_string,
};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{
    CmdqItemWeak, cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command,
    cmdq_get_state_ref, cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
    cmdq_item_weak_from_ptr,
};
use crate::environ::environ_t;
use crate::ffi::__ctype_toupper_loc;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::job::{job_get_data, job_get_status, job_run};
use crate::server::{client_ref_from_ptr, server_client_get_cwd};
use crate::status::status_message_set;
pub use crate::types::*;
use ::std::ffi::CString;
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
pub struct cmd_if_shell_data {
    pub cmd_if: Option<Box<args_command_state>>,
    pub cmd_else: Option<Box<args_command_state>>,
    pub(crate) client_ref: Option<ClientRef>,
    pub(crate) item: Option<CmdqItemWeak>,
}

impl cmd_if_shell_data {
    /// The client the branch runs for, or null when there is none.
    pub(crate) fn client(&self) -> *mut client {
        self.client_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), ClientRef::as_ptr)
    }
}

impl Drop for cmd_if_shell_data {
    fn drop(&mut self) {
        drop(self.cmd_else.take());
        drop(self.cmd_if.take());
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
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub(crate) static cmd_if_shell_entry: cmd_entry = {
    cmd_entry {
        name: c"if-shell",
        alias: Some(c"if"),
        args: args_parse_t {
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
        exec: cmd_if_shell_exec,
    }
};
unsafe fn cmd_if_shell_args_parse(
    _args: &args,
    mut idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    if idx == 1 as u_int || idx == 2 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    ARGS_PARSE_STRING
}
unsafe fn cmd_if_shell_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut cdata = Box::new(cmd_if_shell_data {
            cmd_if: None,
            cmd_else: None,
            client_ref: None,
            item: None,
        });
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut s: *mut session = (*target).session();
        let mut cmdlist: Option<CmdListRef> = None;
        let mut count: u_int = args_count(args);
        let mut wait: ::core::ffi::c_int =
            (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
        let shellcmd = format_single_from_target(
            item,
            ::core::ffi::CStr::from_ptr(args_string(args, 0 as u_int)),
        );
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
            let Some(cmdlist) = cmdlist.as_ref() else {
                return CMD_RETURN_ERROR;
            };
            cmdq_insert_after(
                item,
                cmdq_get_command(cmdlist, Some(cmdq_get_state_ref(item))),
            );
            return CMD_RETURN_NORMAL;
        }
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
            cdata.client_ref = client_ref_from_ptr(cmdq_get_client(&*item));
            cdata.item = cmdq_item_weak_from_ptr(item);
        } else {
            cdata.client_ref = client_ref_from_ptr(tc);
        }
        if job_run(
            shellcmd.as_ptr(),
            &[],
            ::core::ptr::null_mut::<environ_t>(),
            s,
            server_client_get_cwd(cmdq_get_client(&*item), s),
            None,
            Some(cmd_if_shell_callback),
            Some(cmd_if_shell_free),
            JobData::IfShell(cdata),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        )
        .is_null()
        {
            cmdq_error(
                item,
                c"failed to run command: %s".as_ptr(),
                fmt_args![shellcmd.as_ptr()],
            );
            return CMD_RETURN_ERROR;
        }
        if wait == 0 {
            return CMD_RETURN_NORMAL;
        }
        CMD_RETURN_WAIT
    }
}
pub unsafe fn cmd_if_shell_callback(mut job: *mut job) {
    unsafe {
        let cdata: *mut cmd_if_shell_data = match job_get_data(job) {
            JobData::IfShell(data) => {
                data.as_ref() as *const cmd_if_shell_data as *mut cmd_if_shell_data
            }
            _ => panic!("if-shell job data is not if-shell data"),
        };
        let mut c: *mut client = (*cdata).client();
        let item = (*cdata).item.as_ref().and_then(CmdqItemWeak::upgrade);

        let mut cmdlist: Option<CmdListRef> = None;
        let mut error = None;
        let mut status: ::core::ffi::c_int = 0;
        status = job_get_status(job);
        let state = if !(status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int)
            || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            (*cdata).cmd_else.as_deref_mut()
        } else {
            (*cdata).cmd_if.as_deref_mut()
        };
        if let Some(state) = state {
            cmdlist = args_make_commands(state, &[], &mut error);
            if error.is_some() {
                if item.is_none() {
                    if let Some(error) = error.as_mut() {
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
                    cmdq_error(
                        item.as_ptr(),
                        c"%s".as_ptr(),
                        fmt_args![error.as_ref().unwrap().as_ptr()],
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
pub unsafe fn cmd_if_shell_free(data: JobData) {
    let JobData::IfShell(mut cdata) = data else {
        panic!("if-shell job data is not if-shell data");
    };
    let _ = cdata.client_ref.take();
}
