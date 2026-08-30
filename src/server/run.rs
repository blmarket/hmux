use super::acl::server_acl_init;
use super::acl::server_acl_join;
use super::client::server_client_loop;
use super::client::{server_client_create, server_client_lost};
use super::message::server_destroy_pane;
use crate::cmd::cmd_wait_for_flush;
use crate::cmd::cmdq_next;
use crate::cmd::{cmd_find_clear_state, cmd_find_valid_state};
use crate::compat::systemd_create_socket;
use crate::ffi::{
    __errno_location, accept, bind, close, exit, fprintf, gettimeofday, kill, killpg, listen,
    malloc_trim, sigfillset, sigprocmask, socket, stderr, strerror, strlcpy, strsignal, time,
    umask, waitpid,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_tidy_jobs;
use crate::input::input_key_build;
use crate::job::{job_check_died, job_kill_all, job_still_running};
use crate::key_bindings::key_bindings_init;
use crate::list::{Foreach, ForeachSafe, foreach_by, foreach_safe_by};
use crate::log::{fatal, fatalx, log_debug, log_get_level};
use crate::options::{options_codepoint_widths, options_get_number, options_set_number};
use crate::proc::proc_fork_and_daemon;
use crate::proc::{proc_clear_signals, proc_loop, proc_set_signals, proc_start, proc_toggle_log};
use crate::reactor;
use crate::reactor::{Interest, IoWatch, Reactor, Timer, WatchMode};
use crate::session::{session_attached, session_destroy, sessions, sessions_after, sessions_first};
use crate::status::status_prompt_save_history;
use crate::text::utf8_update_width_cache;
use crate::tmux::{get_timer, setblocking};
use crate::tmux::{global_options, socket_path, start_time};
use crate::tree::GlobalQueue;
use crate::tty::tty_create_log;
pub use crate::types::*;
use crate::window::pane_registry_clear;
use crate::window::{
    window_find_by_id_ref, window_pane_destroy_ready, window_panes_first, window_panes_next,
    windows,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::{CString, OsStr};
use ::std::fs::{self, Permissions};
use ::std::os::unix::ffi::OsStrExt;
use ::std::os::unix::fs::PermissionsExt;
pub type mode_t = __mode_t;
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
pub const __S_IREAD: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const __S_IWRITE: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const __S_IEXEC: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;
pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;
pub const SIGINT: ::core::ffi::c_int = 2;
pub const SIGTERM: ::core::ffi::c_int = 15;
pub const ACCESSPERMS: ::core::ffi::c_int = S_IRWXU | S_IRWXG | S_IRWXO;
pub const WNOHANG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const WUNTRACED: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SIGCONT: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const SIGCHLD: ::core::ffi::c_int = 17;
pub const SIGTTIN: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const SIGTTOU: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const SIGUSR1: ::core::ffi::c_int = 10;
pub const SIGUSR2: ::core::ffi::c_int = 12;
pub const SIG_BLOCK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIG_SETMASK: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENAMETOOLONG: ::core::ffi::c_int = 36 as ::core::ffi::c_int;
pub const ECONNABORTED: ::core::ffi::c_int = 103 as ::core::ffi::c_int;
pub const WAIT_ANY: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ECHILD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const EAGAIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const ENFILE: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const EMFILE: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const S_IRUSR: ::core::ffi::c_int = __S_IREAD;
pub const S_IXUSR: ::core::ffi::c_int = __S_IEXEC;
pub const S_IRWXU: ::core::ffi::c_int = __S_IREAD | __S_IWRITE | __S_IEXEC;
pub const S_IRGRP: ::core::ffi::c_int = S_IRUSR >> 3 as ::core::ffi::c_int;
pub const S_IXGRP: ::core::ffi::c_int = S_IXUSR >> 3 as ::core::ffi::c_int;
pub const S_IRWXG: ::core::ffi::c_int = S_IRWXU >> 3 as ::core::ffi::c_int;
pub const S_IROTH: ::core::ffi::c_int = S_IRGRP >> 3 as ::core::ffi::c_int;
pub const S_IXOTH: ::core::ffi::c_int = S_IXGRP >> 3 as ::core::ffi::c_int;
pub const S_IRWXO: ::core::ffi::c_int = S_IRWXG >> 3 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_IDENTIFIED: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const CLIENT_DEFAULTSOCKET: ::core::ffi::c_int = 0x8000000 as ::core::ffi::c_int;
pub const CLIENT_NOFORK: ::core::ffi::c_int = 0x40000000 as ::core::ffi::c_int;
pub(crate) static mut clients: clients_t = clients_t::new();

fn client_at(list: &clients_t, at: usize) -> Option<*mut client> {
    list.get(at).map(ClientRef::as_ptr)
}

/// Walks every client the server holds, oldest first, the way [`foreach`]
/// walks a list: the client after the one the body has just had is read out
/// of the list again afterwards, so the body may connect or lose clients as
/// the walk runs.
pub(crate) fn client_walk() -> Foreach<clients_t, client> {
    unsafe { foreach_by(&raw mut clients, client_at) }
}

/// The oldest client the server holds, or null when it holds none. This is
/// the one the config load and its causes belong to.
pub fn first_client() -> *mut client {
    unsafe {
        clients
            .first()
            .map_or(::core::ptr::null_mut(), ClientRef::as_ptr)
    }
}

/// [`client_walk`] the way [`foreach_safe`] walks a list: the successor is
/// taken before the body runs, so the body may lose the client it has been
/// given.
pub(crate) fn client_walk_safe() -> ForeachSafe<clients_t, client> {
    unsafe { foreach_safe_by(&raw mut clients, client_at) }
}
pub static mut server_proc: *mut tmuxproc = ::core::ptr::null::<tmuxproc>() as *mut tmuxproc;
static mut server_fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
static mut server_client_flags: uint64_t = 0;
static mut server_exit: ::core::ffi::c_int = 0;
static mut server_ev_accept: IoHandle = IoHandle::ZERO;
static mut server_ev_accept_timer: TimerHandle = TimerHandle::ZERO;
static mut server_ev_tidy: TimerHandle = TimerHandle::ZERO;
pub static mut marked_pane: cmd_find_state = cmd_find_state {
    flags: 0,
    s_ref: None,
    wl_idx: None,
    w_ref: None,
    wp_id: None,
    idx: 0,
};
static mut message_next: u_int = 0;
pub static message_log: GlobalQueue<message_entry> = GlobalQueue::new();
pub static mut current_time: time_t = 0;
pub unsafe fn server_set_marked(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    unsafe {
        cmd_find_clear_state(&mut marked_pane, 0 as ::core::ffi::c_int);
        marked_pane.set_session(s);
        marked_pane.set_winlink(wl);
        if !wl.is_null() {
            marked_pane.set_window((*wl).window());
        }
        marked_pane.set_pane(wp);
    }
}
pub fn server_clear_marked() {
    unsafe {
        cmd_find_clear_state(&mut marked_pane, 0 as ::core::ffi::c_int);
    }
}
pub unsafe fn server_is_marked(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    unsafe {
        if s.is_null() || wl.is_null() || wp.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if marked_pane.session() != s || marked_pane.winlink() != wl {
            return 0 as ::core::ffi::c_int;
        }
        if marked_pane.pane() != wp {
            return 0 as ::core::ffi::c_int;
        }
        server_check_marked()
    }
}
pub fn server_check_marked() -> ::core::ffi::c_int {
    unsafe { cmd_find_valid_state(&marked_pane) }
}
pub unsafe fn server_create_socket(
    mut flags: uint64_t,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut sa = sockaddr_un::default();
        let mut size: size_t = 0;
        let mut mask: mode_t = 0;
        let mut fd: ::core::ffi::c_int = 0;
        let mut saved_errno: ::core::ffi::c_int = 0;
        sa.sun_family = AF_UNIX as sa_family_t;
        size = strlcpy(
            &raw mut sa.sun_path as *mut ::core::ffi::c_char,
            socket_path,
            ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as size_t,
        ) as size_t;
        if size >= ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as usize {
            *__errno_location() = ENAMETOOLONG;
        } else {
            let _ = fs::remove_file(OsStr::from_bytes(
                CStr::from_ptr(&raw const sa.sun_path as *const ::core::ffi::c_char).to_bytes(),
            ));
            fd = socket(
                AF_UNIX,
                SOCK_STREAM as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if !(fd == -(1 as ::core::ffi::c_int)) {
                if flags & CLIENT_DEFAULTSOCKET as uint64_t != 0 {
                    mask = umask((S_IXUSR | S_IXGRP | S_IRWXO) as __mode_t) as mode_t;
                } else {
                    mask = umask((S_IXUSR | S_IRWXG | S_IRWXO) as __mode_t) as mode_t;
                }
                if bind(
                    fd,
                    __CONST_SOCKADDR_ARG {
                        __sockaddr__: &raw mut sa as *mut sockaddr,
                    },
                    ::core::mem::size_of::<sockaddr_un>() as socklen_t,
                ) == -(1 as ::core::ffi::c_int)
                {
                    saved_errno = *__errno_location();
                    close(fd);
                    *__errno_location() = saved_errno;
                } else {
                    umask(mask as __mode_t);
                    if listen(fd, 128 as ::core::ffi::c_int) == -(1 as ::core::ffi::c_int) {
                        saved_errno = *__errno_location();
                        close(fd);
                        *__errno_location() = saved_errno;
                    } else {
                        setblocking(fd, 0 as ::core::ffi::c_int);
                        return fd;
                    }
                }
            }
        }
        *cause = Some(xasprintf(
            c"error creating %s (%s)".as_ptr(),
            fmt_args![socket_path, strerror(*__errno_location())],
        ));
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn server_tidy_event() {
    unsafe {
        let mut tv = timeval::from_secs(3600 as __time_t);
        let mut t: uint64_t = get_timer();
        format_tidy_jobs();
        malloc_trim(0 as size_t);
        log_debug(
            c"%s: took %llu milliseconds".as_ptr(),
            fmt_args![
                c"server_tidy_event".as_ptr(),
                get_timer().wrapping_sub(t) as ::core::ffi::c_ulonglong
            ],
        );
        server_ev_tidy.arm(tv);
    }
}
pub unsafe fn server_start(
    mut client: *mut tmuxproc,
    mut flags: uint64_t,
    mut base: reactor::Base,
    mut lockfd: ::core::ffi::c_int,
    mut lockfile: Option<CString>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fd: ::core::ffi::c_int = 0;
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        let mut cause: Option<CString> = None;
        let mut tv = timeval::from_secs(3600 as __time_t);
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if !flags & CLIENT_NOFORK as uint64_t != 0 && {
            let forked;
            (forked, fd) = proc_fork_and_daemon();
            forked != 0 as ::core::ffi::c_int
        } {
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            return fd;
        }
        proc_clear_signals(client, 0 as ::core::ffi::c_int);
        server_client_flags = flags;
        if !base.reinit() {
            fatalx(c"reactor reinit failed".as_ptr(), fmt_args![]);
        }
        server_proc = proc_start(c"server".as_ptr());
        proc_set_signals(server_proc, Some(server_signal));
        sigprocmask(
            SIG_SETMASK,
            &raw mut oldset,
            ::core::ptr::null_mut::<sigset_t>(),
        );
        if log_get_level() > 1 as ::core::ffi::c_int {
            tty_create_log();
        }
        input_key_build();
        utf8_update_width_cache(options_codepoint_widths(global_options));
        windows.map().clear();
        pane_registry_clear();
        super::client::client_registry_clear();
        clients.clear();
        crate::session::session_registry_clear();
        key_bindings_init();
        message_log.queue().clear();
        gettimeofday(&raw mut start_time, ::core::ptr::null_mut());
        server_fd = systemd_create_socket(flags as ::core::ffi::c_int, &mut cause);
        if server_fd != -(1 as ::core::ffi::c_int) {
            server_update_socket();
        }
        if !flags & CLIENT_NOFORK as uint64_t != 0 {
            c = server_client_create(fd);
        } else {
            options_set_number(
                global_options,
                c"exit-empty".as_ptr(),
                0 as ::core::ffi::c_longlong,
            );
        }
        if lockfd >= 0 as ::core::ffi::c_int {
            if let Some(lockfile) = lockfile {
                let _ = fs::remove_file(OsStr::from_bytes(lockfile.to_bytes()));
            }
            close(lockfd);
        }
        if let Some(cause) = cause {
            if !c.is_null() {
                (*c).exit_message = Some(cause);
                (*c).flags |= CLIENT_EXIT as uint64_t;
            } else {
                fprintf(stderr, c"%s\n".as_ptr(), cause.as_ptr());
                crate::reactor::shutdown();
                exit(1 as ::core::ffi::c_int);
            }
        }
        server_ev_tidy.set_callback(move || {
            server_tidy_event();
        });
        server_ev_tidy.arm(tv);
        crate::plugin::init();
        server_acl_init();
        server_add_accept(0 as ::core::ffi::c_int);
        proc_loop(server_proc, Some(server_loop));
        job_kill_all();
        status_prompt_save_history();
        crate::reactor::shutdown();
        exit(0 as ::core::ffi::c_int);
    }
}
fn server_loop() -> ::core::ffi::c_int {
    unsafe {
        let mut items: u_int = 0;
        current_time = time(::core::ptr::null_mut::<time_t>());
        loop {
            items = cmdq_next(::core::ptr::null_mut::<client>());
            for c in client_walk() {
                if (*c).flags & CLIENT_IDENTIFIED as uint64_t != 0 {
                    items = items.wrapping_add(cmdq_next(c));
                }
            }
            if !(items != 0 as u_int) {
                break;
            }
        }
        server_client_loop();
        if options_get_number(global_options, c"exit-empty".as_ptr()) == 0 && server_exit == 0 {
            return 0 as ::core::ffi::c_int;
        }
        if options_get_number(global_options, c"exit-unattached".as_ptr()) == 0
            && !sessions.map().is_empty()
        {
            return 0 as ::core::ffi::c_int;
        }
        for c in client_walk() {
            if !(*c).session.is_null() {
                return 0 as ::core::ffi::c_int;
            }
        }
        cmd_wait_for_flush();
        if !clients.is_empty() {
            return 0 as ::core::ffi::c_int;
        }
        if job_still_running() != 0 {
            return 0 as ::core::ffi::c_int;
        }
        1 as ::core::ffi::c_int
    }
}
fn server_send_exit() {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut s1: *mut session = ::core::ptr::null_mut::<session>();
        cmd_wait_for_flush();
        for c in client_walk_safe() {
            if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
                server_client_lost(c);
            } else {
                (*c).flags |= CLIENT_EXIT as uint64_t;
                (*c).exit_type = CLIENT_EXIT_SHUTDOWN;
            }
            (*c).session = ::core::ptr::null_mut::<session>();
        }
        s = sessions_first();
        while !s.is_null() && {
            s1 = sessions_after(s);
            1 as ::core::ffi::c_int != 0
        } {
            session_destroy(s, 1 as ::core::ffi::c_int, c"server_send_exit".as_ptr());
            s = s1;
        }
    }
}
pub fn server_update_socket() {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        static mut last: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
        let mut n: ::core::ffi::c_int = 0;
        let mut mode: ::core::ffi::c_int = 0;
        n = 0 as ::core::ffi::c_int;
        s = sessions_first();
        while !s.is_null() {
            if session_attached(s) != 0 as u_int {
                n += 1;
                break;
            } else {
                s = sessions_after(s);
            }
        }
        if n != last {
            last = n;
            let path = OsStr::from_bytes(CStr::from_ptr(socket_path).to_bytes());
            let Ok(metadata) = fs::metadata(path) else {
                return;
            };
            mode = (metadata.permissions().mode() & ACCESSPERMS as u32) as ::core::ffi::c_int;
            if n != 0 as ::core::ffi::c_int {
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
unsafe fn server_accept(mut fd: ::core::ffi::c_int, _events: ::core::ffi::c_short) {
    unsafe {
        let mut sa: sockaddr_storage = ::core::mem::zeroed();
        let mut slen: socklen_t = ::core::mem::size_of::<sockaddr_storage>() as socklen_t;
        let mut newfd: ::core::ffi::c_int = 0;
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        server_add_accept(0 as ::core::ffi::c_int);
        newfd = accept(
            fd,
            __SOCKADDR_ARG {
                __sockaddr__: &raw mut sa as *mut sockaddr,
            },
            &raw mut slen,
        );
        if newfd == -(1 as ::core::ffi::c_int) {
            if *__errno_location() == EAGAIN
                || *__errno_location() == EINTR
                || *__errno_location() == ECONNABORTED
            {
                return;
            }
            if *__errno_location() == ENFILE || *__errno_location() == EMFILE {
                server_add_accept(1 as ::core::ffi::c_int);
                return;
            }
            fatal(c"accept failed".as_ptr(), fmt_args![]);
        }
        if server_exit != 0 {
            close(newfd);
            return;
        }
        c = server_client_create(newfd);
        if server_acl_join(c) == 0 {
            (*c).exit_message = Some(c"access not allowed".to_owned());
            (*c).flags |= CLIENT_EXIT as uint64_t;
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
        server_add_accept(0 as ::core::ffi::c_int);
    }
}

pub fn server_add_accept(mut timeout: ::core::ffi::c_int) {
    unsafe {
        let mut tv = timeval::from_secs(timeout as __time_t);
        if server_fd == -(1 as ::core::ffi::c_int) {
            return;
        }
        server_stop_accept();
        if timeout == 0 as ::core::ffi::c_int {
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
fn server_signal(mut sig: ::core::ffi::c_int) {
    unsafe {
        let mut fd: ::core::ffi::c_int = 0;
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"server_signal".as_ptr(), strsignal(sig)],
        );
        match sig {
            SIGINT | SIGTERM => {
                server_exit = 1 as ::core::ffi::c_int;
                server_send_exit();
            }
            SIGCHLD => {
                server_child_signal();
            }
            SIGUSR1 => {
                server_stop_accept();
                fd = server_create_socket(server_client_flags, &mut None);
                if fd != -(1 as ::core::ffi::c_int) {
                    close(server_fd);
                    server_fd = fd;
                    server_update_socket();
                }
                server_add_accept(0 as ::core::ffi::c_int);
            }
            SIGUSR2 => {
                proc_toggle_log(server_proc);
            }
            _ => {}
        };
    }
}
fn server_child_signal() {
    unsafe {
        let mut status: ::core::ffi::c_int = 0;
        let mut pid: pid_t = 0;
        loop {
            pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG | WUNTRACED) as pid_t;
            match pid {
                -1 => {
                    if *__errno_location() == ECHILD {
                        return;
                    }
                    fatal(c"waitpid failed".as_ptr(), fmt_args![]);
                }
                0 => return,
                _ => {}
            }
            if status & 0xff as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
                server_child_stopped(pid, status);
            } else if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_schar as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    > 0 as ::core::ffi::c_int
            {
                server_child_exited(pid, status);
            }
        }
    }
}
fn server_child_exited(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let ids: Vec<u_int> = windows.map().keys().copied().collect();
        for id in ids {
            let Some(w_ref) = window_find_by_id_ref(id) else {
                continue;
            };
            let w = w_ref.as_ptr();
            wp = window_panes_first(w);
            while !wp.is_null() {
                if (*wp).pid == pid {
                    (*wp).status = status;
                    (*wp).flags |= PANE_STATUSREADY;
                    log_debug(c"%%%u exited".as_ptr(), fmt_args![(*wp).id]);
                    (*wp).flags |= PANE_EXITED;
                    if window_pane_destroy_ready(wp) != 0 {
                        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
                    }
                    break;
                } else {
                    wp = window_panes_next(w, wp);
                }
            }
        }
        job_check_died(pid, status);
    }
}
fn server_child_stopped(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        if (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTIN
            || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTOU
        {
            return;
        }
        let ids: Vec<u_int> = windows.map().keys().copied().collect();
        for id in ids {
            let Some(w_ref) = window_find_by_id_ref(id) else {
                continue;
            };
            let w = w_ref.as_ptr();
            wp = window_panes_first(w);
            while !wp.is_null() {
                if (*wp).pid == pid && killpg(pid as __pid_t, SIGCONT) != 0 as ::core::ffi::c_int {
                    kill(pid as __pid_t, SIGCONT);
                }
                wp = window_panes_next(w, wp);
            }
        }
        job_check_died(pid, status);
    }
}
pub unsafe fn server_add_message(mut fmt: *const ::core::ffi::c_char, args: &[FmtArg]) {
    unsafe {
        let mut msg_time = timeval::default();
        let mut limit: u_int = 0;
        let s = format_alloc(fmt, args);
        log_debug(c"message: %s".as_ptr(), fmt_args![s.as_ptr()]);
        gettimeofday(&raw mut msg_time, ::core::ptr::null_mut());
        let fresh0 = message_next;
        message_next = message_next.wrapping_add(1);
        message_log.queue().push_back(message_entry {
            msg: s.clone(),
            msg_num: fresh0,
            msg_time,
        });
        limit = options_get_number(global_options, c"message-limit".as_ptr()) as u_int;
        while message_log
            .queue()
            .front()
            .is_some_and(|msg| msg.msg_num.wrapping_add(limit) < message_next)
        {
            message_log.queue().pop_front();
        }
    }
}
