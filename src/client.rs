use crate::src::arguments::OwnedArgumentVector;
use crate::src::cmd::{cmd_list_any_have, cmd_list_free, cmd_pack_argv};
use crate::src::cmd_parse::cmd_parse_from_arguments;
use crate::src::compat::systemd::systemd_activated;
use crate::src::control::control_wait_exit;
use crate::src::environ::environ_free;
use crate::src::ffi::libc::{
    __errno_location, cfgetispeed, cfgetospeed, cfmakeraw, cfsetispeed, cfsetospeed, close,
    closefrom, connect, dup, environ, execl, fflush, flock, fprintf, free, getenv, getpid, getppid,
    isatty, kill, memcpy, memset, open, printf, setenv, sigaction, sigemptyset, socket, stderr,
    stdout, strerror, strlcpy, strlen, strsignal, system, tcgetattr, tcsetattr, ttyname, unlink,
    waitpid,
};
use crate::src::file::{
    file_read_cancel, file_read_open, file_write_close, file_write_data, file_write_left,
    file_write_open,
};
use crate::src::log::{fatal, fatalx, log_debug};
use crate::src::options::options_free;
use crate::src::proc::{
    proc_add_peer, proc_clear_signals, proc_exit, proc_flush_peer, proc_loop, proc_send,
    proc_set_signals, proc_start,
};
use crate::src::server::{server_start, server_start_owned};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{
    __clock_t, __off64_t, __off_t, __socklen_t, __uid_t, __uint16_t, __uint32_t, socklen_t,
    ssize_t, uint16_t, uint32_t,
};
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_CONTROLCONTROL, CLIENT_CONTROL_WAITEXIT, CLIENT_LOGIN,
    CLIENT_NOSTARTSERVER, CLIENT_STARTSERVER, CLIENT_WRITE_ACK,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::CMD_STARTSERVER;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::errno::{EAGAIN, ECHILD, EINTR, ENAMETOOLONG, ENOENT};
use crate::src::shared::event::*;
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
pub use crate::src::shared::message::msg_command;
use crate::src::shared::message::*;
pub use crate::src::shared::message::{ibuf, imsg};
pub use crate::src::shared::message::{imsg_hdr, IMSG_HEADER_SIZE, MAX_IMSGSIZE, PROTOCOL_VERSION};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::posix_io::{
    O_CREAT, O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, WAIT_ANY, WNOHANG,
};
pub use crate::src::shared::posix_terminal::{ICRNL, ONLCR, OPOST, TCSANOW, VMIN, VTIME};
pub use crate::src::shared::process::{tmuxpeer, tmuxproc};
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::signal::{
    __sighandler_t, __sigset_t, __sigval_t, sigaction, sigaction___sigaction_handler, siginfo_t,
    siginfo_t__sifields, siginfo_t__sifields__kill, siginfo_t__sifields__rt,
    siginfo_t__sifields__sigchld, siginfo_t__sifields__sigfault,
    siginfo_t__sifields__sigfault__bounds, siginfo_t__sifields__sigfault__bounds__addr_bnd,
    siginfo_t__sifields__sigpoll, siginfo_t__sifields__sigsys, siginfo_t__sifields__timer,
    sigset_t, sigval, SA_RESTART, SIGCHLD, SIGCONT, SIGHUP, SIGTERM, SIGTSTP, SIGWINCH, SIG_DFL,
};
pub use crate::src::shared::socket::{
    __socket_type, in6_addr, in6_addr___in6_u, in_addr, in_addr_t, in_port_t, sa_family_t,
    sockaddr, sockaddr_at, sockaddr_ax25, sockaddr_dl, sockaddr_eon, sockaddr_in, sockaddr_in6,
    sockaddr_inarp, sockaddr_ipx, sockaddr_iso, sockaddr_ns, sockaddr_un, sockaddr_x25,
    __CONST_SOCKADDR_ARG, AF_UNIX, PF_LOCAL, PF_UNIX, SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM,
    SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
pub use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tmux::{
    find_cwd, find_home_cstr, global_environ, global_options, global_s_options, global_w_options,
    ptm_fd, setblocking, shell_argv0_cstring, shell_command, socket_path,
};
use crate::src::tty_term::tty_term_read_list;
use crate::src::xmalloc::xsnprintf;
use ::libc;
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_26;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_25;

pub const ECONNREFUSED: ::core::ffi::c_int = 111 as ::core::ffi::c_int;

pub const LOCK_EX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOCK_NB: ::core::ffi::c_int = 4 as ::core::ffi::c_int;

pub const IXANY: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;

pub const CS8: ::core::ffi::c_int = 0o60 as ::core::ffi::c_int;
pub const CREAD: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const HUPCL: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;

pub const TCSAFLUSH: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

static mut client_proc: *mut tmuxproc = ::core::ptr::null::<tmuxproc>() as *mut tmuxproc;
static mut client_peer: *mut tmuxpeer = ::core::ptr::null::<tmuxpeer>() as *mut tmuxpeer;
static mut client_flags: uint64_t = 0;
static mut client_suspended: ::core::ffi::c_int = 0;
static mut client_exitreason: client_exit_reason = CLIENT_EXIT_NONE;
static mut client_exitflag: ::core::ffi::c_int = 0;
static mut client_exitval: ::core::ffi::c_int = 0;
static mut client_exittype: msgtype = 0 as msgtype;
static mut client_exitsession: Option<CString> = None;
static mut client_exitmessage: Option<Vec<u8>> = None;
static mut client_exec_payload: Option<(CString, CString)> = None;
static mut client_attached: ::core::ffi::c_int = 0;
static mut client_files: client_files = client_files { storage: None };
unsafe extern "C" fn client_get_lock(
    mut lockfile: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut lockfd: ::core::ffi::c_int = 0;
    log_debug(
        b"lock file is %s\0" as *const u8 as *const ::core::ffi::c_char,
        lockfile,
    );
    lockfd = open(lockfile, O_WRONLY | O_CREAT, 0o600 as ::core::ffi::c_int);
    if lockfd == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"open failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    if flock(lockfd, LOCK_EX | LOCK_NB) == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"flock failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        if *__errno_location() != EAGAIN {
            return lockfd;
        }
        while flock(lockfd, LOCK_EX) == -(1 as ::core::ffi::c_int) && *__errno_location() == EINTR {
        }
        close(lockfd);
        return -(2 as ::core::ffi::c_int);
    }
    log_debug(b"flock succeeded\0" as *const u8 as *const ::core::ffi::c_char);
    return lockfd;
}
unsafe extern "C" fn client_connect(
    mut base: *mut event_base,
    mut path: *const ::core::ffi::c_char,
    mut flags: uint64_t,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut sa: sockaddr_un = sockaddr_un {
        sun_family: 0,
        sun_path: [0; 108],
    };
    let mut size: size_t = 0;
    let mut fd: ::core::ffi::c_int = 0;
    let mut lockfd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut locked: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lockfile: Option<CString> = None;
    memset(
        &raw mut sa as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<sockaddr_un>() as size_t,
    );
    sa.sun_family = AF_UNIX as sa_family_t;
    size = strlcpy(
        &raw mut sa.sun_path as *mut ::core::ffi::c_char,
        path,
        ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as size_t,
    ) as size_t;
    if size >= ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as usize {
        *__errno_location() = ENAMETOOLONG;
        return -(1 as ::core::ffi::c_int);
    }
    log_debug(
        b"socket is %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    loop {
        fd = socket(
            AF_UNIX,
            SOCK_STREAM as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if fd == -(1 as ::core::ffi::c_int) {
            return -(1 as ::core::ffi::c_int);
        }
        log_debug(b"trying connect\0" as *const u8 as *const ::core::ffi::c_char);
        if !(connect(
            fd,
            __CONST_SOCKADDR_ARG {
                __sockaddr__: &raw mut sa as *mut sockaddr,
            },
            ::core::mem::size_of::<sockaddr_un>() as socklen_t,
        ) == -(1 as ::core::ffi::c_int))
        {
            current_block = 7172762164747879670;
            break;
        }
        log_debug(
            b"connect failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        if *__errno_location() != ECONNREFUSED && *__errno_location() != ENOENT {
            current_block = 16524389688364091157;
            break;
        }
        if flags & CLIENT_NOSTARTSERVER as uint64_t != 0 {
            current_block = 16524389688364091157;
            break;
        }
        if !flags & CLIENT_STARTSERVER as uint64_t != 0 {
            current_block = 16524389688364091157;
            break;
        }
        close(fd);
        if locked == 0 {
            let mut name = std::ffi::CStr::from_ptr(path).to_bytes().to_vec();
            name.extend_from_slice(b".lock");
            lockfile = Some(CString::new(name).expect("socket path has no interior NUL"));
            lockfd = client_get_lock(lockfile.as_ref().unwrap().as_ptr());
            if lockfd < 0 as ::core::ffi::c_int {
                log_debug(
                    b"didn't get lock (%d)\0" as *const u8 as *const ::core::ffi::c_char,
                    lockfd,
                );
                lockfile = None;
                if lockfd == -(2 as ::core::ffi::c_int) {
                    continue;
                }
            }
            log_debug(
                b"got lock (%d)\0" as *const u8 as *const ::core::ffi::c_char,
                lockfd,
            );
            locked = 1 as ::core::ffi::c_int;
        } else {
            if lockfd >= 0 as ::core::ffi::c_int
                && unlink(path) != 0 as ::core::ffi::c_int
                && *__errno_location() != ENOENT
            {
                lockfile.take();
                close(lockfd);
                return -(1 as ::core::ffi::c_int);
            }
            fd = server_start_owned(client_proc, flags, base, lockfd, &mut lockfile);
            current_block = 7172762164747879670;
            break;
        }
    }
    match current_block {
        16524389688364091157 => {
            if locked != 0 {
                lockfile.take();
                close(lockfd);
            }
            close(fd);
            return -(1 as ::core::ffi::c_int);
        }
        _ => {
            if locked != 0 && lockfd >= 0 as ::core::ffi::c_int {
                lockfile.take();
                close(lockfd);
            }
            setblocking(fd, 0 as ::core::ffi::c_int);
            return fd;
        }
    };
}
unsafe extern "C" fn client_exit_message() -> *const ::core::ffi::c_char {
    static mut msg: [::core::ffi::c_char; 256] = [0; 256];
    match client_exitreason as ::core::ffi::c_uint {
        1 => {
            if let Some(session) = client_exitsession.as_ref() {
                xsnprintf(
                    &raw mut msg as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                    b"detached (from session %s)\0" as *const u8 as *const ::core::ffi::c_char,
                    session.as_ptr(),
                );
                return &raw mut msg as *mut ::core::ffi::c_char;
            }
            return b"detached\0" as *const u8 as *const ::core::ffi::c_char;
        }
        2 => {
            if let Some(session) = client_exitsession.as_ref() {
                xsnprintf(
                    &raw mut msg as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                    b"detached and SIGHUP (from session %s)\0" as *const u8
                        as *const ::core::ffi::c_char,
                    session.as_ptr(),
                );
                return &raw mut msg as *mut ::core::ffi::c_char;
            }
            return b"detached and SIGHUP\0" as *const u8 as *const ::core::ffi::c_char;
        }
        3 => return b"lost tty\0" as *const u8 as *const ::core::ffi::c_char,
        4 => return b"terminated\0" as *const u8 as *const ::core::ffi::c_char,
        5 => {
            return b"server exited unexpectedly\0" as *const u8 as *const ::core::ffi::c_char;
        }
        6 => return b"exited\0" as *const u8 as *const ::core::ffi::c_char,
        7 => return b"server exited\0" as *const u8 as *const ::core::ffi::c_char,
        8 => {
            // The message remains owned until client_main has printed the exit reason.
            return client_exitmessage
                .as_ref()
                .map_or(::core::ptr::null(), |message| message.as_ptr().cast());
        }
        0 | _ => {}
    }
    return b"unknown reason\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn client_exit() {
    if file_write_left(&raw mut client_files) == 0 {
        proc_exit(client_proc);
    }
}
#[no_mangle]
pub unsafe extern "C" fn client_main(
    mut base: *mut event_base,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut flags: uint64_t,
    mut feat: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut fd: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut ttynam: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut termname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ppid: pid_t = 0;
    let mut msg: msgtype = 0 as msgtype;
    let mut tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut saved_tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut size: size_t = 0;
    let mut caps = Vec::new();
    if !shell_command.is_null() {
        msg = MSG_SHELL;
        flags |= CLIENT_STARTSERVER as uint64_t;
    } else if argc == 0 as ::core::ffi::c_int {
        msg = MSG_COMMAND;
        flags |= CLIENT_STARTSERVER as uint64_t;
    } else {
        msg = MSG_COMMAND;
        let mut values = OwnedArgumentVector::from_argv(argc, argv);
        pr = cmd_parse_from_arguments(
            values.as_mut_ptr(),
            argc as u_int,
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
        drop(values);
        if (*pr).status as ::core::ffi::c_uint
            == CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if cmd_list_any_have((*pr).cmdlist, CMD_STARTSERVER) != 0 {
                flags |= CLIENT_STARTSERVER as uint64_t;
            }
            cmd_list_free((*pr).cmdlist);
        } else {
            free((*pr).error as *mut ::core::ffi::c_void);
        }
    }
    client_proc = proc_start(b"client\0" as *const u8 as *const ::core::ffi::c_char);
    proc_set_signals(
        client_proc,
        Some(client_signal as unsafe extern "C" fn(::core::ffi::c_int) -> ()),
    );
    client_flags = (flags as ::core::ffi::c_ulonglong | CLIENT_WRITE_ACK) as uint64_t;
    log_debug(
        b"flags are %#llx\0" as *const u8 as *const ::core::ffi::c_char,
        client_flags as ::core::ffi::c_ulonglong,
    );
    if systemd_activated() != 0 {
        fd = server_start(
            client_proc,
            flags,
            base,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    } else {
        fd = client_connect(base, socket_path, client_flags);
    }
    if fd == -(1 as ::core::ffi::c_int) {
        if *__errno_location() == ECONNREFUSED {
            fprintf(
                stderr,
                b"no server running on %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                socket_path,
            );
        } else {
            fprintf(
                stderr,
                b"error connecting to %s (%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                socket_path,
                strerror(*__errno_location()),
            );
        }
        return 1 as ::core::ffi::c_int;
    }
    client_peer = proc_add_peer(
        client_proc,
        fd,
        Some(client_dispatch as unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> ()),
        NULL,
    );
    cwd = find_cwd();
    if cwd.is_null() {
        cwd = find_home_cstr().map_or(
            b"/\0" as *const u8 as *const ::core::ffi::c_char,
            CStr::as_ptr,
        );
    }
    ttynam = ttyname(STDIN_FILENO);
    if ttynam.is_null() {
        ttynam = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    termname = getenv(b"TERM\0" as *const u8 as *const ::core::ffi::c_char);
    if termname.is_null() {
        termname = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        fatal(b"pledge failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if isatty(STDIN_FILENO) != 0 && *termname as ::core::ffi::c_int != '\0' as i32 {
        match tty_term_read_list(termname, STDIN_FILENO) {
            Ok(read_caps) => caps = read_caps,
            Err(cause) => {
                fprintf(
                    stderr,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    cause.as_ptr(),
                );
                return 1 as ::core::ffi::c_int;
            }
        }
    }
    if ptm_fd != -(1 as ::core::ffi::c_int) {
        close(ptm_fd);
    }
    options_free(global_options);
    options_free(global_s_options);
    options_free(global_w_options);
    environ_free(global_environ);
    if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        if tcgetattr(STDIN_FILENO, &raw mut saved_tio) != 0 as ::core::ffi::c_int {
            fprintf(
                stderr,
                b"tcgetattr failed: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
            return 1 as ::core::ffi::c_int;
        }
        cfmakeraw(&raw mut tio);
        tio.c_iflag = (ICRNL | IXANY) as tcflag_t;
        tio.c_oflag = (OPOST | ONLCR) as tcflag_t;
        tio.c_cflag = (CREAD | CS8 | HUPCL) as tcflag_t;
        tio.c_cc[VMIN as usize] = 1 as cc_t;
        tio.c_cc[VTIME as usize] = 0 as cc_t;
        cfsetispeed(&raw mut tio, cfgetispeed(&raw mut saved_tio));
        cfsetospeed(&raw mut tio, cfgetospeed(&raw mut saved_tio));
        tcsetattr(STDIN_FILENO, TCSANOW, &raw mut tio);
    }
    client_send_identify(ttynam, termname, &caps, cwd, feat);
    proc_flush_peer(client_peer);
    if msg as ::core::ffi::c_uint == MSG_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint {
        size = 0 as size_t;
        i = 0 as ::core::ffi::c_int;
        while i < argc {
            size = size.wrapping_add(strlen(*argv.offset(i as isize)).wrapping_add(1 as size_t));
            i += 1;
        }
        if size
            > (MAX_IMSGSIZE as usize).wrapping_sub(::core::mem::size_of::<msg_command>() as usize)
        {
            fprintf(
                stderr,
                b"command too long\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return 1 as ::core::ffi::c_int;
        }
        const _: () = assert!(
            ::core::mem::size_of::<msg_command>() == ::core::mem::size_of::<::core::ffi::c_int>()
        );
        let header_size = ::core::mem::size_of::<msg_command>();
        let mut data = vec![0u8; header_size + size];
        data[..header_size].copy_from_slice(&argc.to_ne_bytes());
        if cmd_pack_argv(
            argc,
            argv,
            data[header_size..].as_mut_ptr() as *mut ::core::ffi::c_char,
            size,
        ) != 0 as ::core::ffi::c_int
        {
            fprintf(
                stderr,
                b"command too long\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return 1 as ::core::ffi::c_int;
        }
        if proc_send(
            client_peer,
            msg,
            -(1 as ::core::ffi::c_int),
            data.as_ptr() as *const ::core::ffi::c_void,
            data.len(),
        ) != 0 as ::core::ffi::c_int
        {
            fprintf(
                stderr,
                b"failed to send command\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return 1 as ::core::ffi::c_int;
        }
    } else if msg as ::core::ffi::c_uint == MSG_SHELL as ::core::ffi::c_int as ::core::ffi::c_uint {
        proc_send(
            client_peer,
            msg,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null::<::core::ffi::c_void>(),
            0 as size_t,
        );
    }
    proc_loop(client_proc, None);
    if client_exittype as ::core::ffi::c_uint
        == MSG_EXEC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
        }
        let (shell, command) = client_exec_payload
            .as_ref()
            .expect("MSG_EXEC requires a stored shell and command");
        client_exec(shell.as_ptr(), command.as_ptr());
    }
    if client_attached != 0 {
        if client_exitreason as ::core::ffi::c_uint
            != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            printf(
                b"[%s]\n\0" as *const u8 as *const ::core::ffi::c_char,
                client_exit_message(),
            );
        }
        ppid = getppid() as pid_t;
        if client_exittype as ::core::ffi::c_uint
            == MSG_DETACHKILL as ::core::ffi::c_int as ::core::ffi::c_uint
            && ppid > 1 as ::core::ffi::c_int
        {
            kill(ppid as __pid_t, SIGHUP);
        }
    } else if client_flags & CLIENT_CONTROL as uint64_t != 0 {
        if client_exitreason as ::core::ffi::c_uint
            != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            printf(
                b"%%exit %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                client_exit_message(),
            );
        } else {
            printf(b"%%exit\n\0" as *const u8 as *const ::core::ffi::c_char);
        }
        fflush(stdout);
        if client_flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
            control_wait_exit(STDIN_FILENO);
        }
        if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            printf(b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char);
            fflush(stdout);
            tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
        }
    } else if client_exitreason as ::core::ffi::c_uint
        != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fprintf(
            stderr,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            client_exit_message(),
        );
    }
    setblocking(STDIN_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDOUT_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDERR_FILENO, 1 as ::core::ffi::c_int);
    client_exitmessage.take();
    client_exitsession.take();
    return client_exitval;
}
unsafe fn client_send_identify(
    mut ttynam: *const ::core::ffi::c_char,
    mut termname: *const ::core::ffi::c_char,
    caps: &[CString],
    mut cwd: *const ::core::ffi::c_char,
    mut feat: ::core::ffi::c_int,
) {
    let mut ss: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut sslen: size_t = 0;
    let mut fd: ::core::ffi::c_int = 0;
    let mut flags: uint64_t = client_flags;
    let mut pid: pid_t = 0;
    proc_send(
        client_peer,
        MSG_IDENTIFY_LONGFLAGS,
        -(1 as ::core::ffi::c_int),
        &raw mut flags as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_LONGFLAGS,
        -(1 as ::core::ffi::c_int),
        &raw mut client_flags as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_TERM,
        -(1 as ::core::ffi::c_int),
        termname as *const ::core::ffi::c_void,
        strlen(termname).wrapping_add(1 as size_t),
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_FEATURES,
        -(1 as ::core::ffi::c_int),
        &raw mut feat as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_TTYNAME,
        -(1 as ::core::ffi::c_int),
        ttynam as *const ::core::ffi::c_void,
        strlen(ttynam).wrapping_add(1 as size_t),
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_CWD,
        -(1 as ::core::ffi::c_int),
        cwd as *const ::core::ffi::c_void,
        strlen(cwd).wrapping_add(1 as size_t),
    );
    for cap in caps {
        proc_send(
            client_peer,
            MSG_IDENTIFY_TERMINFO,
            -(1 as ::core::ffi::c_int),
            cap.as_ptr() as *const ::core::ffi::c_void,
            cap.as_bytes_with_nul().len() as size_t,
        );
    }
    fd = dup(STDIN_FILENO);
    if fd == -(1 as ::core::ffi::c_int) {
        fatal(b"dup failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    proc_send(
        client_peer,
        MSG_IDENTIFY_STDIN,
        fd,
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
    fd = dup(STDOUT_FILENO);
    if fd == -(1 as ::core::ffi::c_int) {
        fatal(b"dup failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    proc_send(
        client_peer,
        MSG_IDENTIFY_STDOUT,
        fd,
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
    pid = getpid() as pid_t;
    proc_send(
        client_peer,
        MSG_IDENTIFY_CLIENTPID,
        -(1 as ::core::ffi::c_int),
        &raw mut pid as *const ::core::ffi::c_void,
        ::core::mem::size_of::<pid_t>() as size_t,
    );
    ss = environ;
    while !(*ss).is_null() {
        sslen = strlen(*ss).wrapping_add(1 as size_t);
        if !(sslen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)) {
            proc_send(
                client_peer,
                MSG_IDENTIFY_ENVIRON,
                -(1 as ::core::ffi::c_int),
                *ss as *const ::core::ffi::c_void,
                sslen,
            );
        }
        ss = ss.offset(1);
    }
    proc_send(
        client_peer,
        MSG_IDENTIFY_DONE,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
}
unsafe extern "C" fn client_exec(
    mut shell: *const ::core::ffi::c_char,
    mut shellcmd: *const ::core::ffi::c_char,
) -> ! {
    log_debug(
        b"shell %s, command %s\0" as *const u8 as *const ::core::ffi::c_char,
        shell,
        shellcmd,
    );
    let argv0 = shell_argv0_cstring(shell, client_flags & CLIENT_LOGIN as uint64_t != 0);
    setenv(
        b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
        shell,
        1 as ::core::ffi::c_int,
    );
    proc_clear_signals(client_proc, 1 as ::core::ffi::c_int);
    setblocking(STDIN_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDOUT_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDERR_FILENO, 1 as ::core::ffi::c_int);
    closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
    execl(
        shell,
        argv0.as_ptr(),
        b"-c\0" as *const u8 as *const ::core::ffi::c_char,
        shellcmd,
        NULL as *mut ::core::ffi::c_char,
    );
    fatal(b"execl failed\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn client_signal(mut sig: ::core::ffi::c_int) {
    let mut sigact: sigaction = sigaction {
        __sigaction_handler: sigaction___sigaction_handler { sa_handler: None },
        sa_mask: __sigset_t { __val: [0; 16] },
        sa_flags: 0,
        sa_restorer: None,
    };
    let mut status: ::core::ffi::c_int = 0;
    let mut pid: pid_t = 0;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"client_signal\0" as *const u8 as *const ::core::ffi::c_char,
        strsignal(sig),
    );
    if sig == SIGCHLD {
        loop {
            pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG) as pid_t;
            if pid == 0 as ::core::ffi::c_int {
                break;
            }
            if !(pid == -(1 as ::core::ffi::c_int)) {
                continue;
            }
            if *__errno_location() == ECHILD {
                break;
            }
            log_debug(
                b"waitpid failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
        }
    } else if client_attached == 0 {
        if sig == SIGTERM || sig == SIGHUP {
            proc_exit(client_proc);
        }
    } else {
        match sig {
            SIGHUP => {
                client_exitreason = CLIENT_EXIT_LOST_TTY;
                client_exitval = 1 as ::core::ffi::c_int;
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
            }
            SIGTERM => {
                if client_suspended == 0 {
                    client_exitreason = CLIENT_EXIT_TERMINATED;
                }
                client_exitval = 1 as ::core::ffi::c_int;
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
            }
            SIGWINCH => {
                proc_send(
                    client_peer,
                    MSG_RESIZE,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
            }
            SIGCONT => {
                memset(
                    &raw mut sigact as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<sigaction>() as size_t,
                );
                sigemptyset(&raw mut sigact.sa_mask);
                sigact.sa_flags = SA_RESTART;
                sigact.__sigaction_handler.sa_handler =
                    ::core::mem::transmute::<::libc::intptr_t, __sighandler_t>(
                        1 as ::core::ffi::c_int as ::libc::intptr_t,
                    );
                if sigaction(
                    SIGTSTP,
                    &raw mut sigact,
                    ::core::ptr::null_mut::<sigaction>(),
                ) != 0 as ::core::ffi::c_int
                {
                    fatal(b"sigaction failed\0" as *const u8 as *const ::core::ffi::c_char);
                }
                proc_send(
                    client_peer,
                    MSG_WAKEUP,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
                client_suspended = 0 as ::core::ffi::c_int;
            }
            _ => {}
        }
    };
}
unsafe extern "C" fn client_file_check_cb(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    if client_exitflag != 0 {
        client_exit();
    }
}
unsafe extern "C" fn client_dispatch(mut imsg: *mut imsg, mut arg: *mut ::core::ffi::c_void) {
    if imsg.is_null() {
        if client_exitflag == 0 {
            client_exitreason = CLIENT_EXIT_LOST_SERVER;
            client_exitval = 1 as ::core::ffi::c_int;
        }
        proc_exit(client_proc);
        return;
    }
    if client_attached != 0 {
        client_dispatch_attached(imsg);
    } else {
        client_dispatch_wait(imsg);
    };
}
unsafe extern "C" fn client_dispatch_exit_message(
    mut data: *mut ::core::ffi::c_char,
    mut datalen: size_t,
) {
    let mut retval: ::core::ffi::c_int = 0;
    if datalen < ::core::mem::size_of::<::core::ffi::c_int>() as usize && datalen != 0 as size_t {
        fatalx(b"bad MSG_EXIT size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if datalen >= ::core::mem::size_of::<::core::ffi::c_int>() as usize {
        memcpy(
            &raw mut retval as *mut ::core::ffi::c_void,
            data as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
        );
        client_exitval = retval;
    }
    if datalen > ::core::mem::size_of::<::core::ffi::c_int>() as usize {
        datalen = (datalen as ::core::ffi::c_ulong)
            .wrapping_sub(
                ::core::mem::size_of::<::core::ffi::c_int>() as usize as ::core::ffi::c_ulong
            ) as size_t as size_t;
        data = data.offset(::core::mem::size_of::<::core::ffi::c_int>() as usize as isize);
        let mut message = ::core::slice::from_raw_parts(data.cast::<u8>(), datalen).to_vec();
        *message.last_mut().unwrap() = 0;
        client_exitmessage = Some(message);
        client_exitreason = CLIENT_EXIT_MESSAGE_PROVIDED;
    }
}
unsafe extern "C" fn client_dispatch_wait(mut imsg: *mut imsg) {
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut datalen: ssize_t = 0;
    static mut pledge_applied: ::core::ffi::c_int = 0;
    if pledge_applied == 0 {
        if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            fatal(b"pledge failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        pledge_applied = 1 as ::core::ffi::c_int;
    }
    data = (*imsg).data as *mut ::core::ffi::c_char;
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
    match (*imsg).hdr.type_0 {
        203 | 210 => {
            client_dispatch_exit_message(data, datalen as size_t);
            client_exitflag = 1 as ::core::ffi::c_int;
            client_exit();
        }
        207 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_READY size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_attached = 1 as ::core::ffi::c_int;
            proc_send(
                client_peer,
                MSG_RESIZE,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        12 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_VERSION size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            fprintf(
                stderr,
                b"protocol version mismatch (client %d, server %u)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                PROTOCOL_VERSION,
                (*imsg).hdr.peerid & 0xff as uint32_t,
            );
            client_exitval = 1 as ::core::ffi::c_int;
            proc_exit(client_proc);
        }
        218 => {
            if datalen as usize != ::core::mem::size_of::<uint64_t>() as usize {
                fatalx(b"bad MSG_FLAGS string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            memcpy(
                &raw mut client_flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            log_debug(
                b"new flags are %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                client_flags as ::core::ffi::c_ulonglong,
            );
        }
        209 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                fatalx(b"bad MSG_SHELL string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_exec(data, shell_command);
        }
        201 | 202 => {
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        204 => {
            proc_exit(client_proc);
        }
        300 => {
            file_read_open(
                &raw mut client_files,
                client_peer,
                imsg,
                1 as ::core::ffi::c_int,
                (client_flags & CLIENT_CONTROL as uint64_t == 0) as ::core::ffi::c_int,
                Some(
                    client_file_check_cb
                        as unsafe extern "C" fn(
                            *mut client,
                            *const ::core::ffi::c_char,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            *mut evbuffer,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                NULL,
            );
        }
        307 => {
            file_read_cancel(&raw mut client_files, imsg);
        }
        303 => {
            file_write_open(
                &raw mut client_files,
                client_peer,
                imsg,
                1 as ::core::ffi::c_int,
                (client_flags & CLIENT_CONTROL as uint64_t == 0) as ::core::ffi::c_int,
                Some(
                    client_file_check_cb
                        as unsafe extern "C" fn(
                            *mut client,
                            *const ::core::ffi::c_char,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            *mut evbuffer,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                NULL,
            );
        }
        304 => {
            file_write_data(&raw mut client_files, imsg);
        }
        306 => {
            file_write_close(&raw mut client_files, imsg);
        }
        211..=213 => {
            fprintf(
                stderr,
                b"server version is too old for client\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            proc_exit(client_proc);
        }
        _ => {
            log_debug(
                b"unknown message type %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*imsg).hdr.type_0,
            );
        }
    };
}
unsafe extern "C" fn client_dispatch_attached(mut imsg: *mut imsg) {
    let mut sigact: sigaction = sigaction {
        __sigaction_handler: sigaction___sigaction_handler { sa_handler: None },
        sa_mask: __sigset_t { __val: [0; 16] },
        sa_flags: 0,
        sa_restorer: None,
    };
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut datalen: ssize_t = 0;
    data = (*imsg).data as *mut ::core::ffi::c_char;
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
    match (*imsg).hdr.type_0 {
        218 => {
            if datalen as usize != ::core::mem::size_of::<uint64_t>() as usize {
                fatalx(b"bad MSG_FLAGS string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            memcpy(
                &raw mut client_flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            log_debug(
                b"new flags are %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                client_flags as ::core::ffi::c_ulonglong,
            );
        }
        201 | 202 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                fatalx(b"bad MSG_DETACH string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_exitsession = Some(std::ffi::CStr::from_ptr(data).to_owned());
            client_exittype = (*imsg).hdr.type_0 as msgtype;
            if (*imsg).hdr.type_0 == MSG_DETACHKILL as ::core::ffi::c_int as uint32_t {
                client_exitreason = CLIENT_EXIT_DETACHED_HUP;
            } else {
                client_exitreason = CLIENT_EXIT_DETACHED;
            }
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        217 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
                || strlen(data).wrapping_add(1 as size_t) == datalen as size_t
            {
                fatalx(b"bad MSG_EXEC string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            let command = std::ffi::CStr::from_ptr(data).to_owned();
            let shell = std::ffi::CStr::from_ptr(data.add(strlen(data) + 1)).to_owned();
            client_exec_payload = Some((shell, command));
            client_exittype = (*imsg).hdr.type_0 as msgtype;
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        203 => {
            client_dispatch_exit_message(data, datalen as size_t);
            if client_exitreason as ::core::ffi::c_uint
                == CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                client_exitreason = CLIENT_EXIT_EXITED;
            }
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        204 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_EXITED size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            proc_exit(client_proc);
        }
        210 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_SHUTDOWN size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
            client_exitreason = CLIENT_EXIT_SERVER_EXITED;
            client_exitval = 1 as ::core::ffi::c_int;
        }
        214 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_SUSPEND size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            memset(
                &raw mut sigact as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<sigaction>() as size_t,
            );
            sigemptyset(&raw mut sigact.sa_mask);
            sigact.sa_flags = SA_RESTART;
            sigact.__sigaction_handler.sa_handler = SIG_DFL;
            if sigaction(
                SIGTSTP,
                &raw mut sigact,
                ::core::ptr::null_mut::<sigaction>(),
            ) != 0 as ::core::ffi::c_int
            {
                fatal(b"sigaction failed\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_suspended = 1 as ::core::ffi::c_int;
            kill(getpid(), SIGTSTP);
        }
        206 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                fatalx(b"bad MSG_LOCK string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            system(data);
            proc_send(
                client_peer,
                MSG_UNLOCK,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        _ => {
            log_debug(
                b"unknown message type %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*imsg).hdr.type_0,
            );
        }
    };
}
