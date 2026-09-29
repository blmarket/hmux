use crate::src::cmd::entries::wait_for::cmd_wait_for_flush;
use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_valid_state};
use crate::src::cmd::queue::cmdq_next;
use crate::src::compat::systemd::systemd_create_socket;
use crate::src::control_notify::control_build_events;
use crate::src::ffi::libc::{
    __errno_location, accept, bind, chmod, close, exit, fprintf, gettimeofday, kill, killpg,
    listen, malloc_trim, memset, sigfillset, sigprocmask, socket, stat, stderr, strerror, strlcpy,
    strsignal, time, umask, unlink, waitpid,
};
use crate::src::format::bytes::format_message_with;
use crate::src::format::format_tidy_jobs;
use crate::src::hooks::hooks_build_events;
use crate::src::input_keys::input_key_build;
use crate::src::job::{job_check_died, job_kill_all, job_still_running};
use crate::src::key_bindings::key_bindings_init;
use crate::src::log::{fatal, fatalx, log_cstr, log_debug, log_get_level};
use crate::src::options::{options_get_number, options_set_number};
use crate::src::proc::{
    proc_clear_signals, proc_fork_and_daemon, proc_loop, proc_set_signals, proc_start,
    proc_toggle_log,
};
use crate::src::prompt_history::prompt_save_history;
use crate::src::reactor::{event_add, event_del, event_initialized, event_reinit, event_set};
use crate::src::server_acl::{server_acl_init, server_acl_join};
use crate::src::server_client::Client as _;
use crate::src::server_client::{server_client_create, server_client_loop, server_client_lost};
use crate::src::server_fn::server_destroy_pane;
use crate::src::session::sessions;
use crate::src::session::Session;
use crate::src::session::{session_destroy, sessions_after, sessions_minmax};
use crate::src::shared::client::ClientRef;
use crate::src::shared::status::message_list;
use crate::src::spawn::spawn_editor_finish;
use crate::src::text::utf8::utf8_update_width_cache;
use crate::src::tmux::{get_timer, global_options, setblocking, socket_path, start_time};
use crate::src::tty::tty_create_log;
use crate::src::window::windows;
use crate::src::window::{
    all_window_panes, window_pane_destroy_ready, window_pane_first, window_pane_next,
    window_pane_wait_finish, windows_minmax, windows_next,
};

use std::ffi::{CStr, CString};

unsafe fn server_clear_messages() {
    message_log.clear();
}

pub use crate::src::server_client::clients;
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__mode_t, socklen_t};
use crate::src::shared::client::client;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    CLIENT_DEFAULTSOCKET, CLIENT_EXIT, CLIENT_IDENTIFIED, CLIENT_NOFORK, CLIENT_SUSPENDED,
};
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::errno::{EAGAIN, ECHILD, EINTR, ENAMETOOLONG};
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_TIMEOUT};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_EXITED, PANE_STATUSREADY};
use crate::src::shared::posix_io::stat;
use crate::src::shared::posix_io::{__S_IEXEC, __S_IREAD, S_IRWXU, WAIT_ANY, WNOHANG};
use crate::src::shared::process::tmuxproc;
use crate::src::shared::session::session;
use crate::src::shared::signal::{
    __sigset_t, sigset_t, ProcessSignal, SIGCHLD, SIGCONT, SIGINT, SIGTERM, SIGTTIN, SIGTTOU,
    SIGUSR1, SIGUSR2, SIG_BLOCK, SIG_SETMASK,
};
use crate::src::shared::socket::{
    sa_family_t, sockaddr, sockaddr_un, __CONST_SOCKADDR_ARG, __SOCKADDR_ARG, AF_UNIX, SOCK_STREAM,
};
use crate::src::shared::time::timespec;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{window, winlink};

pub type mode_t = __mode_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    pub __ss_padding: [::core::ffi::c_char; 118],
    pub __ss_align: ::core::ffi::c_ulong,
}

pub const ACCESSPERMS: ::core::ffi::c_int = S_IRWXU | S_IRWXG | S_IRWXO;

pub const WUNTRACED: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const ECONNABORTED: ::core::ffi::c_int = 103 as ::core::ffi::c_int;

pub const ENFILE: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const EMFILE: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const S_IRUSR: ::core::ffi::c_int = __S_IREAD;
pub const S_IXUSR: ::core::ffi::c_int = __S_IEXEC;

pub const S_IRGRP: ::core::ffi::c_int = S_IRUSR >> 3 as ::core::ffi::c_int;
pub const S_IXGRP: ::core::ffi::c_int = S_IXUSR >> 3 as ::core::ffi::c_int;
pub const S_IRWXG: ::core::ffi::c_int = S_IRWXU >> 3 as ::core::ffi::c_int;
pub const S_IROTH: ::core::ffi::c_int = S_IRGRP >> 3 as ::core::ffi::c_int;
pub const S_IXOTH: ::core::ffi::c_int = S_IXGRP >> 3 as ::core::ffi::c_int;
pub const S_IRWXO: ::core::ffi::c_int = S_IRWXG >> 3 as ::core::ffi::c_int;
pub static mut server_proc: *mut tmuxproc = ::core::ptr::null::<tmuxproc>() as *mut tmuxproc;
static mut server_fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
static mut server_client_flags: uint64_t = 0;
static mut server_exit: ::core::ffi::c_int = 0;
static mut server_ev_accept: event = event::new();
static mut server_ev_tidy: event = event::new();
pub static mut marked_pane: cmd_find_state = cmd_find_state {
    flags: 0,
    s: std::rc::Weak::new(),
    wl: refbox::Weak::new(),
    w: std::rc::Weak::new(),
    wp: std::rc::Weak::new(),
    idx: 0,
};
static mut message_next: u_int = 0;
// Filled through `message_list` methods; the head owns the collection.
pub static mut message_log: message_list = message_list::new();
pub static mut current_time: time_t = 0;
pub unsafe fn server_set_marked(
    s_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
    mut wl: refbox::Weak<winlink>,
    wp_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
) {
    let mut wp = wp_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    cmd_find_clear_state(&raw mut marked_pane, 0 as ::core::ffi::c_int);
    marked_pane.set_s(s_owner);
    marked_pane.set_wl(wl.clone());
    if wl.is_alive() {
        marked_pane.set_w(
            (wl.get_unchecked()
                .window_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .as_ref(),
        );
    }
    marked_pane.set_wp((wp).as_ref());
}
pub unsafe fn server_clear_marked() {
    cmd_find_clear_state(&raw mut marked_pane, 0 as ::core::ffi::c_int);
}
pub unsafe fn server_is_marked(
    s_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
    mut wl: refbox::Weak<winlink>,
    wp_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
) -> ::core::ffi::c_int {
    let mut wp = wp_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    if s_owner.is_none() || !wl.is_alive() || wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !marked_pane
        .s
        .ptr_eq(&std::rc::Rc::downgrade(s_owner.unwrap()))
        || marked_pane.winlink_handle() != wl
    {
        return 0 as ::core::ffi::c_int;
    }
    if marked_pane
        .pane_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get())
        != wp
    {
        return 0 as ::core::ffi::c_int;
    }
    return server_check_marked();
}
pub unsafe fn server_check_marked() -> ::core::ffi::c_int {
    return cmd_find_valid_state(&marked_pane);
}
pub unsafe fn server_create_socket(mut flags: uint64_t) -> Result<::core::ffi::c_int, CString> {
    let mut sa: sockaddr_un = sockaddr_un {
        sun_family: 0,
        sun_path: [0; 108],
    };
    let mut size: size_t = 0;
    let mut mask: mode_t = 0;
    let mut fd: ::core::ffi::c_int = 0;
    let mut saved_errno: ::core::ffi::c_int = 0;
    memset(
        &raw mut sa as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<sockaddr_un>() as size_t,
    );
    sa.sun_family = AF_UNIX as sa_family_t;
    size = strlcpy(
        &raw mut sa.sun_path as *mut ::core::ffi::c_char,
        socket_path,
        ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as size_t,
    ) as size_t;
    if size >= ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as usize {
        *__errno_location() = ENAMETOOLONG;
    } else {
        unlink(&raw mut sa.sun_path as *mut ::core::ffi::c_char);
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
                umask(mask as __mode_t);
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
                    return Ok(fd);
                }
            }
        }
    }
    let saved_errno = *__errno_location();
    let path = CStr::from_ptr(socket_path).to_bytes();
    let reason = CStr::from_ptr(strerror(saved_errno)).to_bytes();
    let mut message = Vec::with_capacity(17 + path.len() + reason.len());
    message.extend_from_slice(b"error creating ");
    message.extend_from_slice(path);
    message.extend_from_slice(b" (");
    message.extend_from_slice(reason);
    message.push(b')');
    Err(CString::new(message).expect("C strings contain no interior NUL"))
}
unsafe fn server_tidy_event() {
    let mut tv: timeval = timeval {
        tv_sec: 3600 as __time_t,
        tv_usec: 0,
    };
    let mut t: uint64_t = get_timer();
    format_tidy_jobs();
    malloc_trim(0 as size_t);
    log_debug(format_args!(
        "{}: took {} milliseconds",
        "server_tidy_event",
        get_timer().wrapping_sub(t) as ::core::ffi::c_ulonglong
    ));
    event_add(&raw mut server_ev_tidy, &raw mut tv);
}
pub(crate) unsafe fn server_start(
    mut client: *mut tmuxproc,
    mut flags: uint64_t,
    mut lockfd: ::core::ffi::c_int,
    lockfile: &mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = 0;
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut c: Option<ClientRef> = None;
    let mut cause: Option<CString> = None;
    let mut tv: timeval = timeval {
        tv_sec: 3600 as __time_t,
        tv_usec: 0,
    };
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    if !flags & CLIENT_NOFORK as uint64_t != 0 {
        if proc_fork_and_daemon(&raw mut fd) != 0 as ::core::ffi::c_int {
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            return fd;
        }
    }
    proc_clear_signals(client, 0 as ::core::ffi::c_int);
    server_client_flags = flags;
    if event_reinit() != 0 as ::core::ffi::c_int {
        fatalx(|out| out.write_all(b"event_reinit failed"));
    }
    let mut process_owner = proc_start(c"server".as_ptr());
    server_proc = &raw mut *process_owner;
    proc_set_signals(
        server_proc,
        Some(Box::new(|sig| unsafe { server_signal(sig) })),
    );
    sigprocmask(
        SIG_SETMASK,
        &raw mut oldset,
        ::core::ptr::null_mut::<sigset_t>(),
    );
    if log_get_level() > 1 as ::core::ffi::c_int {
        tty_create_log();
    }
    if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        fatal(|out| out.write_all(b"pledge failed"));
    }
    input_key_build();
    utf8_update_width_cache();
    windows.storage = None;
    all_window_panes.storage = None;
    clients.clear();
    sessions.storage = None;
    key_bindings_init();
    control_build_events();
    hooks_build_events();
    server_clear_messages();
    gettimeofday(&raw mut start_time, NULL);
    match systemd_create_socket(flags as ::core::ffi::c_int) {
        Ok(socket) => {
            server_fd = socket;
            server_update_socket();
        }
        Err(error) => {
            server_fd = -1;
            cause = Some(error);
        }
    }
    if !flags & CLIENT_NOFORK as uint64_t != 0 {
        let owner = server_client_create(fd);
        c = Some(owner.clone());
    } else {
        options_set_number(
            global_options,
            b"exit-empty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_longlong,
        );
    }
    if lockfd >= 0 as ::core::ffi::c_int {
        if let Some(path) = lockfile.take() {
            unlink(path.as_ptr());
        }
        close(lockfd);
    }
    if let Some(cause) = cause {
        if !c.is_none() {
            c.as_ref()
                .expect("live client")
                .exit_with_message(cause, Some(1));
        } else {
            fprintf(
                stderr,
                b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ptr(),
            );
            crate::src::proc::proc_free(process_owner);
            server_proc = std::ptr::null_mut();
            exit(1 as ::core::ffi::c_int);
        }
    }
    event_set(
        &raw mut server_ev_tidy,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { server_tidy_event() },
    );
    event_add(&raw mut server_ev_tidy, &raw mut tv);
    server_acl_init();
    server_add_accept(0 as ::core::ffi::c_int);
    let mut loop_callback = || unsafe { server_loop() == 0 };
    proc_loop(server_proc, Some(&mut loop_callback));
    job_kill_all();
    prompt_save_history();
    server_clear_messages();
    crate::src::proc::proc_free(process_owner);
    server_proc = std::ptr::null_mut();
    exit(0 as ::core::ffi::c_int);
}
unsafe fn server_loop() -> ::core::ffi::c_int {
    let mut c: Option<ClientRef> = None;
    let mut items: u_int = 0;
    current_time = time(::core::ptr::null_mut::<time_t>());
    loop {
        items = cmdq_next(None);
        let mut registry_c_owner = clients.first();
        c = registry_c_owner.clone();
        while !c.is_none() {
            if c.as_ref().expect("live client").flags() & CLIENT_IDENTIFIED as uint64_t != 0 {
                items = items.wrapping_add(cmdq_next(registry_c_owner.as_ref()));
            }
            registry_c_owner =
                clients.next(registry_c_owner.as_ref().expect("current registry client"));
            c = registry_c_owner.clone();
        }
        if !(items != 0 as u_int) {
            break;
        }
    }
    server_client_loop();
    if options_get_number(
        global_options,
        b"exit-empty\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
        && server_exit == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        global_options,
        b"exit-unattached\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        if sessions.storage.is_some() {
            return 0 as ::core::ffi::c_int;
        }
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.clone();
    while !c.is_none() {
        if !c
            .as_ref()
            .expect("live client")
            .attached_session()
            .upgrade()
            .is_none()
        {
            return 0 as ::core::ffi::c_int;
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.clone();
    }
    cmd_wait_for_flush();
    if clients.first().is_some() {
        return 0 as ::core::ffi::c_int;
    }
    if job_still_running() != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn server_send_exit() {
    let mut s: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    cmd_wait_for_flush();
    let mut registry_c_owner = clients.first();
    while let Some(client_owner) = registry_c_owner {
        let mut c: Option<ClientRef> = Some(client_owner.clone());
        registry_c_owner = clients.next(&client_owner);
        client_owner.shutdown();
    }
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        let name = s.as_ref().expect("live session").name().into_bytes();
        session_destroy(
            s_owner.as_ref().expect("registered session"),
            1 as ::core::ffi::c_int,
            b"server_send_exit\0" as *const u8 as *const ::core::ffi::c_char,
        );
        s_owner = sessions_after(&sessions, &name);
        s = s_owner.clone();
    }
}
pub unsafe fn server_update_socket() {
    let mut s: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    static mut last: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut n: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = 0;
    let mut sb: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    n = 0 as ::core::ffi::c_int;
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        if s.as_ref().expect("live session").is_attached() {
            n += 1;
            break;
        } else {
            s_owner = s.as_ref().expect("live session").next_session();
            s = s_owner.clone();
        }
    }
    if n != last {
        last = n;
        if stat(socket_path, &raw mut sb) != 0 as ::core::ffi::c_int {
            return;
        }
        mode = (sb.st_mode & ACCESSPERMS as __mode_t) as ::core::ffi::c_int;
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
        chmod(socket_path, mode as __mode_t);
    }
}
unsafe fn server_accept(mut fd: ::core::ffi::c_int, mut events: ::core::ffi::c_short) {
    let mut sa: sockaddr_storage = sockaddr_storage {
        ss_family: 0,
        __ss_padding: [0; 118],
        __ss_align: 0,
    };
    let mut slen: socklen_t = ::core::mem::size_of::<sockaddr_storage>() as socklen_t;
    let mut newfd: ::core::ffi::c_int = 0;
    let mut c: Option<ClientRef> = None;
    server_add_accept(0 as ::core::ffi::c_int);
    if events as ::core::ffi::c_int & EV_READ == 0 {
        return;
    }
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
        fatal(|out| out.write_all(b"accept failed"));
    }
    if server_exit != 0 {
        close(newfd);
        return;
    }
    let owner = server_client_create(newfd);
    c = Some(owner.clone());
    if server_acl_join(c.as_ref().expect("live client")) == 0 {
        c.as_ref()
            .expect("live client")
            .exit_with_message(CString::new("access not allowed").unwrap(), Some(1));
    }
}
pub unsafe fn server_add_accept(mut timeout: ::core::ffi::c_int) {
    let mut tv: timeval = timeval {
        tv_sec: timeout as __time_t,
        tv_usec: 0 as __suseconds_t,
    };
    if server_fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    if event_initialized(&*(&raw const server_ev_accept)) != 0 {
        event_del(&raw mut server_ev_accept);
    }
    if timeout == 0 as ::core::ffi::c_int {
        event_set(
            &raw mut server_ev_accept,
            server_fd,
            EV_READ as ::core::ffi::c_short,
            move |fd, flags| unsafe { server_accept(fd, flags) },
        );
        event_add(&raw mut server_ev_accept, ::core::ptr::null::<timeval>());
    } else {
        event_set(
            &raw mut server_ev_accept,
            server_fd,
            EV_TIMEOUT as ::core::ffi::c_short,
            move |fd, flags| unsafe { server_accept(fd, flags) },
        );
        event_add(&raw mut server_ev_accept, &raw mut tv);
    };
}
unsafe fn server_signal(sig: ProcessSignal) {
    let _fd: ::core::ffi::c_int = 0;
    log_debug(format_args!(
        "{}: {}",
        "server_signal",
        log_cstr((strsignal(sig.as_raw())) as *const _)
    ));
    match sig {
        ProcessSignal::Interrupt | ProcessSignal::Terminate => {
            server_exit = 1 as ::core::ffi::c_int;
            server_send_exit();
        }
        ProcessSignal::Child => {
            server_child_signal();
        }
        ProcessSignal::User1 => {
            event_del(&raw mut server_ev_accept);
            if let Ok(fd) = server_create_socket(server_client_flags) {
                close(server_fd);
                server_fd = fd;
                server_update_socket();
            }
            server_add_accept(0 as ::core::ffi::c_int);
        }
        ProcessSignal::User2 => {
            proc_toggle_log(server_proc);
        }
        _ => {}
    };
}
unsafe fn server_child_signal() {
    let mut status: ::core::ffi::c_int = 0;
    let mut pid: pid_t = 0;
    loop {
        pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG | WUNTRACED) as pid_t;
        match pid {
            -1 => {
                if *__errno_location() == ECHILD {
                    return;
                }
                fatal(|out| out.write_all(b"waitpid failed"));
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
unsafe fn server_child_exited(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        w = window_owner.get();
        window_cursor = windows_next(&*w);
        let mut pane_cursor = window_pane_first(w.as_ref());
        while let Some(pane_owner) = pane_cursor {
            wp = pane_owner.get();
            if (*wp).pid == pid {
                (*wp).status = status;
                (*wp).flags |= PANE_STATUSREADY;
                log_debug(format_args!("%{} exited", ((*wp).id) as u32));
                (*wp).flags |= PANE_EXITED;
                window_pane_wait_finish(&(*(wp)).observer.upgrade().expect("live window_pane"));
                spawn_editor_finish(&(*(wp)).observer.upgrade().expect("live window_pane"));
                if window_pane_destroy_ready(&(*(wp)).observer.upgrade().expect("live window_pane"))
                    != 0
                {
                    server_destroy_pane(&pane_owner, 1);
                }
                break;
            } else {
                pane_cursor = window_pane_next(wp.as_ref());
            }
        }
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
    job_check_died(pid, status);
}
unsafe fn server_child_stopped(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTIN
        || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTOU
    {
        return;
    }
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        w = window_owner.get();
        wp = window_pane_first(w.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            if (*wp).pid == pid {
                if killpg(pid as __pid_t, SIGCONT) != 0 as ::core::ffi::c_int {
                    kill(pid as __pid_t, SIGCONT);
                }
            }
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        window_cursor = windows_next(&*w);
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
    job_check_died(pid, status);
}
pub unsafe fn server_add_message(
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let mut limit: u_int = 0;
    let s = format_message_with(write);
    log_debug(format_args!(
        "message: {}",
        log_cstr((s.as_ptr()) as *const _)
    ));
    let fresh0 = message_next;
    message_next = message_next.wrapping_add(1);
    let mut msg_time = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    gettimeofday(&raw mut msg_time, NULL);
    message_log.push_back(s, fresh0, msg_time);
    limit = options_get_number(
        global_options,
        b"message-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    message_log.trim(message_next, limit);
}
