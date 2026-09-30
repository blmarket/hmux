use crate::src::compat::getpeereid::getpeereid;
use crate::src::compat::imsg::PROTOCOL_VERSION;
use crate::src::compat::imsg::*;
use crate::src::compat::imsg::{imsg, imsgbuf};
use crate::src::compat::imsg::{
    imsg_compose, imsgbuf_allow_fdpass, imsgbuf_clear, imsgbuf_flush, imsgbuf_get, imsgbuf_init,
    imsgbuf_queuelen, imsgbuf_read, imsgbuf_write,
};
use crate::src::compat::setproctitle::setproctitle;
use crate::src::ffi::libc::utsname;
use crate::src::ffi::libc::{
    close, daemon, fork, getpid, memset, sigaction, sigemptyset, socketpair, uname,
};
use crate::src::ffi::utf8proc::utf8proc_version;
use crate::src::format::bytes::write_cstr;
use crate::src::log::{fatal, fatalx, log_cstr, log_debug, log_open, log_pointer, log_toggle};
use crate::src::reactor::{
    event_add, event_del, event_get_method, event_get_version, event_loop, event_pending, event_set,
};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{gid_t, uid_t, uint32_t};
use crate::src::shared::event::{EV_PERSIST, EV_READ, EV_SIGNAL, EV_WRITE};
use crate::src::shared::process::PeerMessage;
use crate::src::shared::process::{tmuxpeer, tmuxproc};
use crate::src::shared::signal::ProcessSignal;
pub use crate::src::shared::signal::{
    __sighandler_t, __sigset_t, sigaction, sigaction___sigaction_handler, SA_RESTART, SIGCHLD,
    SIGCONT, SIGHUP, SIGINT, SIGTERM, SIGTSTP, SIGTTIN, SIGTTOU, SIGUSR1, SIGUSR2, SIGWINCH,
    SIG_DFL,
};
use crate::src::shared::socket::{AF_UNIX, PF_UNSPEC, SOCK_STREAM};
use crate::src::tmux::{getversion, socket_path};
use std::ffi::CStr;
use std::os::fd::FromRawFd;
use std::time::Duration;

pub const SIGQUIT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SIGPIPE: ::core::ffi::c_int = 13 as ::core::ffi::c_int;

pub const NCURSES_VERSION_PATCH: ::core::ffi::c_int = 20251230 as ::core::ffi::c_int;
pub const NCURSES_VERSION: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"6.6\0") };

pub const PEER_BAD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
unsafe fn proc_dispatch(peer: *mut tmuxpeer, message: PeerMessage<'_>) -> bool {
    let Some(mut dispatchcb) = (*peer).dispatchcb.take() else {
        return false;
    };
    let tp = (*peer).parent;
    dispatchcb(message);
    if (*tp)
        .peers
        .iter()
        .any(|owned_peer| std::ptr::eq(&**owned_peer, peer))
    {
        (*peer).dispatchcb = Some(dispatchcb);
        true
    } else {
        false
    }
}
unsafe fn proc_event_cb(events: ::core::ffi::c_short, peer: *mut tmuxpeer) {
    if (*peer).flags & PEER_BAD == 0 && events as ::core::ffi::c_int & EV_READ != 0 {
        if !matches!(imsgbuf_read(&mut (*peer).ibuf), Ok(1)) {
            proc_dispatch(peer, PeerMessage::Disconnected);
            return;
        }
        loop {
            let mut imsg = match imsgbuf_get(&mut (*peer).ibuf) {
                Ok(Some(imsg)) => imsg,
                Ok(None) => break,
                Err(_) => {
                    proc_dispatch(peer, PeerMessage::Disconnected);
                    return;
                }
            };
            log_debug(format_args!(
                "peer {} message {}",
                log_pointer((peer) as *const ::core::ffi::c_void),
                (imsg.hdr.type_0) as i32
            ));
            if peer_check_version(peer, &imsg) != 0 as ::core::ffi::c_int {
                break;
            } else {
                let peer_alive = proc_dispatch(peer, PeerMessage::Message(&mut imsg));
                if !peer_alive {
                    return;
                }
            }
        }
    }
    if events as ::core::ffi::c_int & EV_WRITE != 0 {
        if imsgbuf_write(&mut (*peer).ibuf).is_err() {
            proc_dispatch(peer, PeerMessage::Disconnected);
            return;
        }
    }
    if (*peer).flags & PEER_BAD != 0 && imsgbuf_queuelen(&(*peer).ibuf) == 0 as uint32_t {
        proc_dispatch(peer, PeerMessage::Disconnected);
        return;
    }
    proc_update_event(peer);
}
unsafe fn proc_signal_cb(signo: ::core::ffi::c_int, tp: *mut tmuxproc) {
    (*tp)
        .signalcb
        .as_mut()
        .expect("signal callback is installed")(ProcessSignal::from_raw(signo));
}
unsafe fn peer_check_version(peer: *mut tmuxpeer, imsg: &imsg) -> ::core::ffi::c_int {
    let mut version: ::core::ffi::c_int = 0;
    version = (imsg.hdr.peerid & 0xff as uint32_t) as ::core::ffi::c_int;
    if imsg.hdr.type_0 != MSG_VERSION && version != PROTOCOL_VERSION {
        log_debug(format_args!(
            "peer {} bad version {}",
            log_pointer((peer) as *const ::core::ffi::c_void),
            (version) as i32
        ));
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
unsafe fn proc_update_event(mut peer: *mut tmuxpeer) {
    let mut events: ::core::ffi::c_short = 0;
    events = EV_READ as ::core::ffi::c_short;
    if imsgbuf_queuelen(&(*peer).ibuf) > 0 as uint32_t {
        events = (events as ::core::ffi::c_int | EV_WRITE) as ::core::ffi::c_short;
    }
    // Keep the descriptor registration while its interest is unchanged.
    // A callback may rearm it when the output queue changes direction.
    if event_pending(&raw mut (*peer).event, 6, None) == events as ::core::ffi::c_int {
        return;
    }
    event_del(&raw mut (*peer).event);
    event_set(
        &raw mut (*peer).event,
        (*peer).ibuf.fd,
        events | EV_PERSIST as ::core::ffi::c_short,
        move |_, flags| unsafe { proc_event_cb(flags, peer) },
    );
    event_add(&raw mut (*peer).event, None);
}
pub unsafe fn proc_send(
    mut peer: *mut tmuxpeer,
    mut type_0: msgtype,
    mut fd: ::core::ffi::c_int,
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let imsgbuf = &mut (*peer).ibuf;
    if (*peer).flags & PEER_BAD != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    log_debug(format_args!(
        "sending message {} to peer {} ({} bytes)",
        (type_0 as ::core::ffi::c_uint) as i32,
        log_pointer((peer) as *const ::core::ffi::c_void),
        (len) as usize
    ));
    let Some(data) = (len == 0).then_some(&[][..]).or_else(|| {
        (!buf.is_null()).then(|| unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), len) })
    }) else {
        return -1;
    };
    let fd = (fd >= 0).then(|| unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) });
    if imsg_compose(
        imsgbuf,
        type_0,
        PROTOCOL_VERSION as uint32_t,
        -(1 as pid_t),
        fd,
        data,
    )
    .is_err()
    {
        return -(1 as ::core::ffi::c_int);
    }
    proc_update_event(peer);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn proc_start(mut name: *const ::core::ffi::c_char) -> Box<tmuxproc> {
    let mut u: utsname = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    log_open(name);
    setproctitle(|out| {
        write_cstr(out, name)?;
        out.write_all(b" (")?;
        write_cstr(out, socket_path)?;
        out.write_all(b")")
    });
    if uname(&raw mut u) < 0 as ::core::ffi::c_int {
        memset(
            &raw mut u as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<utsname>() as size_t,
        );
    }
    log_debug(format_args!(
        "{} started ({}): version {}, socket {}, protocol {}",
        log_cstr((name) as *const _),
        getpid() as ::core::ffi::c_long,
        log_cstr((getversion()) as *const _),
        log_cstr((socket_path) as *const _),
        (PROTOCOL_VERSION) as i32
    ));
    log_debug(format_args!(
        "on {} {} {}",
        log_cstr((&raw mut u.sysname as *mut ::core::ffi::c_char) as *const _),
        log_cstr((&raw mut u.release as *mut ::core::ffi::c_char) as *const _),
        log_cstr((&raw mut u.version as *mut ::core::ffi::c_char) as *const _)
    ));
    log_debug(format_args!(
        "using runtime {} {}",
        log_cstr((event_get_version()) as *const _),
        log_cstr((event_get_method()) as *const _)
    ));
    log_debug(format_args!(
        "using utf8proc {}",
        log_cstr((utf8proc_version()) as *const _)
    ));
    log_debug(format_args!(
        "using ncurses {} {:06}",
        log_cstr((NCURSES_VERSION.as_ptr()) as *const _),
        (NCURSES_VERSION_PATCH) as u32
    ));
    Box::new(tmuxproc {
        name: CStr::from_ptr(name).to_owned(),
        exit: 0,
        signalcb: None,
        ev_sigint: Default::default(),
        ev_sighup: Default::default(),
        ev_sigchld: Default::default(),
        ev_sigcont: Default::default(),
        ev_sigterm: Default::default(),
        ev_sigusr1: Default::default(),
        ev_sigusr2: Default::default(),
        ev_sigwinch: Default::default(),
        peers: Vec::new(),
    })
}
/// Stop observers and release peers before releasing the process allocation.
/// Call only after dispatch has returned and no further process use is possible.
pub unsafe fn proc_free(mut owner: Box<tmuxproc>) {
    let tp = &raw mut *owner;
    proc_clear_signals(tp, 0);
    while let Some(peer) = (*tp).peers.last_mut() {
        let peer = &raw mut **peer;
        proc_remove_peer(peer);
    }
    drop(owner);
}

pub unsafe fn proc_loop(mut tp: *mut tmuxproc, mut loopcb: Option<&mut dyn FnMut() -> bool>) {
    log_debug(format_args!(
        "{} loop enter",
        crate::src::log::log_bytes((*tp).name.as_bytes())
    ));
    loop {
        event_loop();
        if (*tp).exit != 0 || loopcb.as_mut().is_some_and(|callback| !callback()) {
            break;
        }
    }
    log_debug(format_args!(
        "{} loop exit",
        crate::src::log::log_bytes((*tp).name.as_bytes())
    ));
}
pub unsafe fn proc_exit(mut tp: *mut tmuxproc) {
    for peer in (*tp).peers.iter_mut() {
        let peer: *mut tmuxpeer = &mut **peer;
        let _ = imsgbuf_flush(&mut (*peer).ibuf);
    }
    (*tp).exit = 1 as ::core::ffi::c_int;
}
pub unsafe fn proc_set_signals(
    mut tp: *mut tmuxproc,
    mut signalcb: Option<Box<dyn FnMut(ProcessSignal)>>,
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
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigint, None);
    event_set(
        &raw mut (*tp).ev_sighup,
        1 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sighup, None);
    event_set(
        &raw mut (*tp).ev_sigchld,
        17 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigchld, None);
    event_set(
        &raw mut (*tp).ev_sigcont,
        18 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigcont, None);
    event_set(
        &raw mut (*tp).ev_sigterm,
        15 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigterm, None);
    event_set(
        &raw mut (*tp).ev_sigusr1,
        10 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigusr1, None);
    event_set(
        &raw mut (*tp).ev_sigusr2,
        12 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigusr2, None);
    event_set(
        &raw mut (*tp).ev_sigwinch,
        28 as ::core::ffi::c_int,
        (EV_SIGNAL | EV_PERSIST) as ::core::ffi::c_short,
        move |fd, _| unsafe { proc_signal_cb(fd, tp) },
    );
    event_add(&raw mut (*tp).ev_sigwinch, None);
}
pub unsafe fn proc_clear_signals(mut tp: *mut tmuxproc, mut defaults: ::core::ffi::c_int) {
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
        crate::src::reactor::shutdown_runtime();
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
pub unsafe fn proc_add_peer(
    mut tp: *mut tmuxproc,
    mut fd: ::core::ffi::c_int,
    mut dispatchcb: Box<dyn for<'a> FnMut(PeerMessage<'a>)>,
) -> *mut tmuxpeer {
    let mut owned_peer = Box::new(tmuxpeer::default());
    let peer: *mut tmuxpeer = &mut *owned_peer;
    (*peer).parent = tp;
    (*peer).dispatchcb = Some(dispatchcb);
    if let Err(error) = imsgbuf_init(&mut (*peer).ibuf, fd) {
        fatalx(|out| write!(out, "imsgbuf_init failed (errno {})", (error) as i32));
    }
    imsgbuf_allow_fdpass(&mut (*peer).ibuf);
    event_set(
        &raw mut (*peer).event,
        fd,
        EV_READ as ::core::ffi::c_short,
        move |_, flags| unsafe { proc_event_cb(flags, peer) },
    );
    if getpeereid(fd, &raw mut (*peer).uid, &raw mut (*peer).gid) != 0 as ::core::ffi::c_int {
        (*peer).uid = -(1 as ::core::ffi::c_int) as uid_t;
        (*peer).gid = -(1 as ::core::ffi::c_int) as gid_t;
    }
    log_debug(format_args!(
        "add peer {}: {}",
        log_pointer((peer) as *const ::core::ffi::c_void),
        (fd) as i32
    ));
    (*tp).peers.push(owned_peer);
    proc_update_event(peer);
    return peer;
}
pub unsafe fn proc_remove_peer(peer: *mut tmuxpeer) {
    let peers = &mut (*(*peer).parent).peers;
    let peer_index = peers
        .iter()
        .position(|owned_peer| std::ptr::eq(&**owned_peer, peer))
        .expect("peer must be owned by its parent process");
    let owned_peer = peers.remove(peer_index);
    log_debug(format_args!(
        "remove peer {}",
        log_pointer((peer) as *const ::core::ffi::c_void)
    ));
    event_del(&raw mut (*peer).event);
    imsgbuf_clear(&mut (*peer).ibuf);
    close((*peer).ibuf.fd);
    drop(owned_peer);
}
pub unsafe fn proc_kill_peer(mut peer: *mut tmuxpeer) {
    (*peer).flags |= PEER_BAD;
}
pub unsafe fn proc_flush_peer(mut peer: *mut tmuxpeer) {
    let _ = imsgbuf_flush(&mut (*peer).ibuf);
}
pub unsafe fn proc_toggle_log(mut tp: *mut tmuxproc) {
    log_toggle((*tp).name.as_ptr());
}
pub unsafe fn proc_fork_and_daemon(mut fd: *mut ::core::ffi::c_int) -> pid_t {
    let mut pid: pid_t = 0;
    let mut pair: [::core::ffi::c_int; 2] = [0; 2];
    if socketpair(
        AF_UNIX,
        SOCK_STREAM as ::core::ffi::c_int,
        PF_UNSPEC,
        &raw mut pair as *mut ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        fatal(|out| out.write_all(b"socketpair failed"));
    }
    pid = fork() as pid_t;
    match pid {
        -1 => {
            fatal(|out| out.write_all(b"fork failed"));
        }
        0 => {
            close(pair[0 as ::core::ffi::c_int as usize]);
            *fd = pair[1 as ::core::ffi::c_int as usize];
            if daemon(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
                fatal(|out| out.write_all(b"daemon failed"));
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
pub unsafe fn proc_get_peer_uid(mut peer: *mut tmuxpeer) -> uid_t {
    return (*peer).uid;
}
pub unsafe fn proc_get_peer_gid(mut peer: *mut tmuxpeer) -> gid_t {
    return (*peer).gid;
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn process_free_cancels_events_and_explicitly_closes_remaining_peers() {
        unsafe {
            let mut owner = proc_start(c"process-owner-test".as_ptr());
            let tp = &raw mut *owner;
            let calls = std::rc::Rc::new(std::cell::Cell::new(0));
            let observed = calls.clone();
            event_set(&raw mut (*tp).ev_sigint, -1, 0, move |_, _| {
                observed.set(observed.get() + 1);
            });
            let timeout = Duration::ZERO;
            event_add(&raw mut (*tp).ev_sigint, Some(timeout));

            let mut pair = [0; 2];
            assert_eq!(
                ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, pair.as_mut_ptr()),
                0
            );
            let capture = refbox::RefBox::new(());
            let observer = capture.downgrade();
            proc_add_peer(
                tp,
                pair[0],
                Box::new(move |_| {
                    let _keep_capture = &capture;
                    panic!("peer callback must be cancelled before freeing its owner");
                }),
            );
            proc_free(owner);
            assert_eq!(::libc::fcntl(pair[0], ::libc::F_GETFD), -1);
            assert!(matches!(
                observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
            close(pair[1]);
            event_loop();
            assert_eq!(calls.get(), 0);
            assert_eq!(std::rc::Rc::strong_count(&calls), 1);
            crate::src::reactor::shutdown_runtime();
        }
    }
}
