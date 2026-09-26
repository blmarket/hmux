use crate::src::cfg::cfg_finished;
use crate::src::cmd::queue::cmdq_print;
use crate::src::cmd::{cmd_log_argv, cmd_stringify_argv_cstring};
use crate::src::compat::fdforkpty::fdforkpty;
use crate::src::environ::{
    environ_copy, environ_for_session, environ_push, environ_set, EnvironOwner,
};
use crate::src::ffi::libc::{
    _exit, chdir, close, closefrom, dup2, execl, execvp, fork, ioctl, kill, killpg, memset, open,
    setenv, shutdown, sigfillset, sigprocmask, socketpair, strlcpy,
};
use crate::src::log::{fatal, fatalx, log_debug};
use crate::src::options::options_get_string;
use crate::src::proc::proc_clear_signals;
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_free, bufferevent_get_output,
    bufferevent_new, evbuffer_get_length, evbuffer_pullup,
};
use crate::src::server::server_proc;
use crate::src::shared::abi::*;
use crate::src::shared::command::cmdq_item;
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_WRITE};
use crate::src::shared::job::{
    job, job_complete_cb, job_free_cb, job_state, job_update_cb, JobCompletion, JobExitStatus,
};
use crate::src::shared::job::{
    JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, JOB_SHOWSTDERR,
};
use crate::src::shared::options::options;
use crate::src::shared::posix_io::{
    _PATH_BSHELL, _PATH_DEVNULL, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
use crate::src::shared::posix_terminal::{winsize, TIOCSWINSZ};
use crate::src::shared::session::session;
use crate::src::shared::signal::{
    __sigset_t, sigset_t, SIGCONT, SIGTERM, SIGTTIN, SIGTTOU, SIG_BLOCK, SIG_SETMASK,
};
use crate::src::shared::socket::{AF_UNIX, PF_UNSPEC, SOCK_STREAM};
use crate::src::shared::terminal::*;
use crate::src::tmux::{
    checkshell, find_home_cstr, global_s_options, ptm_fd, setblocking, shell_argv0_cstring,
};
use std::ffi::{CStr, CString};

pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SHUT_RDWR: C2RustUnnamed = 2;
pub const SHUT_WR: C2RustUnnamed = 1;
pub const SHUT_RD: C2RustUnnamed = 0;

pub const JOB_CLOSED: job_state = 2;
pub const JOB_DEAD: job_state = 1;
pub const JOB_RUNNING: job_state = 0;

unsafe fn job_completion(job: *mut job) -> JobCompletion {
    let input = (*(*job).event).input;
    let len = evbuffer_get_length(&*input);
    let output = if len == 0 {
        Vec::new()
    } else {
        let bytes = evbuffer_pullup(input, -(1 as ::core::ffi::c_int) as ssize_t);
        if bytes.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(bytes, len).to_vec()
        }
    };
    JobCompletion {
        status: JobExitStatus::from_wait_status((*job).status),
        output,
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct joblist {
    pub lh_first: *mut job,
}

pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;

static mut all_jobs: joblist = joblist {
    lh_first: ::core::ptr::null::<job>() as *mut job,
};

pub unsafe fn job_run(
    cmd: Option<&CStr>,
    argv: &Vec<CString>,
    mut e: *mut environ,
    mut s: *mut session,
    cwd: Option<&CStr>,
    mut updatecb: job_update_cb,
    mut completecb: job_complete_cb,
    mut freecb: job_free_cb,
    mut flags: ::core::ffi::c_int,
    mut sx: ::core::ffi::c_int,
    mut sy: ::core::ffi::c_int,
) -> *mut job {
    let mut current_block: u64;
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut pid: pid_t = 0;
    let mut nullfd: ::core::ffi::c_int = 0;
    let mut out: [::core::ffi::c_int; 2] = [0; 2];
    let mut master: ::core::ffi::c_int = 0;
    let mut do_close: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut argvp: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut tty: [::core::ffi::c_char; 32] = [0; 32];
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    // This environment is forked for the job and never escapes `job_run`.
    // Keep ownership in a Rust local rather than putting a Drop-bearing value
    // in the C-managed job record. The child reaches exec/_exit, while the
    // parent drops the same owner on both success and failure paths.
    let mut env_owner = Some(EnvironOwner::from_raw(environ_for_session(
        s,
        (cfg_finished == 0) as ::core::ffi::c_int,
    )));
    env = env_owner
        .as_ref()
        .expect("job environment owner must exist")
        .as_ptr();
    if !e.is_null() {
        environ_copy(e, env);
    }
    if !flags & JOB_DEFAULTSHELL != 0 {
        shell = _PATH_BSHELL.as_ptr();
    } else {
        if !s.is_null() {
            oo = (*s).options;
        } else {
            oo = global_s_options;
        }
        shell = options_get_string(
            oo,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if checkshell(shell) == 0 {
            shell = _PATH_BSHELL.as_ptr();
        }
    }
    let argv0 = shell_argv0_cstring(CStr::from_ptr(shell), false);
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    if flags & JOB_PTY != 0 {
        memset(
            &raw mut ws as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<winsize>() as size_t,
        );
        ws.ws_col = sx as ::core::ffi::c_ushort;
        ws.ws_row = sy as ::core::ffi::c_ushort;
        pid = fdforkpty(
            ptm_fd,
            &raw mut master,
            &raw mut tty as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<termios>(),
            &raw mut ws,
        );
        current_block = 224731115979188411;
    } else if socketpair(
        AF_UNIX,
        SOCK_STREAM as ::core::ffi::c_int,
        PF_UNSPEC,
        &raw mut out as *mut ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        current_block = 12393940290395533062;
    } else {
        pid = fork() as pid_t;
        current_block = 224731115979188411;
    }
    match current_block {
        224731115979188411 => {
            if cmd.is_none() {
                cmd_log_argv(argv, c"job_run:");
                log_debug(
                    b"%s: cwd=%s, shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                    cwd.map_or(
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        CStr::as_ptr,
                    ),
                    shell,
                );
            } else {
                log_debug(
                    b"%s: cmd=%s, cwd=%s, shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                    cmd.unwrap().as_ptr(),
                    cwd.map_or(
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        CStr::as_ptr,
                    ),
                    shell,
                );
            }
            match pid {
                -1 => {
                    if !flags & JOB_PTY != 0 {
                        close(out[0 as ::core::ffi::c_int as usize]);
                        close(out[1 as ::core::ffi::c_int as usize]);
                    }
                }
                0 => {
                    proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    if let Some(cwd) = cwd {
                        if chdir(cwd.as_ptr()) == 0 as ::core::ffi::c_int {
                            environ_set(
                                env,
                                b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                cwd.as_ptr(),
                            );
                        } else {
                            home = find_home_cstr().map_or(::core::ptr::null(), CStr::as_ptr);
                            if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                                environ_set(
                                    env,
                                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                    home,
                                );
                            } else if chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char)
                                == 0 as ::core::ffi::c_int
                            {
                                environ_set(
                                    env,
                                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    b"/\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                            } else {
                                _exit(1 as ::core::ffi::c_int);
                            }
                        }
                    }
                    environ_push(env);
                    // The child has a private copy after fork. The parent
                    // keeps its owner for the parent-side return path.
                    drop(env_owner.take());
                    if !flags & JOB_PTY != 0 {
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDIN_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            _exit(1 as ::core::ffi::c_int);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDIN_FILENO)
                            as ::core::ffi::c_int;
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDOUT_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            _exit(1 as ::core::ffi::c_int);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDOUT_FILENO)
                            as ::core::ffi::c_int;
                        if flags & JOB_SHOWSTDERR != 0 {
                            if dup2(out[1 as ::core::ffi::c_int as usize], STDERR_FILENO)
                                == -(1 as ::core::ffi::c_int)
                            {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            do_close = (do_close != 0
                                && out[1 as ::core::ffi::c_int as usize] != STDERR_FILENO)
                                as ::core::ffi::c_int;
                        } else {
                            nullfd = open(_PATH_DEVNULL.as_ptr(), O_RDWR);
                            if nullfd == -(1 as ::core::ffi::c_int) {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            if dup2(nullfd, STDERR_FILENO) == -(1 as ::core::ffi::c_int) {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            if nullfd != STDERR_FILENO {
                                close(nullfd);
                            }
                        }
                        if do_close != 0 {
                            close(out[1 as ::core::ffi::c_int as usize]);
                        }
                        close(out[0 as ::core::ffi::c_int as usize]);
                    }
                    closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
                    if let Some(cmd) = cmd {
                        if flags & JOB_DEFAULTSHELL != 0 {
                            setenv(
                                b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
                                shell,
                                1 as ::core::ffi::c_int,
                            );
                        }
                        execl(
                            shell,
                            argv0.as_ptr(),
                            b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                            cmd.as_ptr(),
                            NULL as *mut ::core::ffi::c_char,
                        );
                        _exit(1 as ::core::ffi::c_int);
                    } else {
                        let mut pointer_view: Vec<_> =
                            argv.iter().map(|arg| arg.as_ptr().cast_mut()).collect();
                        pointer_view.push(::core::ptr::null_mut());
                        argvp = pointer_view.as_mut_ptr();
                        execvp(
                            *argvp.offset(0 as ::core::ffi::c_int as isize),
                            argvp as *const *mut ::core::ffi::c_char,
                        );
                        _exit(1 as ::core::ffi::c_int);
                    }
                }
                _ => {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    drop(env_owner.take());
                    drop(argv0);
                    let cmd_owner = if let Some(cmd) = cmd {
                        Some(cmd.to_owned())
                    } else {
                        cmd_stringify_argv_cstring(argv)
                    };
                    let mut owner = Box::new(job {
                        cmd: cmd_owner,
                        ..job::empty()
                    });

                    job = Box::into_raw(owner).cast::<job>();
                    (*job).state = JOB_RUNNING;
                    (*job).flags = flags;
                    (*job).pid = pid;
                    if flags & JOB_PTY != 0 {
                        strlcpy(
                            &raw mut (*job).tty as *mut ::core::ffi::c_char,
                            &raw mut tty as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                        );
                    }
                    (*job).status = 0 as ::core::ffi::c_int;
                    (*job).entry.le_next = all_jobs.lh_first;
                    if !(*job).entry.le_next.is_null() {
                        (*all_jobs.lh_first).entry.le_prev = &raw mut (*job).entry.le_next;
                    }
                    all_jobs.lh_first = job;
                    (*job).entry.le_prev = &raw mut all_jobs.lh_first;
                    (*job).updatecb = updatecb;
                    (*job).completecb = completecb;
                    (*job).freecb = freecb;
                    if !flags & JOB_PTY != 0 {
                        close(out[1 as ::core::ffi::c_int as usize]);
                        (*job).fd = out[0 as ::core::ffi::c_int as usize];
                    } else {
                        (*job).fd = master;
                    }
                    setblocking((*job).fd, 0 as ::core::ffi::c_int);
                    (*job).event = bufferevent_new(
                        (*job).fd,
                        bufferevent_data_callback(move |stream| unsafe {
                            job_read_callback(stream.as_ptr(), job as *mut ::core::ffi::c_void)
                        }),
                        bufferevent_data_callback(move |stream| unsafe {
                            job_write_callback(stream.as_ptr(), job as *mut ::core::ffi::c_void)
                        }),
                        bufferevent_event_callback(move |stream, flags| unsafe {
                            job_error_callback(
                                stream.as_ptr(),
                                flags,
                                job as *mut ::core::ffi::c_void,
                            )
                        }),
                    );
                    if (*job).event.is_null() {
                        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                    bufferevent_enable((*job).event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
                    log_debug(
                        b"run job %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
                        job,
                        ((*job).cmd)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        (*job).pid as ::core::ffi::c_long,
                    );
                    return job;
                }
            }
        }
        _ => {}
    }
    sigprocmask(
        SIG_SETMASK,
        &raw mut oldset,
        ::core::ptr::null_mut::<sigset_t>(),
    );
    drop(argv0);
    return ::core::ptr::null_mut::<job>();
}
#[no_mangle]
pub unsafe extern "C" fn job_transfer(
    mut job: *mut job,
    mut pid: *mut pid_t,
    mut tty: *mut ::core::ffi::c_char,
    mut ttylen: size_t,
) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = (*job).fd;
    log_debug(
        b"transfer job %p: %s\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        ((*job).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if !pid.is_null() {
        *pid = (*job).pid;
    }
    if !tty.is_null() {
        strlcpy(tty, &raw mut (*job).tty as *mut ::core::ffi::c_char, ttylen);
    }
    if !(*job).entry.le_next.is_null() {
        (*(*job).entry.le_next).entry.le_prev = (*job).entry.le_prev;
    }
    *(*job).entry.le_prev = (*job).entry.le_next;
    if let Some(callback) = (*job).freecb.take() {
        callback();
    }
    if !(*job).event.is_null() {
        bufferevent_free((*job).event);
    }
    drop(Box::from_raw(job));
    return fd;
}
#[no_mangle]
pub unsafe extern "C" fn job_free(mut job: *mut job) {
    log_debug(
        b"free job %p: %s\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        ((*job).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if !(*job).entry.le_next.is_null() {
        (*(*job).entry.le_next).entry.le_prev = (*job).entry.le_prev;
    }
    *(*job).entry.le_prev = (*job).entry.le_next;
    if let Some(callback) = (*job).freecb.take() {
        callback();
    }
    if (*job).pid != -(1 as ::core::ffi::c_int) {
        kill((*job).pid as __pid_t, SIGTERM);
    }
    if !(*job).event.is_null() {
        bufferevent_free((*job).event);
    }
    if (*job).fd != -(1 as ::core::ffi::c_int) {
        close((*job).fd);
    }
    drop(Box::from_raw(job));
}
#[no_mangle]
pub unsafe extern "C" fn job_resize(mut job: *mut job, mut sx: u_int, mut sy: u_int) {
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    if (*job).fd == -(1 as ::core::ffi::c_int) || !(*job).flags & JOB_PTY != 0 {
        return;
    }
    log_debug(
        b"resize job %p: %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        sx,
        sy,
    );
    memset(
        &raw mut ws as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<winsize>() as size_t,
    );
    ws.ws_col = sx as ::core::ffi::c_ushort;
    ws.ws_row = sy as ::core::ffi::c_ushort;
    if ioctl((*job).fd, TIOCSWINSZ as ::core::ffi::c_ulong, &raw mut ws)
        == -(1 as ::core::ffi::c_int)
    {
        fatal(b"ioctl failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
unsafe fn job_read_callback(_bufev: *mut bufferevent, mut data: *mut ::core::ffi::c_void) {
    let mut job: *mut job = data as *mut job;
    let Some(callback_slot) = (*job).updatecb.as_ref().cloned() else {
        return;
    };
    let Some(mut callback) = callback_slot.borrow_mut().take() else {
        return;
    };
    callback(&mut *job);
    let mut callback_owner = callback_slot.borrow_mut();
    if callback_owner.is_none() {
        *callback_owner = Some(callback);
    }
}
unsafe fn job_write_callback(_bufev: *mut bufferevent, mut data: *mut ::core::ffi::c_void) {
    let mut job: *mut job = data as *mut job;
    let mut len: size_t = evbuffer_get_length(&*(bufferevent_get_output((*job).event)));
    log_debug(
        b"job write %p: %s, pid %ld, output left %zu\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        ((*job).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        (*job).pid as ::core::ffi::c_long,
        len,
    );
    if len == 0 as size_t && !(*job).flags & JOB_KEEPWRITE != 0 {
        shutdown((*job).fd, SHUT_WR as ::core::ffi::c_int);
        bufferevent_disable((*job).event, EV_WRITE as ::core::ffi::c_short);
    }
}
unsafe fn job_error_callback(
    _bufev: *mut bufferevent,
    _events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    log_debug(
        b"job error %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        ((*job).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        (*job).pid as ::core::ffi::c_long,
    );
    if (*job).state as ::core::ffi::c_uint == JOB_DEAD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if let Some(callback) = (*job).completecb.take() {
            callback(job_completion(job));
        }
        job_free(job);
    } else {
        bufferevent_disable((*job).event, EV_READ as ::core::ffi::c_short);
        (*job).state = JOB_CLOSED;
    };
}
#[no_mangle]
pub unsafe extern "C" fn job_check_died(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if pid == (*job).pid {
            break;
        }
        job = (*job).entry.le_next;
    }
    if job.is_null() {
        return;
    }
    if status & 0xff as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
        if (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTIN
            || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTOU
        {
            return;
        }
        killpg((*job).pid as __pid_t, SIGCONT);
        return;
    }
    log_debug(
        b"job died %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        ((*job).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        (*job).pid as ::core::ffi::c_long,
    );
    (*job).status = status;
    if (*job).state as ::core::ffi::c_uint
        == JOB_CLOSED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if let Some(callback) = (*job).completecb.take() {
            callback(job_completion(job));
        }
        job_free(job);
    } else {
        (*job).pid = -(1 as ::core::ffi::c_int) as pid_t;
        (*job).state = JOB_DEAD;
    };
}
#[no_mangle]
pub unsafe extern "C" fn job_get_status(mut job: *mut job) -> ::core::ffi::c_int {
    return (*job).status;
}
#[no_mangle]
pub unsafe extern "C" fn job_get_event(mut job: *mut job) -> *mut bufferevent {
    return (*job).event;
}
#[no_mangle]
pub unsafe extern "C" fn job_kill_all() {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if (*job).pid != -(1 as ::core::ffi::c_int) {
            kill((*job).pid as __pid_t, SIGTERM);
        }
        job = (*job).entry.le_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn job_still_running() -> ::core::ffi::c_int {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if !(*job).flags & JOB_NOWAIT != 0
            && (*job).state as ::core::ffi::c_uint
                == JOB_RUNNING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 1 as ::core::ffi::c_int;
        }
        job = (*job).entry.le_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn job_print_summary(
    mut item: *mut cmdq_item,
    mut blank: ::core::ffi::c_int,
) {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let mut n: u_int = 0 as u_int;
    job = all_jobs.lh_first;
    while !job.is_null() {
        if blank != 0 {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
            blank = 0 as ::core::ffi::c_int;
        }
        cmdq_print(
            item,
            b"Job %u: %s [fd=%d, pid=%ld, status=%d]\0" as *const u8 as *const ::core::ffi::c_char,
            n,
            ((*job).cmd)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            (*job).fd,
            (*job).pid as ::core::ffi::c_long,
            (*job).status,
        );
        n = n.wrapping_add(1);
        job = (*job).entry.le_next;
    }
}
