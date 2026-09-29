use crate::src::reactor::BufferEvent;
use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_get_target_client,
};
use crate::src::ffi::libc::{
    __errno_location, _exit, close, closefrom, dup2, execl, fork, memcpy, open, setpgid,
    sigfillset, sigprocmask, socketpair, strerror,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_create_with_client, format_defaults, format_expand_time_cstring, format_free};
use crate::src::log::{fatalx, log_debug};
use crate::src::proc::proc_clear_signals;
use crate::src::reactor::{
    bufferevent_get_input,
    bufferevent_enable, bufferevent_new, bufferevent_write, evbuffer_drain,
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
pub static cmd_pipe_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"pipe-pane",
        alias: Some(c"pipep"),
        args: args_parse {
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
        exec: Some(cmd_pipe_pane_exec),
    }
};
unsafe fn cmd_pipe_pane_exec(mut self_0: refbox::Weak<cmd>, item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    let queue_client = cmdq_get_client((item).as_ref());
    let mut args: *mut args = cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let tc_owner = cmdq_get_target_client((item).as_ref());
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let pane_owner = (*target).wp.upgrade().expect("live pipe target pane");
    let wp = pane_owner.get();
    let mut s: *mut session = (*target).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let mut old_fd: ::core::ffi::c_int = 0;
    let mut pipe_fd: [::core::ffi::c_int; 2] = [0; 2];
    let mut null_fd: ::core::ffi::c_int = 0;
    let mut in_0: ::core::ffi::c_int = 0;
    let mut out: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    if window_pane_exited(&*wp) != 0 {
        cmdq_error(item_handle, |out| out.write_all(b"target pane has exited"));
        return CMD_RETURN_ERROR;
    }
    old_fd = (*wp).pipe_fd;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        std::mem::take(&mut (*wp).pipe_event).free();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
        if window_pane_destroy_ready(&(*(wp)).observer.upgrade().expect("live window_pane")) != 0 {
            server_destroy_pane(&pane_owner, 1);
            return CMD_RETURN_NORMAL;
        }
    }
    if args_count(args) == 0 as u_int
        || *args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()) as ::core::ffi::c_int == '\0' as i32
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
        cmdq_error(item_handle, |out| {
            out.write_all(b"socketpair error: ")?;
            write_cstr(out, strerror(*__errno_location()))
        });
        return CMD_RETURN_ERROR;
    }
    let mut ft_owner = format_create_with_client(
        queue_client.as_ref(),
        Some(item_handle),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    ft = &raw mut *ft_owner;
    format_defaults(ft, (tc).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), (s).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), wl.clone(), (wp).as_ref().and_then(|model| model.observer.upgrade()).as_ref());
    let cmd = format_expand_time_cstring(ft, args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()));
    format_free(ft_owner);
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
            cmdq_error(item_handle, |out| {
                out.write_all(b"fork error: ")?;
                write_cstr(out, strerror(*__errno_location()))
            });
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
            let read_observer = std::rc::Rc::downgrade(&pane_owner);
            let write_observer = read_observer.clone();
            let error_observer = read_observer.clone();
            let stream = bufferevent_new(
                (*wp).pipe_fd,
                bufferevent_data_callback(move |_| unsafe {
                    if let Some(owner) = read_observer.upgrade() {
                        cmd_pipe_pane_read_callback(&owner);
                    }
                }),
                bufferevent_data_callback(move |_| unsafe {
                    if let Some(owner) = write_observer.upgrade() {
                        cmd_pipe_pane_write_callback(&owner);
                    }
                }),
                bufferevent_event_callback(move |_, _| unsafe {
                    if let Some(owner) = error_observer.upgrade() {
                        cmd_pipe_pane_error_callback(&owner);
                    }
                }),
            );
            if stream.is_null() {
                fatalx(|out| out.write_all(b"out of memory"));
            }
            (*wp).pipe_event = crate::src::reactor::StreamHandle::from_ptr(stream);
            if out != 0 {
                let _ = (*wp).pipe_event.with_ptr(|event| unsafe {
                    bufferevent_enable(event, EV_WRITE as ::core::ffi::c_short);
                });
            }
            if in_0 != 0 {
                let _ = (*wp).pipe_event.with_ptr(|event| unsafe {
                    bufferevent_enable(event, EV_READ as ::core::ffi::c_short);
                });
            }
            return CMD_RETURN_NORMAL;
        }
    };
}
unsafe fn cmd_pipe_pane_read_callback(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut available: size_t = 0;
    if !(*wp).pipe_event.is_alive() {
        return;
    }
    let data = (*wp).pipe_event.with_ptr(|event| unsafe {
        let evb = &mut *(*event).input;
        let available = evbuffer_get_length(evb);
        evbuffer_pullup(evb, -1)
            .map_or_else(Vec::new, |bytes| bytes[..available].to_vec())
    }).unwrap_or_default();
    available = data.len();
    log_debug(format_args!(
        "%{} pipe read {}",
        ((*wp).id) as u32,
        (available) as usize
    ));
    let _ = (*wp).event.with_ptr(|event| unsafe {
        bufferevent_write(event, data.as_ptr().cast(), available);
    });
    let _ = (*wp).pipe_event.with_ptr(|event| unsafe {
        evbuffer_drain(bufferevent_get_input(&mut *event), available);
    });
    if window_pane_destroy_ready(&(*(wp)).observer.upgrade().expect("live window_pane")) != 0 {
        server_destroy_pane(&pane_owner, 1);
    }
}
unsafe fn cmd_pipe_pane_write_callback(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    log_debug(format_args!("%{} pipe empty", ((*wp).id) as u32));
    if window_pane_destroy_ready(&(*(wp)).observer.upgrade().expect("live window_pane")) != 0 {
        server_destroy_pane(&pane_owner, 1);
    }
}
unsafe fn cmd_pipe_pane_error_callback(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    log_debug(format_args!("%{} pipe error", ((*wp).id) as u32));
    std::mem::take(&mut (*wp).pipe_event).free();
    close((*wp).pipe_fd);
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    if window_pane_destroy_ready(&(*(wp)).observer.upgrade().expect("live window_pane")) != 0 {
        server_destroy_pane(&pane_owner, 1);
    }
}

#[cfg(test)]
mod pipe_stream_tests {
    use super::*;
    use crate::src::reactor::{bufferevent_free, evbuffer_add, shutdown_runtime};
    use crate::src::shared::rc;

    #[test]
    fn pipe_read_forwards_bytes_and_stream_handle_expires() {
        unsafe {
            let pane_owner = window_pane::new();
            let wp = rc::as_ptr(&pane_owner);
            (*wp).fd = -1;
            (*wp).pipe_fd = -1;
            let main = bufferevent_new(-1, None, None, None);
            (*wp).event = crate::src::reactor::StreamHandle::from_ptr(main);
            let pipe = bufferevent_new(-1, None, None, None);
            (*wp).pipe_event = crate::src::reactor::StreamHandle::from_ptr(pipe);
            let stale = (*wp).pipe_event.clone();
            let bytes = b"pipe output";
            evbuffer_add(&mut (*pipe).input, bytes.as_ptr().cast(), bytes.len());

            cmd_pipe_pane_read_callback(&pane_owner);

            assert_eq!((*wp).event.with_ptr(|event| unsafe {
                evbuffer_get_length(&(*event).output)
            }), Some(bytes.len()));
            assert_eq!(evbuffer_get_length(&(*pipe).input), 0);
            std::mem::take(&mut (*wp).pipe_event).free();
            assert!(!stale.is_alive());
            assert!(!(*wp).pipe_event.is_alive());
            std::mem::take(&mut (*wp).event).free();
            drop(pane_owner);
            shutdown_runtime();
        }
    }
}
