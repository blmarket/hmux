use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_get_target_client,
};
use crate::src::ffi::libc::{
    __errno_location, _exit, close, closefrom, dup2, execl, fork, memcpy, open, setpgid,
    sigfillset, sigprocmask, socketpair, strerror,
};
use crate::src::format::{format_create, format_defaults, format_expand_time_cstring, format_free};
use crate::src::log::{fatalx, log_debug};
use crate::src::proc::proc_clear_signals;
use crate::src::reactor::{
    bufferevent_enable, bufferevent_free, bufferevent_new, bufferevent_write, evbuffer_drain,
    evbuffer_get_length, evbuffer_pullup,
};
use crate::src::server::server_proc;
use crate::src::server_fn::server_destroy_pane;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_WRITE};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::window_pane_offset;
use crate::src::shared::posix_io::{
    _PATH_BSHELL, _PATH_DEVNULL, O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
use crate::src::shared::session::session;
use crate::src::shared::signal::{__sigset_t, sigset_t, SIG_BLOCK, SIG_SETMASK};
use crate::src::shared::socket::{AF_UNIX, PF_UNSPEC, SOCK_STREAM};
use crate::src::shared::window::winlink;
use crate::src::tmux::setblocking;
use crate::src::window::{window_pane_destroy_ready, window_pane_exited};
pub static mut cmd_pipe_pane_entry: cmd_entry =  {
    cmd_entry {
        name: c"pipe-pane",
        alias: Some(c"pipep"),
        args: args_parse {
            template: b"IOot:\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(cmd_pipe_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_pipe_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let mut old_fd: ::core::ffi::c_int = 0;
    let mut pipe_fd: [::core::ffi::c_int; 2] = [0; 2];
    let mut null_fd: ::core::ffi::c_int = 0;
    let mut in_0: ::core::ffi::c_int = 0;
    let mut out: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    if window_pane_exited(wp) != 0 {
        cmdq_error(
            item,
            b"target pane has exited\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    old_fd = (*wp).pipe_fd;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
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
            b"socketpair error: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        return CMD_RETURN_ERROR;
    }
    ft = format_create(
        cmdq_get_client(item),
        item,
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, tc, s, wl, wp);
    let cmd = format_expand_time_cstring(ft, args_string(args, 0 as u_int));
    format_free(ft);
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
                b"fork error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
            close(pipe_fd[0 as ::core::ffi::c_int as usize]);
            close(pipe_fd[1 as ::core::ffi::c_int as usize]);
            return CMD_RETURN_ERROR;
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
                b"sh\0" as *const u8 as *const ::core::ffi::c_char,
                b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                cmd.as_ptr(),
                NULL as *mut ::core::ffi::c_char,
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
            memcpy(
                wpo as *mut ::core::ffi::c_void,
                &raw mut (*wp).offset as *const ::core::ffi::c_void,
                ::core::mem::size_of::<window_pane_offset>() as size_t,
            );
            setblocking((*wp).pipe_fd, 0 as ::core::ffi::c_int);
            (*wp).pipe_event = bufferevent_new(
                (*wp).pipe_fd,
                bufferevent_data_callback(move |stream| unsafe {
                    cmd_pipe_pane_read_callback(stream.as_ptr(), wp as *mut ::core::ffi::c_void)
                }),
                bufferevent_data_callback(move |stream| unsafe {
                    cmd_pipe_pane_write_callback(stream.as_ptr(), wp as *mut ::core::ffi::c_void)
                }),
                bufferevent_event_callback(move |stream, flags| unsafe {
                    cmd_pipe_pane_error_callback(
                        stream.as_ptr(),
                        flags,
                        wp as *mut ::core::ffi::c_void,
                    )
                }),
            );
            if (*wp).pipe_event.is_null() {
                fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
            }
            if out != 0 {
                bufferevent_enable((*wp).pipe_event, EV_WRITE as ::core::ffi::c_short);
            }
            if in_0 != 0 {
                bufferevent_enable((*wp).pipe_event, EV_READ as ::core::ffi::c_short);
            }
            return CMD_RETURN_NORMAL;
        }
    };
}
unsafe fn cmd_pipe_pane_read_callback(
    _bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut available: size_t = 0;
    if (*wp).pipe_event.is_null() {
        return;
    }
    evb = (*(*wp).pipe_event).input;
    available = evbuffer_get_length(&*(evb));
    log_debug(
        b"%%%u pipe read %zu\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        available,
    );
    bufferevent_write(
        (*wp).event,
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *const ::core::ffi::c_void,
        available,
    );
    evbuffer_drain(evb, available);
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
unsafe fn cmd_pipe_pane_write_callback(
    _bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%%%u pipe empty\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
unsafe fn cmd_pipe_pane_error_callback(
    _bufev: *mut bufferevent,
    _what: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%%%u pipe error\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    bufferevent_free((*wp).pipe_event);
    close((*wp).pipe_fd);
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
