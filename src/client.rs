use crate::arguments::args_from_vector;
use crate::cmd::cmd_list_any_have;
use crate::cmd::cmd_pack_argv;
use crate::cmd::cmd_parse_from_arguments;
use crate::compat::imsgbuf_flush;
use crate::compat::systemd_activated;
use crate::environ::environ_process;
use crate::ffi::{
    __errno_location, cfgetispeed, cfgetospeed, cfmakeraw, cfsetispeed, cfsetospeed, close,
    closefrom, connect, dup, execl, fflush, flock, fprintf, getenv, getpid, getppid, isatty, kill,
    open, printf, setenv, sigaction, sigemptyset, socket, stderr, stdout, strerror, strlcpy,
    strlen, strsignal, system, tcgetattr, tcsetattr, ttyname, waitpid,
};
use crate::file::{
    file_read_cancel, file_read_open, file_write_close, file_write_data, file_write_left,
    file_write_open,
};
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::log::{fatal, fatalx, log_debug};
use crate::proc::{
    proc_add_peer, proc_clear_signals, proc_exit, proc_flush_peer, proc_loop, proc_send,
    proc_set_signals, proc_start,
};
use crate::reactor;
use crate::server::server_start;
use crate::terminfo::tty_term_read_list;
use crate::tmux::{find_cwd, find_home, setblocking, shell_argv0};
use crate::tmux::{global_options_free, ptm_fd, shell_command, socket_path};
use crate::tree::GlobalQueue;
use crate::tree::GlobalTree;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use std::ffi::{CStr, OsStr};
use std::fs;
use std::io::{BufRead, ErrorKind};
use std::os::unix::ffi::OsStrExt;
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
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const CLIENT_EXIT_MESSAGE_PROVIDED: client_exit_reason = 8;
pub const CLIENT_EXIT_SERVER_EXITED: client_exit_reason = 7;
pub const CLIENT_EXIT_EXITED: client_exit_reason = 6;
pub const CLIENT_EXIT_LOST_SERVER: client_exit_reason = 5;
pub const CLIENT_EXIT_TERMINATED: client_exit_reason = 4;
pub const CLIENT_EXIT_LOST_TTY: client_exit_reason = 3;
pub const CLIENT_EXIT_DETACHED_HUP: client_exit_reason = 2;
pub const CLIENT_EXIT_DETACHED: client_exit_reason = 1;
pub const CLIENT_EXIT_NONE: client_exit_reason = 0;
pub type client_exit_reason = ::core::ffi::c_uint;
pub const SIG_DFL: __sighandler_t = None;
pub const SIGTERM: ::core::ffi::c_int = 15;
pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;
pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;
pub const WNOHANG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SIGTSTP: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SIGCONT: ::core::ffi::c_int = 18;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const SIGWINCH: ::core::ffi::c_int = 28;
pub const SA_RESTART: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENAMETOOLONG: ::core::ffi::c_int = 36 as ::core::ffi::c_int;
pub const ECONNREFUSED: ::core::ffi::c_int = 111 as ::core::ffi::c_int;
pub const WAIT_ANY: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const LOCK_EX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOCK_NB: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ECHILD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const EAGAIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const VTIME: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const VMIN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const ICRNL: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const IXANY: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const OPOST: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ONLCR: ::core::ffi::c_int = 0o4 as ::core::ffi::c_int;
pub const CS8: ::core::ffi::c_int = 0o60 as ::core::ffi::c_int;
pub const CREAD: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const HUPCL: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const TCSANOW: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TCSAFLUSH: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const PROTOCOL_VERSION: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CLIENT_LOGIN: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_NOSTARTSERVER: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_CONTROLCONTROL: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const CLIENT_STARTSERVER: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_WAITEXIT: ::core::ffi::c_ulonglong =
    0x200000000 as ::core::ffi::c_ulonglong;
pub(crate) static mut client_proc: *mut tmuxproc = ::core::ptr::null::<tmuxproc>() as *mut tmuxproc;
pub(crate) static mut client_peer: *mut tmuxpeer = ::core::ptr::null::<tmuxpeer>() as *mut tmuxpeer;

/// The peer the client speaks to the server through. `client_peer` is the
/// borrowed view of the box parked here.
static client_peers: GlobalQueue<Box<tmuxpeer>> = GlobalQueue::new();

/// Flushes what the client still owes the server, then asks the loop to stop.
unsafe fn client_exit_proc() {
    unsafe {
        if !client_peer.is_null() {
            imsgbuf_flush(&mut (*client_peer).ibuf);
        }
        proc_exit(client_proc);
    }
}
pub(crate) static mut client_flags: uint64_t = 0;
pub(crate) static mut client_suspended: ::core::ffi::c_int = 0;
pub(crate) static mut client_exitreason: client_exit_reason = CLIENT_EXIT_NONE;
pub(crate) static mut client_exitflag: ::core::ffi::c_int = 0;
pub(crate) static mut client_exitval: ::core::ffi::c_int = 0;
pub(crate) static mut client_exittype: msgtype = 0 as msgtype;
pub(crate) static mut client_exitsession: Option<::std::ffi::CString> = None;
pub(crate) static mut client_exitmessage: Option<::std::ffi::CString> = None;
pub(crate) static mut client_execshell: Option<::std::ffi::CString> = None;
pub(crate) static mut client_execcmd: Option<::std::ffi::CString> = None;
pub(crate) static mut client_attached: ::core::ffi::c_int = 0;
pub(crate) static client_files: GlobalTree<::core::ffi::c_int, ClientFileRef> = GlobalTree::new();

pub(crate) unsafe fn client_get_lock(mut lockfile: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut lockfd: ::core::ffi::c_int = 0;
        log_debug(c"lock file is %s".as_ptr(), fmt_args![lockfile]);
        lockfd = open(lockfile, O_WRONLY | O_CREAT, 0o600 as ::core::ffi::c_int);
        if lockfd == -(1 as ::core::ffi::c_int) {
            log_debug(
                c"open failed: %s".as_ptr(),
                fmt_args![strerror(*__errno_location())],
            );
            return -(1 as ::core::ffi::c_int);
        }
        if flock(lockfd, LOCK_EX | LOCK_NB) == -(1 as ::core::ffi::c_int) {
            log_debug(
                c"flock failed: %s".as_ptr(),
                fmt_args![strerror(*__errno_location())],
            );
            if *__errno_location() != EAGAIN {
                return lockfd;
            }
            while flock(lockfd, LOCK_EX) == -(1 as ::core::ffi::c_int)
                && *__errno_location() == EINTR
            {}
            close(lockfd);
            return -(2 as ::core::ffi::c_int);
        }
        log_debug(c"flock succeeded".as_ptr(), fmt_args![]);
        lockfd
    }
}
pub(crate) unsafe fn client_connect(
    mut base: reactor::Base,
    mut path: *const ::core::ffi::c_char,
    mut flags: uint64_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut current_block: u64;
        let mut sa = sockaddr_un::default();
        let mut size: size_t = 0;
        let mut fd: ::core::ffi::c_int = 0;
        let mut lockfd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
        let mut locked: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lockfile: Option<::std::ffi::CString> = None;
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
        log_debug(c"socket is %s".as_ptr(), fmt_args![path]);
        loop {
            fd = socket(
                AF_UNIX,
                SOCK_STREAM as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if fd == -(1 as ::core::ffi::c_int) {
                return -(1 as ::core::ffi::c_int);
            }
            log_debug(c"trying connect".as_ptr(), fmt_args![]);
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
                c"connect failed: %s".as_ptr(),
                fmt_args![strerror(*__errno_location())],
            );
            if *__errno_location() != ECONNREFUSED && *__errno_location() != ENOENT {
                current_block = 15782065984124042354;
                break;
            }
            if flags & CLIENT_NOSTARTSERVER as uint64_t != 0 {
                current_block = 15782065984124042354;
                break;
            }
            if !flags & CLIENT_STARTSERVER as uint64_t != 0 {
                current_block = 15782065984124042354;
                break;
            }
            close(fd);
            if locked == 0 {
                let new_lockfile = xasprintf(c"%s.lock".as_ptr(), fmt_args![path]);
                lockfd = client_get_lock(new_lockfile.as_ptr() as *mut ::core::ffi::c_char);
                if lockfd < 0 as ::core::ffi::c_int {
                    log_debug(c"didn't get lock (%d)".as_ptr(), fmt_args![lockfd]);
                    drop(new_lockfile);
                    lockfile = None;
                    if lockfd == -(2 as ::core::ffi::c_int) {
                        continue;
                    }
                } else {
                    lockfile = Some(new_lockfile);
                }
                log_debug(c"got lock (%d)".as_ptr(), fmt_args![lockfd]);
                locked = 1 as ::core::ffi::c_int;
            } else {
                if lockfd >= 0 as ::core::ffi::c_int
                    && let Err(err) =
                        fs::remove_file(OsStr::from_bytes(CStr::from_ptr(path).to_bytes()))
                    && err.kind() != ErrorKind::NotFound
                {
                    drop(lockfile.take());
                    close(lockfd);
                    return -(1 as ::core::ffi::c_int);
                }
                fd = server_start(client_proc, flags, base, lockfd, lockfile.take());
                current_block = 7172762164747879670;
                break;
            }
        }
        match current_block {
            15782065984124042354 => {
                if locked != 0 {
                    drop(lockfile.take());
                    close(lockfd);
                }
                close(fd);
                -(1 as ::core::ffi::c_int)
            }
            _ => {
                if locked != 0 && lockfd >= 0 as ::core::ffi::c_int {
                    drop(lockfile.take());
                    close(lockfd);
                }
                setblocking(fd, 0 as ::core::ffi::c_int);
                fd
            }
        }
    }
}
/// Why the client is going, as the caller's own string.
pub(crate) unsafe fn client_exit_message() -> ::std::ffi::CString {
    unsafe {
        match client_exitreason {
            CLIENT_EXIT_DETACHED => {
                let exitsession = &raw const client_exitsession;
                match (*exitsession).as_ref() {
                    Some(session) => format_alloc(
                        c"detached (from session %s)".as_ptr(),
                        fmt_args![session.as_ptr()],
                    ),
                    None => c"detached".to_owned(),
                }
            }
            CLIENT_EXIT_DETACHED_HUP => {
                let exitsession = &raw const client_exitsession;
                match (*exitsession).as_ref() {
                    Some(session) => format_alloc(
                        c"detached and SIGHUP (from session %s)".as_ptr(),
                        fmt_args![session.as_ptr()],
                    ),
                    None => c"detached and SIGHUP".to_owned(),
                }
            }
            CLIENT_EXIT_LOST_TTY => c"lost tty".to_owned(),
            CLIENT_EXIT_TERMINATED => c"terminated".to_owned(),
            CLIENT_EXIT_LOST_SERVER => c"server exited unexpectedly".to_owned(),
            CLIENT_EXIT_EXITED => c"exited".to_owned(),
            CLIENT_EXIT_SERVER_EXITED => c"server exited".to_owned(),
            CLIENT_EXIT_MESSAGE_PROVIDED => {
                let exitmessage = &raw const client_exitmessage;
                match (*exitmessage).as_ref() {
                    Some(message) => message.clone(),
                    None => c"unknown reason".to_owned(),
                }
            }
            _ => c"unknown reason".to_owned(),
        }
    }
}
pub(crate) unsafe fn client_exit() {
    unsafe {
        if file_write_left(client_files.map() as *mut client_files_t) == 0 {
            client_exit_proc();
        }
    }
}
pub unsafe fn client_main(
    mut base: reactor::Base,
    argv: &[::std::ffi::CString],
    mut flags: uint64_t,
    mut feat: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fd: ::core::ffi::c_int = 0;
        let mut ttynam: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut termname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut ppid: pid_t = 0;
        let mut msg: msgtype = 0 as msgtype;
        let mut tio: termios = ::core::mem::zeroed();
        let mut saved_tio: termios = ::core::mem::zeroed();
        let mut size: size_t = 0;
        let argc = argv.len() as ::core::ffi::c_int;
        if shell_command.is_some() {
            msg = MSG_SHELL;
            flags |= CLIENT_STARTSERVER as uint64_t;
        } else if argv.is_empty() {
            msg = MSG_COMMAND;
            flags |= CLIENT_STARTSERVER as uint64_t;
        } else {
            msg = MSG_COMMAND;
            let mut values = args_from_vector(argv);
            let mut pr = cmd_parse_from_arguments(
                values.as_mut_ptr(),
                argc as u_int,
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            if pr.status as ::core::ffi::c_uint
                == CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if cmd_list_any_have(pr.cmdlist.as_ref().unwrap(), CMD_STARTSERVER) != 0 {
                    flags |= CLIENT_STARTSERVER as uint64_t;
                }
                let _ = pr.cmdlist.take();
            } else {
                let _ = pr.error.take();
            }
        }
        client_proc = proc_start(c"client".as_ptr());
        proc_set_signals(client_proc, Some(client_signal));
        client_flags = flags;
        log_debug(
            c"flags are %#llx".as_ptr(),
            fmt_args![client_flags as ::core::ffi::c_ulonglong],
        );
        if systemd_activated() != 0 {
            fd = server_start(client_proc, flags, base, 0 as ::core::ffi::c_int, None);
        } else {
            fd = client_connect(
                base,
                socket_path
                    .as_deref()
                    .map_or(::core::ptr::null(), CStr::as_ptr),
                client_flags,
            );
        }
        if fd == -(1 as ::core::ffi::c_int) {
            if *__errno_location() == ECONNREFUSED {
                fprintf(
                    stderr,
                    c"no server running on %s\n".as_ptr(),
                    socket_path
                        .as_deref()
                        .map_or(::core::ptr::null(), CStr::as_ptr),
                );
            } else {
                fprintf(
                    stderr,
                    c"error connecting to %s (%s)\n".as_ptr(),
                    socket_path
                        .as_deref()
                        .map_or(::core::ptr::null(), CStr::as_ptr),
                    strerror(*__errno_location()),
                );
            }
            return 1 as ::core::ffi::c_int;
        }
        let mut peer_box = proc_add_peer(client_proc, fd, Some(client_dispatch), None);
        client_peer = &raw mut *peer_box;
        client_peers.queue().push_back(peer_box);
        let found_cwd = find_cwd();
        cwd = match (&found_cwd, find_home()) {
            (Some(found), _) => found.as_ptr(),
            (None, Some(home)) => home.as_ptr(),
            (None, None) => c"/".as_ptr(),
        };
        ttynam = ttyname(STDIN_FILENO);
        if ttynam.is_null() {
            ttynam = c"".as_ptr();
        }
        termname = getenv(c"TERM".as_ptr());
        if termname.is_null() {
            termname = c"".as_ptr();
        }
        let caps = if isatty(STDIN_FILENO) != 0 && *termname as ::core::ffi::c_int != '\0' as i32 {
            match tty_term_read_list(termname, STDIN_FILENO) {
                Ok(caps) => caps,
                Err(cause) => {
                    fprintf(stderr, c"%s\n".as_ptr(), cause.as_ptr());
                    return 1 as ::core::ffi::c_int;
                }
            }
        } else {
            Vec::new()
        };
        if ptm_fd != -(1 as ::core::ffi::c_int) {
            close(ptm_fd);
        }
        global_options_free();
        if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            if tcgetattr(STDIN_FILENO, &raw mut saved_tio) != 0 as ::core::ffi::c_int {
                fprintf(
                    stderr,
                    c"tcgetattr failed: %s\n".as_ptr(),
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
            size = argv.iter().map(|arg| arg.as_bytes_with_nul().len()).sum();
            let header_size = ::core::mem::size_of::<msg_command>();
            if size > (MAX_IMSGSIZE as usize).wrapping_sub(header_size) {
                fprintf(stderr, c"command too long\n".as_ptr());
                return 1 as ::core::ffi::c_int;
            }
            let header = msg_command { argc };
            let mut data: Vec<u8> = Vec::with_capacity(header_size.wrapping_add(size as usize));
            data.extend_from_slice(::core::slice::from_raw_parts(
                &raw const header as *const u8,
                header_size,
            ));
            data.resize(header_size.wrapping_add(size as usize), 0u8);
            if cmd_pack_argv(
                argv,
                data.as_mut_ptr().add(header_size) as *mut ::core::ffi::c_char,
                size,
            ) != 0 as ::core::ffi::c_int
            {
                fprintf(stderr, c"command too long\n".as_ptr());
                return 1 as ::core::ffi::c_int;
            }
            if proc_send(
                client_peer,
                msg,
                -(1 as ::core::ffi::c_int),
                data.as_ptr(),
                data.len() as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                fprintf(stderr, c"failed to send command\n".as_ptr());
                return 1 as ::core::ffi::c_int;
            }
        } else if msg as ::core::ffi::c_uint
            == MSG_SHELL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            proc_send(
                client_peer,
                msg,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<u8>(),
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
            let execshell = &raw const client_execshell;
            let execcmd = &raw const client_execcmd;
            client_exec((*execshell).as_deref(), (*execcmd).as_deref());
        }
        setblocking(STDIN_FILENO, 1 as ::core::ffi::c_int);
        setblocking(STDOUT_FILENO, 1 as ::core::ffi::c_int);
        setblocking(STDERR_FILENO, 1 as ::core::ffi::c_int);
        if client_attached != 0 {
            if client_exitreason as ::core::ffi::c_uint
                != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                printf(c"[%s]\n".as_ptr(), client_exit_message().as_ptr());
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
                printf(c"%%exit %s\n".as_ptr(), client_exit_message().as_ptr());
            } else {
                printf(c"%%exit\n".as_ptr());
            }
            fflush(stdout);
            if client_flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
                let mut stdin = ::std::io::stdin().lock();
                let mut line = Vec::<u8>::new();
                loop {
                    line.clear();
                    match stdin.read_until(b'\n', &mut line) {
                        Ok(linelen) if linelen > 1 => {}
                        _ => break,
                    }
                }
            }
            if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
                printf(c"\x1B\\".as_ptr());
                fflush(stdout);
                tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
            }
        } else if client_exitreason as ::core::ffi::c_uint
            != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            fprintf(stderr, c"%s\n".as_ptr(), client_exit_message().as_ptr());
        }
        client_exitval
    }
}
unsafe fn client_send_identify(
    mut ttynam: *const ::core::ffi::c_char,
    mut termname: *const ::core::ffi::c_char,
    caps: &[::std::ffi::CString],
    mut cwd: *const ::core::ffi::c_char,
    mut feat: ::core::ffi::c_int,
) {
    unsafe {
        let mut fd: ::core::ffi::c_int = 0;
        let mut flags: uint64_t = client_flags;
        let mut pid: pid_t = 0;
        proc_send(
            client_peer,
            MSG_IDENTIFY_LONGFLAGS,
            -(1 as ::core::ffi::c_int),
            &raw mut flags as *const u8,
            ::core::mem::size_of::<uint64_t>() as size_t,
        );
        proc_send(
            client_peer,
            MSG_IDENTIFY_LONGFLAGS,
            -(1 as ::core::ffi::c_int),
            &raw mut client_flags as *const u8,
            ::core::mem::size_of::<uint64_t>() as size_t,
        );
        proc_send(
            client_peer,
            MSG_IDENTIFY_TERM,
            -(1 as ::core::ffi::c_int),
            termname as *const u8,
            strlen(termname).wrapping_add(1 as size_t),
        );
        proc_send(
            client_peer,
            MSG_IDENTIFY_FEATURES,
            -(1 as ::core::ffi::c_int),
            &raw mut feat as *const u8,
            ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
        );
        proc_send(
            client_peer,
            MSG_IDENTIFY_TTYNAME,
            -(1 as ::core::ffi::c_int),
            ttynam as *const u8,
            strlen(ttynam).wrapping_add(1 as size_t),
        );
        proc_send(
            client_peer,
            MSG_IDENTIFY_CWD,
            -(1 as ::core::ffi::c_int),
            cwd as *const u8,
            strlen(cwd).wrapping_add(1 as size_t),
        );
        for cap in caps {
            proc_send(
                client_peer,
                MSG_IDENTIFY_TERMINFO,
                -(1 as ::core::ffi::c_int),
                cap.as_ptr() as *const u8,
                cap.as_bytes_with_nul().len() as size_t,
            );
        }
        fd = dup(STDIN_FILENO);
        if fd == -(1 as ::core::ffi::c_int) {
            fatal(c"dup failed".as_ptr(), fmt_args![]);
        }
        proc_send(
            client_peer,
            MSG_IDENTIFY_STDIN,
            fd,
            ::core::ptr::null::<u8>(),
            0 as size_t,
        );
        fd = dup(STDOUT_FILENO);
        if fd == -(1 as ::core::ffi::c_int) {
            fatal(c"dup failed".as_ptr(), fmt_args![]);
        }
        proc_send(
            client_peer,
            MSG_IDENTIFY_STDOUT,
            fd,
            ::core::ptr::null::<u8>(),
            0 as size_t,
        );
        pid = getpid() as pid_t;
        proc_send(
            client_peer,
            MSG_IDENTIFY_CLIENTPID,
            -(1 as ::core::ffi::c_int),
            &raw mut pid as *const u8,
            ::core::mem::size_of::<pid_t>() as size_t,
        );
        for var in environ_process() {
            let sslen = var.to_bytes_with_nul().len() as size_t;
            if !(sslen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)) {
                proc_send(
                    client_peer,
                    MSG_IDENTIFY_ENVIRON,
                    -(1 as ::core::ffi::c_int),
                    var.as_ptr() as *const u8,
                    sslen,
                );
            }
        }
        proc_send(
            client_peer,
            MSG_IDENTIFY_DONE,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null::<u8>(),
            0 as size_t,
        );
    }
}
unsafe fn client_exec(shell: Option<&CStr>, shellcmd: Option<&CStr>) -> ! {
    unsafe {
        log_debug(c"shell %s, command %s".as_ptr(), fmt_args![shell, shellcmd]);
        let shell = shell.map_or(::core::ptr::null(), CStr::as_ptr);
        let argv0 = shell_argv0(
            shell,
            (client_flags & CLIENT_LOGIN as uint64_t != 0) as ::core::ffi::c_int,
        );
        setenv(c"SHELL".as_ptr(), shell, 1 as ::core::ffi::c_int);
        proc_clear_signals(client_proc, 1 as ::core::ffi::c_int);
        setblocking(STDIN_FILENO, 1 as ::core::ffi::c_int);
        setblocking(STDOUT_FILENO, 1 as ::core::ffi::c_int);
        setblocking(STDERR_FILENO, 1 as ::core::ffi::c_int);
        closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
        execl(
            shell,
            argv0.as_ptr() as *mut ::core::ffi::c_char,
            c"-c".as_ptr(),
            shellcmd.map_or(::core::ptr::null(), CStr::as_ptr),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        fatal(c"execl failed".as_ptr(), fmt_args![]);
    }
}
pub(crate) unsafe fn client_signal(mut sig: ::core::ffi::c_int) {
    unsafe {
        let mut sigact: libc::sigaction = ::core::mem::zeroed();
        let mut status: ::core::ffi::c_int = 0;
        let mut pid: pid_t = 0;
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"client_signal".as_ptr(), strsignal(sig)],
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
                    c"waitpid failed: %s".as_ptr(),
                    fmt_args![strerror(*__errno_location())],
                );
            }
        } else if client_attached == 0 {
            if sig == SIGTERM || sig == SIGHUP {
                client_exit_proc();
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
                        ::core::ptr::null::<u8>(),
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
                        ::core::ptr::null::<u8>(),
                        0 as size_t,
                    );
                }
                SIGWINCH => {
                    proc_send(
                        client_peer,
                        MSG_RESIZE,
                        -(1 as ::core::ffi::c_int),
                        ::core::ptr::null::<u8>(),
                        0 as size_t,
                    );
                }
                SIGCONT => {
                    sigact = ::core::mem::zeroed();
                    sigemptyset(&raw mut sigact.sa_mask);
                    sigact.sa_flags = SA_RESTART;
                    sigact.sa_sigaction = ::libc::SIG_IGN;
                    if sigaction(
                        SIGTSTP,
                        &raw mut sigact,
                        ::core::ptr::null_mut::<libc::sigaction>(),
                    ) != 0 as ::core::ffi::c_int
                    {
                        fatal(c"sigaction failed".as_ptr(), fmt_args![]);
                    }
                    proc_send(
                        client_peer,
                        MSG_WAKEUP,
                        -(1 as ::core::ffi::c_int),
                        ::core::ptr::null::<u8>(),
                        0 as size_t,
                    );
                    client_suspended = 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        };
    }
}
pub(crate) unsafe fn client_file_check_cb(
    _c: *mut client,
    _path: *const ::core::ffi::c_char,
    _error: ::core::ffi::c_int,
    _closed: ::core::ffi::c_int,
    _buffer: *mut Buf,
    _data: ClientFileData,
) {
    unsafe {
        if client_exitflag != 0 {
            client_exit();
        }
    }
}
pub(crate) unsafe fn client_dispatch(mut imsg: *mut imsg, _arg: *mut client) {
    unsafe {
        if imsg.is_null() {
            if client_exitflag == 0 {
                client_exitreason = CLIENT_EXIT_LOST_SERVER;
                client_exitval = 1 as ::core::ffi::c_int;
            }
            client_exit_proc();
            return;
        }
        if client_attached != 0 {
            client_dispatch_attached(imsg);
        } else {
            client_dispatch_wait(imsg);
        };
    }
}
pub(crate) unsafe fn client_dispatch_exit_message(
    mut data: *mut ::core::ffi::c_char,
    mut datalen: size_t,
) {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0;
        if datalen < ::core::mem::size_of::<::core::ffi::c_int>() as usize && datalen != 0 as size_t
        {
            fatalx(c"bad MSG_EXIT size".as_ptr(), fmt_args![]);
        }
        if datalen >= ::core::mem::size_of::<::core::ffi::c_int>() as usize {
            retval = ::core::ptr::read_unaligned(data as *const ::core::ffi::c_int);
            client_exitval = retval;
        }
        if datalen > ::core::mem::size_of::<::core::ffi::c_int>() as usize {
            datalen = (datalen as ::core::ffi::c_ulong)
                .wrapping_sub(
                    ::core::mem::size_of::<::core::ffi::c_int>() as usize as ::core::ffi::c_ulong
                ) as size_t as size_t;
            data = data.add(::core::mem::size_of::<::core::ffi::c_int>() as usize);
            let bytes = ::core::slice::from_raw_parts(data as *const u8, datalen);
            let end = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(datalen.wrapping_sub(1 as size_t));
            client_exitmessage = ::std::ffi::CString::new(&bytes[..end]).ok();
            client_exitreason = CLIENT_EXIT_MESSAGE_PROVIDED;
        }
    }
}
pub(crate) unsafe fn client_dispatch_wait(mut imsg: *mut imsg) {
    unsafe {
        let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut datalen: ssize_t = 0;
        data = (*imsg).data as *mut ::core::ffi::c_char;
        datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
        match (*imsg).hdr.type_0 {
            MSG_EXIT | MSG_SHUTDOWN => {
                client_dispatch_exit_message(data, datalen as size_t);
                client_exitflag = 1 as ::core::ffi::c_int;
                client_exit();
            }
            MSG_READY => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_READY size".as_ptr(), fmt_args![]);
                }
                client_attached = 1 as ::core::ffi::c_int;
                proc_send(
                    client_peer,
                    MSG_RESIZE,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            MSG_VERSION => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_VERSION size".as_ptr(), fmt_args![]);
                }
                fprintf(
                    stderr,
                    c"protocol version mismatch (client %d, server %u)\n".as_ptr(),
                    PROTOCOL_VERSION,
                    (*imsg).hdr.peerid & 0xff as uint32_t,
                );
                client_exitval = 1 as ::core::ffi::c_int;
                client_exit_proc();
            }
            MSG_FLAGS => {
                if datalen as usize != ::core::mem::size_of::<uint64_t>() as usize {
                    fatalx(c"bad MSG_FLAGS string".as_ptr(), fmt_args![]);
                }
                client_flags = ::core::ptr::read_unaligned(data as *const uint64_t);
                log_debug(
                    c"new flags are %#llx".as_ptr(),
                    fmt_args![client_flags as ::core::ffi::c_ulonglong],
                );
            }
            MSG_SHELL => {
                if datalen == 0 as ssize_t
                    || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    fatalx(c"bad MSG_SHELL string".as_ptr(), fmt_args![]);
                }
                client_exec(Some(CStr::from_ptr(data)), shell_command.as_deref());
            }
            MSG_DETACH | MSG_DETACHKILL => {
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            MSG_EXITED => {
                client_exit_proc();
            }
            MSG_READ_OPEN => {
                file_read_open(
                    client_files.map() as *mut client_files_t,
                    client_peer,
                    imsg,
                    1 as ::core::ffi::c_int,
                    (client_flags & CLIENT_CONTROL as uint64_t == 0) as ::core::ffi::c_int,
                    Some(client_file_check_cb),
                    ClientFileData::None,
                );
            }
            MSG_READ_CANCEL => {
                file_read_cancel(client_files.map() as *mut client_files_t, imsg);
            }
            MSG_WRITE_OPEN => {
                file_write_open(
                    client_files.map() as *mut client_files_t,
                    client_peer,
                    imsg,
                    1 as ::core::ffi::c_int,
                    (client_flags & CLIENT_CONTROL as uint64_t == 0) as ::core::ffi::c_int,
                    Some(client_file_check_cb),
                    ClientFileData::None,
                );
            }
            MSG_WRITE => {
                file_write_data(client_files.map() as *mut client_files_t, imsg);
            }
            MSG_WRITE_CLOSE => {
                file_write_close(client_files.map() as *mut client_files_t, imsg);
            }
            MSG_OLDSTDERR | MSG_OLDSTDIN | MSG_OLDSTDOUT => {
                fprintf(stderr, c"server version is too old for client\n".as_ptr());
                client_exit_proc();
            }
            _ => {}
        };
    }
}
pub(crate) unsafe fn client_dispatch_attached(mut imsg: *mut imsg) {
    unsafe {
        let mut sigact: libc::sigaction = ::core::mem::zeroed();
        let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut datalen: ssize_t = 0;
        data = (*imsg).data as *mut ::core::ffi::c_char;
        datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
        match (*imsg).hdr.type_0 {
            MSG_FLAGS => {
                if datalen as usize != ::core::mem::size_of::<uint64_t>() as usize {
                    fatalx(c"bad MSG_FLAGS string".as_ptr(), fmt_args![]);
                }
                client_flags = ::core::ptr::read_unaligned(data as *const uint64_t);
                log_debug(
                    c"new flags are %#llx".as_ptr(),
                    fmt_args![client_flags as ::core::ffi::c_ulonglong],
                );
            }
            MSG_DETACH | MSG_DETACHKILL => {
                if datalen == 0 as ssize_t
                    || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    fatalx(c"bad MSG_DETACH string".as_ptr(), fmt_args![]);
                }
                client_exitsession = Some(::std::ffi::CStr::from_ptr(data).to_owned());
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
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            MSG_EXEC => {
                if datalen == 0 as ssize_t
                    || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                        != '\0' as i32
                    || strlen(data).wrapping_add(1 as size_t) == datalen as size_t
                {
                    fatalx(c"bad MSG_EXEC string".as_ptr(), fmt_args![]);
                }
                client_execcmd = Some(::std::ffi::CStr::from_ptr(data).to_owned());
                client_execshell = Some(
                    ::std::ffi::CStr::from_ptr(
                        data.add(strlen(data))
                            .offset(1 as ::core::ffi::c_int as isize),
                    )
                    .to_owned(),
                );
                client_exittype = (*imsg).hdr.type_0 as msgtype;
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            MSG_EXIT => {
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
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            MSG_EXITED => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_EXITED size".as_ptr(), fmt_args![]);
                }
                client_exit_proc();
            }
            MSG_SHUTDOWN => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_SHUTDOWN size".as_ptr(), fmt_args![]);
                }
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
                client_exitreason = CLIENT_EXIT_SERVER_EXITED;
                client_exitval = 1 as ::core::ffi::c_int;
            }
            MSG_SUSPEND => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_SUSPEND size".as_ptr(), fmt_args![]);
                }
                sigact = ::core::mem::zeroed();
                sigemptyset(&raw mut sigact.sa_mask);
                sigact.sa_flags = SA_RESTART;
                sigact.sa_sigaction = ::libc::SIG_DFL;
                if sigaction(
                    SIGTSTP,
                    &raw mut sigact,
                    ::core::ptr::null_mut::<libc::sigaction>(),
                ) != 0 as ::core::ffi::c_int
                {
                    fatal(c"sigaction failed".as_ptr(), fmt_args![]);
                }
                client_suspended = 1 as ::core::ffi::c_int;
                kill(getpid(), SIGTSTP);
            }
            MSG_LOCK => {
                if datalen == 0 as ssize_t
                    || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    fatalx(c"bad MSG_LOCK string".as_ptr(), fmt_args![]);
                }
                system(data);
                proc_send(
                    client_peer,
                    MSG_UNLOCK,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            _ => {}
        };
    }
}
