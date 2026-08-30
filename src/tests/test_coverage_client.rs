//! Unit tests for [`crate::client`] — the client half of the protocol:
//! the constants the wire behaviour hangs on, the reason-to-text mapping for
//! exits, the MSG_EXIT payload parser, the lock-file handshake behind
//! `client_get_lock`, the refusal paths of `client_connect`, and the message
//! router (`client_dispatch`) with both of its arms — waiting and attached —
//! plus the signal handler that feeds them.
//!
//! Replies are read back off a real peer: each test wires a zeroed process
//! and peer whose message buffer sits on one end of a socket pair, installs
//! them into the client statics, and afterwards drains what was queued for
//! the server through the far end. Every assertion about "the client told
//! the server X" therefore reads an actually composed message.
//!
//! Paths that would end the test process or reach outside it are left alone,
//! deliberately: the fatal size-validation arms (they abort), MSG_SHELL and
//! `client_exec` (they execl), MSG_SUSPEND (it stops this process with
//! SIGTSTP), MSG_LOCK (it runs a shell command), SIGCHLD handling (reaping
//! children is shared process state), the contended half of
//! `client_get_lock` (it blocks until the holder lets go), the server-start
//! halves of `client_connect` and `client_main` (they spawn or need a live
//! daemon), and the file-transfer messages (they want descriptors handed
//! over by a live exchange; even the cancel arm aborts on an unknown
//! stream). The statics these functions share are reset under one lock by
//! every test that touches them.

use crate::client::{
    CLIENT_CONTROL, CLIENT_CONTROL_WAITEXIT, CLIENT_CONTROLCONTROL, CLIENT_EXIT_DETACH,
    CLIENT_EXIT_DETACHED, CLIENT_EXIT_DETACHED_HUP, CLIENT_EXIT_EXITED, CLIENT_EXIT_LOST_SERVER,
    CLIENT_EXIT_MESSAGE_PROVIDED, CLIENT_EXIT_NONE, CLIENT_EXIT_RETURN, CLIENT_EXIT_SERVER_EXITED,
    CLIENT_EXIT_SHUTDOWN, CLIENT_EXIT_TERMINATED, CLIENT_LOGIN, CLIENT_NOSTARTSERVER,
    CLIENT_STARTSERVER, CMD_STARTSERVER, EAGAIN, ECONNREFUSED, EINTR, ENAMETOOLONG, ENOENT,
    IMSG_HEADER_SIZE, LOCK_EX, LOCK_NB, MAX_IMSGSIZE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL,
    MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_LOCK, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_VERSION, MSG_WAKEUP, O_CREAT, O_WRONLY,
    PROTOCOL_VERSION, client_attached, client_connect, client_dispatch,
    client_dispatch_exit_message, client_execcmd, client_execshell, client_exit,
    client_exit_message, client_exitflag, client_exitmessage, client_exitreason,
    client_exitsession, client_exittype, client_exitval, client_file_check_cb, client_files,
    client_flags, client_get_lock, client_peer, client_proc, client_signal, client_suspended,
};
use crate::compat::{
    imsg_free, imsg_get, imsg_get_type, imsgbuf_allow_fdpass, imsgbuf_clear, imsgbuf_flush,
    imsgbuf_init, imsgbuf_queuelen, imsgbuf_read,
};
use crate::ffi::{__errno_location, close, sigaction, socketpair};
use crate::reactor;
use crate::reactor::{Interest, IoWatch, WatchMode};
use crate::tests::test_fixtures::{ensure_reactor, seen, zeroed};
use crate::types::*;
use ::core::ffi::{c_char, c_int, c_short, c_void};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;
use ::std::sync::{Mutex, MutexGuard};

/// The handler the fixture peer's event is bound with. The fixture never runs
/// the event loop, so nothing ever reaches it.
unsafe fn never(_fd: c_int, _events: c_short, _arg: *mut c_void) {}

/// A turn at the client's process-wide statics **and** at the rest of the
/// crate's shared state. Cargo runs tests on parallel threads; everything in
/// [`crate::client`] shares these statics, and the fixture peer's events
/// live on the same process-wide ensure_reactor base every other suite uses, so
/// each test that reaches any of it holds this guard.
fn turn() -> (MutexGuard<'static, ()>, MutexGuard<'static, ()>) {
    static TURN: Mutex<()> = Mutex::new(());
    (
        TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner()),
        crate::tests::test_fixtures::globals(),
    )
}

/// Puts every client static back the way a fresh process would find it,
/// freeing the strings earlier tests left behind.
unsafe fn reset() {
    unsafe {
        client_exitmessage = None;
        client_exitsession = None;
        client_execshell = None;
        client_execcmd = None;
        client_proc = null_mut();
        client_peer = null_mut();
        client_flags = 0;
        client_suspended = 0;
        client_attached = 0;
        client_exitflag = 0;
        client_exitval = 0;
        client_exittype = 0;
        client_exitreason = CLIENT_EXIT_NONE;
        client_files.map().clear();
    }
}

unsafe fn client_string_ptr(value: *const Option<CString>) -> *const c_char {
    unsafe { (*value).as_ref().map_or(null(), |value| value.as_ptr()) }
}

/// A client process with one peer to talk to. The process is zeroed apart
/// from its empty peer list — enough for `proc_exit`, which only walks the
/// list and sets the exit flag — and the peer is zeroed apart from a message
/// buffer on one end of a socket pair, which is all `proc_send` needs to
/// compose into. Both are installed into the client statics; dropping the
/// harness takes them out again before its buffers go away.
struct Harness {
    pr: Box<tmuxproc>,
    peer: Box<tmuxpeer>,
    far: Box<imsgbuf>,
    fds: [c_int; 2],
}

impl Harness {
    fn new() -> Harness {
        ensure_reactor();
        let mut fds: [c_int; 2] = [-1, -1];
        unsafe {
            assert_eq!(
                socketpair(
                    crate::client::AF_UNIX,
                    crate::client::SOCK_STREAM as c_int,
                    0,
                    fds.as_mut_ptr()
                ),
                0
            );
        }
        let mut h = Harness {
            pr: Box::new(tmuxproc::default()),
            peer: zeroed::<tmuxpeer>(),
            far: zeroed::<imsgbuf>(),
            fds,
        };
        unsafe {
            assert_eq!(imsgbuf_init(&mut h.peer.ibuf, h.fds[0]), 0);
            imsgbuf_allow_fdpass(&mut h.peer.ibuf);
            assert_eq!(imsgbuf_init(&mut h.far, h.fds[1]), 0);
            h.peer.event.set_callback(
                h.fds[0],
                Interest::Read,
                WatchMode::Once,
                move |fd, events| never(fd, events, null_mut()),
            );
            client_proc = &raw mut *h.pr;
            client_peer = &raw mut *h.peer;
        }
        h
    }

    fn pr(&mut self) -> *mut tmuxproc {
        &raw mut *self.pr
    }

    /// Everything the client has queued for the server since last asked, by
    /// message type. Queued messages are flushed over the socket pair, read
    /// at the far end, and taken off it; an empty queue touches neither end.
    fn sent(&mut self) -> Vec<uint32_t> {
        unsafe {
            let mut out = Vec::new();
            while imsgbuf_queuelen(&mut self.peer.ibuf) > 0 {
                assert_eq!(imsgbuf_flush(&mut self.peer.ibuf), 0);
                let rv = imsgbuf_read(&mut self.far);
                assert_eq!(rv, 1, "imsgbuf_read answered {rv}");
            }
            loop {
                let mut m = Box::new(imsg::default());
                match imsg_get(&mut self.far, &raw mut *m) {
                    0 => break,
                    len if len > 0 => out.push(imsg_get_type(&raw mut *m)),
                    other => panic!("imsg_get answered {other}"),
                }
                imsg_free(&raw mut *m);
            }
            out
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        unsafe {
            client_proc = null_mut();
            client_peer = null_mut();
            self.peer.event.disable();
            imsgbuf_clear(&mut self.peer.ibuf);
            imsgbuf_clear(&mut self.far);
            close(self.fds[0]);
            close(self.fds[1]);
        }
    }
}

/// A message of type `ty` carrying `payload`, shaped as dispatch reads one:
/// only the header type and length and the data pointer are filled in.
unsafe fn incoming(ty: uint32_t, payload: &[u8]) -> Box<imsg> {
    let mut m = Box::new(imsg::default());
    m.hdr.type_0 = ty;
    m.hdr.len = (IMSG_HEADER_SIZE + payload.len()) as uint32_t;
    m.data = payload.as_ptr() as *mut u8;
    m
}

/// Hands the server's message of type `ty` carrying `payload` to the router.
unsafe fn deliver(ty: uint32_t, payload: &[u8]) {
    unsafe {
        let mut m = incoming(ty, payload);
        client_dispatch(&raw mut *m, null_mut());
    }
}

/// An eight-byte MSG_FLAGS payload.
fn flags_payload(flags: u64) -> Vec<u8> {
    flags.to_le_bytes().to_vec()
}

/// A MSG_EXIT payload: an exit value, then the message text including its
/// terminating NUL when there is one.
fn exit_payload(retval: c_int, message: &[u8]) -> Vec<u8> {
    let mut v = retval.to_ne_bytes().to_vec();
    v.extend_from_slice(message);
    v
}

/// Copies of the client's bookkeeping, taken under the caller's turn so that
/// assertions never hold a reference into the statics themselves.
unsafe fn exit_value() -> c_int {
    unsafe { client_exitval }
}

/// The reason currently recorded for leaving.
unsafe fn exit_reason() -> uint32_t {
    unsafe { client_exitreason }
}

/// The message type the next exec or detach will act on.
unsafe fn exit_type() -> uint32_t {
    unsafe { client_exittype }
}

/// Whether an exit was already asked for.
unsafe fn exit_asked() -> c_int {
    unsafe { client_exitflag }
}

/// The flags the server last sent.
unsafe fn flags_now() -> uint64_t {
    unsafe { client_flags }
}

/// Whether the client considers itself attached.
unsafe fn attached_now() -> c_int {
    unsafe { client_attached }
}

/// Whether the client considers itself suspended.
unsafe fn suspended_now() -> c_int {
    unsafe { client_suspended }
}

#[test]
fn session_message_types_match_upstream() {
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_DETACHKILL, 202);
    assert_eq!(MSG_EXIT, 203);
    assert_eq!(MSG_EXITED, 204);
    assert_eq!(MSG_EXITING, 205);
    assert_eq!(MSG_LOCK, 206);
    assert_eq!(MSG_READY, 207);
    assert_eq!(MSG_RESIZE, 208);
    assert_eq!(MSG_SHELL, 209);
    assert_eq!(MSG_SHUTDOWN, 210);
    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_FLAGS, 218);
    assert_eq!(MSG_WAKEUP, 216);
    assert_eq!(MSG_SUSPEND, 214);
    assert_eq!(MSG_EXEC, 217);
}

/// The identify block rides below the session block, the file block above
/// it, so no type can be mistaken for another on the wire.
#[test]
fn identify_and_file_message_types_do_not_overlap_the_session_block() {
    use crate::client::{
        MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
        MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
        MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
        MSG_IDENTIFY_TTYNAME, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_WRITE,
        MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY,
    };
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_TTYNAME, 102);
    assert_eq!(MSG_IDENTIFY_OLDCWD, 103);
    assert_eq!(MSG_IDENTIFY_STDIN, 104);
    assert_eq!(MSG_IDENTIFY_ENVIRON, 105);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    assert_eq!(MSG_IDENTIFY_CLIENTPID, 107);
    assert_eq!(MSG_IDENTIFY_CWD, 108);
    assert_eq!(MSG_IDENTIFY_FEATURES, 109);
    assert_eq!(MSG_IDENTIFY_STDOUT, 110);
    assert_eq!(MSG_IDENTIFY_LONGFLAGS, 111);
    assert_eq!(MSG_IDENTIFY_TERMINFO, 112);

    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_READY, 305);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert_eq!(MSG_READ_CANCEL, 307);

    assert_eq!(PROTOCOL_VERSION, 8);
    assert_eq!(MAX_IMSGSIZE, 16384);
}

/// The flag bits a command line or the server can set, all distinct.
#[test]
fn client_flag_bits_match_upstream_and_stay_distinct() {
    assert_eq!(CMD_STARTSERVER, 0x1);
    assert_eq!(CLIENT_LOGIN, 0x2);
    assert_eq!(CLIENT_NOSTARTSERVER, 0x1000);
    assert_eq!(CLIENT_CONTROL, 0x2000);
    assert_eq!(CLIENT_CONTROLCONTROL, 0x4000);
    assert_eq!(CLIENT_STARTSERVER, 0x10000000);
    assert_eq!(CLIENT_CONTROL_WAITEXIT, 0x200000000u64);

    let bits = [
        CMD_STARTSERVER as u64,
        CLIENT_LOGIN as u64,
        CLIENT_NOSTARTSERVER as u64,
        CLIENT_CONTROL as u64,
        CLIENT_CONTROLCONTROL as u64,
        CLIENT_STARTSERVER as u64,
        CLIENT_CONTROL_WAITEXIT,
    ];
    for i in 0..bits.len() {
        for j in (i + 1)..bits.len() {
            assert_ne!(bits[i], bits[j]);
        }
    }
}

/// The exit reasons the server can report and the exit types the client acts
/// on, together with the errno and flock constants behind the connect and
/// lock paths.
#[test]
fn exit_reason_type_and_connect_constants_match_upstream() {
    use crate::client::{AF_UNIX, PF_UNIX};
    assert_eq!(CLIENT_EXIT_NONE, 0);
    assert_eq!(CLIENT_EXIT_DETACHED, 1);
    assert_eq!(CLIENT_EXIT_DETACHED_HUP, 2);
    assert_eq!(crate::client::CLIENT_EXIT_LOST_TTY, 3);
    assert_eq!(CLIENT_EXIT_TERMINATED, 4);
    assert_eq!(CLIENT_EXIT_LOST_SERVER, 5);
    assert_eq!(CLIENT_EXIT_EXITED, 6);
    assert_eq!(CLIENT_EXIT_SERVER_EXITED, 7);
    assert_eq!(CLIENT_EXIT_MESSAGE_PROVIDED, 8);

    assert_eq!(CLIENT_EXIT_RETURN, 0);
    assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
    assert_eq!(CLIENT_EXIT_DETACH, 2);

    assert_eq!(ENOENT, 2);
    assert_eq!(EINTR, 4);
    assert_eq!(EAGAIN, 11);
    assert_eq!(ENAMETOOLONG, 36);
    assert_eq!(ECONNREFUSED, 111);
    assert_eq!(O_WRONLY, 1);
    assert_eq!(O_CREAT, 0o100);
    assert_eq!(LOCK_EX, 2);
    assert_eq!(LOCK_NB, 4);
    assert_eq!(AF_UNIX, PF_UNIX);
    assert_ne!(MSG_VERSION, MSG_COMMAND);
}

/// Every reason names itself, and anything else comes back as unknown. With
/// no session recorded, the detached reasons do not name one.
#[test]
fn every_exit_reason_names_itself() {
    let _t = turn();
    unsafe {
        reset();

        struct Case {
            reason: uint32_t,
            want: &'static [u8],
        }
        let cases = [
            Case {
                reason: CLIENT_EXIT_DETACHED,
                want: b"detached",
            },
            Case {
                reason: CLIENT_EXIT_DETACHED_HUP,
                want: b"detached and SIGHUP",
            },
            Case {
                reason: crate::client::CLIENT_EXIT_LOST_TTY,
                want: b"lost tty",
            },
            Case {
                reason: CLIENT_EXIT_TERMINATED,
                want: b"terminated",
            },
            Case {
                reason: CLIENT_EXIT_LOST_SERVER,
                want: b"server exited unexpectedly",
            },
            Case {
                reason: CLIENT_EXIT_EXITED,
                want: b"exited",
            },
            Case {
                reason: CLIENT_EXIT_SERVER_EXITED,
                want: b"server exited",
            },
            Case {
                reason: CLIENT_EXIT_NONE,
                want: b"unknown reason",
            },
            Case {
                reason: 99,
                want: b"unknown reason",
            },
        ];
        for case in cases {
            client_exitreason = case.reason;
            assert_eq!(
                client_exit_message().to_bytes(),
                case.want,
                "reason {}",
                case.reason
            );
        }

        client_exitreason = CLIENT_EXIT_NONE;
    }
}

/// A detach that names its session says where it came from, both plain and
/// with the SIGHUP that follows a killed detachment.
#[test]
fn a_detached_client_names_the_session_it_left() {
    let _t = turn();
    unsafe {
        reset();
        let held = CString::new("work").unwrap();

        client_exitreason = CLIENT_EXIT_DETACHED;
        client_exitsession = Some(held);
        assert_eq!(
            client_exit_message().to_bytes(),
            b"detached (from session work)"
        );

        client_exitreason = CLIENT_EXIT_DETACHED_HUP;
        assert_eq!(
            client_exit_message().to_bytes(),
            b"detached and SIGHUP (from session work)"
        );

        client_exitsession = None;
        client_exitreason = CLIENT_EXIT_NONE;
    }
}

/// MESSAGE_PROVIDED hands back the stored message itself rather than one of
/// the canned strings.
#[test]
fn a_provided_exit_message_is_returned_verbatim() {
    let _t = turn();
    unsafe {
        reset();
        client_exitreason = CLIENT_EXIT_MESSAGE_PROVIDED;
        client_exitmessage = Some(CString::new("server is going down").unwrap());
        assert_eq!(client_exit_message().to_bytes(), b"server is going down");

        client_exitmessage = None;
        client_exitreason = CLIENT_EXIT_NONE;
    }
}

/// An empty MSG_EXIT payload changes nothing at all.
#[test]
fn an_empty_msg_exit_payload_changes_nothing() {
    let _t = turn();
    unsafe {
        reset();
        client_exitval = 4;
        client_exitreason = CLIENT_EXIT_DETACHED;

        client_dispatch_exit_message(null_mut(), 0);

        assert_eq!(exit_value(), 4);
        assert_eq!(exit_reason(), CLIENT_EXIT_DETACHED);
        assert!(client_exitmessage.is_none());

        client_exitreason = CLIENT_EXIT_NONE;
        client_exitval = 0;
    }
}

/// A four-byte payload carries just the exit value: the reason and any
/// stored message stay alone.
#[test]
fn a_bare_msg_exit_sets_only_the_exit_value() {
    let _t = turn();
    unsafe {
        reset();
        let payload = exit_payload(-7, b"");

        client_dispatch_exit_message(payload.as_ptr() as *mut c_char, payload.len());

        assert_eq!(exit_value(), -7);
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);
        assert!(client_exitmessage.is_none());
    }
}

/// A payload with text past the value records both: the text is copied out
/// NUL-terminated and the reason becomes MESSAGE_PROVIDED.
#[test]
fn a_msg_exit_with_a_message_carries_both() {
    let _t = turn();
    unsafe {
        reset();
        let payload = exit_payload(9, b"done\0");

        client_dispatch_exit_message(payload.as_ptr() as *mut c_char, payload.len());

        assert_eq!(exit_value(), 9);
        assert_eq!(exit_reason(), CLIENT_EXIT_MESSAGE_PROVIDED);
        assert_eq!(
            seen(client_string_ptr(&raw const client_exitmessage)),
            "done"
        );

        client_exitmessage = None;
        client_exitreason = CLIENT_EXIT_NONE;
    }
}

/// A message whose bytes carry a NUL of their own stops there, the way the
/// C string the payload stands for ends at its first NUL.
#[test]
fn a_msg_exit_message_ends_at_its_first_nul() {
    let _t = turn();
    unsafe {
        reset();
        let payload = exit_payload(1, b"stopped\0trailing\0");

        client_dispatch_exit_message(payload.as_ptr() as *mut c_char, payload.len());

        assert_eq!(exit_reason(), CLIENT_EXIT_MESSAGE_PROVIDED);
        assert_eq!(
            seen(client_string_ptr(&raw const client_exitmessage)),
            "stopped"
        );

        client_exitmessage = None;
        client_exitreason = CLIENT_EXIT_NONE;
        client_exitval = 0;
    }
}

/// A message with no NUL at all loses its last byte, which is where the
/// terminator would have gone.
#[test]
fn an_unterminated_msg_exit_message_drops_its_last_byte() {
    let _t = turn();
    unsafe {
        reset();
        let payload = exit_payload(1, b"cut");

        client_dispatch_exit_message(payload.as_ptr() as *mut c_char, payload.len());

        assert_eq!(exit_reason(), CLIENT_EXIT_MESSAGE_PROVIDED);
        assert_eq!(seen(client_string_ptr(&raw const client_exitmessage)), "cu");

        client_exitmessage = None;
        client_exitreason = CLIENT_EXIT_NONE;
        client_exitval = 0;
    }
}

/// The lock file of a fresh socket path is created mode-less but present,
/// and the descriptor handed back holds it.
#[test]
fn a_fresh_lock_file_is_created_and_held() {
    let path = std::env::temp_dir().join(format!("tmux-c2rs-client-lock-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let cpath = CString::new(path.to_str().unwrap()).unwrap();

    let fd = unsafe { client_get_lock(cpath.as_ptr() as *mut c_char) };
    assert!(fd >= 0, "no lock descriptor");
    assert!(path.exists(), "the lock file was not created");

    unsafe { close(fd) };
    std::fs::remove_file(&path).unwrap();
}

/// A lock file that cannot even be opened — here because its directory does
/// not exist — is refused with -1.
#[test]
fn an_unopenable_lock_file_is_refused() {
    let path = std::env::temp_dir()
        .join(format!("tmux-c2rs-client-missing-{}", std::process::id()))
        .join("lock");
    let cpath = CString::new(path.to_str().unwrap()).unwrap();

    unsafe {
        *__errno_location() = 0;
        assert_eq!(client_get_lock(cpath.as_ptr() as *mut c_char), -1);
        assert_eq!(*__errno_location(), ENOENT);
    }
}

/// A socket path too long for sockaddr_un is refused before any socket is
/// even created.
#[test]
fn an_over_long_socket_path_is_refused() {
    let long = CString::new("a".repeat(200)).unwrap();
    unsafe {
        *__errno_location() = 0;
        assert_eq!(
            client_connect(
                reactor::current(),
                long.as_ptr(),
                CLIENT_NOSTARTSERVER as uint64_t
            ),
            -1
        );
        assert_eq!(*__errno_location(), ENAMETOOLONG);
    }
}

/// With no server allowed to be started, connecting to a path with nothing
/// listening is refused, and the errno says why. A missing socket reports
/// ENOENT.
#[test]
fn a_missing_socket_is_refused_when_no_server_may_be_started() {
    let path = std::env::temp_dir().join("tmux-c2rs-client-no-such.sock");
    let _ = std::fs::remove_file(&path);
    let cpath = CString::new(path.to_str().unwrap()).unwrap();
    unsafe {
        *__errno_location() = 0;
        assert_eq!(
            client_connect(
                reactor::current(),
                cpath.as_ptr(),
                CLIENT_NOSTARTSERVER as uint64_t
            ),
            -1
        );
        assert_eq!(*__errno_location(), ENOENT);
    }
}

/// Same refusal for a path that exists but has nothing listening behind it:
/// the connection is refused, and that is what the caller gets told.
#[test]
fn a_deaf_socket_path_reports_connection_refused() {
    let path = std::env::temp_dir().join(format!("tmux-c2rs-client-deaf-{}", std::process::id()));
    std::fs::File::create(&path).unwrap();
    let cpath = CString::new(path.to_str().unwrap()).unwrap();
    unsafe {
        *__errno_location() = 0;
        assert_eq!(
            client_connect(
                reactor::current(),
                cpath.as_ptr(),
                CLIENT_NOSTARTSERVER as uint64_t
            ),
            -1
        );
        assert_eq!(*__errno_location(), ECONNREFUSED);
    }
    std::fs::remove_file(&path).unwrap();
}

/// Without either start-server bit the client never tries to bring a server
/// up: a refused connection ends the attempt with -1.
#[test]
fn a_refused_socket_without_start_server_flags_is_not_retried() {
    let path = std::env::temp_dir().join(format!("tmux-c2rs-client-noflag-{}", std::process::id()));
    std::fs::File::create(&path).unwrap();
    let cpath = CString::new(path.to_str().unwrap()).unwrap();
    unsafe {
        assert_eq!(
            client_connect(
                reactor::current(),
                cpath.as_ptr(),
                (CLIENT_LOGIN | CLIENT_CONTROL) as uint64_t
            ),
            -1
        );
    }
    std::fs::remove_file(&path).unwrap();
}

/// Losing the server while waiting — the null-message call the event loop
/// makes when the peer goes away — reports LOST_SERVER and asks the process
/// to end.
#[test]
fn losing_the_server_ends_an_unattached_wait() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        client_dispatch(null_mut(), null_mut());

        assert_eq!(exit_reason(), CLIENT_EXIT_LOST_SERVER);
        assert_eq!(exit_value(), 1);
        assert_eq!((*h.pr()).exit, 1);
        assert!(h.sent().is_empty());
    }
}

/// Losing the server after an exit was already asked leaves the first
/// reason standing.
#[test]
fn a_reported_loss_does_not_overwrite_an_earlier_reason() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_exitflag = 1;
        client_exitreason = CLIENT_EXIT_MESSAGE_PROVIDED;

        client_dispatch(null_mut(), null_mut());

        assert_eq!(exit_reason(), CLIENT_EXIT_MESSAGE_PROVIDED);
        assert_eq!((*h.pr()).exit, 1);

        client_exitreason = CLIENT_EXIT_NONE;
        client_exitflag = 0;
    }
}

/// New flags from the server replace the client's own, byte for byte.
#[test]
fn the_wait_state_tracks_new_flags_from_the_server() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_FLAGS as uint32_t, &flags_payload(0x123456789abcdef0));

        assert_eq!(flags_now(), 0x123456789abcdef0);
        assert!(h.sent().is_empty());
    }
}

/// While waiting, a detach request is answered by announcing the exit —
/// nothing is recorded yet, which is what tells the two dispatch arms apart.
#[test]
fn the_wait_state_answers_a_detach_by_leaving() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_DETACH as uint32_t, b"ignored");
        deliver(MSG_DETACHKILL as uint32_t, b"ignored");

        assert_eq!(h.sent(), [MSG_EXITING as uint32_t, MSG_EXITING as uint32_t]);
        assert!(client_string_ptr(&raw const client_exitsession).is_null());
        assert_eq!(exit_type(), 0);
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);
    }
}

/// Readiness attaches the client and answers with a resize, since the
/// terminal may have changed while it waited.
#[test]
fn msg_readiness_attaches_the_client() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_READY as uint32_t, b"");

        assert_eq!(attached_now(), 1);
        assert_eq!(h.sent(), [MSG_RESIZE as uint32_t]);

        client_attached = 0;
    }
}

/// An empty MSG_EXIT from the server asks the wait state to finish: the
/// exit flag goes up, and with no files still writing the process follows.
#[test]
fn an_empty_msg_exit_ends_the_wait() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_EXIT as uint32_t, b"");

        assert_eq!(exit_asked(), 1);
        assert_eq!(exit_value(), 0);
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);
        assert_eq!((*h.pr()).exit, 1);

        client_exitflag = 0;
    }
}

/// A MSG_EXIT that carries text records the value and the message before
/// finishing, exactly as the bare parser leaves them.
#[test]
fn a_msg_exit_with_text_records_both_before_finishing() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        let payload = exit_payload(3, b"shutting down\0");

        deliver(MSG_EXIT as uint32_t, &payload);

        assert_eq!(exit_value(), 3);
        assert_eq!(
            seen(client_string_ptr(&raw const client_exitmessage)),
            "shutting down"
        );
        assert_eq!(exit_reason(), CLIENT_EXIT_MESSAGE_PROVIDED);
        assert_eq!(exit_asked(), 1);
        assert_eq!((*h.pr()).exit, 1);

        client_exitmessage = None;
        client_exitreason = CLIENT_EXIT_NONE;
        client_exitflag = 0;
    }
}

/// Shutdown reaches the wait state through the same arm as exit: a bare
/// value-only payload sets the value, raises the flag and ends the process.
#[test]
fn msg_shutdown_walks_the_same_wait_path_as_msg_exit() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_SHUTDOWN as uint32_t, &exit_payload(2, b""));

        assert_eq!(exit_value(), 2);
        assert!(client_exitmessage.is_none());
        assert_eq!(exit_asked(), 1);
        assert_eq!((*h.pr()).exit, 1);

        client_exitflag = 0;
    }
}

/// MSG_EXITED simply ends the process; nothing else moves.
#[test]
fn msg_exited_from_the_server_ends_the_process() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_EXITED as uint32_t, b"");

        assert_eq!((*h.pr()).exit, 1);
        assert_eq!(exit_asked(), 0);
        assert_eq!(exit_value(), 0);
    }
}

/// A version greeting from an incompatible server is fatal to the waiting
/// client, with the mismatch counted against it.
#[test]
fn a_protocol_mismatch_is_fatal_to_the_wait_state() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        let mut m = incoming(MSG_VERSION as uint32_t, b"");
        m.hdr.peerid = 0x0109;
        client_dispatch(&raw mut *m, null_mut());

        assert_eq!(exit_value(), 1);
        assert_eq!((*h.pr()).exit, 1);
    }
}

/// A stream from a server older than any known protocol is refused without
/// ceremony: the process ends, but the exit value stays untouched.
#[test]
fn an_old_server_stream_is_refused_without_touching_the_exit_value() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(211, b"");

        assert_eq!((*h.pr()).exit, 1);
        assert_eq!(exit_value(), 0);
    }
}

/// Messages the wait state has no interest in are dropped silently: MSG_COMMAND
/// belongs to the server, and nothing unknown should disturb anything.
#[test]
fn uninteresting_messages_are_dropped_while_waiting() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        deliver(MSG_COMMAND as uint32_t, b"junk");
        deliver(999, b"junk");

        assert!(h.sent().is_empty());
        assert_eq!((*h.pr()).exit, 0);
        assert_eq!(exit_asked(), 0);
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);
    }
}

/// A detach reaching the attached arm is recorded: the session name is kept,
/// the exit type remembers how the detach was asked for, and the reason says
/// whether the server will hang up behind it. Both kinds answer with the
/// exiting announcement.
#[test]
fn an_attached_client_records_who_detached_it_and_how() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        deliver(MSG_DETACH as uint32_t, b"sessions\0");

        assert_eq!(
            seen(client_string_ptr(&raw const client_exitsession)),
            "sessions"
        );
        assert_eq!(exit_type(), MSG_DETACH as uint32_t);
        assert_eq!(exit_reason(), CLIENT_EXIT_DETACHED);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);
        client_exitsession = None;
        client_exitreason = CLIENT_EXIT_NONE;

        deliver(MSG_DETACHKILL as uint32_t, b"killed\0");

        assert_eq!(
            seen(client_string_ptr(&raw const client_exitsession)),
            "killed"
        );
        assert_eq!(exit_type(), MSG_DETACHKILL as uint32_t);
        assert_eq!(exit_reason(), CLIENT_EXIT_DETACHED_HUP);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        client_exitsession = None;
        client_exitreason = CLIENT_EXIT_NONE;
        client_attached = 0;
    }
}

/// An exec request splits its two NUL-separated strings into the command to
/// run and the shell to run it with, and answers by announcing the exit.
#[test]
fn an_attached_client_prepares_an_exec_request() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        deliver(MSG_EXEC as uint32_t, b"tmux attach -t work\0/bin/sh\0");

        assert_eq!(
            seen(client_string_ptr(&raw const client_execcmd)),
            "tmux attach -t work"
        );
        assert_eq!(
            seen(client_string_ptr(&raw const client_execshell)),
            "/bin/sh"
        );
        assert_eq!(exit_type(), MSG_EXEC as uint32_t);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        client_execcmd = None;
        client_execshell = None;
        client_attached = 0;
    }
}

/// An exit arriving once attached announces the exit and names it EXITED,
/// but only if nothing better was already recorded — an earlier reason wins.
/// Unlike the wait arm it does not end the process by itself.
#[test]
fn an_attached_client_names_msg_exit_but_keeps_an_earlier_reason() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        client_exitreason = CLIENT_EXIT_DETACHED;
        deliver(MSG_EXIT as uint32_t, b"");
        assert_eq!(exit_reason(), CLIENT_EXIT_DETACHED);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        client_exitreason = CLIENT_EXIT_NONE;
        deliver(MSG_EXIT as uint32_t, b"");
        assert_eq!(exit_reason(), CLIENT_EXIT_EXITED);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        assert_eq!((*h.pr()).exit, 0);

        client_exitreason = CLIENT_EXIT_NONE;
        client_attached = 0;
    }
}

/// A shutdown while attached tells the client the server went away: the
/// announcement goes out first, then the reason and value are set.
#[test]
fn msg_shutdown_tells_an_attached_client_the_server_is_gone() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        deliver(MSG_SHUTDOWN as uint32_t, b"");

        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);
        assert_eq!(exit_reason(), CLIENT_EXIT_SERVER_EXITED);
        assert_eq!(exit_value(), 1);

        client_exitreason = CLIENT_EXIT_NONE;
        client_exitval = 0;
        client_attached = 0;
    }
}

/// MSG_EXITED while attached still means the process ends, but only with an
/// empty body — the size check is deliberately not provoked here.
#[test]
fn msg_exited_while_attached_ends_the_process() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        deliver(MSG_EXITED as uint32_t, b"");

        assert_eq!((*h.pr()).exit, 1);

        client_attached = 0;
    }
}

/// Attached clients track flag updates exactly as waiting ones do.
#[test]
fn an_attached_client_tracks_new_flags_too() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        deliver(MSG_FLAGS as uint32_t, &flags_payload(CLIENT_CONTROL as u64));

        assert_eq!(flags_now(), CLIENT_CONTROL as uint64_t);
        assert!(h.sent().is_empty());

        client_attached = 0;
    }
}

/// Uninteresting signals leave an unattached client completely alone.
#[test]
fn an_unattached_client_ignores_uninteresting_signals() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        client_signal(crate::client::SIGWINCH);

        assert_eq!((*h.pr()).exit, 0);
        assert!(h.sent().is_empty());
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);
        assert_eq!(exit_value(), 0);
    }
}

/// SIGTERM and SIGHUP ask an unattached client to end straight away.
#[test]
fn an_unattached_client_ends_on_sigterm_and_sighup() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        client_signal(crate::client::SIGTERM);
        assert_eq!((*h.pr()).exit, 1);

        (*h.pr()).exit = 0;
        client_signal(crate::client::SIGHUP);
        assert_eq!((*h.pr()).exit, 1);
    }
}

/// Attached, SIGHUP means the controlling terminal went away: the reason is
/// lost-tty, the value 1, and the server is told the client is exiting.
#[test]
fn an_attached_client_reports_lost_tty_on_sighup() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        client_signal(crate::client::SIGHUP);

        assert_eq!(exit_reason(), crate::client::CLIENT_EXIT_LOST_TTY);
        assert_eq!(exit_value(), 1);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        client_exitreason = CLIENT_EXIT_NONE;
        client_attached = 0;
    }
}

/// Attached, SIGTERM reports termination unless the client was suspended at
/// the time — a resumed client knows why it stopped and keeps that story —
/// and either way the value is 1 and the server hears about the exit.
#[test]
fn an_attached_client_reports_sigterm_according_to_suspension() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        client_signal(crate::client::SIGTERM);
        assert_eq!(exit_reason(), CLIENT_EXIT_TERMINATED);
        assert_eq!(exit_value(), 1);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        client_exitreason = CLIENT_EXIT_NONE;
        client_suspended = 1;
        client_signal(crate::client::SIGTERM);
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);
        assert_eq!(exit_value(), 1);
        assert_eq!(h.sent(), [MSG_EXITING as uint32_t]);

        client_suspended = 0;
        client_attached = 0;
    }
}

/// A window change while attached only asks for a resize; nothing else is
/// disturbed.
#[test]
fn an_attached_client_asks_for_a_resize_on_sigwinch() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;

        client_signal(crate::client::SIGWINCH);

        assert_eq!(h.sent(), [MSG_RESIZE as uint32_t]);
        assert_eq!((*h.pr()).exit, 0);
        assert_eq!(exit_reason(), CLIENT_EXIT_NONE);

        client_attached = 0;
    }
}

/// Resuming clears the suspension, tells the server the client woke up, and
/// ignores further stop signals on the way — the disposition the handler
/// installs is put back again before the test ends.
#[test]
fn an_attached_client_wakes_up_on_sigcont() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();
        client_attached = 1;
        client_suspended = 1;

        let mut old: libc::sigaction = ::core::mem::zeroed();
        assert_eq!(sigaction(crate::client::SIGTSTP, null(), &raw mut old), 0);

        client_signal(crate::client::SIGCONT);

        let mut now: libc::sigaction = ::core::mem::zeroed();
        assert_eq!(sigaction(crate::client::SIGTSTP, null(), &raw mut now), 0);
        assert_eq!(now.sa_sigaction as usize, ::libc::SIG_IGN);
        assert_eq!(
            sigaction(crate::client::SIGTSTP, &raw const old, null_mut()),
            0
        );

        assert_eq!(suspended_now(), 0);
        assert_eq!(h.sent(), [MSG_WAKEUP as uint32_t]);

        client_attached = 0;
    }
}

/// With nothing left writing, ending the client ends the process.
#[test]
fn client_exit_ends_the_process_when_no_files_are_pending() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        assert_eq!((*h.pr()).exit, 0);
        client_exit();

        assert_eq!((*h.pr()).exit, 1);
    }
}

/// The file callback only ends the process once an exit was already asked
/// for; until then it lets pending transfers carry on.
#[test]
fn the_file_callback_defers_to_pending_transfers_until_asked_to_exit() {
    let _t = turn();
    unsafe {
        reset();
        let mut h = Harness::new();

        client_file_check_cb(null_mut(), null(), 0, 0, null_mut(), ClientFileData::None);
        assert_eq!((*h.pr()).exit, 0);

        client_exitflag = 1;
        client_file_check_cb(null_mut(), null(), 0, 0, null_mut(), ClientFileData::None);
        assert_eq!((*h.pr()).exit, 1);

        client_exitflag = 0;
    }
}
