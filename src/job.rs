use crate::cmd::cmdq_item;
use crate::cfg::cfg_finished;
use crate::cmd::cmd_log_argv;

use crate::compat::fdforkpty;
use crate::environ::EnvironmentStore;
use crate::environ::{RustEnvironment, environment_for_session, push_environment_to_process};
use crate::ffi::{
    chdir, close, closefrom, dup2, execl, execvp, fork, ioctl, kill, killpg, open, setenv,
    shutdown, sigfillset, sigprocmask, socketpair,
};
use crate::fmt_args;
use crate::log::{fatal, fatalx, log_debug};

use crate::proc::proc_clear_signals;
use crate::reactor::Interest;
use crate::server::server_proc;

pub use crate::consts::{
    _PATH_BSHELL, _PATH_DEVNULL, AF_UNIX, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN,
    CLIENT_EXIT_SHUTDOWN, JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, JOB_SHOWSTDERR,
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL,
    MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN,
    MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK,
    MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK,
    MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY,
    PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE,
    PANE_LINES_SPACES, PF_LOCAL, PF_UNIX, PF_UNSPEC, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, SIG_BLOCK, SIG_SETMASK, SIGCONT, SIGTERM, SIGTTIN, SIGTTOU,
    SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM,
    SOCK_SEQPACKET, SOCK_STREAM, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, TIOCSWINSZ,
};
use crate::tmux::{checkshell, find_home, setblocking, shell_argv0};
use crate::tmux::{global_s_options, ptm_fd};
use crate::tree::GlobalQueue;
pub use crate::types::*;
use crate::{CommandTextCodec, RustCommandTextCodec};
use ::core::ffi::CStr;

pub type shut_how = core::ffi::c_uint;
pub const SHUT_RDWR: shut_how = 2;
pub const SHUT_WR: shut_how = 1;
pub const SHUT_RD: shut_how = 0;

#[repr(C)]
pub struct job {
    /// What the job is called by. A job is named by its id and nothing else,
    /// so an observer never names one that has finished.
    pub id: u_int,
    pub state: job_state,
    pub flags: core::ffi::c_int,
    pub cmd: Option<std::ffi::CString>,
    pub pid: pid_t,
    pub tty: [u8; 32],
    pub status: core::ffi::c_int,
    pub fd: core::ffi::c_int,
    pub event: Stream,
    pub updatecb: job_update_cb,
    pub completecb: job_complete_cb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobEvent {
    pub id: u_int,
    pub event: Stream,
    pub status: core::ffi::c_int,
}

impl job {
    pub fn event_state(&self) -> JobEvent {
        JobEvent {
            id: self.id,
            event: self.event,
            status: self.status,
        }
    }
}
pub type job_state = core::ffi::c_uint;
pub const JOB_CLOSED: job_state = 2;
pub const JOB_DEAD: job_state = 1;
pub const JOB_RUNNING: job_state = 0;

pub const O_RDWR: core::ffi::c_int = 0o2 as core::ffi::c_int;

pub use crate::consts::EV_READ;
pub use crate::consts::EV_WRITE;

/// Every job the server has started, newest first, and the owner of each.
/// [`job_free`] is what takes one off.
static all_jobs: GlobalQueue<Box<job>> = GlobalQueue::new();

thread_local! {
    static NEXT_JOB_ID: std::cell::Cell<Option<u_int>> = const { std::cell::Cell::new(Some(0)) };
}

/// The event stream of a registered job, or nothing after it leaves the registry.
pub fn job_event_by_id(id: u_int) -> Option<Stream> {
    all_jobs
        .queue()
        .iter()
        .find(|job| job.id == id)
        .map(|job| job.event)
}

/// Removes the named job and transfers its ownership to the caller.
fn take_job(id: u_int) -> Option<Box<job>> {
    let at = all_jobs.queue().iter().position(|listed| listed.id == id)?;
    all_jobs.queue().remove(at)
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn job_run(
    cmd: Option<&CStr>,
    argv: &[std::ffi::CString],
    e: Option<&RustEnvironment>,
    s: Option<&session>,
    cwd: Option<&CStr>,
    updatecb: job_update_cb,
    completecb: job_complete_cb,
    flags: core::ffi::c_int,
    sx: core::ffi::c_int,
    sy: core::ffi::c_int,
) -> Option<u_int> {
    unsafe {
        let Some(id) = crate::entity_id::try_next_entity_id(&NEXT_JOB_ID) else {
            drop(updatecb);
            drop(completecb);
            *crate::ffi::__errno_location() = libc::EOVERFLOW;
            return None;
        };
        let current_block: u64;
        let mut pid: pid_t = 0;
        let nullfd: core::ffi::c_int;
        let mut out: [core::ffi::c_int; 2] = [0; 2];
        let mut do_close: core::ffi::c_int = 1 as core::ffi::c_int;
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let mut ws = winsize::default();
        let mut master: core::ffi::c_int = 0;
        let mut tty: [u8; 32] = [0; 32];
        let mut env = environment_for_session(s, (cfg_finished == 0) as core::ffi::c_int);
        if let Some(e) = e {
            env.copy_from(e);
        }
        let configured_shell;
        let shell = if flags & JOB_DEFAULTSHELL == 0 {
            _PATH_BSHELL
        } else {
            let options = match s {
                Some(s) => (s).options_ref().clone(),
                None => global_s_options
                    .as_ref()
                    .expect("global options are initialized")
                    .clone(),
            };
            configured_shell = options.string_ref(c"default-shell");
            if checkshell(Some(&configured_shell)) == 0 {
                _PATH_BSHELL
            } else {
                &configured_shell
            }
        };
        let argv0 = shell_argv0(shell, 0 as core::ffi::c_int);
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if flags & JOB_PTY != 0 {
            ws.ws_col = sx as core::ffi::c_ushort;
            ws.ws_row = sy as core::ffi::c_ushort;
            let forkpty = fdforkpty(ptm_fd, None, Some(&ws));
            pid = forkpty.pid;
            master = forkpty.master_fd;
            tty = forkpty.tty_name;
            current_block = 224731115979188411;
        } else if socketpair(
            AF_UNIX,
            SOCK_STREAM as core::ffi::c_int,
            PF_UNSPEC,
            &raw mut out as *mut core::ffi::c_int,
        ) != 0 as core::ffi::c_int
        {
            current_block = 10893307061255223821;
        } else {
            pid = fork() as pid_t;
            current_block = 224731115979188411;
        }
        if current_block == 224731115979188411 {
            if cmd.is_none() {
                cmd_log_argv(argv, c"%s:", fmt_args![c"job_run"]);
                log_debug(
                    c"%s: cwd=%s, shell=%s",
                    fmt_args![c"job_run", cwd.unwrap_or(c""), shell],
                );
            } else {
                log_debug(
                    c"%s: cmd=%s, cwd=%s, shell=%s",
                    fmt_args![c"job_run", cmd.unwrap_or(c""), cwd.unwrap_or(c""), shell],
                );
            }
            match pid {
                -1 => {
                    if !flags & JOB_PTY != 0 {
                        close(out[0 as core::ffi::c_int as usize]);
                        close(out[1 as core::ffi::c_int as usize]);
                    }
                }
                0 => {
                    proc_clear_signals(
                        &mut server_proc
                            .as_ref()
                            .expect("server process is initialized")
                            .borrow_mut(),
                        1 as core::ffi::c_int,
                    );
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        core::ptr::null_mut::<sigset_t>(),
                    );
                    if let Some(cwd) = cwd {
                        if chdir(cwd.as_ptr()) == 0 as core::ffi::c_int {
                            env.set(c"PWD", 0, cwd);
                        } else {
                            if let Some(home) = find_home().filter(|home| chdir(home.as_ptr()) == 0)
                            {
                                env.set(c"PWD", 0, home);
                            } else if chdir(c"/".as_ptr()) == 0 as core::ffi::c_int {
                                env.set(c"PWD", 0, c"/");
                            } else {
                                fatal(c"chdir failed", fmt_args![]);
                            }
                        }
                    }
                    push_environment_to_process(&env);
                    if !flags & JOB_PTY != 0 {
                        if dup2(out[1 as core::ffi::c_int as usize], STDIN_FILENO)
                            == -(1 as core::ffi::c_int)
                        {
                            fatal(c"dup2 failed", fmt_args![]);
                        }
                        do_close = (do_close != 0
                            && out[1 as core::ffi::c_int as usize] != STDIN_FILENO)
                            as core::ffi::c_int;
                        if dup2(out[1 as core::ffi::c_int as usize], STDOUT_FILENO)
                            == -(1 as core::ffi::c_int)
                        {
                            fatal(c"dup2 failed", fmt_args![]);
                        }
                        do_close = (do_close != 0
                            && out[1 as core::ffi::c_int as usize] != STDOUT_FILENO)
                            as core::ffi::c_int;
                        if flags & JOB_SHOWSTDERR != 0 {
                            if dup2(out[1 as core::ffi::c_int as usize], STDERR_FILENO)
                                == -(1 as core::ffi::c_int)
                            {
                                fatal(c"dup2 failed", fmt_args![]);
                            }
                            do_close = (do_close != 0
                                && out[1 as core::ffi::c_int as usize] != STDERR_FILENO)
                                as core::ffi::c_int;
                        } else {
                            nullfd = open(_PATH_DEVNULL.as_ptr(), O_RDWR);
                            if nullfd == -(1 as core::ffi::c_int) {
                                fatal(c"open failed", fmt_args![]);
                            }
                            if dup2(nullfd, STDERR_FILENO) == -(1 as core::ffi::c_int) {
                                fatal(c"dup2 failed", fmt_args![]);
                            }
                            if nullfd != STDERR_FILENO {
                                close(nullfd);
                            }
                        }
                        if do_close != 0 {
                            close(out[1 as core::ffi::c_int as usize]);
                        }
                        close(out[0 as core::ffi::c_int as usize]);
                    }
                    closefrom(STDERR_FILENO + 1 as core::ffi::c_int);
                    if let Some(cmd) = cmd {
                        if flags & JOB_DEFAULTSHELL != 0 {
                            setenv(c"SHELL".as_ptr(), shell.as_ptr(), 1 as core::ffi::c_int);
                        }
                        execl(
                            shell.as_ptr(),
                            argv0.as_ptr(),
                            c"-c".as_ptr(),
                            cmd.as_ptr(),
                            core::ptr::null_mut::<core::ffi::c_char>(),
                        );
                        fatal(c"execl failed", fmt_args![]);
                    } else {
                        let argvp: Vec<*mut core::ffi::c_char> = argv
                            .iter()
                            .map(|arg| arg.as_ptr() as *mut core::ffi::c_char)
                            .chain(core::iter::once(core::ptr::null_mut()))
                            .collect();
                        execvp(argvp[0], argvp.as_ptr());
                        fatal(c"execvp failed", fmt_args![]);
                    }
                }
                _ => {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        core::ptr::null_mut::<sigset_t>(),
                    );
                    let mut job_box = Box::new(job {
                        id,
                        state: JOB_RUNNING,
                        flags,
                        cmd: if let Some(cmd) = cmd {
                            Some(cmd.to_owned())
                        } else {
                            Some(RustCommandTextCodec.stringify(argv))
                        },
                        pid,
                        tty: [0; 32],
                        status: 0,
                        fd: -1,
                        event: Stream::NONE,
                        updatecb,
                        completecb,
                    });
                    let job = &mut *job_box;
                    if flags & JOB_PTY != 0 {
                        job.tty = tty;
                    }
                    if !flags & JOB_PTY != 0 {
                        close(out[1 as core::ffi::c_int as usize]);
                        job.fd = out[0 as core::ffi::c_int as usize];
                    } else {
                        job.fd = master;
                    }
                    setblocking(job.fd, 0 as core::ffi::c_int);
                    job.event = Stream::new(
                        job.fd,
                        Some(on_job(id, job_read_callback)),
                        Some(on_job(id, job_write_callback)),
                        Some(on_job_error(id, job_error_callback)),
                    );
                    if job.event.is_none() {
                        fatalx(c"out of memory", fmt_args![]);
                    }
                    job.event.enable(Interest::ReadWrite);
                    log_debug(
                        c"run job %u: %s, pid %ld",
                        fmt_args![id, job.cmd.as_deref(), job.pid as core::ffi::c_long],
                    );
                    all_jobs.queue().push_front(job_box);
                    return Some(id);
                }
            }
        }
        sigprocmask(
            SIG_SETMASK,
            &raw mut oldset,
            core::ptr::null_mut::<sigset_t>(),
        );
        None
    }
}
/// Hands the job's file descriptor and process id to the caller and takes the
/// job off the list, copying its tty name into `tty` when one is given.
pub unsafe fn job_transfer(id: u_int, tty: Option<&mut [u8]>) -> Option<(core::ffi::c_int, pid_t)> {
    {
        let mut job = take_job(id)?;
        let fd: core::ffi::c_int = job.fd;
        log_debug(c"transfer job %u: %s", fmt_args![id, job.cmd.as_deref()]);
        let pid = job.pid;
        if let Some(tty) = tty.filter(|tty| !tty.is_empty()) {
            let len = job
                .tty
                .iter()
                .position(|ch| *ch == 0)
                .unwrap_or(job.tty.len());
            let len = len.min(tty.len() - 1);
            tty[..len].copy_from_slice(&job.tty[..len]);
            tty[len] = 0;
        }
        job.cmd = None;
        if !job.event.is_none() {
            job.event.free();
        }
        Some((fd, pid))
    }
}
pub unsafe fn job_free(id: u_int) {
    unsafe {
        let Some(mut job) = take_job(id) else {
            return;
        };
        log_debug(c"free job %u: %s", fmt_args![id, job.cmd.as_deref()]);
        job.cmd = None;
        if job.pid != -(1 as core::ffi::c_int) {
            kill(job.pid as __pid_t, SIGTERM);
        }
        if !job.event.is_none() {
            job.event.free();
        }
        if job.fd != -(1 as core::ffi::c_int) {
            close(job.fd);
        }
    }
}
pub unsafe fn job_resize(id: u_int, sx: u_int, sy: u_int) {
    unsafe {
        let jobs = all_jobs.queue();
        let Some(job) = jobs.iter().find(|job| job.id == id) else {
            return;
        };
        let mut ws = winsize::default();
        if job.fd == -(1 as core::ffi::c_int) || !job.flags & JOB_PTY != 0 {
            return;
        }
        log_debug(c"resize job %u: %ux%u", fmt_args![id, sx, sy]);
        ws.ws_col = sx as core::ffi::c_ushort;
        ws.ws_row = sy as core::ffi::c_ushort;
        if ioctl(job.fd, TIOCSWINSZ as core::ffi::c_ulong, &raw mut ws) == -(1 as core::ffi::c_int)
        {
            fatal(c"ioctl failed", fmt_args![]);
        }
    }
}
/// Routes stream events by job identity. Handlers ignore removed jobs.
fn on_job(id: u_int, body: impl Fn(u_int) + 'static) -> std::rc::Rc<dyn Fn(Stream)> {
    std::rc::Rc::new(move |_stream| body(id))
}

/// Routes stream errors by job identity.
fn on_job_error(
    id: u_int,
    body: impl Fn(u_int) + 'static,
) -> std::rc::Rc<dyn Fn(Stream, core::ffi::c_short)> {
    std::rc::Rc::new(move |_stream, _what| body(id))
}

fn job_read_callback(id: u_int) {
    let (callback, event) = {
        let jobs = all_jobs.queue();
        let Some(job) = jobs.iter().find(|job| job.id == id) else {
            return;
        };
        (job.updatecb.clone(), job.event_state())
    };
    if let Some(callback) = callback {
        callback(event);
    }
}

fn job_write_callback(id: u_int) {
    let jobs = all_jobs.queue();
    let Some(job) = jobs.iter().find(|job| job.id == id) else {
        return;
    };
    let len = job.event.output_len();
    {
        log_debug(
            c"job write %u: %s, pid %ld, output left %zu",
            fmt_args![id, job.cmd.as_deref(), job.pid as core::ffi::c_long, len],
        );
    }
    if len == 0 && job.flags & JOB_KEEPWRITE == 0 {
        unsafe { shutdown(job.fd, SHUT_WR as core::ffi::c_int) };
        job.event.disable(Interest::Write);
    }
}

fn job_error_callback(id: u_int) {
    let (callback, event) = {
        let mut jobs = all_jobs.queue();
        let Some(job) = jobs.iter_mut().find(|job| job.id == id) else {
            return;
        };
        {
            log_debug(
                c"job error %u: %s, pid %ld",
                fmt_args![id, job.cmd.as_deref(), job.pid as core::ffi::c_long],
            );
        }
        if job.state != JOB_DEAD {
            job.event.disable(Interest::Read);
            job.state = JOB_CLOSED;
            return;
        }
        (job.completecb.take(), job.event_state())
    };
    if let Some(callback) = callback {
        callback(event);
    }
    unsafe { job_free(id) };
}

pub fn job_check_died(pid: pid_t, status: core::ffi::c_int) {
    let (callback, event) = {
        let mut jobs = all_jobs.queue();
        let Some(job) = jobs.iter_mut().find(|job| job.pid == pid) else {
            return;
        };
        if status & 0xff == 0x7f {
            if (status & 0xff00) >> 8 == SIGTTIN || (status & 0xff00) >> 8 == SIGTTOU {
                return;
            }
            unsafe { killpg(job.pid, SIGCONT) };
            return;
        }
        {
            log_debug(
                c"job died %u: %s, pid %ld",
                fmt_args![job.id, job.cmd.as_deref(), job.pid as core::ffi::c_long],
            );
        }
        job.status = status;
        if job.state != JOB_CLOSED {
            job.pid = -1;
            job.state = JOB_DEAD;
            return;
        }
        (job.completecb.take(), job.event_state())
    };
    if let Some(callback) = callback {
        callback(event);
    }
    unsafe { job_free(event.id) };
}
pub fn job_kill_all() {
    unsafe {
        for job in all_jobs.queue().iter() {
            if job.pid != -(1 as core::ffi::c_int) {
                kill(job.pid as __pid_t, SIGTERM);
            }
        }
    }
}
pub fn job_still_running() -> core::ffi::c_int {
    all_jobs
        .queue()
        .iter()
        .any(|job| job.flags & JOB_NOWAIT == 0 && job.state == JOB_RUNNING) as core::ffi::c_int
}
pub unsafe fn job_print_summary(item: &cmdq_item, mut blank: core::ffi::c_int) {
    let lines: Vec<_> = all_jobs
        .queue()
        .iter()
        .enumerate()
        .map(|(n, job)| {
            crate::fmt_engine::format_alloc(
                c"Job %u: %s [fd=%d, pid=%ld, status=%d]",
                fmt_args![
                    n as u_int,
                    job.cmd.as_deref(),
                    job.fd,
                    job.pid as core::ffi::c_long,
                    job.status
                ],
            )
        })
        .collect();
    for line in lines {
        if blank != 0 {
            unsafe { item.print(c"%s", fmt_args![c""]) };
            blank = 0;
        }
        unsafe { item.print(c"%s", fmt_args![line.as_c_str()]) };
    }
}

#[cfg(test)]
#[path = "tests/test_job_identity.rs"]
mod identity_tests;

/// Starts the existing job implementation from an optional session handle.
/// Environment/cwd resolution, failure cleanup and callback ownership are unchanged;
/// the session is only viewed during job setup, not newly retained by the job.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn job_run_for_session(
    cmd: Option<&CStr>,
    argv: &[std::ffi::CString],
    e: Option<&RustEnvironment>,
    s: Option<&SessionRef>,
    cwd: Option<&CStr>,
    updatecb: job_update_cb,
    completecb: job_complete_cb,
    flags: core::ffi::c_int,
    sx: core::ffi::c_int,
    sy: core::ffi::c_int,
) -> Option<u_int> {
    unsafe {
        job_run(
            cmd,
            argv,
            e,
            s.map(|s| s.as_session()),
            cwd,
            updatecb,
            completecb,
            flags,
            sx,
            sy,
        )
    }
}
