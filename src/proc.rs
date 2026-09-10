use crate::compat::getpeereid;
use crate::compat::setproctitle;
use crate::compat::{
    imsg_compose, imsgbuf_allow_fdpass, imsgbuf_clear, imsgbuf_flush, imsgbuf_init,
    imsgbuf_queuelen, imsgbuf_read, imsgbuf_write,
};
use crate::compat::{imsg_free, imsg_get};
pub use crate::consts::{
    AF_UNIX, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING,
    MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, PF_LOCAL, PF_UNIX, PF_UNSPEC, PROTOCOL_VERSION, SA_RESTART, SIG_DFL, SIGCHLD,
    SIGCONT, SIGHUP, SIGINT, SIGTERM, SIGTSTP, SIGTTIN, SIGTTOU, SIGUSR1, SIGUSR2, SIGWINCH,
    SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM,
    SOCK_SEQPACKET, SOCK_STREAM,
};
use crate::ffi::{
    close, daemon, fork, getpid, sigaction, sigemptyset, socketpair, uname, utf8proc_version,
};
use crate::fmt_args;
use crate::log::{fatal, log_debug, log_open, log_toggle};
use crate::reactor;
use crate::reactor::{Interest, IoWatch, Reactor, SignalWatch, WatchMode};
use crate::tmux::getversion;
use crate::tmux::socket_path;
pub use crate::types::*;
use ::core::ffi::CStr;

pub type PeerDispatch = dyn for<'a> Fn(Option<&'a mut imsg>);
pub type ProcSignalCallback = dyn Fn(core::ffi::c_int);
pub type ProcessRef = std::rc::Rc<std::cell::RefCell<tmuxproc>>;
#[derive(Clone)]
pub struct PeerRef(std::rc::Rc<std::cell::RefCell<tmuxpeer>>);

impl PeerRef {
    pub fn new(mut peer: tmuxpeer) -> Self {
        Self(std::rc::Rc::new_cyclic(|owner| {
            peer.owner = Some(owner.clone());
            std::cell::RefCell::new(peer)
        }))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, tmuxpeer> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, tmuxpeer> {
        self.0.borrow_mut()
    }
}

#[repr(C)]
pub struct tmuxpeer {
    owner: Option<std::rc::Weak<std::cell::RefCell<tmuxpeer>>>,
    pub ibuf: imsgbuf,
    pub event: IoHandle,
    pub uid: uid_t,
    pub flags: core::ffi::c_int,
    /// Retained separately while it runs because dispatch may drop this peer.
    pub dispatchcb: Option<std::rc::Rc<PeerDispatch>>,
}
#[derive(Default)]
#[repr(C)]
pub struct tmuxproc {
    pub(crate) config: crate::cfg::ConfigState,
    pub name: Option<std::ffi::CString>,
    pub exit: core::ffi::c_int,
    pub signalcb: Option<std::rc::Rc<ProcSignalCallback>>,
    pub ev_sigint: SignalHandle,
    pub ev_sighup: SignalHandle,
    pub ev_sigchld: SignalHandle,
    pub ev_sigcont: SignalHandle,
    pub ev_sigterm: SignalHandle,
    pub ev_sigusr1: SignalHandle,
    pub ev_sigusr2: SignalHandle,
    pub ev_sigwinch: SignalHandle,
}

pub const SIGQUIT: core::ffi::c_int = 3 as core::ffi::c_int;
pub const SIGPIPE: core::ffi::c_int = 13 as core::ffi::c_int;

pub const NCURSES_VERSION_PATCH: core::ffi::c_int = 20251230 as core::ffi::c_int;
pub const NCURSES_VERSION: [core::ffi::c_char; 4] =
    unsafe { core::mem::transmute::<[u8; 4], [core::ffi::c_char; 4]>(*b"6.6\0") };
pub const EVLOOP_ONCE: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub use crate::consts::EV_READ;
pub use crate::consts::EV_WRITE;
pub const EV_SIGNAL: core::ffi::c_int = 0x8 as core::ffi::c_int;
pub const EV_PERSIST: core::ffi::c_int = 0x10 as core::ffi::c_int;

pub const PEER_BAD: core::ffi::c_int = 0x1 as core::ffi::c_int;

pub unsafe fn proc_start(name: &CStr) -> ProcessRef {
    unsafe {
        let mut u: utsname = core::mem::zeroed();
        log_open(name);
        setproctitle(c"%s (%s)", fmt_args![name, socket_path.get().as_deref()]);
        if uname(&raw mut u) < 0 as core::ffi::c_int {
            u = core::mem::zeroed();
        }
        log_debug(
            c"%s started (%ld): version %s, socket %s, protocol %d",
            fmt_args![
                name,
                getpid() as core::ffi::c_long,
                getversion(),
                socket_path.get().as_deref(),
                PROTOCOL_VERSION
            ],
        );
        log_debug(
            c"on %s %s %s",
            fmt_args![
                &raw mut u.sysname as *mut core::ffi::c_char,
                &raw mut u.release as *mut core::ffi::c_char,
                &raw mut u.version as *mut core::ffi::c_char
            ],
        );
        let reactor = reactor::current().describe();
        log_debug(c"using %s", fmt_args![reactor.as_ptr()]);
        log_debug(c"using utf8proc %s", fmt_args![utf8proc_version()]);
        log_debug(
            c"using ncurses %s %06u",
            fmt_args![NCURSES_VERSION.as_ptr(), NCURSES_VERSION_PATCH],
        );
        std::rc::Rc::new(std::cell::RefCell::new(tmuxproc {
            name: Some(name.to_owned()),
            ..tmuxproc::default()
        }))
    }
}
/// Dispatches callbacks between checked borrows of the retained process.
pub fn proc_loop(tp: &ProcessRef, mut should_exit: impl FnMut() -> bool) {
    log_debug(c"%s loop enter", fmt_args![tp.borrow().name.as_deref()]);
    loop {
        reactor::current().run_once();
        let exited = tp.borrow().exit != 0;
        if exited || should_exit() {
            break;
        }
    }
    log_debug(c"%s loop exit", fmt_args![tp.borrow().name.as_deref()]);
}

/// Asks the loop to stop. Whoever owns a peer flushes it first; the process
/// keeps no list of them.
pub fn proc_exit(tp: &mut tmuxproc) {
    tp.exit = 1 as core::ffi::c_int;
}

pub unsafe fn proc_set_signals(tp: &mut tmuxproc, signalcb: impl Fn(core::ffi::c_int) + 'static) {
    unsafe {
        let mut sa: libc::sigaction = core::mem::zeroed();
        tp.signalcb = Some(std::rc::Rc::new(signalcb));
        sigemptyset(&raw mut sa.sa_mask);
        sa.sa_flags = SA_RESTART;
        sa.sa_sigaction = libc::SIG_IGN;
        sigaction(
            SIGPIPE,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTSTP,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTTIN,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTTOU,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGQUIT,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        for (watch, signo) in [
            (&mut tp.ev_sigint, SIGINT),
            (&mut tp.ev_sighup, SIGHUP),
            (&mut tp.ev_sigchld, SIGCHLD),
            (&mut tp.ev_sigcont, SIGCONT),
            (&mut tp.ev_sigterm, SIGTERM),
            (&mut tp.ev_sigusr1, SIGUSR1),
            (&mut tp.ev_sigusr2, SIGUSR2),
            (&mut tp.ev_sigwinch, SIGWINCH),
        ] {
            let callback =
                std::rc::Rc::downgrade(tp.signalcb.as_ref().expect("the signal callback was set"));
            watch.set_callback(signo, move |signo, _| {
                if let Some(callback) = callback.upgrade() {
                    callback(signo);
                }
            });
        }
    }
}
pub unsafe fn proc_clear_signals(tp: &mut tmuxproc, defaults: core::ffi::c_int) {
    unsafe {
        let mut sa: libc::sigaction = core::mem::zeroed();
        sigemptyset(&raw mut sa.sa_mask);
        sa.sa_flags = SA_RESTART;
        sa.sa_sigaction = libc::SIG_DFL;
        sigaction(
            SIGPIPE,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        sigaction(
            SIGTSTP,
            &raw mut sa,
            core::ptr::null_mut::<libc::sigaction>(),
        );
        tp.ev_sigint.unwatch();
        tp.ev_sighup.unwatch();
        tp.ev_sigchld.unwatch();
        tp.ev_sigcont.unwatch();
        tp.ev_sigterm.unwatch();
        tp.ev_sigusr1.unwatch();
        tp.ev_sigusr2.unwatch();
        tp.ev_sigwinch.unwatch();
        if defaults != 0 {
            sigaction(
                SIGINT,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGQUIT,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGHUP,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGCHLD,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGCONT,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGTERM,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGUSR1,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGUSR2,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
            sigaction(
                SIGWINCH,
                &raw mut sa,
                core::ptr::null_mut::<libc::sigaction>(),
            );
        }
    }
}

pub fn proc_toggle_log(tp: &mut tmuxproc) {
    {
        log_toggle(tp.name.as_deref().expect("a process retains its log name"));
    }
}
/// Forks a daemon, answering the child's process id and this end of the
/// socket pair the two halves talk over.
pub unsafe fn proc_fork_and_daemon() -> (pid_t, core::ffi::c_int) {
    unsafe {
        let mut pair: [core::ffi::c_int; 2] = [0; 2];
        if socketpair(
            AF_UNIX,
            SOCK_STREAM as core::ffi::c_int,
            PF_UNSPEC,
            &raw mut pair as *mut core::ffi::c_int,
        ) != 0 as core::ffi::c_int
        {
            fatal(c"socketpair failed", fmt_args![]);
        }
        let pid: pid_t = fork() as pid_t;
        match pid {
            -1 => {
                fatal(c"fork failed", fmt_args![]);
            }
            0 => {
                close(pair[0 as core::ffi::c_int as usize]);
                let fd = pair[1 as core::ffi::c_int as usize];
                if daemon(1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0 as core::ffi::c_int {
                    fatal(c"daemon failed", fmt_args![]);
                }
                (0 as pid_t, fd)
            }
            _ => {
                close(pair[1 as core::ffi::c_int as usize]);
                (pid, pair[0 as core::ffi::c_int as usize])
            }
        }
    }
}

#[cfg(test)]
mod signal_tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn process_loop_releases_borrows_before_reactor_callbacks() {
        let _guard = crate::tests::test_fixtures::globals();
        crate::tests::test_fixtures::ensure_reactor();
        let process = ProcessRef::default();
        let weak = Rc::downgrade(&process);
        let callback_process = process.clone();
        reactor::current().defer(move || {
            proc_exit(&mut callback_process.borrow_mut());
        });
        {
            proc_loop(&process, || panic!("the process already requested exit"));
        }
        assert_eq!(process.borrow().exit, 1);
        drop(process);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn process_loop_releases_borrows_before_exit_predicate() {
        let _guard = crate::tests::test_fixtures::globals();
        crate::tests::test_fixtures::ensure_reactor();
        let process = ProcessRef::default();
        let mut calls = 0;
        {
            proc_loop(&process, || {
                calls += 1;
                proc_exit(&mut process.borrow_mut());
                false
            });
        }
        assert_eq!(calls, 1);
        assert_eq!(process.borrow().exit, 1);
    }

    #[test]
    fn peer_dispatch_can_send_and_remove_its_own_peer() {
        let _guard = crate::tests::test_fixtures::globals();
        crate::tests::test_fixtures::ensure_reactor();
        unsafe {
            let mut fds = [-1; 2];
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0
            );
            let peer = PeerRef::from_fd(fds[0], None);
            let sender = PeerRef::from_fd(fds[1], None);
            let weak = Rc::downgrade(&peer.0);
            let callback_peer = weak.clone();
            let calls = Rc::new(Cell::new(0));
            let observed = calls.clone();
            peer.borrow_mut().dispatchcb = Some(Rc::new(move |message| {
                assert!(message.is_some());
                observed.set(observed.get() + 1);
                let peer = PeerRef(callback_peer.upgrade().unwrap());
                assert_eq!(peer.send(MSG_READY, -1, &[]), 0);
                peer.close();
            }));
            assert_eq!(sender.send(MSG_COMMAND, -1, &[]), 0);
            assert_eq!(sender.send(MSG_COMMAND, -1, &[]), 0);
            sender.flush();
            sender.borrow_mut().event.disable();
            for _ in 0..8 {
                reactor::current().run_once();
                if calls.get() == 1 {
                    break;
                }
            }
            assert_eq!(calls.get(), 1);
            assert_eq!(peer.borrow().ibuf.fd, -1);
            assert_eq!(peer.send(MSG_READY, -1, &[]), -1);
            peer.on_event(EV_READ as _);
            assert_eq!(calls.get(), 1);
            sender.close();
            drop(peer);
            assert!(weak.upgrade().is_none());
        }
    }

    #[test]
    fn file_transfer_retains_a_closed_peer_until_its_last_owner_is_dropped() {
        let _guard = crate::tests::test_fixtures::globals();
        crate::tests::test_fixtures::ensure_reactor();
        unsafe {
            let mut fds = [-1; 2];
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0
            );
            let peer = PeerRef::from_fd(fds[0], None);
            let weak = Rc::downgrade(&peer.0);
            let files = Rc::new(std::cell::RefCell::new(client_files_t::new()));
            let file = ClientFileRef::create_with_peer(
                Some(peer.clone()),
                &FileOwner::Shared(Rc::downgrade(&files)),
                1,
                None,
                ClientFileData::None,
            );
            peer.close();
            assert!(weak.upgrade().is_some());
            {
                let file = file.borrow();
                let peer = file.peer.as_ref().unwrap();
                assert_eq!(peer.borrow().ibuf.fd, -1);
                assert_eq!(peer.send(MSG_READ, -1, &[]), -1);
            }
            file.close();
            assert!(weak.upgrade().is_none());
            close(fds[1]);
        }
    }

    fn deliver(calls: &Cell<usize>, expected: usize) {
        for _ in 0..2 {
            reactor::current().run_once();
        }
        assert_eq!(unsafe { libc::raise(SIGUSR1) }, 0);
        for _ in 0..8 {
            reactor::current().run_once();
            if calls.get() == expected {
                return;
            }
        }
        assert_eq!(calls.get(), expected);
    }

    #[test]
    fn signal_watches_follow_callback_ownership_across_process_moves_and_replacement() {
        let _guard = crate::tests::test_fixtures::globals();
        crate::tests::test_fixtures::ensure_reactor();
        let mut process = Box::new(tmuxproc::default());
        let old_calls = Rc::new(Cell::new(0));
        let observed = old_calls.clone();
        unsafe {
            proc_set_signals(&mut process, move |signo| {
                if signo == SIGUSR1 {
                    observed.set(observed.get() + 1);
                }
            })
        };
        let old_callback = Rc::downgrade(process.signalcb.as_ref().unwrap());
        let mut moved = *process;
        deliver(&old_calls, 1);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        unsafe {
            proc_set_signals(&mut moved, move |signo| {
                if signo == SIGUSR1 {
                    observed.set(observed.get() + 1);
                }
            })
        };
        assert!(old_callback.upgrade().is_none());
        deliver(&calls, 1);
        assert_eq!(old_calls.get(), 1);
        let callback = Rc::downgrade(moved.signalcb.as_ref().unwrap());
        let mut watches = [
            moved.ev_sigint,
            moved.ev_sighup,
            moved.ev_sigchld,
            moved.ev_sigcont,
            moved.ev_sigterm,
            moved.ev_sigusr1,
            moved.ev_sigusr2,
            moved.ev_sigwinch,
        ];
        drop(moved);
        assert!(callback.upgrade().is_none());
        assert_eq!(unsafe { libc::raise(SIGUSR1) }, 0);
        for _ in 0..2 {
            reactor::current().run_once();
        }
        assert_eq!(calls.get(), 1);
        for watch in &mut watches {
            watch.unwatch();
        }
    }
}

impl tmuxpeer {
    unsafe fn check_version(&mut self, imsg: &imsg) -> core::ffi::c_int {
        let peer = self;
        unsafe {
            let version: core::ffi::c_int =
                (imsg.hdr.peerid & 0xff as uint32_t) as core::ffi::c_int;
            if imsg.hdr.type_0 != MSG_VERSION as core::ffi::c_int as uint32_t
                && version != PROTOCOL_VERSION
            {
                log_debug(
                    c"peer %p bad version %d",
                    fmt_args![peer as *mut tmuxpeer, version],
                );
                peer.send(MSG_VERSION, -(1 as core::ffi::c_int), &[]);
                peer.flags |= PEER_BAD;
                return -(1 as core::ffi::c_int);
            }
            0 as core::ffi::c_int
        }
    }
    unsafe fn update_event(&mut self) {
        let peer = self;
        unsafe {
            let mut interest: Interest = Interest::Read;
            peer.event.disable();
            if imsgbuf_queuelen(&peer.ibuf) > 0 as uint32_t {
                interest = Interest::ReadWrite;
            }
            let owner = peer.owner.as_ref().expect("a peer has an owner").clone();
            peer.event
                .set_callback(peer.ibuf.fd, interest, WatchMode::Once, move |_, events| {
                    if let Some(owner) = owner.upgrade() {
                        (PeerRef(owner)).on_event(events);
                    }
                });
            peer.event.enable();
        }
    }
    pub unsafe fn send(
        &mut self,
        type_0: msgtype,
        fd: core::ffi::c_int,
        buf: &[u8],
    ) -> core::ffi::c_int {
        let peer = self;
        unsafe {
            let peer_ptr = peer as *mut tmuxpeer;
            let ibuf: &mut imsgbuf = &mut peer.ibuf;

            if peer.flags & PEER_BAD != 0 {
                return -(1 as core::ffi::c_int);
            }
            log_debug(
                c"sending message %d to peer %p (%zu bytes)",
                fmt_args![type_0 as core::ffi::c_uint, peer_ptr, buf.len()],
            );
            let retval: core::ffi::c_int = imsg_compose(
                &mut *ibuf,
                type_0 as uint32_t,
                PROTOCOL_VERSION as uint32_t,
                -(1 as pid_t),
                fd,
                buf,
            );
            if retval != 1 as core::ffi::c_int {
                return -(1 as core::ffi::c_int);
            }
            peer.update_event();
            0 as core::ffi::c_int
        }
    }
    pub fn mark_bad(&mut self) {
        let peer = self;
        {
            peer.flags |= PEER_BAD;
        }
    }
    pub unsafe fn flush(&mut self) {
        let peer = self;
        unsafe {
            imsgbuf_flush(&mut peer.ibuf);
        }
    }
    pub fn uid(&self) -> uid_t {
        let peer = self;
        peer.uid
    }
}
impl PeerRef {
    unsafe fn on_event(&self, events: core::ffi::c_short) {
        let owner = self;
        unsafe {
            let dispatch = owner.borrow().dispatchcb.clone();
            let mut peer = owner.borrow_mut();
            if peer.ibuf.fd == -1 {
                return;
            }
            if peer.flags & PEER_BAD == 0 && events as core::ffi::c_int & EV_READ != 0 {
                if imsgbuf_read(&mut peer.ibuf) != 1 {
                    drop(peer);
                    dispatch.as_ref().expect("non-null dispatch callback")(None);
                    return;
                }
                loop {
                    let mut message = match imsg_get(&mut peer.ibuf) {
                        Ok(Some(message)) => message.0,
                        Ok(None) => break,
                        Err(_) => {
                            drop(peer);
                            dispatch.as_ref().expect("non-null dispatch callback")(None);
                            return;
                        }
                    };
                    log_debug(
                        c"peer %p message %d",
                        fmt_args![core::ptr::from_ref(&*peer), message.hdr.type_0],
                    );
                    if peer.check_version(&message) != 0 {
                        imsg_free(message);
                        break;
                    }
                    drop(peer);
                    dispatch.as_ref().expect("non-null dispatch callback")(Some(&mut message));
                    imsg_free(message);
                    peer = owner.borrow_mut();
                    if peer.ibuf.fd == -1 {
                        return;
                    }
                }
            }
            if events as core::ffi::c_int & EV_WRITE != 0 && imsgbuf_write(&mut peer.ibuf) == -1 {
                drop(peer);
                dispatch.as_ref().expect("non-null dispatch callback")(None);
                return;
            }
            if peer.flags & PEER_BAD != 0 && imsgbuf_queuelen(&peer.ibuf) == 0 {
                drop(peer);
                dispatch.as_ref().expect("non-null dispatch callback")(None);
                return;
            }
            peer.update_event();
        }
    }
    pub unsafe fn from_fd(
        fd: core::ffi::c_int,
        dispatchcb: Option<std::rc::Rc<PeerDispatch>>,
    ) -> PeerRef {
        unsafe {
            let mut gid: gid_t = 0;
            let owner = PeerRef::new(tmuxpeer {
                owner: None,
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
            });
            let mut peer = owner.borrow_mut();
            if imsgbuf_init(&mut peer.ibuf, fd) == -(1 as core::ffi::c_int) {
                fatal(c"imsgbuf_init", fmt_args![]);
            }
            imsgbuf_allow_fdpass(&mut peer.ibuf);
            if getpeereid(fd, &mut peer.uid, &mut gid) != 0 as core::ffi::c_int {
                peer.uid = -(1 as core::ffi::c_int) as uid_t;
            }
            log_debug(
                c"add peer %p: %d",
                fmt_args![core::ptr::from_ref(&*peer), fd],
            );
            peer.update_event();
            drop(peer);
            owner
        }
    }
    pub unsafe fn close(self) {
        let owner = self;
        unsafe {
            let mut peer = owner.borrow_mut();
            let peer_ptr = core::ptr::from_ref(&*peer);
            log_debug(c"remove peer %p", fmt_args![peer_ptr]);
            peer.event.disable();
            imsgbuf_clear(&mut peer.ibuf);
            close(peer.ibuf.fd);
            peer.ibuf.fd = -1;
            peer.flags |= PEER_BAD;
        }
    }
    /// Sends a protocol message through the peer.
    ///
    /// # Safety
    /// A nonnegative descriptor must be valid for the message's descriptor transfer.
    pub unsafe fn send(
        &self,
        message: msgtype,
        fd: core::ffi::c_int,
        bytes: &[u8],
    ) -> core::ffi::c_int {
        unsafe { self.borrow_mut().send(message, fd, bytes) }
    }

    pub fn mark_bad(&self) {
        self.borrow_mut().mark_bad();
    }

    pub unsafe fn flush(&self) {
        unsafe { self.borrow_mut().flush() };
    }

    pub fn uid(&self) -> uid_t {
        self.borrow().uid()
    }
}
