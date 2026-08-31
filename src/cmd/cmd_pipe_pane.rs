use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_get_target_client};
use crate::ffi::{
    __errno_location, _exit, close, closefrom, dup2, execl, fork, open, setpgid, sigfillset,
    sigprocmask, socketpair, strerror,
};
use crate::fmt_args;
use crate::format::{format_create, format_defaults, format_expand_time};
use crate::log::{fatalx, log_debug};
use crate::proc::proc_clear_signals;
use crate::reactor::Interest;
use crate::server::server_destroy_pane;
use crate::server::server_proc;
use crate::tmux::setblocking;
pub use crate::types::*;
use crate::window::{on_pane, on_pane_error};
use crate::window::{window_pane_destroy_ready, window_pane_exited};
pub const SOCK_NONBLOCK: __socket_type = 2048;
pub const SOCK_CLOEXEC: __socket_type = 524288;
pub const SOCK_PACKET: __socket_type = 10;
pub const SOCK_DCCP: __socket_type = 6;
pub const SOCK_SEQPACKET: __socket_type = 5;
pub const SOCK_RDM: __socket_type = 4;
pub const SOCK_RAW: __socket_type = 3;
pub const SOCK_DGRAM: __socket_type = 2;
pub const SOCK_STREAM: __socket_type = 1;
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
pub const PF_UNSPEC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;
pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const SIG_BLOCK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIG_SETMASK: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const _PATH_BSHELL: &::core::ffi::CStr = c"/bin/sh";
pub const _PATH_DEVNULL: &::core::ffi::CStr = c"/dev/null";
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub(crate) static cmd_pipe_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"pipe-pane",
        alias: Some(c"pipep"),
        args: args_parse_t {
            template: c"IOot:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-IOo] [-t target-pane] [shell-command]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_pipe_pane_exec,
    }
};
unsafe fn cmd_pipe_pane_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut wp: *mut window_pane = (*target).pane();
        let mut s: *mut session = (*target).session();
        let mut wl: *mut winlink = (*target).winlink();
        let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
        let mut old_fd: ::core::ffi::c_int = 0;
        let mut pipe_fd: [::core::ffi::c_int; 2] = [0; 2];
        let mut null_fd: ::core::ffi::c_int = 0;
        let mut in_0: ::core::ffi::c_int = 0;
        let mut out: ::core::ffi::c_int = 0;
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        if window_pane_exited(wp) != 0 {
            cmdq_error(item, c"target pane has exited".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        old_fd = (*wp).pipe_fd;
        if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
            (*wp).pipe_event.free();
            close((*wp).pipe_fd);
            (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
            if window_pane_destroy_ready(wp) != 0 {
                server_destroy_pane(wp, 1 as ::core::ffi::c_int);
                return CMD_RETURN_NORMAL;
            }
        }
        if args_count(args) == 0 as u_int
            || *args_string(args, 0 as u_int) as ::core::ffi::c_int == '\0' as i32
        {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'o' as i32 as u_char) != 0 && old_fd != -(1 as ::core::ffi::c_int) {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'I' as i32 as u_char) != 0 {
            in_0 = 1 as ::core::ffi::c_int;
            out = args_has(args, 'O' as i32 as u_char);
        } else {
            in_0 = 0 as ::core::ffi::c_int;
            out = 1 as ::core::ffi::c_int;
        }
        if socketpair(
            AF_UNIX,
            SOCK_STREAM as ::core::ffi::c_int,
            PF_UNSPEC,
            &raw mut pipe_fd as *mut ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            cmdq_error(
                item,
                c"socketpair error: %s".as_ptr(),
                fmt_args![strerror(*__errno_location())],
            );
            return CMD_RETURN_ERROR;
        }
        let mut ft = format_create(
            cmdq_get_client(&*item),
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        format_defaults(&mut ft, tc, s, wl, wp);
        let cmd = format_expand_time(
            &mut ft,
            ::core::ffi::CStr::from_ptr(args_string(args, 0 as u_int)),
        );
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        (*wp).pipe_pid = fork() as pid_t;
        match (*wp).pipe_pid {
            -1 => {
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    ::core::ptr::null_mut::<sigset_t>(),
                );
                cmdq_error(
                    item,
                    c"fork error: %s".as_ptr(),
                    fmt_args![strerror(*__errno_location())],
                );
                close(pipe_fd[0 as ::core::ffi::c_int as usize]);
                close(pipe_fd[1 as ::core::ffi::c_int as usize]);
                CMD_RETURN_ERROR
            }
            0 => {
                proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    ::core::ptr::null_mut::<sigset_t>(),
                );
                close(pipe_fd[0 as ::core::ffi::c_int as usize]);
                if setpgid(0 as __pid_t, 0 as __pid_t) == -(1 as ::core::ffi::c_int) {
                    _exit(1 as ::core::ffi::c_int);
                }
                null_fd = open(_PATH_DEVNULL.as_ptr(), O_WRONLY);
                if out != 0 {
                    if dup2(pipe_fd[1 as ::core::ffi::c_int as usize], STDIN_FILENO)
                        == -(1 as ::core::ffi::c_int)
                    {
                        _exit(1 as ::core::ffi::c_int);
                    }
                } else if dup2(null_fd, STDIN_FILENO) == -(1 as ::core::ffi::c_int) {
                    _exit(1 as ::core::ffi::c_int);
                }
                if in_0 != 0 {
                    if dup2(pipe_fd[1 as ::core::ffi::c_int as usize], STDOUT_FILENO)
                        == -(1 as ::core::ffi::c_int)
                    {
                        _exit(1 as ::core::ffi::c_int);
                    }
                    if pipe_fd[1 as ::core::ffi::c_int as usize] != STDOUT_FILENO {
                        close(pipe_fd[1 as ::core::ffi::c_int as usize]);
                    }
                } else if dup2(null_fd, STDOUT_FILENO) == -(1 as ::core::ffi::c_int) {
                    _exit(1 as ::core::ffi::c_int);
                }
                if dup2(null_fd, STDERR_FILENO) == -(1 as ::core::ffi::c_int) {
                    _exit(1 as ::core::ffi::c_int);
                }
                closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
                execl(
                    _PATH_BSHELL.as_ptr(),
                    c"sh".as_ptr(),
                    c"-c".as_ptr(),
                    cmd.as_ptr(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                _exit(1 as ::core::ffi::c_int);
            }
            _ => {
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    ::core::ptr::null_mut::<sigset_t>(),
                );
                close(pipe_fd[1 as ::core::ffi::c_int as usize]);
                (*wp).pipe_fd = pipe_fd[0 as ::core::ffi::c_int as usize];
                *wpo = (*wp).offset;
                setblocking((*wp).pipe_fd, 0 as ::core::ffi::c_int);
                let id = (*wp).id;
                (*wp).pipe_event = Stream::new(
                    (*wp).pipe_fd,
                    Some(on_pane(id, |wp| cmd_pipe_pane_read_callback(wp))),
                    Some(on_pane(id, |wp| cmd_pipe_pane_write_callback(wp))),
                    Some(on_pane_error(id, |wp| cmd_pipe_pane_error_callback(wp))),
                );
                if (*wp).pipe_event.is_none() {
                    fatalx(c"out of memory".as_ptr(), fmt_args![]);
                }
                if out != 0 {
                    (*wp).pipe_event.enable(Interest::Write);
                }
                if in_0 != 0 {
                    (*wp).pipe_event.enable(Interest::Read);
                }
                CMD_RETURN_NORMAL
            }
        }
    }
}
unsafe fn cmd_pipe_pane_read_callback(mut wp: *mut window_pane) {
    unsafe {
        let data = (*wp)
            .pipe_event
            .with_input(|buffer| buffer.copy_to_bytes(buffer.len()))
            .unwrap_or_default();
        let available = data.len();
        log_debug(
            c"%%%u pipe read %zu".as_ptr(),
            fmt_args![(*wp).id, available],
        );
        (*wp).event.write(data.as_ptr(), available);
        if window_pane_destroy_ready(wp) != 0 {
            server_destroy_pane(wp, 1 as ::core::ffi::c_int);
        }
    }
}
unsafe fn cmd_pipe_pane_write_callback(mut wp: *mut window_pane) {
    unsafe {
        log_debug(c"%%%u pipe empty".as_ptr(), fmt_args![(*wp).id]);
        if window_pane_destroy_ready(wp) != 0 {
            server_destroy_pane(wp, 1 as ::core::ffi::c_int);
        }
    }
}
unsafe fn cmd_pipe_pane_error_callback(mut wp: *mut window_pane) {
    unsafe {
        log_debug(c"%%%u pipe error".as_ptr(), fmt_args![(*wp).id]);
        (*wp).pipe_event.free();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
        if window_pane_destroy_ready(wp) != 0 {
            server_destroy_pane(wp, 1 as ::core::ffi::c_int);
        }
    }
}
