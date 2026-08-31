use crate::compat::getpeereid;
use crate::compat::setproctitle;
use crate::compat::{
    imsg_compose, imsgbuf_allow_fdpass, imsgbuf_clear, imsgbuf_flush, imsgbuf_init,
    imsgbuf_queuelen, imsgbuf_read, imsgbuf_write,
};
use crate::compat::{imsg_free, imsg_get};
use crate::ffi::{
    close, daemon, fork, getpid, sigaction, sigemptyset, socketpair, uname, utf8proc_version,
};
use crate::fmt_args;
use crate::log::{fatal, log_debug, log_open, log_toggle};
use crate::reactor;
use crate::reactor::{Interest, IoWatch, Reactor, SignalWatch, WatchMode};
use crate::tmux::getversion;
use crate::tmux::socket_path;
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
#[repr(C)]
pub struct tmuxpeer {
    pub parent: *mut tmuxproc,
    pub ibuf: imsgbuf,
    pub event: IoHandle,
    pub uid: uid_t,
    pub flags: ::core::ffi::c_int,
    pub dispatchcb: Option<unsafe fn(*mut imsg, *mut client) -> ()>,
    /// The client this peer speaks for, observed rather than held: a
    /// dispatch that loses the client drops the peer with it, so the
    /// callback stays a plain function pointer and the client is upgraded
    /// for the length of one call.
    pub owner: Option<ClientWeak>,
}
#[derive(Default)]
#[repr(C)]
pub struct tmuxproc {
    pub name: Option<::std::ffi::CString>,
    pub exit: ::core::ffi::c_int,
    pub signalcb: Option<unsafe fn(::core::ffi::c_int) -> ()>,
    pub ev_sigint: SignalHandle,
    pub ev_sighup: SignalHandle,
    pub ev_sigchld: SignalHandle,
    pub ev_sigcont: SignalHandle,
    pub ev_sigterm: SignalHandle,
    pub ev_sigusr1: SignalHandle,
    pub ev_sigusr2: SignalHandle,
    pub ev_sigwinch: SignalHandle,
}
impl tmuxproc {
    /// The name the process logs under.
    pub(crate) fn name_ptr(&self) -> *mut ::core::ffi::c_char {
        cstr_ptr(&self.name)
    }
}
pub const PF_UNSPEC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;
pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;
pub const SIG_DFL: __sighandler_t = None;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SIGQUIT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SIGPIPE: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const SIGTSTP: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SIGCONT: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const SIGTTIN: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const SIGTTOU: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const SIGUSR1: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const SIGUSR2: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const SIGWINCH: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const SA_RESTART: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;
pub const NCURSES_VERSION_PATCH: ::core::ffi::c_int = 20251230 as ::core::ffi::c_int;
pub const NCURSES_VERSION: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"6.6\0") };
pub const EVLOOP_ONCE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const EV_SIGNAL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const EV_PERSIST: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PROTOCOL_VERSION: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const PEER_BAD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
/// The client a peer speaks for, upgraded for as long as the caller holds
/// the result. `None` means the peer never had one; a peer whose client has
/// gone is dropped along with it.
unsafe fn peer_owner(peer: *mut tmuxpeer) -> Option<ClientRef> {
    unsafe { (*peer).owner.as_ref().and_then(ClientWeak::upgrade) }
}

unsafe fn peer_owner_ptr(peer: *mut tmuxpeer) -> *mut client {
    unsafe { peer_owner(peer).map_or(::core::ptr::null_mut::<client>(), |owner| owner.as_ptr()) }
}

unsafe fn proc_event_cb(mut events: ::core::ffi::c_short, peer: *mut tmuxpeer) {
    unsafe {
        let mut n: ssize_t = 0;
        let mut imsg = imsg::default();
        let owner = peer_owner(peer);
        let arg = owner
            .as_ref()
            .map_or(::core::ptr::null_mut::<client>(), ClientRef::as_ptr);
        if (*peer).flags & PEER_BAD == 0 && events as ::core::ffi::c_int & EV_READ != 0 {
            if imsgbuf_read(&mut (*peer).ibuf) != 1 as ::core::ffi::c_int {
                (*peer).dispatchcb.expect("non-null function pointer")(
                    ::core::ptr::null_mut::<imsg>(),
                    arg,
                );
                return;
            }
            loop {
                n = imsg_get(&mut (*peer).ibuf, &raw mut imsg);
                if n == -(1 as ::core::ffi::c_int) as ssize_t {
                    (*peer).dispatchcb.expect("non-null function pointer")(
                        ::core::ptr::null_mut::<imsg>(),
                        arg,
                    );
                    return;
                }
                if n == 0 as ssize_t {
                    break;
                }
                log_debug(
                    c"peer %p message %d".as_ptr(),
                    fmt_args![peer, imsg.hdr.type_0],
                );
                if peer_check_version(peer, &raw mut imsg) != 0 as ::core::ffi::c_int {
                    imsg_free(&raw mut imsg);
                    break;
                } else {
                    (*peer).dispatchcb.expect("non-null function pointer")(&raw mut imsg, arg);
                    imsg_free(&raw mut imsg);
                }
            }
        }
        if events as ::core::ffi::c_int & EV_WRITE != 0
            && imsgbuf_write(&mut (*peer).ibuf) == -(1 as ::core::ffi::c_int)
        {
            (*peer).dispatchcb.expect("non-null function pointer")(
                ::core::ptr::null_mut::<imsg>(),
                arg,
            );
            return;
        }
        if (*peer).flags & PEER_BAD != 0 && imsgbuf_queuelen(&mut (*peer).ibuf) == 0 as uint32_t {
            (*peer).dispatchcb.expect("non-null function pointer")(
                ::core::ptr::null_mut::<imsg>(),
                arg,
            );
            return;
        }
        proc_update_event(peer);
    }
}
unsafe fn proc_signal_cb(mut signo: ::core::ffi::c_int, tp: *mut tmuxproc) {
    unsafe {
        (*tp).signalcb.expect("non-null function pointer")(signo);
    }
}
unsafe fn peer_check_version(mut peer: *mut tmuxpeer, mut imsg: *mut imsg) -> ::core::ffi::c_int {
    unsafe {
        let mut version: ::core::ffi::c_int = 0;
        version = ((*imsg).hdr.peerid & 0xff as uint32_t) as ::core::ffi::c_int;
        if (*imsg).hdr.type_0 != MSG_VERSION as ::core::ffi::c_int as uint32_t
            && version != PROTOCOL_VERSION
        {
            log_debug(c"peer %p bad version %d".as_ptr(), fmt_args![peer, version]);
            proc_send(
                peer,
                MSG_VERSION,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<u8>(),
                0 as size_t,
            );
            (*peer).flags |= PEER_BAD;
            return -(1 as ::core::ffi::c_int);
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn proc_update_event(mut peer: *mut tmuxpeer) {
    unsafe {
        let mut interest: Interest = Interest::Read;
        (*peer).event.disable();
        if imsgbuf_queuelen(&mut (*peer).ibuf) > 0 as uint32_t {
            interest = Interest::ReadWrite;
        }
        (*peer).event.set_callback(
            (*peer).ibuf.fd,
            interest,
            WatchMode::Once,
            move |_, events| proc_event_cb(events, peer),
        );
        (*peer).event.enable();
    }
}
pub unsafe fn proc_send(
    mut peer: *mut tmuxpeer,
    mut type_0: msgtype,
    mut fd: ::core::ffi::c_int,
    mut buf: *const u8,
    mut len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ibuf: *mut imsgbuf = &mut (*peer).ibuf;
        let mut retval: ::core::ffi::c_int = 0;
        if (*peer).flags & PEER_BAD != 0 {
            return -(1 as ::core::ffi::c_int);
        }
        log_debug(
            c"sending message %d to peer %p (%zu bytes)".as_ptr(),
            fmt_args![type_0 as ::core::ffi::c_uint, peer, len],
        );
        retval = imsg_compose(
            &mut *ibuf,
            type_0 as uint32_t,
            PROTOCOL_VERSION as uint32_t,
            -(1 as pid_t),
            fd,
            buf,
            len,
        );
        if retval != 1 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        proc_update_event(peer);
        0 as ::core::ffi::c_int
    }
}
/// Every process description this program has started. `proc_start` hands
/// out a view into the box it parks here; the process outlives them all.
static procs: GlobalQueue<Box<tmuxproc>> = GlobalQueue::new();

pub unsafe fn proc_start(mut name: *const ::core::ffi::c_char) -> *mut tmuxproc {
    unsafe {
        let mut tp: *mut tmuxproc = ::core::ptr::null_mut::<tmuxproc>();
        let mut u: utsname = ::core::mem::zeroed();
        log_open(name);
        setproctitle(c"%s (%s)".as_ptr(), fmt_args![name, socket_path.as_deref()]);
        if uname(&raw mut u) < 0 as ::core::ffi::c_int {
            u = ::core::mem::zeroed();
        }
        log_debug(
            c"%s started (%ld): version %s, socket %s, protocol %d".as_ptr(),
            fmt_args![
                name,
                getpid() as ::core::ffi::c_long,
                getversion(),
                socket_path.as_deref(),
                PROTOCOL_VERSION
            ],
        );
        log_debug(
            c"on %s %s %s".as_ptr(),
            fmt_args![
                &raw mut u.sysname as *mut ::core::ffi::c_char,
                &raw mut u.release as *mut ::core::ffi::c_char,
                &raw mut u.version as *mut ::core::ffi::c_char
            ],
        );
        let reactor = ::std::ffi::CString::new(reactor::current().describe())
            .expect("a reactor description without a NUL");
        log_debug(c"using %s".as_ptr(), fmt_args![reactor.as_ptr()]);
        log_debug(c"using utf8proc %s".as_ptr(), fmt_args![utf8proc_version()]);
        log_debug(
            c"using ncurses %s %06u".as_ptr(),
            fmt_args![NCURSES_VERSION.as_ptr(), NCURSES_VERSION_PATCH],
        );
        let mut tp_box = Box::new(tmuxproc {
            name: Some(CStr::from_ptr(name).to_owned()),
            ..tmuxproc::default()
        });
        tp = &raw mut *tp_box;
        procs.queue().push_back(tp_box);
        tp
    }
}
pub unsafe fn proc_loop(
    mut tp: *mut tmuxproc,
    mut loopcb: Option<unsafe fn() -> ::core::ffi::c_int>,
) {
    unsafe {
        log_debug(c"%s loop enter".as_ptr(), fmt_args![(*tp).name.as_deref()]);
        loop {
            reactor::current().run_once();
            if !((*tp).exit == 0
                && (loopcb.is_none() || loopcb.expect("non-null function pointer")() == 0))
            {
                break;
            }
        }
        log_debug(c"%s loop exit".as_ptr(), fmt_args![(*tp).name.as_deref()]);
    }
}
/// Asks the loop to stop. Whoever owns a peer flushes it first; the process
/// keeps no list of them.
pub unsafe fn proc_exit(mut tp: *mut tmuxproc) {
    unsafe {
        (*tp).exit = 1 as ::core::ffi::c_int;
    }
}
pub unsafe fn proc_set_signals(
    mut tp: *mut tmuxproc,
    mut signalcb: Option<unsafe fn(::core::ffi::c_int) -> ()>,
) {
    unsafe {
        let mut sa: libc::sigaction = ::core::mem::zeroed();
        (*tp).signalcb = signalcb;
        sigemptyset(&raw mut sa.sa_mask);
        sa.sa_flags = SA_RESTART;
        sa.sa_sigaction = ::libc::SIG_IGN;
        sigaction(
            SIGPIPE,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTSTP,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTTIN,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTTOU,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGQUIT,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        (*tp)
            .ev_sigint
            .set_callback(2 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sighup
            .set_callback(1 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sigchld
            .set_callback(17 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sigcont
            .set_callback(18 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sigterm
            .set_callback(15 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sigusr1
            .set_callback(10 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sigusr2
            .set_callback(12 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
        (*tp)
            .ev_sigwinch
            .set_callback(28 as ::core::ffi::c_int, move |signo, _| {
                proc_signal_cb(signo, tp)
            });
    }
}
pub unsafe fn proc_clear_signals(mut tp: *mut tmuxproc, mut defaults: ::core::ffi::c_int) {
    unsafe {
        let mut sa: libc::sigaction = ::core::mem::zeroed();
        sigemptyset(&raw mut sa.sa_mask);
        sa.sa_flags = SA_RESTART;
        sa.sa_sigaction = ::libc::SIG_DFL;
        sigaction(
            SIGPIPE,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTSTP,
            &raw mut sa,
            ::core::ptr::null_mut::<libc::sigaction>(),
        );
        (*tp).ev_sigint.unwatch();
        (*tp).ev_sighup.unwatch();
        (*tp).ev_sigchld.unwatch();
        (*tp).ev_sigcont.unwatch();
        (*tp).ev_sigterm.unwatch();
        (*tp).ev_sigusr1.unwatch();
        (*tp).ev_sigusr2.unwatch();
        (*tp).ev_sigwinch.unwatch();
        if defaults != 0 {
            sigaction(
                SIGINT,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGQUIT,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGHUP,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGCHLD,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGCONT,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGTERM,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGUSR1,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGUSR2,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGWINCH,
                &raw mut sa,
                ::core::ptr::null_mut::<libc::sigaction>(),
            );
        }
    }
}
pub unsafe fn proc_add_peer(
    mut tp: *mut tmuxproc,
    mut fd: ::core::ffi::c_int,
    mut dispatchcb: Option<unsafe fn(*mut imsg, *mut client) -> ()>,
    owner: Option<ClientWeak>,
) -> Box<tmuxpeer> {
    unsafe {
        let mut gid: gid_t = 0;
        let mut peer_box = Box::new(tmuxpeer {
            parent: tp,
            ibuf: imsgbuf {
                w: None,
                pid: 0,
                maxsize: 0,
                fd: -1,
                flags: 0,
            },
            event: IoHandle(0),
            uid: 0,
            flags: 0,
            dispatchcb,
            owner,
        });
        let peer = &raw mut *peer_box;
        if imsgbuf_init(&mut (*peer).ibuf, fd) == -(1 as ::core::ffi::c_int) {
            fatal(c"imsgbuf_init".as_ptr(), fmt_args![]);
        }
        imsgbuf_allow_fdpass(&mut (*peer).ibuf);
        (*peer)
            .event
            .set_callback(fd, Interest::Read, WatchMode::Once, move |_, events| {
                proc_event_cb(events, peer)
            });
        if getpeereid(fd, &mut (*peer).uid, &mut gid) != 0 as ::core::ffi::c_int {
            (*peer).uid = -(1 as ::core::ffi::c_int) as uid_t;
        }
        log_debug(
            c"add peer %p: %d (%p)".as_ptr(),
            fmt_args![peer, fd, peer_owner_ptr(peer)],
        );
        proc_update_event(peer);
        peer_box
    }
}
/// The peer a client holds, as the borrowed view the message calls take, or
/// null for a client that has none.
pub fn peer_ptr(value: &Option<Box<tmuxpeer>>) -> *mut tmuxpeer {
    value
        .as_ref()
        .map(|peer| &raw const **peer as *mut tmuxpeer)
        .unwrap_or(::core::ptr::null_mut::<tmuxpeer>())
}
pub unsafe fn proc_remove_peer(mut peer: Box<tmuxpeer>) {
    unsafe {
        let peer_ptr = &raw mut *peer;
        log_debug(c"remove peer %p".as_ptr(), fmt_args![peer_ptr]);
        (*peer_ptr).event.disable();
        imsgbuf_clear(&mut (*peer_ptr).ibuf);
        close((*peer_ptr).ibuf.fd);
        drop(peer);
    }
}
pub unsafe fn proc_kill_peer(mut peer: *mut tmuxpeer) {
    unsafe {
        (*peer).flags |= PEER_BAD;
    }
}
pub unsafe fn proc_flush_peer(mut peer: *mut tmuxpeer) {
    unsafe {
        imsgbuf_flush(&mut (*peer).ibuf);
    }
}
pub unsafe fn proc_toggle_log(mut tp: *mut tmuxproc) {
    unsafe {
        log_toggle((*tp).name_ptr());
    }
}
/// Forks a daemon, answering the child's process id and this end of the
/// socket pair the two halves talk over.
pub unsafe fn proc_fork_and_daemon() -> (pid_t, ::core::ffi::c_int) {
    unsafe {
        let mut pid: pid_t = 0;
        let mut pair: [::core::ffi::c_int; 2] = [0; 2];
        if socketpair(
            AF_UNIX,
            SOCK_STREAM as ::core::ffi::c_int,
            PF_UNSPEC,
            &raw mut pair as *mut ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            fatal(c"socketpair failed".as_ptr(), fmt_args![]);
        }
        pid = fork() as pid_t;
        match pid {
            -1 => {
                fatal(c"fork failed".as_ptr(), fmt_args![]);
            }
            0 => {
                close(pair[0 as ::core::ffi::c_int as usize]);
                let fd = pair[1 as ::core::ffi::c_int as usize];
                if daemon(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                    != 0 as ::core::ffi::c_int
                {
                    fatal(c"daemon failed".as_ptr(), fmt_args![]);
                }
                (0 as pid_t, fd)
            }
            _ => {
                close(pair[1 as ::core::ffi::c_int as usize]);
                (pid, pair[0 as ::core::ffi::c_int as usize])
            }
        }
    }
}
pub unsafe fn proc_get_peer_uid(mut peer: *mut tmuxpeer) -> uid_t {
    unsafe { (*peer).uid }
}
