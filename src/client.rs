use crate::ImsgMessage;
use crate::args::args_from_vector;

use crate::cmd::cmd_parse_from_arguments;
use crate::compat::imsgbuf_flush;
use crate::compat::systemd_activated;
use crate::compat::{error_message, signal_description};
pub use crate::consts::{
    AF_UNIX, CLIENT_CONTROL, CLIENT_CONTROL_WAITEXIT, CLIENT_CONTROLCONTROL, CLIENT_EXIT_DETACH,
    CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CLIENT_LOGIN, CLIENT_NOSTARTSERVER, CMD_PARSE_ERROR,
    CMD_PARSE_SUCCESS, CMD_STARTSERVER, EAGAIN, ECHILD, EINTR, ENAMETOOLONG, ENOENT, ICRNL,
    IMSG_HEADER_SIZE, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MAX_IMSGSIZE,
    MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING,
    MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, O_CREAT, O_WRONLY, ONLCR, OPOST, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PF_LOCAL, PF_UNIX,
    PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL,
    PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID,
    PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, PROTOCOL_VERSION,
    SA_RESTART, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, SIG_DFL, SIGCHLD, SIGCONT, SIGHUP, SIGTERM, SIGTSTP, SIGWINCH,
    SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM,
    SOCK_SEQPACKET, SOCK_STREAM, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    TCSANOW, THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, VMIN, VTIME, WAIT_ANY, WNOHANG,
};
use crate::environ::{process_environment, process_environment_value};
use crate::ffi::{
    __errno_location, cfgetispeed, cfgetospeed, cfmakeraw, cfsetispeed, cfsetospeed, close,
    closefrom, connect, dup, execl, flock, getpid, getppid, isatty, kill, open, setenv, sigaction,
    sigemptyset, socket, system, tcgetattr, tcsetattr, waitpid,
};
use crate::file::{
    file_read_cancel, file_read_open, file_write_close, file_write_data, file_write_left,
    file_write_open,
};
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::log::{fatal, fatalx, log_debug};
use crate::proc::{proc_clear_signals, proc_exit, proc_loop, proc_set_signals, proc_start};
use crate::reactor;
use crate::server::server_start;
use crate::socket_address::UnixSocketAddress;
use crate::terminfo::tty_term_read_list;
use crate::tmux::{find_cwd, find_home, setblocking, shell_argv0};
use crate::tmux::{global_options_free, ptm_fd, shell_command, socket_path};
use crate::tree::GlobalTree;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use crate::{CommandTextCodec, RustCommandTextCodec};
use std::ffi::{CStr, OsStr};
use std::fs;
use std::io::{BufRead, ErrorKind, Write};
use std::os::unix::ffi::OsStrExt;

pub const CLIENT_EXIT_MESSAGE_PROVIDED: client_exit_reason = 8;
pub const CLIENT_EXIT_SERVER_EXITED: client_exit_reason = 7;
pub const CLIENT_EXIT_EXITED: client_exit_reason = 6;
pub const CLIENT_EXIT_LOST_SERVER: client_exit_reason = 5;
pub const CLIENT_EXIT_TERMINATED: client_exit_reason = 4;
pub const CLIENT_EXIT_LOST_TTY: client_exit_reason = 3;
pub const CLIENT_EXIT_DETACHED_HUP: client_exit_reason = 2;
pub const CLIENT_EXIT_DETACHED: client_exit_reason = 1;
pub const CLIENT_EXIT_NONE: client_exit_reason = 0;
pub type client_exit_reason = core::ffi::c_uint;

pub const ECONNREFUSED: core::ffi::c_int = 111 as core::ffi::c_int;

pub const LOCK_EX: core::ffi::c_int = 2 as core::ffi::c_int;
pub const LOCK_NB: core::ffi::c_int = 4 as core::ffi::c_int;

pub const IXANY: core::ffi::c_int = 0o4000 as core::ffi::c_int;

pub const CS8: core::ffi::c_int = 0o60 as core::ffi::c_int;
pub const CREAD: core::ffi::c_int = 0o200 as core::ffi::c_int;
pub const HUPCL: core::ffi::c_int = 0o2000 as core::ffi::c_int;

pub const TCSAFLUSH: core::ffi::c_int = 2 as core::ffi::c_int;

pub const CLIENT_STARTSERVER: core::ffi::c_int = 0x10000000 as core::ffi::c_int;

pub(crate) static mut client_proc: Option<ProcessRef> = None;
pub(crate) static mut client_peer: Option<PeerRef> = None;

fn client_peer_ref() -> PeerRef {
    unsafe {
        client_peer
            .as_ref()
            .expect("client peer is initialized")
            .clone()
    }
}

/// Flushes what the client still owes the server, then asks the loop to stop.
unsafe fn client_exit_proc() {
    unsafe {
        if let Some(peer) = client_peer.as_ref() {
            imsgbuf_flush(&mut peer.borrow_mut().ibuf);
        }
        proc_exit(
            &mut client_proc
                .as_ref()
                .expect("client process is initialized")
                .borrow_mut(),
        );
    }
}
pub(crate) static mut client_flags: uint64_t = 0;
pub(crate) static mut client_suspended: core::ffi::c_int = 0;
pub(crate) static mut client_exitreason: client_exit_reason = CLIENT_EXIT_NONE;
pub(crate) static mut client_exitflag: core::ffi::c_int = 0;
pub(crate) static mut client_exitval: core::ffi::c_int = 0;
pub(crate) static mut client_exittype: msgtype = 0 as msgtype;
pub(crate) static mut client_exitsession: Option<std::ffi::CString> = None;
pub(crate) static mut client_exitmessage: Option<std::ffi::CString> = None;
pub(crate) static mut client_execshell: Option<std::ffi::CString> = None;
pub(crate) static mut client_execcmd: Option<std::ffi::CString> = None;
pub(crate) static mut client_attached: core::ffi::c_int = 0;
pub(crate) static client_files: GlobalTree<core::ffi::c_int, ClientFileRef> = GlobalTree::new();

pub(crate) unsafe fn client_get_lock(lockfile: &CStr) -> core::ffi::c_int {
    unsafe {
        log_debug(c"lock file is %s", fmt_args![lockfile]);
        let lockfd: core::ffi::c_int = open(
            lockfile.as_ptr(),
            O_WRONLY | O_CREAT,
            0o600 as core::ffi::c_int,
        );
        if lockfd == -(1 as core::ffi::c_int) {
            log_debug(
                c"open failed: %s",
                fmt_args![error_message(*__errno_location()).as_c_str()],
            );
            return -(1 as core::ffi::c_int);
        }
        if flock(lockfd, LOCK_EX | LOCK_NB) == -(1 as core::ffi::c_int) {
            log_debug(
                c"flock failed: %s",
                fmt_args![error_message(*__errno_location()).as_c_str()],
            );
            if *__errno_location() != EAGAIN {
                return lockfd;
            }
            while flock(lockfd, LOCK_EX) == -(1 as core::ffi::c_int) && *__errno_location() == EINTR
            {
            }
            close(lockfd);
            return -(2 as core::ffi::c_int);
        }
        log_debug(c"flock succeeded", fmt_args![]);
        lockfd
    }
}
pub(crate) unsafe fn client_connect(
    base: reactor::Base,
    path: &CStr,
    flags: uint64_t,
) -> core::ffi::c_int {
    unsafe {
        let current_block: u64;
        let mut sa = sockaddr_un::default();
        let mut fd: core::ffi::c_int;
        let mut lockfd: core::ffi::c_int = -(1 as core::ffi::c_int);
        let mut locked: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut lockfile: Option<std::ffi::CString> = None;
        if !sa.set_unix_socket_address(AF_UNIX as sa_family_t, path) {
            *__errno_location() = ENAMETOOLONG;
            return -(1 as core::ffi::c_int);
        }
        log_debug(c"socket is %s", fmt_args![path]);
        loop {
            fd = socket(
                AF_UNIX,
                SOCK_STREAM as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            if fd == -(1 as core::ffi::c_int) {
                return -(1 as core::ffi::c_int);
            }
            log_debug(c"trying connect", fmt_args![]);
            if !(connect(
                fd,
                (&raw const sa).cast(),
                size_of::<sockaddr_un>() as socklen_t,
            ) == -(1 as core::ffi::c_int))
            {
                current_block = 7172762164747879670;
                break;
            }
            log_debug(
                c"connect failed: %s",
                fmt_args![error_message(*__errno_location()).as_c_str()],
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
                let new_lockfile = xasprintf(c"%s.lock", fmt_args![path]);
                lockfd = client_get_lock(&new_lockfile);
                if lockfd < 0 as core::ffi::c_int {
                    log_debug(c"didn't get lock (%d)", fmt_args![lockfd]);
                    drop(new_lockfile);
                    lockfile = None;
                    if lockfd == -(2 as core::ffi::c_int) {
                        continue;
                    }
                } else {
                    lockfile = Some(new_lockfile);
                }
                log_debug(c"got lock (%d)", fmt_args![lockfd]);
                locked = 1 as core::ffi::c_int;
            } else {
                if lockfd >= 0 as core::ffi::c_int
                    && let Err(err) = fs::remove_file(OsStr::from_bytes(path.to_bytes()))
                    && err.kind() != ErrorKind::NotFound
                {
                    drop(lockfile.take());
                    close(lockfd);
                    return -(1 as core::ffi::c_int);
                }
                fd = server_start(
                    &client_proc
                        .as_ref()
                        .expect("client process is initialized")
                        .clone(),
                    flags,
                    base,
                    lockfd,
                    lockfile.take(),
                );
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
                -(1 as core::ffi::c_int)
            }
            _ => {
                if locked != 0 && lockfd >= 0 as core::ffi::c_int {
                    drop(lockfile.take());
                    close(lockfd);
                }
                setblocking(fd, 0 as core::ffi::c_int);
                fd
            }
        }
    }
}
/// Why the client is going, as the caller's own string.
pub(crate) unsafe fn client_exit_message() -> std::ffi::CString {
    unsafe {
        match client_exitreason {
            CLIENT_EXIT_DETACHED => match client_exitsession.as_deref() {
                Some(session) => format_alloc(c"detached (from session %s)", fmt_args![session]),
                None => c"detached".to_owned(),
            },
            CLIENT_EXIT_DETACHED_HUP => match client_exitsession.as_deref() {
                Some(session) => {
                    format_alloc(c"detached and SIGHUP (from session %s)", fmt_args![session])
                }
                None => c"detached and SIGHUP".to_owned(),
            },
            CLIENT_EXIT_LOST_TTY => c"lost tty".to_owned(),
            CLIENT_EXIT_TERMINATED => c"terminated".to_owned(),
            CLIENT_EXIT_LOST_SERVER => c"server exited unexpectedly".to_owned(),
            CLIENT_EXIT_EXITED => c"exited".to_owned(),
            CLIENT_EXIT_SERVER_EXITED => c"server exited".to_owned(),
            CLIENT_EXIT_MESSAGE_PROVIDED => match client_exitmessage.as_ref() {
                Some(message) => message.clone(),
                None => c"unknown reason".to_owned(),
            },
            _ => c"unknown reason".to_owned(),
        }
    }
}
pub(crate) unsafe fn client_exit() {
    unsafe {
        if file_write_left(&client_files.map()) == 0 {
            client_exit_proc();
        }
    }
}
pub(crate) unsafe fn client_main(
    base: reactor::Base,
    argv: &[std::ffi::CString],
    mut flags: uint64_t,
    feat: core::ffi::c_int,
    config: crate::cfg::ConfigState,
) -> core::ffi::c_int {
    unsafe {
        let ppid: pid_t;
        let msg: msgtype;
        let mut tio: termios = core::mem::zeroed();
        let mut saved_tio: termios = core::mem::zeroed();
        let size: size_t;
        let argc = argv.len() as core::ffi::c_int;
        if shell_command.is_some() {
            msg = MSG_SHELL;
            flags |= CLIENT_STARTSERVER as uint64_t;
        } else if argv.is_empty() {
            msg = MSG_COMMAND;
            flags |= CLIENT_STARTSERVER as uint64_t;
        } else {
            msg = MSG_COMMAND;
            let values = args_from_vector(argv);
            let mut pr = cmd_parse_from_arguments(&values, None);
            if pr.status as core::ffi::c_uint
                == CMD_PARSE_SUCCESS as core::ffi::c_int as core::ffi::c_uint
            {
                if (pr.cmdlist.as_ref().unwrap()).any_have(CMD_STARTSERVER) != 0 {
                    flags |= CLIENT_STARTSERVER as uint64_t;
                }
                let _ = pr.cmdlist.take();
            } else {
                let _ = pr.error.take();
            }
        }
        let process = proc_start(c"client");
        process.borrow_mut().config = config;
        client_proc = Some(process);
        proc_set_signals(
            &mut client_proc
                .as_ref()
                .expect("client process is initialized")
                .borrow_mut(),
            |sig| client_signal(sig),
        );
        client_flags = flags;
        log_debug(
            c"flags are %#llx",
            fmt_args![client_flags as core::ffi::c_ulonglong],
        );
        let fd: core::ffi::c_int = if systemd_activated() != 0 {
            server_start(
                &client_proc
                    .as_ref()
                    .expect("client process is initialized")
                    .clone(),
                flags,
                base,
                0 as core::ffi::c_int,
                None,
            )
        } else {
            client_connect(
                base,
                socket_path.as_deref().expect("socket path was selected"),
                client_flags,
            )
        };
        if fd == -(1 as core::ffi::c_int) {
            if *__errno_location() == ECONNREFUSED {
                let _ = std::io::stderr().lock().write_all(
                    &[
                        b"no server running on ".as_slice(),
                        socket_path
                            .as_deref()
                            .expect("socket path was selected")
                            .to_bytes(),
                        b"\n",
                    ]
                    .concat(),
                );
            } else {
                let error = error_message(*__errno_location());
                let _ = std::io::stderr().lock().write_all(
                    &[
                        b"error connecting to ".as_slice(),
                        socket_path
                            .as_deref()
                            .expect("socket path was selected")
                            .to_bytes(),
                        b" (",
                        error.to_bytes(),
                        b")\n",
                    ]
                    .concat(),
                );
            }
            return 1 as core::ffi::c_int;
        }
        let dispatch = std::rc::Rc::new(client_dispatch);
        client_peer = Some(PeerRef::from_fd(fd, Some(dispatch)));
        let found_cwd = find_cwd();
        let cwd = match (&found_cwd, find_home()) {
            (Some(found), _) => found.as_c_str(),
            (None, Some(home)) => home,
            (None, None) => c"/",
        };
        let ttynam = crate::tty::terminal_device_name(STDIN_FILENO).unwrap_or_default();
        let term = process_environment_value(c"TERM");
        let termname = term.as_deref().unwrap_or(c"");
        let caps = if isatty(STDIN_FILENO) != 0 && !termname.to_bytes().is_empty() {
            match tty_term_read_list(termname, STDIN_FILENO) {
                Ok(caps) => caps,
                Err(cause) => {
                    let _ = std::io::stderr()
                        .lock()
                        .write_all(&[cause.to_bytes(), b"\n"].concat());
                    return 1 as core::ffi::c_int;
                }
            }
        } else {
            Vec::new()
        };
        if ptm_fd != -(1 as core::ffi::c_int) {
            close(ptm_fd);
        }
        global_options_free();
        if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            if tcgetattr(STDIN_FILENO, &raw mut saved_tio) != 0 as core::ffi::c_int {
                let error = error_message(*__errno_location());
                let _ = std::io::stderr().lock().write_all(
                    &[b"tcgetattr failed: ".as_slice(), error.to_bytes(), b"\n"].concat(),
                );
                return 1 as core::ffi::c_int;
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
        client_send_identify(&ttynam, termname, &caps, cwd, feat);
        (client_peer_ref()).flush();
        if msg as core::ffi::c_uint == MSG_COMMAND as core::ffi::c_int as core::ffi::c_uint {
            size = argv.iter().map(|arg| arg.as_bytes_with_nul().len()).sum();
            let header_size = size_of::<msg_command>();
            if size > (MAX_IMSGSIZE as usize).wrapping_sub(header_size) {
                let _ = std::io::stderr().lock().write_all(b"command too long\n");
                return 1 as core::ffi::c_int;
            }
            let header = msg_command { argc };
            let mut data: Vec<u8> = Vec::with_capacity(header_size.wrapping_add(size as usize));
            data.extend_from_slice(&header.argc.to_ne_bytes());
            data.resize(header_size.wrapping_add(size as usize), 0u8);
            let packed = &mut data[header_size..];
            if !RustCommandTextCodec.pack(argv, packed) {
                let _ = std::io::stderr().lock().write_all(b"command too long\n");
                return 1 as core::ffi::c_int;
            }
            if (client_peer_ref()).send(msg, -(1 as core::ffi::c_int), &data)
                != 0 as core::ffi::c_int
            {
                let _ = std::io::stderr()
                    .lock()
                    .write_all(b"failed to send command\n");
                return 1 as core::ffi::c_int;
            }
        } else if msg as core::ffi::c_uint == MSG_SHELL as core::ffi::c_int as core::ffi::c_uint {
            (client_peer_ref()).send(msg, -(1 as core::ffi::c_int), &[]);
        }
        proc_loop(
            &client_proc
                .as_ref()
                .expect("client process is initialized")
                .clone(),
            || false,
        );
        if client_exittype as core::ffi::c_uint == MSG_EXEC as core::ffi::c_int as core::ffi::c_uint
        {
            if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
                tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
            }
            let execshell = client_execshell.take().expect("MSG_EXEC supplies a shell");
            let execcmd = client_execcmd.take();
            client_exec(&execshell, execcmd.as_deref());
        }
        setblocking(STDIN_FILENO, 1 as core::ffi::c_int);
        setblocking(STDOUT_FILENO, 1 as core::ffi::c_int);
        setblocking(STDERR_FILENO, 1 as core::ffi::c_int);
        if client_attached != 0 {
            if client_exitreason as core::ffi::c_uint
                != CLIENT_EXIT_NONE as core::ffi::c_int as core::ffi::c_uint
            {
                let _ = std::io::stdout().lock().write_all(
                    &[b"[".as_slice(), client_exit_message().to_bytes(), b"]\n"].concat(),
                );
            }
            ppid = getppid() as pid_t;
            if client_exittype as core::ffi::c_uint
                == MSG_DETACHKILL as core::ffi::c_int as core::ffi::c_uint
                && ppid > 1 as core::ffi::c_int
            {
                kill(ppid as __pid_t, SIGHUP);
            }
        } else if client_flags & CLIENT_CONTROL as uint64_t != 0 {
            if client_exitreason as core::ffi::c_uint
                != CLIENT_EXIT_NONE as core::ffi::c_int as core::ffi::c_uint
            {
                let _ = std::io::stdout().lock().write_all(
                    &[
                        b"%exit ".as_slice(),
                        client_exit_message().to_bytes(),
                        b"\n",
                    ]
                    .concat(),
                );
            } else {
                let _ = std::io::stdout().lock().write_all(b"%exit\n");
            }
            let _ = std::io::stdout().lock().flush();
            if client_flags as core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
                let mut stdin = std::io::stdin().lock();
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
                let _ = std::io::stdout().lock().write_all(b"\x1B\\");
                let _ = std::io::stdout().lock().flush();
                tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
            }
        } else if client_exitreason as core::ffi::c_uint
            != CLIENT_EXIT_NONE as core::ffi::c_int as core::ffi::c_uint
        {
            let _ = std::io::stderr()
                .lock()
                .write_all(&[client_exit_message().to_bytes(), b"\n"].concat());
        }
        client_exitval
    }
}
unsafe fn client_send_identify(
    ttynam: &CStr,
    termname: &CStr,
    caps: &[std::ffi::CString],
    cwd: &CStr,
    feat: core::ffi::c_int,
) {
    unsafe {
        let mut fd: core::ffi::c_int;
        let flags: uint64_t = client_flags;

        (client_peer_ref()).send(
            MSG_IDENTIFY_LONGFLAGS,
            -(1 as core::ffi::c_int),
            &flags.to_ne_bytes(),
        );
        (client_peer_ref()).send(
            MSG_IDENTIFY_LONGFLAGS,
            -(1 as core::ffi::c_int),
            &client_flags.to_ne_bytes(),
        );
        (client_peer_ref()).send(
            MSG_IDENTIFY_TERM,
            -(1 as core::ffi::c_int),
            termname.to_bytes_with_nul(),
        );
        (client_peer_ref()).send(
            MSG_IDENTIFY_FEATURES,
            -(1 as core::ffi::c_int),
            &feat.to_ne_bytes(),
        );
        (client_peer_ref()).send(
            MSG_IDENTIFY_TTYNAME,
            -(1 as core::ffi::c_int),
            ttynam.to_bytes_with_nul(),
        );
        (client_peer_ref()).send(
            MSG_IDENTIFY_CWD,
            -(1 as core::ffi::c_int),
            cwd.to_bytes_with_nul(),
        );
        for cap in caps {
            (client_peer_ref()).send(
                MSG_IDENTIFY_TERMINFO,
                -(1 as core::ffi::c_int),
                cap.as_bytes_with_nul(),
            );
        }
        fd = dup(STDIN_FILENO);
        if fd == -(1 as core::ffi::c_int) {
            fatal(c"dup failed", fmt_args![]);
        }
        (client_peer_ref()).send(MSG_IDENTIFY_STDIN, fd, &[]);
        fd = dup(STDOUT_FILENO);
        if fd == -(1 as core::ffi::c_int) {
            fatal(c"dup failed", fmt_args![]);
        }
        (client_peer_ref()).send(MSG_IDENTIFY_STDOUT, fd, &[]);
        let pid: pid_t = getpid() as pid_t;
        (client_peer_ref()).send(
            MSG_IDENTIFY_CLIENTPID,
            -(1 as core::ffi::c_int),
            &pid.to_ne_bytes(),
        );
        for var in process_environment() {
            let bytes = var.to_bytes_with_nul();
            if !(bytes.len() > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)) {
                (client_peer_ref()).send(MSG_IDENTIFY_ENVIRON, -(1 as core::ffi::c_int), bytes);
            }
        }
        (client_peer_ref()).send(MSG_IDENTIFY_DONE, -(1 as core::ffi::c_int), &[]);
    }
}
unsafe fn client_exec(shell: &CStr, shellcmd: Option<&CStr>) -> ! {
    unsafe {
        log_debug(c"shell %s, command %s", fmt_args![shell, shellcmd]);
        let argv0 = shell_argv0(
            shell,
            (client_flags & CLIENT_LOGIN as uint64_t != 0) as core::ffi::c_int,
        );
        setenv(c"SHELL".as_ptr(), shell.as_ptr(), 1 as core::ffi::c_int);
        proc_clear_signals(
            &mut client_proc
                .as_ref()
                .expect("client process is initialized")
                .borrow_mut(),
            1 as core::ffi::c_int,
        );
        setblocking(STDIN_FILENO, 1 as core::ffi::c_int);
        setblocking(STDOUT_FILENO, 1 as core::ffi::c_int);
        setblocking(STDERR_FILENO, 1 as core::ffi::c_int);
        closefrom(STDERR_FILENO + 1 as core::ffi::c_int);
        execl(
            shell.as_ptr(),
            argv0.as_ptr(),
            c"-c".as_ptr(),
            shellcmd.map_or(core::ptr::null(), CStr::as_ptr),
            core::ptr::null_mut::<core::ffi::c_char>(),
        );
        fatal(c"execl failed", fmt_args![]);
    }
}
pub(crate) unsafe fn client_signal(sig: core::ffi::c_int) {
    unsafe {
        let mut sigact: libc::sigaction;
        let mut status: core::ffi::c_int = 0;
        let mut pid: pid_t;
        log_debug(
            c"%s: %s",
            fmt_args![c"client_signal", signal_description(sig).as_deref()],
        );
        if sig == SIGCHLD {
            loop {
                pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG) as pid_t;
                if pid == 0 as core::ffi::c_int {
                    break;
                }
                if !(pid == -(1 as core::ffi::c_int)) {
                    continue;
                }
                if *__errno_location() == ECHILD {
                    break;
                }
                log_debug(
                    c"waitpid failed: %s",
                    fmt_args![error_message(*__errno_location()).as_c_str()],
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
                    client_exitval = 1 as core::ffi::c_int;
                    (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
                }
                SIGTERM => {
                    if client_suspended == 0 {
                        client_exitreason = CLIENT_EXIT_TERMINATED;
                    }
                    client_exitval = 1 as core::ffi::c_int;
                    (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
                }
                SIGWINCH => {
                    (client_peer_ref()).send(MSG_RESIZE, -(1 as core::ffi::c_int), &[]);
                }
                SIGCONT => {
                    sigact = core::mem::zeroed();
                    sigemptyset(&raw mut sigact.sa_mask);
                    sigact.sa_flags = SA_RESTART;
                    sigact.sa_sigaction = libc::SIG_IGN;
                    if sigaction(
                        SIGTSTP,
                        &raw mut sigact,
                        core::ptr::null_mut::<libc::sigaction>(),
                    ) != 0 as core::ffi::c_int
                    {
                        fatal(c"sigaction failed", fmt_args![]);
                    }
                    (client_peer_ref()).send(MSG_WAKEUP, -(1 as core::ffi::c_int), &[]);
                    client_suspended = 0 as core::ffi::c_int;
                }
                _ => {}
            }
        };
    }
}
pub(crate) fn client_file_check_cb(event: ClientFileEvent<'_>) {
    unsafe {
        if !matches!(event, ClientFileEvent::CheckExit) {
            return;
        }
        if client_exitflag != 0 {
            client_exit();
        }
    }
}
pub(crate) fn client_dispatch(imsg: Option<&mut imsg>) {
    unsafe {
        let Some(imsg) = imsg else {
            if client_exitflag == 0 {
                client_exitreason = CLIENT_EXIT_LOST_SERVER;
                client_exitval = 1 as core::ffi::c_int;
            }
            client_exit_proc();
            return;
        };
        if client_attached != 0 {
            client_dispatch_attached(imsg);
        } else {
            client_dispatch_wait(imsg);
        };
    }
}
pub(crate) unsafe fn client_dispatch_exit_message(data: &[u8]) {
    unsafe {
        let retval: core::ffi::c_int;
        if data.len() < size_of::<core::ffi::c_int>() && !data.is_empty() {
            fatalx(c"bad MSG_EXIT size", fmt_args![]);
        }
        if data.len() >= size_of::<core::ffi::c_int>() {
            retval = core::ffi::c_int::from_ne_bytes(
                data[..size_of::<core::ffi::c_int>()]
                    .try_into()
                    .expect("exit status size checked"),
            );
            client_exitval = retval;
        }
        if data.len() > size_of::<core::ffi::c_int>() {
            let bytes = &data[size_of::<core::ffi::c_int>()..];
            let end = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(bytes.len().wrapping_sub(1));
            client_exitmessage = std::ffi::CString::new(&bytes[..end]).ok();
            client_exitreason = CLIENT_EXIT_MESSAGE_PROVIDED;
        }
    }
}
pub(crate) unsafe fn client_dispatch_wait(imsg: &imsg) {
    unsafe {
        let payload = imsg.imsg_message_data();
        let datalen = payload.len() as ssize_t;
        match imsg.hdr.type_0 {
            MSG_EXIT | MSG_SHUTDOWN => {
                client_dispatch_exit_message(payload);
                client_exitflag = 1 as core::ffi::c_int;
                client_exit();
            }
            MSG_READY => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_READY size", fmt_args![]);
                }
                client_attached = 1 as core::ffi::c_int;
                (client_peer_ref()).send(MSG_RESIZE, -(1 as core::ffi::c_int), &[]);
            }
            MSG_VERSION => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_VERSION size", fmt_args![]);
                }
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "protocol version mismatch (client {}, server {})",
                    PROTOCOL_VERSION,
                    imsg.hdr.peerid & 0xff
                );
                client_exitval = 1 as core::ffi::c_int;
                client_exit_proc();
            }
            MSG_FLAGS => {
                if datalen as usize != size_of::<uint64_t>() {
                    fatalx(c"bad MSG_FLAGS string", fmt_args![]);
                }
                client_flags =
                    uint64_t::from_ne_bytes(payload.try_into().expect("flag size checked"));
                log_debug(
                    c"new flags are %#llx",
                    fmt_args![client_flags as core::ffi::c_ulonglong],
                );
            }
            MSG_SHELL => {
                if payload.last() != Some(&0) {
                    fatalx(c"bad MSG_SHELL string", fmt_args![]);
                }
                client_exec(
                    CStr::from_bytes_until_nul(payload).expect("terminated shell"),
                    shell_command.as_deref(),
                );
            }
            MSG_DETACH | MSG_DETACHKILL => {
                (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
            }
            MSG_EXITED => {
                client_exit_proc();
            }
            MSG_READ_OPEN => {
                file_read_open(
                    &FileOwner::Global(&client_files),
                    &client_peer_ref(),
                    imsg,
                    1 as core::ffi::c_int,
                    (client_flags & CLIENT_CONTROL as uint64_t == 0) as core::ffi::c_int,
                    Some(std::rc::Rc::new(client_file_check_cb)),
                    ClientFileData::None,
                );
            }
            MSG_READ_CANCEL => {
                file_read_cancel(&FileOwner::Global(&client_files), imsg);
            }
            MSG_WRITE_OPEN => {
                file_write_open(
                    &FileOwner::Global(&client_files),
                    &client_peer_ref(),
                    imsg,
                    1 as core::ffi::c_int,
                    (client_flags & CLIENT_CONTROL as uint64_t == 0) as core::ffi::c_int,
                    Some(std::rc::Rc::new(client_file_check_cb)),
                    ClientFileData::None,
                );
            }
            MSG_WRITE => {
                file_write_data(&client_files.map(), imsg);
            }
            MSG_WRITE_CLOSE => {
                file_write_close(&FileOwner::Global(&client_files), imsg);
            }
            MSG_OLDSTDERR | MSG_OLDSTDIN | MSG_OLDSTDOUT => {
                let _ = std::io::stderr()
                    .lock()
                    .write_all(b"server version is too old for client\n");
                client_exit_proc();
            }
            _ => {}
        };
    }
}
pub(crate) unsafe fn client_dispatch_attached(imsg: &imsg) {
    unsafe {
        let mut sigact: libc::sigaction;
        let payload = imsg.imsg_message_data();
        let datalen = payload.len() as ssize_t;
        match imsg.hdr.type_0 {
            MSG_FLAGS => {
                if datalen as usize != size_of::<uint64_t>() {
                    fatalx(c"bad MSG_FLAGS string", fmt_args![]);
                }
                client_flags =
                    uint64_t::from_ne_bytes(payload.try_into().expect("flag size checked"));
                log_debug(
                    c"new flags are %#llx",
                    fmt_args![client_flags as core::ffi::c_ulonglong],
                );
            }
            MSG_DETACH | MSG_DETACHKILL => {
                if payload.last() != Some(&0) {
                    fatalx(c"bad MSG_DETACH string", fmt_args![]);
                }
                client_exitsession = Some(
                    CStr::from_bytes_until_nul(payload)
                        .expect("terminated session")
                        .to_owned(),
                );
                client_exittype = imsg.hdr.type_0 as msgtype;
                if imsg.hdr.type_0 == MSG_DETACHKILL as core::ffi::c_int as uint32_t {
                    client_exitreason = CLIENT_EXIT_DETACHED_HUP;
                } else {
                    client_exitreason = CLIENT_EXIT_DETACHED;
                }
                (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
            }
            MSG_EXEC => {
                if payload.last() != Some(&0) {
                    fatalx(c"bad MSG_EXEC string", fmt_args![]);
                }
                let command = CStr::from_bytes_until_nul(payload).expect("terminated command");
                let shell = &payload[command.to_bytes_with_nul().len()..];
                if shell.is_empty() {
                    fatalx(c"bad MSG_EXEC string", fmt_args![]);
                }
                client_execcmd = Some(command.to_owned());
                client_execshell = Some(
                    CStr::from_bytes_until_nul(shell)
                        .expect("terminated shell")
                        .to_owned(),
                );
                client_exittype = imsg.hdr.type_0 as msgtype;
                (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
            }
            MSG_EXIT => {
                client_dispatch_exit_message(payload);
                if client_exitreason as core::ffi::c_uint
                    == CLIENT_EXIT_NONE as core::ffi::c_int as core::ffi::c_uint
                {
                    client_exitreason = CLIENT_EXIT_EXITED;
                }
                (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
            }
            MSG_EXITED => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_EXITED size", fmt_args![]);
                }
                client_exit_proc();
            }
            MSG_SHUTDOWN => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_SHUTDOWN size", fmt_args![]);
                }
                (client_peer_ref()).send(MSG_EXITING, -(1 as core::ffi::c_int), &[]);
                client_exitreason = CLIENT_EXIT_SERVER_EXITED;
                client_exitval = 1 as core::ffi::c_int;
            }
            MSG_SUSPEND => {
                if datalen != 0 as ssize_t {
                    fatalx(c"bad MSG_SUSPEND size", fmt_args![]);
                }
                sigact = core::mem::zeroed();
                sigemptyset(&raw mut sigact.sa_mask);
                sigact.sa_flags = SA_RESTART;
                sigact.sa_sigaction = libc::SIG_DFL;
                if sigaction(
                    SIGTSTP,
                    &raw mut sigact,
                    core::ptr::null_mut::<libc::sigaction>(),
                ) != 0 as core::ffi::c_int
                {
                    fatal(c"sigaction failed", fmt_args![]);
                }
                client_suspended = 1 as core::ffi::c_int;
                kill(getpid(), SIGTSTP);
            }
            MSG_LOCK => {
                if payload.last() != Some(&0) {
                    fatalx(c"bad MSG_LOCK string", fmt_args![]);
                }
                system(
                    CStr::from_bytes_until_nul(payload)
                        .expect("terminated lock command")
                        .as_ptr(),
                );
                (client_peer_ref()).send(MSG_UNLOCK, -(1 as core::ffi::c_int), &[]);
            }
            _ => {}
        };
    }
}

#[cfg(test)]
#[path = "client_focused_tests.rs"]
mod focused_tests;
