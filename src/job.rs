use crate::cfg::cfg_finished;
use crate::cmd::cmdq_print;
use crate::cmd::{cmd_log_argv, cmd_stringify_argv};
use crate::compat::fdforkpty;
use crate::environ::{environ_copy, environ_for_session, environ_push, environ_set, environ_t};
use crate::ffi::{
    chdir, close, closefrom, dup2, execl, execvp, fork, ioctl, kill, killpg, open, setenv,
    shutdown, sigfillset, sigprocmask, socketpair, strlcpy,
};
use crate::fmt_args;
use crate::log::{fatal, fatalx, log_debug};
use crate::options::options_get_string;
use crate::proc::proc_clear_signals;
use crate::reactor::Interest;
use crate::server::server_proc;
use crate::session::session_options;
use crate::tmux::{checkshell, find_home, setblocking, shell_argv0};
use crate::tmux::{global_s_options, ptm_fd};
use crate::tree::GlobalQueue;
pub use crate::types::*;
use ::core::ffi::CStr;
pub const SOCK_NONBLOCK: __socket_type = 2048;
pub const SOCK_CLOEXEC: __socket_type = 524288;
pub const SOCK_PACKET: __socket_type = 10;
pub const SOCK_DCCP: __socket_type = 6;
pub const SOCK_SEQPACKET: __socket_type = 5;
pub const SOCK_RDM: __socket_type = 4;
pub const SOCK_RAW: __socket_type = 3;
pub const SOCK_DGRAM: __socket_type = 2;
pub const SOCK_STREAM: __socket_type = 1;
pub type shut_how = ::core::ffi::c_uint;
pub const SHUT_RDWR: shut_how = 2;
pub const SHUT_WR: shut_how = 1;
pub const SHUT_RD: shut_how = 0;
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
#[repr(C)]
pub struct job {
    /// What the job is called by. A job is named by its id and nothing else,
    /// so an observer never names one that has finished.
    pub id: u_int,
    pub state: job_state,
    pub flags: ::core::ffi::c_int,
    pub cmd: Option<::std::ffi::CString>,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub event: Stream,
    pub updatecb: job_update_cb,
    pub completecb: job_complete_cb,
    pub freecb: job_free_cb,
    pub data: JobData,
}
pub type job_state = ::core::ffi::c_uint;
pub const JOB_CLOSED: job_state = 2;
pub const JOB_DEAD: job_state = 1;
pub const JOB_RUNNING: job_state = 0;
pub const TIOCSWINSZ: ::core::ffi::c_int = 0x5414 as ::core::ffi::c_int;
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const PF_UNSPEC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;
pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;
pub const SIGCONT: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const SIGTTIN: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const SIGTTOU: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const SIG_BLOCK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIG_SETMASK: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const _PATH_BSHELL: &CStr = c"/bin/sh";
pub const _PATH_DEVNULL: &CStr = c"/dev/null";
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const JOB_KEEPWRITE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const JOB_PTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const JOB_DEFAULTSHELL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const JOB_SHOWSTDERR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
/// Every job the server has started, newest first, and the owner of each.
/// [`job_free`] is what takes one off.
static all_jobs: GlobalQueue<Box<job>> = GlobalQueue::new();

static mut next_job_id: u_int = 0;

/// The job called `id`, or null once it has finished.
pub fn job_find_by_id(id: u_int) -> *mut job {
    all_jobs
        .queue()
        .iter()
        .find(|job| job.id == id)
        .map_or(::core::ptr::null_mut::<job>(), |job| {
            &raw const **job as *mut job
        })
}

/// What a job is called by, or nothing for a job that never started.
pub fn job_id(job: Option<&job>) -> Option<u_int> {
    job.map(|job| job.id)
}

/// Every job on the list, newest first, as the borrowed pointers the walks
/// take. The list is read into a run of pointers first, since a walk may free
/// the job it is looking at.
fn jobs() -> Vec<*mut job> {
    all_jobs
        .queue()
        .iter()
        .map(|job| &raw const **job as *mut job)
        .collect()
}

/// Takes `job` off the list and hands it to the caller, who gives it up by
/// dropping it once it has finished reading the job.
unsafe fn take_job(job: *mut job) -> Option<Box<job>> {
    let at = all_jobs
        .queue()
        .iter()
        .position(|listed| std::ptr::eq(&raw const **listed, job))?;
    all_jobs.queue().remove(at)
}
pub unsafe fn job_run(
    mut cmd: *const ::core::ffi::c_char,
    argv: &[::std::ffi::CString],
    mut e: *mut environ_t,
    mut s: *mut session,
    mut cwd: *const ::core::ffi::c_char,
    mut updatecb: job_update_cb,
    mut completecb: job_complete_cb,
    mut freecb: job_free_cb,
    mut data: JobData,
    mut flags: ::core::ffi::c_int,
    mut sx: ::core::ffi::c_int,
    mut sy: ::core::ffi::c_int,
) -> *mut job {
    unsafe {
        let mut current_block: u64;
        let mut job: *mut job = ::core::ptr::null_mut::<job>();
        let mut pid: pid_t = 0;
        let mut nullfd: ::core::ffi::c_int = 0;
        let mut out: [::core::ffi::c_int; 2] = [0; 2];
        let mut master: ::core::ffi::c_int = 0;
        let mut do_close: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let mut ws = winsize::default();
        let mut tty: [::core::ffi::c_char; 32] = [0; 32];
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let mut env = environ_for_session(s, (cfg_finished == 0) as ::core::ffi::c_int);
        if !e.is_null() {
            environ_copy(e, &mut *env);
        }
        if !flags & JOB_DEFAULTSHELL != 0 {
            shell = _PATH_BSHELL.as_ptr();
        } else {
            if !s.is_null() {
                oo = session_options(s);
            } else {
                oo = global_s_options;
            }
            shell = options_get_string(oo, c"default-shell".as_ptr());
            if checkshell(shell) == 0 {
                shell = _PATH_BSHELL.as_ptr();
            }
        }
        let argv0 = shell_argv0(shell, 0 as ::core::ffi::c_int);
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if flags & JOB_PTY != 0 {
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
            current_block = 10893307061255223821;
        } else {
            pid = fork() as pid_t;
            current_block = 224731115979188411;
        }
        if current_block == 224731115979188411 {
            if cmd.is_null() {
                cmd_log_argv(argv, c"%s:".as_ptr(), fmt_args![c"job_run".as_ptr()]);
                log_debug(
                    c"%s: cwd=%s, shell=%s".as_ptr(),
                    fmt_args![
                        c"job_run".as_ptr(),
                        if cwd.is_null() { c"".as_ptr() } else { cwd },
                        shell
                    ],
                );
            } else {
                log_debug(
                    c"%s: cmd=%s, cwd=%s, shell=%s".as_ptr(),
                    fmt_args![
                        c"job_run".as_ptr(),
                        cmd,
                        if cwd.is_null() { c"".as_ptr() } else { cwd },
                        shell
                    ],
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
                    if !cwd.is_null() {
                        if chdir(cwd) == 0 as ::core::ffi::c_int {
                            environ_set(
                                &mut *env,
                                c"PWD".as_ptr(),
                                0 as ::core::ffi::c_int,
                                c"%s".as_ptr(),
                                fmt_args![cwd],
                            );
                        } else {
                            home = find_home().map_or(::core::ptr::null(), CStr::as_ptr);
                            if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                                environ_set(
                                    &mut *env,
                                    c"PWD".as_ptr(),
                                    0 as ::core::ffi::c_int,
                                    c"%s".as_ptr(),
                                    fmt_args![home],
                                );
                            } else if chdir(c"/".as_ptr()) == 0 as ::core::ffi::c_int {
                                environ_set(
                                    &mut *env,
                                    c"PWD".as_ptr(),
                                    0 as ::core::ffi::c_int,
                                    c"/".as_ptr(),
                                    fmt_args![],
                                );
                            } else {
                                fatal(c"chdir failed".as_ptr(), fmt_args![]);
                            }
                        }
                    }
                    environ_push(&env);
                    if !flags & JOB_PTY != 0 {
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDIN_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            fatal(c"dup2 failed".as_ptr(), fmt_args![]);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDIN_FILENO)
                            as ::core::ffi::c_int;
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDOUT_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            fatal(c"dup2 failed".as_ptr(), fmt_args![]);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDOUT_FILENO)
                            as ::core::ffi::c_int;
                        if flags & JOB_SHOWSTDERR != 0 {
                            if dup2(out[1 as ::core::ffi::c_int as usize], STDERR_FILENO)
                                == -(1 as ::core::ffi::c_int)
                            {
                                fatal(c"dup2 failed".as_ptr(), fmt_args![]);
                            }
                            do_close = (do_close != 0
                                && out[1 as ::core::ffi::c_int as usize] != STDERR_FILENO)
                                as ::core::ffi::c_int;
                        } else {
                            nullfd = open(_PATH_DEVNULL.as_ptr(), O_RDWR);
                            if nullfd == -(1 as ::core::ffi::c_int) {
                                fatal(c"open failed".as_ptr(), fmt_args![]);
                            }
                            if dup2(nullfd, STDERR_FILENO) == -(1 as ::core::ffi::c_int) {
                                fatal(c"dup2 failed".as_ptr(), fmt_args![]);
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
                    if !cmd.is_null() {
                        if flags & JOB_DEFAULTSHELL != 0 {
                            setenv(c"SHELL".as_ptr(), shell, 1 as ::core::ffi::c_int);
                        }
                        execl(
                            shell,
                            argv0.as_ptr() as *mut ::core::ffi::c_char,
                            c"-c".as_ptr(),
                            cmd,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                        fatal(c"execl failed".as_ptr(), fmt_args![]);
                    } else {
                        let argvp: Vec<*mut ::core::ffi::c_char> = argv
                            .iter()
                            .map(|arg| arg.as_ptr() as *mut ::core::ffi::c_char)
                            .chain(::core::iter::once(::core::ptr::null_mut()))
                            .collect();
                        execvp(argvp[0], argvp.as_ptr());
                        fatal(c"execvp failed".as_ptr(), fmt_args![]);
                    }
                }
                _ => {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    let id = next_job_id;
                    next_job_id = next_job_id.wrapping_add(1);
                    let mut job_box = Box::new(job {
                        id,
                        state: JOB_RUNNING,
                        flags,
                        cmd: if !cmd.is_null() {
                            Some(CStr::from_ptr(cmd).to_owned())
                        } else {
                            Some(cmd_stringify_argv(argv))
                        },
                        pid,
                        tty: [0; 32],
                        status: 0,
                        fd: -1,
                        event: Stream::NONE,
                        updatecb,
                        completecb,
                        freecb,
                        data,
                    });
                    job = &raw mut *job_box;
                    all_jobs.queue().push_front(job_box);
                    if flags & JOB_PTY != 0 {
                        strlcpy(
                            &raw mut (*job).tty as *mut ::core::ffi::c_char,
                            &raw mut tty as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                        );
                    }
                    if !flags & JOB_PTY != 0 {
                        close(out[1 as ::core::ffi::c_int as usize]);
                        (*job).fd = out[0 as ::core::ffi::c_int as usize];
                    } else {
                        (*job).fd = master;
                    }
                    setblocking((*job).fd, 0 as ::core::ffi::c_int);
                    (*job).event = Stream::new(
                        (*job).fd,
                        Some(on_job(job, |job| job_read_callback(job))),
                        Some(on_job(job, |job| job_write_callback(job))),
                        Some(on_job_error(job, |job| job_error_callback(job))),
                    );
                    if (*job).event.is_none() {
                        fatalx(c"out of memory".as_ptr(), fmt_args![]);
                    }
                    (*job).event.enable(Interest::ReadWrite);
                    log_debug(
                        c"run job %p: %s, pid %ld".as_ptr(),
                        fmt_args![
                            job,
                            (*job).cmd.as_deref(),
                            (*job).pid as ::core::ffi::c_long
                        ],
                    );
                    return job;
                }
            }
        }
        sigprocmask(
            SIG_SETMASK,
            &raw mut oldset,
            ::core::ptr::null_mut::<sigset_t>(),
        );
        ::core::ptr::null_mut::<job>()
    }
}
pub unsafe fn job_transfer(
    mut job: *mut job,
    mut pid: *mut pid_t,
    mut tty: *mut ::core::ffi::c_char,
    mut ttylen: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fd: ::core::ffi::c_int = (*job).fd;
        log_debug(
            c"transfer job %p: %s".as_ptr(),
            fmt_args![job, (*job).cmd.as_deref()],
        );
        let listed = take_job(job);
        if !pid.is_null() {
            *pid = (*job).pid;
        }
        if !tty.is_null() {
            strlcpy(tty, &raw mut (*job).tty as *mut ::core::ffi::c_char, ttylen);
        }
        (*job).cmd = None;
        if let Some(freecb) = (*job).freecb
            && !matches!(&(*job).data, JobData::None)
        {
            let data = ::core::mem::take(&mut (*job).data);
            freecb(data);
        }
        if !(*job).event.is_none() {
            (*job).event.free();
        }
        drop(listed);
        fd
    }
}
pub unsafe fn job_free(mut job: *mut job) {
    unsafe {
        log_debug(
            c"free job %p: %s".as_ptr(),
            fmt_args![job, (*job).cmd.as_deref()],
        );
        let listed = take_job(job);
        (*job).cmd = None;
        if let Some(freecb) = (*job).freecb
            && !matches!(&(*job).data, JobData::None)
        {
            let data = ::core::mem::take(&mut (*job).data);
            freecb(data);
        }
        if (*job).pid != -(1 as ::core::ffi::c_int) {
            kill((*job).pid as __pid_t, SIGTERM);
        }
        if !(*job).event.is_none() {
            (*job).event.free();
        }
        if (*job).fd != -(1 as ::core::ffi::c_int) {
            close((*job).fd);
        }
        drop(listed);
    }
}
pub unsafe fn job_resize(mut job: *mut job, mut sx: u_int, mut sy: u_int) {
    unsafe {
        let mut ws = winsize::default();
        if (*job).fd == -(1 as ::core::ffi::c_int) || !(*job).flags & JOB_PTY != 0 {
            return;
        }
        log_debug(c"resize job %p: %ux%u".as_ptr(), fmt_args![job, sx, sy]);
        ws.ws_col = sx as ::core::ffi::c_ushort;
        ws.ws_row = sy as ::core::ffi::c_ushort;
        if ioctl((*job).fd, TIOCSWINSZ as ::core::ffi::c_ulong, &raw mut ws)
            == -(1 as ::core::ffi::c_int)
        {
            fatal(c"ioctl failed".as_ptr(), fmt_args![]);
        }
    }
}
/// A stream callback that runs `body` on the job whose stream it is. The job
/// owns the stream, so it outlives every call the stream makes.
fn on_job(job: *mut job, body: unsafe fn(*mut job)) -> ::std::rc::Rc<dyn Fn(Stream)> {
    ::std::rc::Rc::new(move |_stream| unsafe { body(job) })
}

/// The same, for the callback a failed stream makes.
fn on_job_error(
    job: *mut job,
    body: unsafe fn(*mut job),
) -> ::std::rc::Rc<dyn Fn(Stream, ::core::ffi::c_short)> {
    ::std::rc::Rc::new(move |_stream, _what| unsafe { body(job) })
}

unsafe fn job_read_callback(mut job: *mut job) {
    unsafe {
        if (*job).updatecb.is_some() {
            (*job).updatecb.expect("non-null function pointer")(job);
        }
    }
}
unsafe fn job_write_callback(mut job: *mut job) {
    unsafe {
        let mut len: size_t = (*job).event.output_len();
        log_debug(
            c"job write %p: %s, pid %ld, output left %zu".as_ptr(),
            fmt_args![
                job,
                (*job).cmd.as_deref(),
                (*job).pid as ::core::ffi::c_long,
                len
            ],
        );
        if len == 0 as size_t && !(*job).flags & JOB_KEEPWRITE != 0 {
            shutdown((*job).fd, SHUT_WR as ::core::ffi::c_int);
            (*job).event.disable(Interest::Write);
        }
    }
}
unsafe fn job_error_callback(mut job: *mut job) {
    unsafe {
        log_debug(
            c"job error %p: %s, pid %ld".as_ptr(),
            fmt_args![
                job,
                (*job).cmd.as_deref(),
                (*job).pid as ::core::ffi::c_long
            ],
        );
        if (*job).state as ::core::ffi::c_uint
            == JOB_DEAD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*job).completecb.is_some() {
                (*job).completecb.expect("non-null function pointer")(job);
            }
            job_free(job);
        } else {
            (*job).event.disable(Interest::Read);
            (*job).state = JOB_CLOSED;
        };
    }
}
pub fn job_check_died(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    unsafe {
        let Some(job) = jobs().into_iter().find(|&job| pid == (*job).pid) else {
            return;
        };
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
            c"job died %p: %s, pid %ld".as_ptr(),
            fmt_args![
                job,
                (*job).cmd.as_deref(),
                (*job).pid as ::core::ffi::c_long
            ],
        );
        (*job).status = status;
        if (*job).state as ::core::ffi::c_uint
            == JOB_CLOSED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*job).completecb.is_some() {
                (*job).completecb.expect("non-null function pointer")(job);
            }
            job_free(job);
        } else {
            (*job).pid = -(1 as ::core::ffi::c_int) as pid_t;
            (*job).state = JOB_DEAD;
        };
    }
}
pub unsafe fn job_get_status(mut job: *mut job) -> ::core::ffi::c_int {
    unsafe { (*job).status }
}
pub unsafe fn job_get_data<'a>(mut job: *mut job) -> &'a JobData {
    unsafe { &(*job).data }
}
pub unsafe fn job_get_event(mut job: *mut job) -> Stream {
    unsafe { (*job).event }
}
pub fn job_kill_all() {
    unsafe {
        for job in jobs() {
            if (*job).pid != -(1 as ::core::ffi::c_int) {
                kill((*job).pid as __pid_t, SIGTERM);
            }
        }
    }
}
pub fn job_still_running() -> ::core::ffi::c_int {
    unsafe {
        for job in jobs() {
            if !(*job).flags & JOB_NOWAIT != 0
                && (*job).state as ::core::ffi::c_uint
                    == JOB_RUNNING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return 1 as ::core::ffi::c_int;
            }
        }
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn job_print_summary(mut item: *mut cmdq_item, mut blank: ::core::ffi::c_int) {
    unsafe {
        let mut n: u_int = 0 as u_int;
        for job in jobs() {
            if blank != 0 {
                cmdq_print(item, c"%s".as_ptr(), fmt_args![c"".as_ptr()]);
                blank = 0 as ::core::ffi::c_int;
            }
            cmdq_print(
                item,
                c"Job %u: %s [fd=%d, pid=%ld, status=%d]".as_ptr(),
                fmt_args![
                    n,
                    (*job).cmd.as_deref(),
                    (*job).fd,
                    (*job).pid as ::core::ffi::c_long,
                    (*job).status
                ],
            );
            n = n.wrapping_add(1);
        }
    }
}
