use crate::src::cmd_find::{cmd_find_clear_state, cmd_find_valid_state};
use crate::src::cmd_queue::cmdq_next;
use crate::src::cmd_wait_for::cmd_wait_for_flush;
use crate::src::compat::systemd::systemd_create_socket;
use crate::src::control_notify::control_build_events;
use crate::src::ffi::libc::{
    __errno_location, accept, bind, chmod, close, exit, fprintf, free, gettimeofday, kill, killpg,
    listen, malloc_trim, memset, sigfillset, sigprocmask, socket, stat, stderr, strerror, strlcpy,
    strsignal, time, umask, unlink, waitpid,
};
use crate::src::format::format_tidy_jobs;
use crate::src::hooks::hooks_build_events;
use crate::src::input_keys::input_key_build;
use crate::src::job::{job_check_died, job_kill_all, job_still_running};
use crate::src::key_bindings::key_bindings_init;
use crate::src::log::{fatal, fatalx, log_debug, log_get_level};
use crate::src::options::{options_get_number, options_set_number};
use crate::src::proc::{
    proc_clear_signals, proc_fork_and_daemon, proc_loop, proc_set_signals, proc_start,
    proc_toggle_log,
};
use crate::src::prompt_history::prompt_save_history;
use crate::src::reactor::{event_add, event_del, event_initialized, event_reinit, event_set};
use crate::src::server_acl::{server_acl_init, server_acl_join};
use crate::src::server_client::{server_client_create, server_client_loop, server_client_lost};
use crate::src::server_fn::server_destroy_pane;
pub use crate::src::session::sessions;
use crate::src::session::{session_destroy, sessions_RB_MINMAX, sessions_RB_NEXT};
pub use crate::src::shared::status::{message_entry, message_entry_entry, message_list};
use crate::src::spawn::spawn_editor_finish;
use crate::src::tmux::{get_timer, global_options, setblocking, socket_path, start_time};
use crate::src::tty::tty_create_log;
use crate::src::utf8::utf8_update_width_cache;
pub use crate::src::window::windows;
use crate::src::window::{
    all_window_panes, window_pane_destroy_ready, window_pane_wait_finish, windows_minmax,
    windows_next,
};
use crate::src::xmalloc::{xasprintf, xcalloc, xstrdup, xvasprintf};

use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{
    __blkcnt_t, __blksize_t, __dev_t, __gid_t, __ino_t, __mode_t, __nlink_t, __off64_t, __off_t,
    __socklen_t, __syscall_slong_t, __uid_t, __uint16_t, __uint32_t, socklen_t, uint16_t, uint32_t,
};
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::client::clients;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{
    CLIENT_DEFAULTSOCKET, CLIENT_EXIT, CLIENT_IDENTIFIED, CLIENT_NOFORK, CLIENT_SUSPENDED,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::errno::{EAGAIN, ECHILD, EINTR, ENAMETOOLONG};
use crate::src::shared::event::*;
pub use crate::src::shared::event::{EV_READ, EV_TIMEOUT};
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::{options, options_entry};
pub use crate::src::shared::pane::window_pane_tree;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_EXITED, PANE_STATUSREADY,
};
pub use crate::src::shared::posix_io::stat;
pub use crate::src::shared::posix_io::{
    __S_IEXEC, __S_IREAD, __S_IWRITE, S_IRWXU, WAIT_ANY, WNOHANG,
};
pub use crate::src::shared::process::{tmuxpeer, tmuxproc};
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::signal::{
    __sigset_t, sigset_t, SIGCHLD, SIGCONT, SIGINT, SIGTERM, SIGTTIN, SIGTTOU, SIGUSR1, SIGUSR2,
    SIG_BLOCK, SIG_SETMASK,
};
pub use crate::src::shared::socket::{
    __socket_type, in6_addr, in6_addr___in6_u, in_addr, in_addr_t, in_port_t, sa_family_t,
    sockaddr, sockaddr_at, sockaddr_ax25, sockaddr_dl, sockaddr_eon, sockaddr_in, sockaddr_in6,
    sockaddr_inarp, sockaddr_ipx, sockaddr_iso, sockaddr_ns, sockaddr_un, sockaddr_x25,
    __CONST_SOCKADDR_ARG, __SOCKADDR_ARG, AF_UNIX, PF_LOCAL, PF_UNIX, SOCK_CLOEXEC, SOCK_DCCP,
    SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
pub use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::time::timespec;
pub use crate::src::shared::tree::RB_NEGINF;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};

pub type mode_t = __mode_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    pub __ss_padding: [::core::ffi::c_char; 118],
    pub __ss_align: ::core::ffi::c_ulong,
}

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

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
#[no_mangle]
pub static mut clients: clients = clients {
    tqh_first: ::core::ptr::null::<client>() as *mut client,
    tqh_last: ::core::ptr::null::<*mut client>() as *mut *mut client,
};
#[no_mangle]
pub static mut server_proc: *mut tmuxproc = ::core::ptr::null::<tmuxproc>() as *mut tmuxproc;
static mut server_fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
static mut server_client_flags: uint64_t = 0;
static mut server_exit: ::core::ffi::c_int = 0;
static mut server_ev_accept: event = event {
    ev_evcallback: event_callback {
        evcb_active_next: event_callback_entry {
            tqe_next: ::core::ptr::null::<event_callback>() as *mut event_callback,
            tqe_prev: ::core::ptr::null::<*mut event_callback>() as *mut *mut event_callback,
        },
        evcb_flags: 0,
        evcb_pri: 0,
        evcb_closure: 0,
        evcb_cb_union: event_callback_union {
            evcb_callback: None,
        },
        evcb_arg: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    },
    ev_timeout_pos: event_timeout_pos {
        ev_next_with_common_timeout: event_timeout_entry {
            tqe_next: ::core::ptr::null::<event>() as *mut event,
            tqe_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
        },
    },
    ev_fd: 0,
    ev_base: ::core::ptr::null::<event_base>() as *mut event_base,
    ev_: event_io_or_signal {
        ev_io: event_io {
            ev_io_next: event_io_entry {
                le_next: ::core::ptr::null::<event>() as *mut event,
                le_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
            },
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
    },
    ev_events: 0,
    ev_res: 0,
    ev_timeout: timeval {
        tv_sec: 0,
        tv_usec: 0,
    },
};
static mut server_ev_tidy: event = event {
    ev_evcallback: event_callback {
        evcb_active_next: event_callback_entry {
            tqe_next: ::core::ptr::null::<event_callback>() as *mut event_callback,
            tqe_prev: ::core::ptr::null::<*mut event_callback>() as *mut *mut event_callback,
        },
        evcb_flags: 0,
        evcb_pri: 0,
        evcb_closure: 0,
        evcb_cb_union: event_callback_union {
            evcb_callback: None,
        },
        evcb_arg: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    },
    ev_timeout_pos: event_timeout_pos {
        ev_next_with_common_timeout: event_timeout_entry {
            tqe_next: ::core::ptr::null::<event>() as *mut event,
            tqe_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
        },
    },
    ev_fd: 0,
    ev_base: ::core::ptr::null::<event_base>() as *mut event_base,
    ev_: event_io_or_signal {
        ev_io: event_io {
            ev_io_next: event_io_entry {
                le_next: ::core::ptr::null::<event>() as *mut event,
                le_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
            },
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
    },
    ev_events: 0,
    ev_res: 0,
    ev_timeout: timeval {
        tv_sec: 0,
        tv_usec: 0,
    },
};
#[no_mangle]
pub static mut marked_pane: cmd_find_state = cmd_find_state {
    flags: 0,
    current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
    s: ::core::ptr::null::<session>() as *mut session,
    wl: ::core::ptr::null::<winlink>() as *mut winlink,
    w: ::core::ptr::null::<window>() as *mut window,
    wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
    idx: 0,
};
static mut message_next: u_int = 0;
#[no_mangle]
pub static mut message_log: message_list = message_list {
    tqh_first: ::core::ptr::null::<message_entry>() as *mut message_entry,
    tqh_last: ::core::ptr::null::<*mut message_entry>() as *mut *mut message_entry,
};
#[no_mangle]
pub static mut current_time: time_t = 0;
#[no_mangle]
pub unsafe extern "C" fn server_set_marked(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    cmd_find_clear_state(&raw mut marked_pane, 0 as ::core::ffi::c_int);
    marked_pane.s = s;
    marked_pane.wl = wl;
    if !wl.is_null() {
        marked_pane.w = (*wl).window;
    }
    marked_pane.wp = wp;
}
#[no_mangle]
pub unsafe extern "C" fn server_clear_marked() {
    cmd_find_clear_state(&raw mut marked_pane, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn server_is_marked(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if s.is_null() || wl.is_null() || wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if marked_pane.s != s || marked_pane.wl != wl {
        return 0 as ::core::ffi::c_int;
    }
    if marked_pane.wp != wp {
        return 0 as ::core::ffi::c_int;
    }
    return server_check_marked();
}
#[no_mangle]
pub unsafe extern "C" fn server_check_marked() -> ::core::ffi::c_int {
    return cmd_find_valid_state(&raw mut marked_pane);
}
#[no_mangle]
pub unsafe extern "C" fn server_create_socket(
    mut flags: uint64_t,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
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
                    return fd;
                }
            }
        }
    }
    if !cause.is_null() {
        xasprintf(
            cause,
            b"error creating %s (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            socket_path,
            strerror(*__errno_location()),
        );
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn server_tidy_event(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut tv: timeval = timeval {
        tv_sec: 3600 as __time_t,
        tv_usec: 0,
    };
    let mut t: uint64_t = get_timer();
    format_tidy_jobs();
    malloc_trim(0 as size_t);
    log_debug(
        b"%s: took %llu milliseconds\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_tidy_event\0" as *const u8 as *const ::core::ffi::c_char,
        get_timer().wrapping_sub(t) as ::core::ffi::c_ulonglong,
    );
    event_add(&raw mut server_ev_tidy, &raw mut tv);
}
#[no_mangle]
pub unsafe extern "C" fn server_start(
    mut client: *mut tmuxproc,
    mut flags: uint64_t,
    mut base: *mut event_base,
    mut lockfd: ::core::ffi::c_int,
    mut lockfile: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = 0;
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
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
    if event_reinit(base) != 0 as ::core::ffi::c_int {
        fatalx(b"event_reinit failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    server_proc = proc_start(b"server\0" as *const u8 as *const ::core::ffi::c_char);
    proc_set_signals(
        server_proc,
        Some(server_signal as unsafe extern "C" fn(::core::ffi::c_int) -> ()),
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
        fatal(b"pledge failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    input_key_build();
    utf8_update_width_cache();
    windows.storage = std::ptr::null_mut();
    all_window_panes.storage = std::ptr::null_mut();
    clients.tqh_first = ::core::ptr::null_mut::<client>();
    clients.tqh_last = &raw mut clients.tqh_first;
    sessions.rbh_root = ::core::ptr::null_mut::<session>();
    key_bindings_init();
    control_build_events();
    hooks_build_events();
    message_log.tqh_first = ::core::ptr::null_mut::<message_entry>();
    message_log.tqh_last = &raw mut message_log.tqh_first;
    gettimeofday(&raw mut start_time, NULL);
    server_fd = systemd_create_socket(flags as ::core::ffi::c_int, &raw mut cause);
    if server_fd != -(1 as ::core::ffi::c_int) {
        server_update_socket();
    }
    if !flags & CLIENT_NOFORK as uint64_t != 0 {
        c = server_client_create(fd);
    } else {
        options_set_number(
            global_options,
            b"exit-empty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_longlong,
        );
    }
    if lockfd >= 0 as ::core::ffi::c_int {
        unlink(lockfile);
        free(lockfile as *mut ::core::ffi::c_void);
        close(lockfd);
    }
    if !cause.is_null() {
        if !c.is_null() {
            (*c).exit_message = cause;
            (*c).retval = 1 as ::core::ffi::c_int;
            (*c).flags |= CLIENT_EXIT as uint64_t;
        } else {
            fprintf(
                stderr,
                b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            exit(1 as ::core::ffi::c_int);
        }
    }
    event_set(
        &raw mut server_ev_tidy,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_tidy_event
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    );
    event_add(&raw mut server_ev_tidy, &raw mut tv);
    server_acl_init();
    server_add_accept(0 as ::core::ffi::c_int);
    proc_loop(
        server_proc,
        Some(server_loop as unsafe extern "C" fn() -> ::core::ffi::c_int),
    );
    job_kill_all();
    prompt_save_history();
    exit(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn server_loop() -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut items: u_int = 0;
    current_time = time(::core::ptr::null_mut::<time_t>());
    loop {
        items = cmdq_next(::core::ptr::null_mut::<client>());
        c = clients.tqh_first;
        while !c.is_null() {
            if (*c).flags & CLIENT_IDENTIFIED as uint64_t != 0 {
                items = items.wrapping_add(cmdq_next(c));
            }
            c = (*c).entry.tqe_next;
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
        if !sessions.rbh_root.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        c = (*c).entry.tqe_next;
    }
    cmd_wait_for_flush();
    if !clients.tqh_first.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if job_still_running() != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_send_exit() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut c1: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s1: *mut session = ::core::ptr::null_mut::<session>();
    cmd_wait_for_flush();
    c = clients.tqh_first;
    while !c.is_null() && {
        c1 = (*c).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
            server_client_lost(c);
        } else {
            (*c).flags |= CLIENT_EXIT as uint64_t;
            (*c).exit_type = CLIENT_EXIT_SHUTDOWN;
        }
        (*c).session = ::core::ptr::null_mut::<session>();
        c = c1;
    }
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() && {
        s1 = sessions_RB_NEXT(s);
        1 as ::core::ffi::c_int != 0
    } {
        session_destroy(
            s,
            1 as ::core::ffi::c_int,
            b"server_send_exit\0" as *const u8 as *const ::core::ffi::c_char,
        );
        s = s1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_update_socket() {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
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
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if (*s).attached != 0 as u_int {
            n += 1;
            break;
        } else {
            s = sessions_RB_NEXT(s);
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
unsafe extern "C" fn server_accept(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut sa: sockaddr_storage = sockaddr_storage {
        ss_family: 0,
        __ss_padding: [0; 118],
        __ss_align: 0,
    };
    let mut slen: socklen_t = ::core::mem::size_of::<sockaddr_storage>() as socklen_t;
    let mut newfd: ::core::ffi::c_int = 0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
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
        fatal(b"accept failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if server_exit != 0 {
        close(newfd);
        return;
    }
    c = server_client_create(newfd);
    if server_acl_join(c) == 0 {
        (*c).exit_message =
            xstrdup(b"access not allowed\0" as *const u8 as *const ::core::ffi::c_char);
        (*c).retval = 1 as ::core::ffi::c_int;
        (*c).flags |= CLIENT_EXIT as uint64_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_add_accept(mut timeout: ::core::ffi::c_int) {
    let mut tv: timeval = timeval {
        tv_sec: timeout as __time_t,
        tv_usec: 0 as __suseconds_t,
    };
    if server_fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    if event_initialized(&raw mut server_ev_accept) != 0 {
        event_del(&raw mut server_ev_accept);
    }
    if timeout == 0 as ::core::ffi::c_int {
        event_set(
            &raw mut server_ev_accept,
            server_fd,
            EV_READ as ::core::ffi::c_short,
            Some(
                server_accept
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            NULL,
        );
        event_add(&raw mut server_ev_accept, ::core::ptr::null::<timeval>());
    } else {
        event_set(
            &raw mut server_ev_accept,
            server_fd,
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                server_accept
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            NULL,
        );
        event_add(&raw mut server_ev_accept, &raw mut tv);
    };
}
unsafe extern "C" fn server_signal(mut sig: ::core::ffi::c_int) {
    let mut fd: ::core::ffi::c_int = 0;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_signal\0" as *const u8 as *const ::core::ffi::c_char,
        strsignal(sig),
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
            event_del(&raw mut server_ev_accept);
            fd = server_create_socket(
                server_client_flags,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
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
unsafe extern "C" fn server_child_signal() {
    let mut status: ::core::ffi::c_int = 0;
    let mut pid: pid_t = 0;
    loop {
        pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG | WUNTRACED) as pid_t;
        match pid {
            -1 => {
                if *__errno_location() == ECHILD {
                    return;
                }
                fatal(b"waitpid failed\0" as *const u8 as *const ::core::ffi::c_char);
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
unsafe extern "C" fn server_child_exited(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut w1: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() && {
        w1 = windows_next(w);
        1 as ::core::ffi::c_int != 0
    } {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).pid == pid {
                (*wp).status = status;
                (*wp).flags |= PANE_STATUSREADY;
                log_debug(
                    b"%%%u exited\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                (*wp).flags |= PANE_EXITED;
                window_pane_wait_finish(wp);
                spawn_editor_finish(wp);
                if window_pane_destroy_ready(wp) != 0 {
                    server_destroy_pane(wp, 1 as ::core::ffi::c_int);
                }
                break;
            } else {
                wp = (*wp).entry.tqe_next;
            }
        }
        w = w1;
    }
    job_check_died(pid, status);
}
unsafe extern "C" fn server_child_stopped(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTIN
        || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTOU
    {
        return;
    }
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).pid == pid {
                if killpg(pid as __pid_t, SIGCONT) != 0 as ::core::ffi::c_int {
                    kill(pid as __pid_t, SIGCONT);
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        w = windows_next(w);
    }
    job_check_died(pid, status);
}
#[no_mangle]
pub unsafe extern "C" fn server_add_message(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut msg: *mut message_entry = ::core::ptr::null_mut::<message_entry>();
    let mut msg1: *mut message_entry = ::core::ptr::null_mut::<message_entry>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ap: ::core::ffi::VaList;
    let mut limit: u_int = 0;
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    log_debug(
        b"message: %s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    msg = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<message_entry>() as size_t,
    ) as *mut message_entry;
    gettimeofday(&raw mut (*msg).msg_time, NULL);
    let fresh0 = message_next;
    message_next = message_next.wrapping_add(1);
    (*msg).msg_num = fresh0;
    (*msg).msg = s;
    (*msg).entry.tqe_next = ::core::ptr::null_mut::<message_entry>();
    (*msg).entry.tqe_prev = message_log.tqh_last;
    *message_log.tqh_last = msg;
    message_log.tqh_last = &raw mut (*msg).entry.tqe_next;
    limit = options_get_number(
        global_options,
        b"message-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    msg = message_log.tqh_first;
    while !msg.is_null() && {
        msg1 = (*msg).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if (*msg).msg_num.wrapping_add(limit) >= message_next {
            break;
        }
        free((*msg).msg as *mut ::core::ffi::c_void);
        if !(*msg).entry.tqe_next.is_null() {
            (*(*msg).entry.tqe_next).entry.tqe_prev = (*msg).entry.tqe_prev;
        } else {
            message_log.tqh_last = (*msg).entry.tqe_prev;
        }
        *(*msg).entry.tqe_prev = (*msg).entry.tqe_next;
        free(msg as *mut ::core::ffi::c_void);
        msg = msg1;
    }
}
