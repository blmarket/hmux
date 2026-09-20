pub use crate::src::shared::message::{ibuf, ibuf_entry, imsg, imsgbuf, msgbuf};
pub use crate::src::shared::process::{tmuxpeer, tmuxpeer_entry, tmuxproc, tmuxproc_peers};
pub use crate::src::shared::message::{PROTOCOL_VERSION, imsg_hdr};
pub use crate::src::shared::socket::{
    __socket_type, AF_UNIX, PF_LOCAL, PF_UNIX, PF_UNSPEC, SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM,
    SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
};
pub use crate::src::shared::signal::{
    __sighandler_t, __sigset_t, __sigval_t, sigaction, sigaction___sigaction_handler, siginfo_t,
    siginfo_t__sifields, siginfo_t__sifields__kill, siginfo_t__sifields__rt,
    siginfo_t__sifields__sigchld, siginfo_t__sifields__sigfault,
    siginfo_t__sifields__sigfault__bounds, siginfo_t__sifields__sigfault__bounds__addr_bnd,
    siginfo_t__sifields__sigpoll, siginfo_t__sifields__sigsys, siginfo_t__sifields__timer,
    sigset_t, sigval, SA_RESTART, SIGCHLD, SIGCONT, SIGHUP, SIGINT, SIGTERM, SIGTSTP, SIGTTIN,
    SIGTTOU, SIGUSR1, SIGUSR2, SIGWINCH, SIG_DFL,
};
pub use crate::src::shared::abi::{__clock_t, __gid_t, __uid_t, __uint32_t, gid_t, uid_t, uint32_t};
pub use crate::src::shared::event::{EV_PERSIST, EV_READ, EV_SIGNAL, EV_WRITE};
use crate::src::shared::event::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use ::libc;
extern "C" {

    fn socketpair(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
        __fds: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uname(__name: *mut utsname) -> ::core::ffi::c_int;
    fn sigemptyset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn sigaction(
        __sig: ::core::ffi::c_int,
        __act: *const sigaction,
        __oact: *mut sigaction,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn getpid() -> __pid_t;
    fn fork() -> __pid_t;
    fn daemon(__nochdir: ::core::ffi::c_int, __noclose: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_get_version() -> *const ::core::ffi::c_char;
    fn event_loop(_: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn event_get_method() -> *const ::core::ffi::c_char;
    fn event_set(
        _: *mut event,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn utf8proc_version() -> *const ::core::ffi::c_char;
    fn imsgbuf_init(_: *mut imsgbuf, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn imsgbuf_allow_fdpass(imsgbuf: *mut imsgbuf);
    fn imsgbuf_read(_: *mut imsgbuf) -> ::core::ffi::c_int;
    fn imsgbuf_write(_: *mut imsgbuf) -> ::core::ffi::c_int;
    fn imsgbuf_flush(_: *mut imsgbuf) -> ::core::ffi::c_int;
    fn imsgbuf_clear(_: *mut imsgbuf);
    fn imsgbuf_queuelen(_: *mut imsgbuf) -> uint32_t;
    fn imsgbuf_get(_: *mut imsgbuf, _: *mut imsg) -> ::core::ffi::c_int;
    fn imsg_compose(
        _: *mut imsgbuf,
        _: uint32_t,
        _: uint32_t,
        _: pid_t,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_void,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn imsg_free(_: *mut imsg);
    fn getpeereid(_: ::core::ffi::c_int, _: *mut uid_t, _: *mut gid_t) -> ::core::ffi::c_int;
    fn setproctitle(_: *const ::core::ffi::c_char, ...);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut socket_path: *const ::core::ffi::c_char;
    fn getversion() -> *const ::core::ffi::c_char;
    fn log_open(_: *const ::core::ffi::c_char);
    fn log_toggle(_: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct utsname {
    pub sysname: [::core::ffi::c_char; 65],
    pub nodename: [::core::ffi::c_char; 65],
    pub release: [::core::ffi::c_char; 65],
    pub version: [::core::ffi::c_char; 65],
    pub machine: [::core::ffi::c_char; 65],
    pub domainname: [::core::ffi::c_char; 65],
}

pub const SIGQUIT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SIGPIPE: ::core::ffi::c_int = 13 as ::core::ffi::c_int;

pub const NCURSES_VERSION_PATCH: ::core::ffi::c_int = 20251230 as ::core::ffi::c_int;
pub const NCURSES_VERSION: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"6.6\0") };
pub const EVLOOP_ONCE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

pub const PEER_BAD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
unsafe extern "C" fn proc_event_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut peer: *mut tmuxpeer = arg as *mut tmuxpeer;
    let mut n: ::core::ffi::c_int = 0;
    let mut imsg: imsg = imsg {
        hdr: imsg_hdr {
            type_0: 0,
            len: 0,
            peerid: 0,
            pid: 0,
        },
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        buf: ::core::ptr::null_mut::<ibuf>(),
    };
    if (*peer).flags & PEER_BAD == 0 && events as ::core::ffi::c_int & EV_READ != 0 {
        if imsgbuf_read(&raw mut (*peer).ibuf) != 1 as ::core::ffi::c_int {
            (*peer).dispatchcb.expect("non-null function pointer")(
                ::core::ptr::null_mut::<imsg>(),
                (*peer).arg,
            );
            return;
        }
        loop {
            n = imsgbuf_get(&raw mut (*peer).ibuf, &raw mut imsg);
            if n == -(1 as ::core::ffi::c_int) {
                (*peer).dispatchcb.expect("non-null function pointer")(
                    ::core::ptr::null_mut::<imsg>(),
                    (*peer).arg,
                );
                return;
            }
            if n == 0 as ::core::ffi::c_int {
                break;
            }
            log_debug(
                b"peer %p message %d\0" as *const u8 as *const ::core::ffi::c_char,
                peer,
                imsg.hdr.type_0,
            );
            if peer_check_version(peer, &raw mut imsg) != 0 as ::core::ffi::c_int {
                imsg_free(&raw mut imsg);
                break;
            } else {
                (*peer).dispatchcb.expect("non-null function pointer")(&raw mut imsg, (*peer).arg);
                imsg_free(&raw mut imsg);
            }
        }
    }
    if events as ::core::ffi::c_int & EV_WRITE != 0 {
        if imsgbuf_write(&raw mut (*peer).ibuf) == -(1 as ::core::ffi::c_int) {
            (*peer).dispatchcb.expect("non-null function pointer")(
                ::core::ptr::null_mut::<imsg>(),
                (*peer).arg,
            );
            return;
        }
    }
    if (*peer).flags & PEER_BAD != 0 && imsgbuf_queuelen(&raw mut (*peer).ibuf) == 0 as uint32_t {
        (*peer).dispatchcb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<imsg>(),
            (*peer).arg,
        );
        return;
    }
    proc_update_event(peer);
}
unsafe extern "C" fn proc_signal_cb(
    mut signo: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut tp: *mut tmuxproc = arg as *mut tmuxproc;
    (*tp).signalcb.expect("non-null function pointer")(signo);
}
unsafe extern "C" fn peer_check_version(
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut version: ::core::ffi::c_int = 0;
    version = ((*imsg).hdr.peerid & 0xff as uint32_t) as ::core::ffi::c_int;
    if (*imsg).hdr.type_0 != MSG_VERSION as ::core::ffi::c_int as uint32_t
        && version != PROTOCOL_VERSION
    {
        log_debug(
            b"peer %p bad version %d\0" as *const u8 as *const ::core::ffi::c_char,
            peer,
            version,
        );
        proc_send(
            peer,
            MSG_VERSION,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null::<::core::ffi::c_void>(),
            0 as size_t,
        );
        (*peer).flags |= PEER_BAD;
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn proc_update_event(mut peer: *mut tmuxpeer) {
    let mut events: ::core::ffi::c_short = 0;
    event_del(&raw mut (*peer).event);
    events = EV_READ as ::core::ffi::c_short;
    if imsgbuf_queuelen(&raw mut (*peer).ibuf) > 0 as uint32_t {
        events = (events as ::core::ffi::c_int | EV_WRITE) as ::core::ffi::c_short;
    }
    event_set(
        &raw mut (*peer).event,
        (*peer).ibuf.fd,
        events,
        Some(
            proc_event_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        peer as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*peer).event, ::core::ptr::null::<timeval>());
}
#[no_mangle]
pub unsafe extern "C" fn proc_send(
    mut peer: *mut tmuxpeer,
    mut type_0: msgtype,
    mut fd: ::core::ffi::c_int,
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut ibuf: *mut imsgbuf = &raw mut (*peer).ibuf;
    let mut vp: *mut ::core::ffi::c_void = buf as *mut ::core::ffi::c_void;
    let mut retval: ::core::ffi::c_int = 0;
    if (*peer).flags & PEER_BAD != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    log_debug(
        b"sending message %d to peer %p (%zu bytes)\0" as *const u8 as *const ::core::ffi::c_char,
        type_0 as ::core::ffi::c_uint,
        peer,
        len,
    );
    retval = imsg_compose(
        ibuf,
        type_0 as uint32_t,
        PROTOCOL_VERSION as uint32_t,
        -(1 as pid_t),
        fd,
        vp,
        len,
    );
    if retval != 1 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    proc_update_event(peer);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn proc_start(mut name: *const ::core::ffi::c_char) -> *mut tmuxproc {
    let mut tp: *mut tmuxproc = ::core::ptr::null_mut::<tmuxproc>();
    let mut u: utsname = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    log_open(name);
    setproctitle(
        b"%s (%s)\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        socket_path,
    );
    if uname(&raw mut u) < 0 as ::core::ffi::c_int {
        memset(
            &raw mut u as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<utsname>() as size_t,
        );
    }
    log_debug(
        b"%s started (%ld): version %s, socket %s, protocol %d\0" as *const u8
            as *const ::core::ffi::c_char,
        name,
        getpid() as ::core::ffi::c_long,
        getversion(),
        socket_path,
        PROTOCOL_VERSION,
    );
    log_debug(
        b"on %s %s %s\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut u.sysname as *mut ::core::ffi::c_char,
        &raw mut u.release as *mut ::core::ffi::c_char,
        &raw mut u.version as *mut ::core::ffi::c_char,
    );
    log_debug(
        b"using libevent %s %s\0" as *const u8 as *const ::core::ffi::c_char,
        event_get_version(),
        event_get_method(),
    );
    log_debug(
        b"using utf8proc %s\0" as *const u8 as *const ::core::ffi::c_char,
        utf8proc_version(),
    );
    log_debug(
        b"using ncurses %s %06u\0" as *const u8 as *const ::core::ffi::c_char,
        NCURSES_VERSION.as_ptr(),
        NCURSES_VERSION_PATCH,
    );
    tp = xcalloc(1 as size_t, ::core::mem::size_of::<tmuxproc>() as size_t) as *mut tmuxproc;
    (*tp).name = xstrdup(name);
    (*tp).peers.tqh_first = ::core::ptr::null_mut::<tmuxpeer>();
    (*tp).peers.tqh_last = &raw mut (*tp).peers.tqh_first;
    return tp;
}
#[no_mangle]
pub unsafe extern "C" fn proc_loop(
    mut tp: *mut tmuxproc,
    mut loopcb: Option<unsafe extern "C" fn() -> ::core::ffi::c_int>,
) {
    log_debug(
        b"%s loop enter\0" as *const u8 as *const ::core::ffi::c_char,
        (*tp).name,
    );
    loop {
        event_loop(EVLOOP_ONCE);
        if !((*tp).exit == 0
            && (loopcb.is_none() || loopcb.expect("non-null function pointer")() == 0))
        {
            break;
        }
    }
    log_debug(
        b"%s loop exit\0" as *const u8 as *const ::core::ffi::c_char,
        (*tp).name,
    );
}
#[no_mangle]
pub unsafe extern "C" fn proc_exit(mut tp: *mut tmuxproc) {
    let mut peer: *mut tmuxpeer = ::core::ptr::null_mut::<tmuxpeer>();
    peer = (*tp).peers.tqh_first;
    while !peer.is_null() {
        imsgbuf_flush(&raw mut (*peer).ibuf);
        peer = (*peer).entry.tqe_next;
    }
    (*tp).exit = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn proc_set_signals(
    mut tp: *mut tmuxproc,
    mut signalcb: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>,
) {
    let mut sa: sigaction = sigaction {
        __sigaction_handler: sigaction___sigaction_handler { sa_handler: None },
        sa_mask: __sigset_t { __val: [0; 16] },
        sa_flags: 0,
        sa_restorer: None,
    };
    (*tp).signalcb = signalcb;
    memset(
        &raw mut sa as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<sigaction>() as size_t,
    );
    sigemptyset(&raw mut sa.sa_mask);
    sa.sa_flags = SA_RESTART;
    sa.__sigaction_handler.sa_handler = ::core::mem::transmute::<::libc::intptr_t, __sighandler_t>(
        1 as ::core::ffi::c_int as ::libc::intptr_t,
    );
    sigaction(SIGPIPE, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    sigaction(SIGTSTP, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    sigaction(SIGTTIN, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    sigaction(SIGTTOU, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    sigaction(SIGQUIT, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    event_set(
        &raw mut (*tp).ev_sigint,
        2 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigint, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sighup,
        1 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sighup, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sigchld,
        17 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigchld, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sigcont,
        18 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigcont, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sigterm,
        15 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigterm, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sigusr1,
        10 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigusr1, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sigusr2,
        12 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigusr2, ::core::ptr::null::<timeval>());
    event_set(
        &raw mut (*tp).ev_sigwinch,
        28 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        Some(
            proc_signal_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        tp as *mut ::core::ffi::c_void,
    );
    event_add(&raw mut (*tp).ev_sigwinch, ::core::ptr::null::<timeval>());
}
#[no_mangle]
pub unsafe extern "C" fn proc_clear_signals(
    mut tp: *mut tmuxproc,
    mut defaults: ::core::ffi::c_int,
) {
    let mut sa: sigaction = sigaction {
        __sigaction_handler: sigaction___sigaction_handler { sa_handler: None },
        sa_mask: __sigset_t { __val: [0; 16] },
        sa_flags: 0,
        sa_restorer: None,
    };
    memset(
        &raw mut sa as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<sigaction>() as size_t,
    );
    sigemptyset(&raw mut sa.sa_mask);
    sa.sa_flags = SA_RESTART;
    sa.__sigaction_handler.sa_handler = SIG_DFL;
    sigaction(SIGPIPE, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    sigaction(SIGTSTP, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    event_del(&raw mut (*tp).ev_sigint);
    event_del(&raw mut (*tp).ev_sighup);
    event_del(&raw mut (*tp).ev_sigchld);
    event_del(&raw mut (*tp).ev_sigcont);
    event_del(&raw mut (*tp).ev_sigterm);
    event_del(&raw mut (*tp).ev_sigusr1);
    event_del(&raw mut (*tp).ev_sigusr2);
    event_del(&raw mut (*tp).ev_sigwinch);
    if defaults != 0 {
        sigaction(SIGINT, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGQUIT, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGHUP, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGCHLD, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGCONT, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGTERM, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGUSR1, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGUSR2, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
        sigaction(SIGWINCH, &raw mut sa, ::core::ptr::null_mut::<sigaction>());
    }
}
#[no_mangle]
pub unsafe extern "C" fn proc_add_peer(
    mut tp: *mut tmuxproc,
    mut fd: ::core::ffi::c_int,
    mut dispatchcb: Option<unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> ()>,
    mut arg: *mut ::core::ffi::c_void,
) -> *mut tmuxpeer {
    let mut peer: *mut tmuxpeer = ::core::ptr::null_mut::<tmuxpeer>();
    peer = xcalloc(1 as size_t, ::core::mem::size_of::<tmuxpeer>() as size_t) as *mut tmuxpeer;
    (*peer).parent = tp;
    (*peer).dispatchcb = dispatchcb;
    (*peer).arg = arg;
    if imsgbuf_init(&raw mut (*peer).ibuf, fd) == -(1 as ::core::ffi::c_int) {
        fatal(b"imsgbuf_init\0" as *const u8 as *const ::core::ffi::c_char);
    }
    imsgbuf_allow_fdpass(&raw mut (*peer).ibuf);
    event_set(
        &raw mut (*peer).event,
        fd,
        EV_READ as ::core::ffi::c_short,
        Some(
            proc_event_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        peer as *mut ::core::ffi::c_void,
    );
    if getpeereid(fd, &raw mut (*peer).uid, &raw mut (*peer).gid) != 0 as ::core::ffi::c_int {
        (*peer).uid = -(1 as ::core::ffi::c_int) as uid_t;
        (*peer).gid = -(1 as ::core::ffi::c_int) as gid_t;
    }
    log_debug(
        b"add peer %p: %d (%p)\0" as *const u8 as *const ::core::ffi::c_char,
        peer,
        fd,
        arg,
    );
    (*peer).entry.tqe_next = ::core::ptr::null_mut::<tmuxpeer>();
    (*peer).entry.tqe_prev = (*tp).peers.tqh_last;
    *(*tp).peers.tqh_last = peer;
    (*tp).peers.tqh_last = &raw mut (*peer).entry.tqe_next;
    proc_update_event(peer);
    return peer;
}
#[no_mangle]
pub unsafe extern "C" fn proc_remove_peer(mut peer: *mut tmuxpeer) {
    if !(*peer).entry.tqe_next.is_null() {
        (*(*peer).entry.tqe_next).entry.tqe_prev = (*peer).entry.tqe_prev;
    } else {
        (*(*peer).parent).peers.tqh_last = (*peer).entry.tqe_prev;
    }
    *(*peer).entry.tqe_prev = (*peer).entry.tqe_next;
    log_debug(
        b"remove peer %p\0" as *const u8 as *const ::core::ffi::c_char,
        peer,
    );
    event_del(&raw mut (*peer).event);
    imsgbuf_clear(&raw mut (*peer).ibuf);
    close((*peer).ibuf.fd);
    free(peer as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn proc_kill_peer(mut peer: *mut tmuxpeer) {
    (*peer).flags |= PEER_BAD;
}
#[no_mangle]
pub unsafe extern "C" fn proc_flush_peer(mut peer: *mut tmuxpeer) {
    imsgbuf_flush(&raw mut (*peer).ibuf);
}
#[no_mangle]
pub unsafe extern "C" fn proc_toggle_log(mut tp: *mut tmuxproc) {
    log_toggle((*tp).name);
}
#[no_mangle]
pub unsafe extern "C" fn proc_fork_and_daemon(mut fd: *mut ::core::ffi::c_int) -> pid_t {
    let mut pid: pid_t = 0;
    let mut pair: [::core::ffi::c_int; 2] = [0; 2];
    if socketpair(
        AF_UNIX,
        SOCK_STREAM as ::core::ffi::c_int,
        PF_UNSPEC,
        &raw mut pair as *mut ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        fatal(b"socketpair failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    pid = fork() as pid_t;
    match pid {
        -1 => {
            fatal(b"fork failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        0 => {
            close(pair[0 as ::core::ffi::c_int as usize]);
            *fd = pair[1 as ::core::ffi::c_int as usize];
            if daemon(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
                fatal(b"daemon failed\0" as *const u8 as *const ::core::ffi::c_char);
            }
            return 0 as pid_t;
        }
        _ => {
            close(pair[1 as ::core::ffi::c_int as usize]);
            *fd = pair[0 as ::core::ffi::c_int as usize];
            return pid;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn proc_get_peer_uid(mut peer: *mut tmuxpeer) -> uid_t {
    return (*peer).uid;
}
#[no_mangle]
pub unsafe extern "C" fn proc_get_peer_gid(mut peer: *mut tmuxpeer) -> gid_t {
    return (*peer).gid;
}
