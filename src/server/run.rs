use super::acl::server_acl_init;
use super::acl::server_acl_join;
use super::client::server_client_loop;

use super::defaults::server_default_options;
use super::message::server_destroy_pane;
use crate::WindowPane;
use crate::cmd::cmd_wait_for_flush;
use crate::cmd::cmdq_next;
use crate::cmd::{cmd_find_clear_state, cmd_find_valid_state};
use crate::compat::systemd_create_socket;
use crate::compat::{error_message, signal_description};
use crate::ffi::{
    __errno_location, accept, bind, close, exit, kill, killpg, listen, malloc_trim, sigfillset,
    sigprocmask, socket, time, umask, waitpid,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_tidy_jobs;
use crate::input::input_key_build;
use crate::job::{job_check_died, job_kill_all, job_still_running};
use crate::key_bindings::key_bindings_init;
use crate::log::{fatal, fatalx, log_debug, log_get_level};
use crate::message_log::{MessageLogStore, with_message_log_mut};
use crate::socket_address::UnixSocketAddress;

use crate::pane_exit::PaneExitState;
use crate::pane_identity::PaneIdentity;
use crate::proc::proc_fork_and_daemon;
use crate::proc::{proc_clear_signals, proc_loop, proc_set_signals, proc_start, proc_toggle_log};
use crate::reactor;
use crate::reactor::{Interest, IoWatch, Reactor, Timer, WatchMode};
use crate::session::{SESSIONS, sessions_empty};
use crate::status::status_prompt_save_history;
use crate::text::utf8_update_width_cache;
use crate::tmux::{get_timer, setblocking};
use crate::tmux::{global_options, socket_path, start_time};
use crate::tty::tty_create_log;
pub use crate::types::*;
use crate::window::{WINDOWS, window_pane_destroy_ready};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::{CString, OsStr};
use ::std::fs::{self, Permissions};
use ::std::io::Write;
use ::std::os::unix::ffi::OsStrExt;
use ::std::os::unix::fs::PermissionsExt;
pub type mode_t = __mode_t;
pub use crate::consts::{
    __S_IEXEC, __S_IREAD, AF_UNIX, CLIENT_DEFAULTSOCKET, CLIENT_EXIT, CLIENT_EXIT_SHUTDOWN,
    CLIENT_IDENTIFIED, CLIENT_NOFORK, CLIENT_SUSPENDED, EAGAIN, ECHILD, EINTR, ENAMETOOLONG,
    PANE_EXITED, PANE_STATUSREADY, S_IRWXU, SIG_BLOCK, SIG_SETMASK, SIGCHLD, SIGCONT, SIGINT,
    SIGTERM, SIGTTIN, SIGTTOU, SIGUSR1, SIGUSR2, SOCK_STREAM, WAIT_ANY, WNOHANG,
};

pub const ACCESSPERMS: core::ffi::c_int = S_IRWXU | S_IRWXG | S_IRWXO;

pub const WUNTRACED: core::ffi::c_int = 2 as core::ffi::c_int;

pub const ECONNABORTED: core::ffi::c_int = 103 as core::ffi::c_int;

pub const ENFILE: core::ffi::c_int = 23 as core::ffi::c_int;
pub const EMFILE: core::ffi::c_int = 24 as core::ffi::c_int;
pub const S_IRUSR: core::ffi::c_int = __S_IREAD;
pub const S_IXUSR: core::ffi::c_int = __S_IEXEC;

pub const S_IRGRP: core::ffi::c_int = S_IRUSR >> 3 as core::ffi::c_int;
pub const S_IXGRP: core::ffi::c_int = S_IXUSR >> 3 as core::ffi::c_int;
pub const S_IRWXG: core::ffi::c_int = S_IRWXU >> 3 as core::ffi::c_int;
pub const S_IROTH: core::ffi::c_int = S_IRGRP >> 3 as core::ffi::c_int;
pub const S_IXOTH: core::ffi::c_int = S_IXGRP >> 3 as core::ffi::c_int;
pub const S_IRWXO: core::ffi::c_int = S_IRWXG >> 3 as core::ffi::c_int;

thread_local! {
    static CLIENTS: std::cell::RefCell<clients_t> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Borrows the server thread's client registry without retaining its clients.
/// Nested reads are allowed; registering or removing clients during the visit panics.
/// References into the registry cannot escape the visit.
pub(crate) fn with_clients<R>(visit: impl FnOnce(&[ClientRef]) -> R) -> R {
    CLIENTS.with_borrow(|clients| visit(clients))
}

pub(crate) fn with_clients_mut<R>(visit: impl FnOnce(&mut clients_t) -> R) -> R {
    CLIENTS.with_borrow_mut(visit)
}

/// Walks every client the server holds, oldest first. The client after the one
/// the body has just had is read out of the list again afterwards, so the body
/// may connect or lose clients as the walk runs. Each yielded client is owned
/// for the duration of the loop body.
pub(crate) fn client_walk() -> ClientWalk {
    ClientWalk { at: 0, last: None }
}

pub(crate) struct ClientWalk {
    at: usize,
    last: Option<ClientWeak>,
}

/// Saves the next client before each loop body, as tmux's TAILQ_FOREACH_SAFE does.
fn client_walk_safe() -> impl Iterator<Item = ClientRef> {
    let mut next = first_client();
    std::iter::from_fn(move || {
        let current = next.take()?;
        next = with_clients(|clients| {
            clients
                .iter()
                .position(|client| client.ptr_eq(&current))
                .and_then(|at| clients.get(at + 1))
                .cloned()
        });
        Some(current)
    })
}

impl Iterator for ClientWalk {
    type Item = ClientRef;

    fn next(&mut self) -> Option<Self::Item> {
        with_clients(|clients| {
            let at = match &self.last {
                None => 0,
                Some(last)
                    if clients
                        .get(self.at)
                        .is_some_and(|client| last.ptr_eq(&client.downgrade())) =>
                {
                    self.at + 1
                }
                Some(last) => clients
                    .iter()
                    .position(|client| last.ptr_eq(&client.downgrade()))
                    .map_or(self.at, |moved| moved + 1),
            };
            let client = clients.get(at)?.clone();
            self.at = at;
            self.last = Some(client.downgrade());
            Some(client)
        })
    }
}

/// The oldest client the server holds, if it holds one. This is the one the
/// config load and its causes belong to.
pub fn first_client() -> Option<ClientRef> {
    with_clients(|clients| clients.first().cloned())
}

pub static mut server_proc: Option<ProcessRef> = None;
static mut server_fd: core::ffi::c_int = -(1 as core::ffi::c_int);
static mut server_client_flags: uint64_t = 0;
static mut server_exit: core::ffi::c_int = 0;
static mut server_ev_accept: IoHandle = IoHandle::ZERO;
static mut server_ev_accept_timer: TimerHandle = TimerHandle::ZERO;
static mut server_ev_tidy: TimerHandle = TimerHandle::ZERO;
pub static mut marked_pane: cmd_find_state = cmd_find_state {
    flags: 0,
    s_ref: None,
    wl_idx: None,
    w_ref: None,
    wp_ref: None,
    idx: 0,
};
pub static mut current_time: time_t = 0;
pub unsafe fn server_set_marked(
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) {
    unsafe {
        cmd_find_clear_state(&mut marked_pane, 0 as core::ffi::c_int);
        marked_pane.set_session(s);
        marked_pane.set_winlink(wl);
        if let Some(wl) = wl {
            marked_pane.set_window_ref(wl.window_handle());
        }
        marked_pane.set_pane(wp);
    }
}
pub fn server_clear_marked() {
    unsafe {
        cmd_find_clear_state(&mut marked_pane, 0 as core::ffi::c_int);
    }
}
pub unsafe fn server_is_marked(
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) -> core::ffi::c_int {
    unsafe {
        let (Some(s), Some(wl), Some(wp)) = (s, wl, wp) else {
            return 0;
        };
        let (Some(session), Some(link), Some(pane)) = (
            marked_pane.session(),
            marked_pane.winlink_ref(),
            marked_pane.pane_ref(),
        ) else {
            return 0;
        };
        if !session.points_to(s)
            || !link.get().is_some_and(|marked| core::ptr::eq(marked, wl))
            || !pane
                .get()
                .is_some_and(|marked| core::ptr::addr_eq(marked, wp))
        {
            return 0;
        }
        server_check_marked()
    }
}
pub fn server_check_marked() -> core::ffi::c_int {
    unsafe { cmd_find_valid_state(&marked_pane) }
}
pub unsafe fn server_create_socket(
    flags: uint64_t,
    cause: &mut Option<CString>,
) -> core::ffi::c_int {
    unsafe {
        let mut sa = sockaddr_un::default();
        let mask: mode_t;
        let fd: core::ffi::c_int;
        let saved_errno: core::ffi::c_int;
        let path = socket_path
            .as_deref()
            .expect("the server has a socket path");
        if !sa.set_unix_socket_address(AF_UNIX as sa_family_t, path) {
            *__errno_location() = ENAMETOOLONG;
        } else {
            let _ = fs::remove_file(OsStr::from_bytes(path.to_bytes()));
            fd = socket(
                AF_UNIX,
                SOCK_STREAM as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            if !(fd == -(1 as core::ffi::c_int)) {
                if flags & CLIENT_DEFAULTSOCKET as uint64_t != 0 {
                    mask = umask((S_IXUSR | S_IXGRP | S_IRWXO) as __mode_t) as mode_t;
                } else {
                    mask = umask((S_IXUSR | S_IRWXG | S_IRWXO) as __mode_t) as mode_t;
                }
                if bind(
                    fd,
                    (&raw const sa).cast(),
                    size_of::<sockaddr_un>() as socklen_t,
                ) == -(1 as core::ffi::c_int)
                {
                    saved_errno = *__errno_location();
                    close(fd);
                    *__errno_location() = saved_errno;
                } else {
                    umask(mask as __mode_t);
                    if listen(fd, 128 as core::ffi::c_int) == -(1 as core::ffi::c_int) {
                        saved_errno = *__errno_location();
                        close(fd);
                        *__errno_location() = saved_errno;
                    } else {
                        setblocking(fd, 0 as core::ffi::c_int);
                        return fd;
                    }
                }
            }
        }
        *cause = Some(xasprintf(
            c"error creating %s (%s)",
            fmt_args![
                socket_path.as_deref(),
                error_message(*__errno_location()).as_c_str()
            ],
        ));
        -(1 as core::ffi::c_int)
    }
}
unsafe fn server_tidy_event() {
    unsafe {
        let tv = timeval::from_secs(3600 as __time_t);
        let t: uint64_t = get_timer();
        format_tidy_jobs();
        malloc_trim(0 as size_t);
        log_debug(
            c"%s: took %llu milliseconds",
            fmt_args![
                c"server_tidy_event",
                get_timer().wrapping_sub(t) as core::ffi::c_ulonglong
            ],
        );
        server_ev_tidy.arm(tv);
    }
}
pub unsafe fn server_start(
    client: &ProcessRef,
    flags: uint64_t,
    mut base: reactor::Base,
    lockfd: core::ffi::c_int,
    lockfile: Option<CString>,
) -> core::ffi::c_int {
    unsafe {
        let mut fd: core::ffi::c_int = 0;
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let mut c: Option<ClientRef> = None;
        let mut cause: Option<CString> = None;
        let tv = timeval::from_secs(3600 as __time_t);
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if !flags & CLIENT_NOFORK as uint64_t != 0 && {
            let forked;
            (forked, fd) = proc_fork_and_daemon();
            forked != 0 as core::ffi::c_int
        } {
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                core::ptr::null_mut::<sigset_t>(),
            );
            return fd;
        }
        proc_clear_signals(&mut client.borrow_mut(), 0 as core::ffi::c_int);
        server_client_flags = flags;
        if !base.reinit() {
            fatalx(c"reactor reinit failed", fmt_args![]);
        }
        server_proc = Some(proc_start(c"server"));
        proc_set_signals(
            &mut server_proc
                .as_ref()
                .expect("server process is initialized")
                .borrow_mut(),
            server_signal,
        );
        sigprocmask(
            SIG_SETMASK,
            &raw mut oldset,
            core::ptr::null_mut::<sigset_t>(),
        );
        if log_get_level() > 1 as core::ffi::c_int {
            tty_create_log();
        }
        input_key_build();
        utf8_update_width_cache(
            (global_options
                .as_ref()
                .expect("global options are initialized"))
            .codepoint_widths(),
        );
        key_bindings_init();
        start_time = timeval::now();
        server_fd = systemd_create_socket(flags as core::ffi::c_int, &mut cause);
        if server_fd != -(1 as core::ffi::c_int) {
            server_update_socket();
        }
        if !flags & CLIENT_NOFORK as uint64_t != 0 {
            c = Some(ClientRef::from_fd(fd));
        } else {
            (global_options
                .as_ref()
                .expect("global options are initialized"))
            .set_number(c"exit-empty", 0 as core::ffi::c_longlong);
        }
        if lockfd >= 0 as core::ffi::c_int {
            if let Some(lockfile) = lockfile {
                let _ = fs::remove_file(OsStr::from_bytes(lockfile.to_bytes()));
            }
            close(lockfd);
        }
        if let Some(cause) = cause {
            if let Some(mut c) = c {
                c.as_client_mut().exit_message = Some(cause);
                *c.flags_mut() |= CLIENT_EXIT as uint64_t;
            } else {
                let _ = std::io::stderr()
                    .lock()
                    .write_all(&[cause.to_bytes(), b"\n"].concat());
                reactor::shutdown();
                exit(1 as core::ffi::c_int);
            }
        }
        server_ev_tidy.set_callback(move || {
            server_tidy_event();
        });
        server_ev_tidy.arm(tv);
        crate::plugin::init();
        server_default_options();
        server_acl_init();
        server_add_accept(0 as core::ffi::c_int);
        proc_loop(
            &server_proc
                .as_ref()
                .expect("server process is initialized")
                .clone(),
            || server_loop() != 0,
        );
        job_kill_all();
        status_prompt_save_history();
        reactor::shutdown();
        exit(0 as core::ffi::c_int);
    }
}
fn server_loop() -> core::ffi::c_int {
    unsafe {
        let mut items: u_int;
        current_time = time(core::ptr::null_mut::<time_t>());
        loop {
            items = cmdq_next(None);
            for c in client_walk() {
                if c.flags() & CLIENT_IDENTIFIED as uint64_t != 0 {
                    items = items.wrapping_add(cmdq_next(Some(&c)));
                }
            }
            if !(items != 0 as u_int) {
                break;
            }
        }
        server_client_loop();
        if (global_options
            .as_ref()
            .expect("global options are initialized"))
        .number(c"exit-empty")
            == 0
            && server_exit == 0
        {
            return 0 as core::ffi::c_int;
        }
        if (global_options
            .as_ref()
            .expect("global options are initialized"))
        .number(c"exit-unattached")
            == 0
            && !sessions_empty()
        {
            return 0 as core::ffi::c_int;
        }
        if with_clients(|clients| clients.iter().any(|c| c.attached_session().is_some())) {
            return 0 as core::ffi::c_int;
        }
        cmd_wait_for_flush();
        if with_clients(|clients| !clients.is_empty()) {
            return 0 as core::ffi::c_int;
        }
        if job_still_running() != 0 {
            return 0 as core::ffi::c_int;
        }
        1 as core::ffi::c_int
    }
}
fn server_send_exit() {
    unsafe {
        cmd_wait_for_flush();
        for mut c in client_walk_safe() {
            if c.flags() & CLIENT_SUSPENDED as uint64_t != 0 {
                (c.clone()).on_lost();
            } else {
                *c.flags_mut() |= CLIENT_EXIT as uint64_t;
                c.as_client_mut().exit_type = CLIENT_EXIT_SHUTDOWN;
            }
            c.set_attached_session(None);
        }
        for s in SESSIONS.walk_safe() {
            s.destroy(1 as core::ffi::c_int, c"server_send_exit");
        }
    }
}
pub fn server_update_socket() {
    unsafe {
        static mut last: core::ffi::c_int = -(1 as core::ffi::c_int);
        let mut mode: core::ffi::c_int;
        let n = SESSIONS.read().values().any(|s| s.attached() != 0 as u_int) as core::ffi::c_int;
        if n != last {
            last = n;
            let Some(path) = socket_path
                .as_deref()
                .map(|path| OsStr::from_bytes(path.to_bytes()))
            else {
                return;
            };
            let Ok(metadata) = fs::metadata(path) else {
                return;
            };
            mode = (metadata.permissions().mode() & ACCESSPERMS as u32) as core::ffi::c_int;
            if n != 0 as core::ffi::c_int {
                if mode & S_IRUSR != 0 {
                    mode |= S_IXUSR;
                }
                if mode & S_IRGRP != 0 {
                    mode |= S_IXGRP;
                }
                if mode & S_IROTH != 0 {
                    mode |= S_IXOTH;
                }
            } else {
                mode &= !(S_IXUSR | S_IXGRP | S_IXOTH);
            }
            let _ = fs::set_permissions(path, Permissions::from_mode(mode as u32));
        }
    }
}
unsafe fn server_accept(fd: core::ffi::c_int, _events: core::ffi::c_short) {
    unsafe {
        let mut sa: sockaddr_storage = core::mem::zeroed();
        let mut slen: socklen_t = size_of::<sockaddr_storage>() as socklen_t;

        server_add_accept(0 as core::ffi::c_int);
        let newfd: core::ffi::c_int = accept(fd, (&raw mut sa).cast(), &raw mut slen);
        if newfd == -(1 as core::ffi::c_int) {
            if *__errno_location() == EAGAIN
                || *__errno_location() == EINTR
                || *__errno_location() == ECONNABORTED
            {
                return;
            }
            if *__errno_location() == ENFILE || *__errno_location() == EMFILE {
                server_add_accept(1 as core::ffi::c_int);
                return;
            }
            fatal(c"accept failed", fmt_args![]);
        }
        if server_exit != 0 {
            close(newfd);
            return;
        }
        let mut c = ClientRef::from_fd(newfd);
        if server_acl_join(c.as_client_mut()) == 0 {
            c.as_client_mut().exit_message = Some(c"access not allowed".to_owned());
            *c.flags_mut() |= CLIENT_EXIT as uint64_t;
        }
    }
}
/// Takes both halves of the accept watch off the loop. The socket is watched
/// for a connection or, while the server is out of descriptors, left alone
/// until a timer says to look again; only one of the two is ever on, and this
/// is what takes whichever it is off.
fn server_stop_accept() {
    unsafe {
        server_ev_accept.disable();
        server_ev_accept_timer.disarm();
    }
}

/// The end of the pause `server_add_accept` starts when `accept` runs the
/// server out of descriptors: it just goes back to watching the socket.
unsafe fn server_accept_timer() {
    {
        server_add_accept(0 as core::ffi::c_int);
    }
}

pub fn server_add_accept(timeout: core::ffi::c_int) {
    unsafe {
        let tv = timeval::from_secs(timeout as __time_t);
        if server_fd == -(1 as core::ffi::c_int) {
            return;
        }
        server_stop_accept();
        if timeout == 0 as core::ffi::c_int {
            server_ev_accept.set_callback(
                server_fd,
                Interest::Read,
                WatchMode::Once,
                move |fd, events| server_accept(fd, events),
            );
            server_ev_accept.enable();
        } else {
            server_ev_accept_timer.set_callback(move || {
                server_accept_timer();
            });
            server_ev_accept_timer.arm(tv);
        };
    }
}
fn server_signal(sig: core::ffi::c_int) {
    unsafe {
        let fd: core::ffi::c_int;
        log_debug(
            c"%s: %s",
            fmt_args![c"server_signal", signal_description(sig).as_deref()],
        );
        match sig {
            SIGINT | SIGTERM => {
                server_exit = 1 as core::ffi::c_int;
                server_send_exit();
            }
            SIGCHLD => {
                server_child_signal();
            }
            SIGUSR1 => {
                server_stop_accept();
                fd = server_create_socket(server_client_flags, &mut None);
                if fd != -(1 as core::ffi::c_int) {
                    close(server_fd);
                    server_fd = fd;
                    server_update_socket();
                }
                server_add_accept(0 as core::ffi::c_int);
            }
            SIGUSR2 => {
                proc_toggle_log(
                    &mut server_proc
                        .as_ref()
                        .expect("server process is initialized")
                        .borrow_mut(),
                );
            }
            _ => {}
        };
    }
}
fn server_child_signal() {
    unsafe {
        let mut status: core::ffi::c_int = 0;
        let mut pid: pid_t;
        loop {
            pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG | WUNTRACED) as pid_t;
            match pid {
                -1 => {
                    if *__errno_location() == ECHILD {
                        return;
                    }
                    fatal(c"waitpid failed", fmt_args![]);
                }
                0 => return,
                _ => {}
            }
            if status & 0xff as core::ffi::c_int == 0x7f as core::ffi::c_int {
                server_child_stopped(pid, status);
            } else if status & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int
                || ((status & 0x7f as core::ffi::c_int) + 1 as core::ffi::c_int)
                    as core::ffi::c_schar as core::ffi::c_int
                    >> 1 as core::ffi::c_int
                    > 0 as core::ffi::c_int
            {
                server_child_exited(pid, status);
            }
        }
    }
}
fn server_child_exited(pid: pid_t, status: core::ffi::c_int) {
    WINDOWS.with(|windows| unsafe {
        for owner in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            let mut window = owner.as_window_mut();
            let Some(pane) = window
                .panes
                .iter_mut()
                .find(|pane| *pane.as_pane().pid() == pid)
            else {
                continue;
            };
            let wp = pane.as_pane_mut();
            let pane_id = wp.pane_id();
            wp.set_exit_status(status);
            *wp.flags_mut() |= PANE_STATUSREADY;
            log_debug(c"%%%u exited", fmt_args![pane_id]);
            *wp.flags_mut() |= PANE_EXITED;
            if window_pane_destroy_ready(wp) != 0 {
                let pane = crate::window::window_pane_ref_of(wp).expect("the pane is owned");
                drop(window);
                server_destroy_pane(&pane, 1);
            }
        }
        job_check_died(pid, status);
    });
}
fn server_child_stopped(pid: pid_t, status: core::ffi::c_int) {
    WINDOWS.with(|windows| unsafe {
        if (status & 0xff00) >> 8 == SIGTTIN || (status & 0xff00) >> 8 == SIGTTOU {
            return;
        }
        for owner in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            for pane in &owner.as_window().panes {
                if *pane.as_pane().pid() == pid && killpg(pid, SIGCONT) != 0 {
                    kill(pid, SIGCONT);
                }
            }
        }
        job_check_died(pid, status);
    });
}
pub(crate) unsafe fn server_add_message(fmt: &CStr, args: &[FmtArg]) {
    unsafe {
        let s = format_alloc(fmt, args);
        log_debug(c"message: %s", fmt_args![s.as_c_str()]);
        let limit = (global_options
            .as_ref()
            .expect("global options are initialized"))
        .number(c"message-limit") as u_int;
        with_message_log_mut(|log| log.add(&s, limit));
    }
}

#[cfg(test)]
#[path = "../tests/test_server_run_focused.rs"]
mod focused_tests;

/// Toggles or clears the marked target and refreshes both affected panes.
///
/// Keeps the existing validity check for the previous mark while still refreshing
/// the resulting mark. Floating-pane selection remains caller policy.
///
/// # Safety
/// The link and pane must be live resolved targets. Run on the server thread
/// without conflicting session, window, pane or marked-target access. Notifications
/// remain deferred; no queue hooks are dispatched inline.
pub(crate) unsafe fn server_toggle_marked_pane(
    link: &crate::window::WinlinkRef,
    pane: &RustWindowPaneWeak,
    clear: bool,
) {
    unsafe {
        let previous = if server_check_marked() != 0 {
            marked_pane.pane_ref()
        } else {
            None
        };
        if clear || server_is_marked(Some(link.session().as_session()), link.get(), pane.get()) != 0
        {
            server_clear_marked();
        } else {
            server_set_marked(Some(link.session().as_session()), link.get(), pane.get());
        }
        for marked in [previous, marked_pane.pane_ref()].into_iter().flatten() {
            marked.add_flags(
                crate::window::PANE_REDRAW
                    | crate::window::PANE_STYLECHANGED
                    | crate::window::PANE_THEMECHANGED,
            );
            if let Some(window) = marked.window() {
                window.redraw_borders();
                window.redraw_status();
            }
        }
    }
}

#[cfg(test)]
pub use crate::consts::{
    MSG_COMMAND, MSG_FLAGS, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_TERM, MSG_READ, MSG_READ_DONE, MSG_READ_OPEN, MSG_VERSION, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, PANE_LINES_DOUBLE, PANE_LINES_SINGLE,
};
