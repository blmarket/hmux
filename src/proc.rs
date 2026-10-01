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
use crate::src::reactor::{self, poll_runtime};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{gid_t, uid_t, uint32_t};
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
use hmux_rt::AsyncFd as _;
use hmux_rt::{Handle as _, Signals as _};
use std::ffi::CStr;
use std::os::fd::FromRawFd;

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
unsafe fn proc_io_ready(peer: *mut tmuxpeer, readable: bool, writable: bool) {
    if (*peer).flags & PEER_BAD == 0 && readable {
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
    if writable {
        if imsgbuf_write(&mut (*peer).ibuf).is_err() {
            proc_dispatch(peer, PeerMessage::Disconnected);
            return;
        }
    }
    if (*peer).flags & PEER_BAD != 0 && imsgbuf_queuelen(&(*peer).ibuf) == 0 as uint32_t {
        proc_dispatch(peer, PeerMessage::Disconnected);
        return;
    }
    proc_update_io(peer);
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
unsafe fn proc_update_io(peer: *mut tmuxpeer) {
    let writable = imsgbuf_queuelen(&(*peer).ibuf) > 0;
    if (*peer).io_task.is_some() && (*peer).io_writable == writable {
        return;
    }
    (*peer).io_writable = writable;
    let fd = (*peer).ibuf.fd;
    crate::src::reactor::task_start(&mut (*peer).io_task, move || {
        let source = reactor::descriptor(fd)?;
        Ok(async move {
            loop {
                let (readable, writable) =
                    source.ready(true, writable).await.expect("peer I/O wait");
                unsafe { proc_io_ready(peer, readable, writable) };
                // Dispatch can remove the peer or replace this task. Yield
                // before another wait so cancellation drops the old future.
                reactor::yield_now().await;
            }
        })
    })
    .expect("start peer I/O");
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
    proc_update_io(peer);
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
        log_cstr(getversion().as_ptr()),
        log_cstr((socket_path) as *const _),
        (PROTOCOL_VERSION) as i32
    ));
    log_debug(format_args!(
        "on {} {} {}",
        log_cstr((&raw mut u.sysname as *mut ::core::ffi::c_char) as *const _),
        log_cstr((&raw mut u.release as *mut ::core::ffi::c_char) as *const _),
        log_cstr((&raw mut u.version as *mut ::core::ffi::c_char) as *const _)
    ));
    log_debug(format_args!("using runtime hmux-rt mio"));
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
        signal_task: Default::default(),
        peers: Vec::new(),
    })
}
/// Stop observers and release peers before releasing the process allocation.
/// Call only after dispatch has returned and no further process use is possible.
pub unsafe fn proc_free(mut owner: Box<tmuxproc>) {
    let tp = &raw mut *owner;
    proc_clear_signals(tp, 0);
    // Remove through the owner, not a peer's parent back-pointer: that pointer
    // predates the unique Box passed to this function and must not mutate it.
    while let Some(peer) = owner.peers.pop() {
        proc_free_peer(peer);
    }
    drop(owner);
}

pub unsafe fn proc_loop(mut tp: *mut tmuxproc, mut loopcb: Option<&mut dyn FnMut() -> bool>) {
    log_debug(format_args!(
        "{} loop enter",
        crate::src::log::log_bytes((*tp).name.as_bytes())
    ));
    loop {
        poll_runtime();
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
    crate::src::reactor::task_start(&mut (*tp).signal_task, move || {
        let mut signals = reactor::handle().signals(&[
            SIGINT, SIGHUP, SIGCHLD, SIGCONT, SIGTERM, SIGUSR1, SIGUSR2, SIGWINCH,
        ])?;
        Ok(async move {
            loop {
                let signo = signals.recv().await.expect("process signal wait");
                unsafe { proc_signal_cb(signo, tp) };
                reactor::yield_now().await;
            }
        })
    })
    .expect("start process signals");
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
    drop((*tp).signal_task.take());
    if defaults != 0 {
        // Pre-exec cleanup may run inside a runtime callback. The runtime
        // remains borrowed until exec replaces the process. Reset signals here.
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
    proc_update_io(peer);
    return peer;
}
pub unsafe fn proc_remove_peer(peer: *mut tmuxpeer) {
    let peers = &mut (*(*peer).parent).peers;
    let peer_index = peers
        .iter()
        .position(|owned_peer| std::ptr::eq(&**owned_peer, peer))
        .expect("peer must be owned by its parent process");
    let owned_peer = peers.remove(peer_index);
    proc_free_peer(owned_peer);
}

unsafe fn proc_free_peer(mut owned_peer: Box<tmuxpeer>) {
    let peer = &raw mut *owned_peer;
    log_debug(format_args!(
        "remove peer {}",
        log_pointer((peer) as *const ::core::ffi::c_void)
    ));
    drop((*peer).io_task.take());
    reactor::forget_descriptor((*peer).ibuf.fd);
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
    assert!(
        !reactor::runtime_initialized(),
        "daemonize before initializing the runtime"
    );
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
    fn peer_readiness_switches_direction_and_dispatch_can_remove_its_peer() {
        use std::cell::{Cell, RefCell};
        use std::os::fd::IntoRawFd;
        use std::os::unix::net::UnixStream;
        use std::rc::Rc;
        use std::time::Duration;
        unsafe {
            let mut owner = proc_start(c"peer-future-test".as_ptr());
            let tp = &raw mut *owner;
            let (sender, receiver) = UnixStream::pair().unwrap();
            sender.set_nonblocking(true).unwrap();
            receiver.set_nonblocking(true).unwrap();
            let sender_fd = sender.into_raw_fd();
            let receiver_fd = receiver.into_raw_fd();
            let calls = Rc::new(RefCell::new(Vec::new()));
            let sender_slot = Rc::new(Cell::new(std::ptr::null_mut()));
            let receiver_slot = Rc::new(Cell::new(std::ptr::null_mut()));
            let observed = calls.clone();
            let slot = sender_slot.clone();
            let sender = proc_add_peer(
                tp,
                sender_fd,
                Box::new(move |message| {
                    assert!(matches!(message, PeerMessage::Disconnected));
                    observed.borrow_mut().push("disconnected");
                    proc_remove_peer(slot.get());
                }),
            );
            sender_slot.set(sender);
            let observed = calls.clone();
            let slot = receiver_slot.clone();
            let receiver = proc_add_peer(
                tp,
                receiver_fd,
                Box::new(move |message| {
                    assert!(matches!(message, PeerMessage::Message(_)));
                    observed.borrow_mut().push("message");
                    proc_remove_peer(slot.get());
                }),
            );
            receiver_slot.set(receiver);
            assert!(!(*sender).io_writable);
            assert_eq!(proc_send(sender, MSG_COMMAND, -1, std::ptr::null(), 0), 0);
            assert!((*sender).io_writable);

            let expired = Rc::new(Cell::new(false));
            let observed = expired.clone();
            let timeout =
                reactor::Timer::new(Duration::from_secs(2), move || observed.set(true)).unwrap();
            while !owner.peers.is_empty() && !expired.get() {
                poll_runtime();
            }
            assert_eq!(&*calls.borrow(), &["message", "disconnected"]);
            assert!(owner.peers.is_empty());
            assert_eq!(libc::fcntl(sender_fd, libc::F_GETFD), -1);
            assert_eq!(libc::fcntl(receiver_fd, libc::F_GETFD), -1);
            drop(timeout);
            proc_free(owner);
            reactor::shutdown_runtime();
        }
    }

    #[test]
    fn process_signal_futures_dispatch_and_can_be_reinstalled() {
        use std::cell::RefCell;
        use std::rc::Rc;
        reactor::shutdown_runtime();
        unsafe {
            let pid = libc::fork();
            assert!(pid >= 0);
            if pid == 0 {
                libc::alarm(5);
                reactor::init_runtime();
                let mut owner = proc_start(c"signal-future-test".as_ptr());
                let tp = &raw mut *owner;
                let calls = Rc::new(RefCell::new(Vec::new()));
                for signo in [SIGUSR1, SIGUSR2] {
                    let observed = calls.clone();
                    proc_set_signals(
                        tp,
                        Some(Box::new(move |signal| {
                            observed.borrow_mut().push(signal.as_raw());
                        })),
                    );
                    libc::raise(signo);
                    poll_runtime();
                }
                assert_eq!(&*calls.borrow(), &[SIGUSR1, SIGUSR2]);
                // Pre-exec signal cleanup must also work inside a callback,
                // while the runtime owner is mutably borrowed by poll.
                let mut cleanup = None::<hmux_rt::mio::Task>;
                crate::src::reactor::task_start(&mut cleanup, move || {
                    Ok(async move { proc_clear_signals(tp, 1) })
                })
                .unwrap();
                poll_runtime();
                proc_free(owner);
                libc::_exit(0);
            }
            let mut status = 0;
            assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
            assert!(libc::WIFEXITED(status), "signal child status {status}");
            assert_eq!(libc::WEXITSTATUS(status), 0);
        }
    }

    #[test]
    fn process_free_cancels_tasks_and_explicitly_closes_remaining_peers() {
        unsafe {
            let mut owner = proc_start(c"process-owner-test".as_ptr());
            let tp = &raw mut *owner;
            let calls = std::rc::Rc::new(std::cell::Cell::new(0));
            let observed = calls.clone();
            crate::src::reactor::task_start(&mut (*tp).signal_task, move || {
                let observed = observed.clone();
                Ok(async move {
                    observed.set(observed.get() + 1);
                })
            })
            .unwrap();

            let mut pair = [0; 2];
            assert_eq!(
                ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, pair.as_mut_ptr()),
                0
            );
            crate::src::tmux::setblocking(pair[0], 0);
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
            poll_runtime();
            assert_eq!(calls.get(), 0);
            assert_eq!(std::rc::Rc::strong_count(&calls), 1);
            crate::src::reactor::shutdown_runtime();
        }
    }
}
