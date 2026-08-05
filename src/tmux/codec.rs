//! imsg framing over `sendmsg`/`recvmsg`, including `SCM_RIGHTS` fd passing.
//!
//! Wire layout (compat/imsg.h, compat/imsg-buffer.c), all native byte order to
//! match a same-arch tmux:
//!
//! ```text
//!   struct imsg_hdr { u32 type; u32 len; u32 peerid; u32 pid; }   // 16 bytes
//!   <payload: len_masked - 16 bytes>
//! ```
//!
//! * `len` is the *total* message size including the header.
//! * Bit 31 of `len` is `IMSG_FD_MARK` (0x8000_0000): set when an fd rides
//!   alongside via `SCM_RIGHTS`. It is masked off to get the real length.
//! * `peerid & 0xff` is the protocol version. `pid` is filler tmux never checks;
//!   we send our own and ignore the peer's.
//!
//! fd ordering: tmux sends at most one fd per `sendmsg`, attached to the first
//! message of that batch (imsg-buffer.c `msgbuf_write`). We mirror imsg's
//! ordered fd queue: received fds are queued in arrival order and popped by the
//! next message whose header has `IMSG_FD_MARK` set (imsg.c `imsg_parse_hdr`).

use std::collections::VecDeque;
use std::io;
use std::mem;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;

use libc::{c_int, c_void};

use super::message::{Frame, Message};
use super::traits::{
    NonblockingFrameReader, NonblockingFrameWriter, WriteQueueFull,
};

/// `IMSG_HEADER_SIZE` — `sizeof(struct imsg_hdr)`.
pub const HEADER_SIZE: usize = 16;
/// `IMSG_FD_MARK` — high bit of `len` signalling an attached fd.
const IMSG_FD_MARK: u32 = 0x8000_0000;
/// `MAX_IMSGSIZE` — largest single imsg (compat/imsg.h).
pub const MAX_IMSGSIZE: usize = 16384;
/// `IBUF_READ_SIZE` — recvmsg chunk size (compat/imsg.h).
const READ_CHUNK: usize = 65535;
/// Default private output queue high-water mark: 64 maximum-sized imsg frames.
const DEFAULT_WRITE_QUEUE_LIMIT: usize = MAX_IMSGSIZE * 64;

/// Encode a frame to its on-wire header+payload bytes. The fd (if any) is
/// carried separately by [`ImsgWriter`] and [`NonblockingImsgWriter`] as
/// ancillary data; this returns only the stream bytes, with `IMSG_FD_MARK` set
/// in `len` when `frame.fd` is present.
pub fn encode_bytes(frame: &Frame) -> Vec<u8> {
    let (type_, payload) = frame.msg.encode();
    let mut len = (HEADER_SIZE + payload.len()) as u32;
    if frame.fd.is_some() {
        len |= IMSG_FD_MARK;
    }
    let peerid = frame.version as u32; // version rides the low byte of peerid
    let pid = std::process::id();

    let mut out = Vec::with_capacity(HEADER_SIZE + payload.len());
    out.extend_from_slice(&type_.to_ne_bytes());
    out.extend_from_slice(&len.to_ne_bytes());
    out.extend_from_slice(&peerid.to_ne_bytes());
    out.extend_from_slice(&pid.to_ne_bytes());
    out.extend_from_slice(&payload);
    out
}

/// Read half of a connection: framed imsg reads over `recvmsg`.
pub struct ImsgReader {
    fd: OwnedFd,
    /// Bytes received but not yet parsed into whole frames.
    buf: Vec<u8>,
    /// fds received, awaiting the next `IMSG_FD_MARK` frame (arrival order).
    fds: VecDeque<OwnedFd>,
}

/// Blocking write half of a connection: framed imsg writes over `sendmsg`.
pub struct ImsgWriter {
    fd: OwnedFd,
}

/// Nonblocking write half of a connection with a bounded private output queue.
pub struct NonblockingImsgWriter {
    fd: OwnedFd,
    pending: VecDeque<ImsgWriteBuffer>,
    pending_bytes: usize,
    max_pending_bytes: usize,
}

#[derive(Debug)]
struct ImsgWriteBuffer {
    bytes: Vec<u8>,
    written: usize,
    fd: Option<OwnedFd>,
}

impl ImsgWriteBuffer {
    fn new(frame: Frame) -> Self {
        let bytes = encode_bytes(&frame);
        Self {
            bytes,
            written: 0,
            fd: frame.fd,
        }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.written
    }
}

impl ImsgReader {
    pub fn new(fd: OwnedFd) -> Self {
        ImsgReader {
            fd,
            buf: Vec::with_capacity(READ_CHUNK),
            fds: VecDeque::new(),
        }
    }

    /// Block until a whole frame is available, then decode it. Returns
    /// `UnexpectedEof` when the peer closes cleanly at a frame boundary.
    pub fn recv(&mut self) -> io::Result<Frame> {
        loop {
            if let Some(frame) = self.try_parse()? {
                return Ok(frame);
            }
            self.fill(0)?;
        }
    }

    /// Return the next complete frame without blocking.
    ///
    /// Partial bytes and received descriptors remain buffered when this
    /// returns `WouldBlock`.
    pub fn try_recv(&mut self) -> io::Result<Frame> {
        loop {
            if let Some(frame) = self.try_parse()? {
                return Ok(frame);
            }
            self.fill(libc::MSG_DONTWAIT)?;
        }
    }

    /// Whether a complete frame is already in the userspace receive buffer.
    /// Used by the native attach event loop: the kernel fd may no longer be
    /// readable after one `recvmsg` delivered several frames at once.
    pub(crate) fn has_buffered_frame(&self) -> bool {
        if self.buf.len() < HEADER_SIZE {
            return false;
        }
        let raw_len = u32::from_ne_bytes(self.buf[4..8].try_into().unwrap());
        let total = (raw_len & !IMSG_FD_MARK) as usize;
        (HEADER_SIZE..=MAX_IMSGSIZE).contains(&total) && self.buf.len() >= total
    }

    /// Try to pop one complete frame from the buffer.
    fn try_parse(&mut self) -> io::Result<Option<Frame>> {
        if self.buf.len() < HEADER_SIZE {
            return Ok(None);
        }
        let type_ = u32::from_ne_bytes(self.buf[0..4].try_into().unwrap());
        let raw_len = u32::from_ne_bytes(self.buf[4..8].try_into().unwrap());
        let peerid = u32::from_ne_bytes(self.buf[8..12].try_into().unwrap());
        let has_fd = raw_len & IMSG_FD_MARK != 0;
        let total = (raw_len & !IMSG_FD_MARK) as usize;

        if !(HEADER_SIZE..=MAX_IMSGSIZE).contains(&total) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("imsg length {total} out of range"),
            ));
        }
        if self.buf.len() < total {
            return Ok(None); // need more bytes
        }

        let payload = self.buf[HEADER_SIZE..total].to_vec();
        self.buf.drain(..total);

        let fd = if has_fd { self.fds.pop_front() } else { None };
        let version = (peerid & 0xff) as u8;
        let msg = Message::decode(type_, &payload);
        Ok(Some(Frame { version, msg, fd }))
    }

    /// One `recvmsg`: append received bytes to `buf` and any fds to `fds`.
    fn fill(&mut self, flags: c_int) -> io::Result<()> {
        let start = self.buf.len();
        self.buf.resize(start + READ_CHUNK, 0);

        // On any recvmsg error, roll the buffer back to its pre-read length
        // before propagating. This matters for the non-blocking / `SO_RCVTIMEO`
        // reader (the attach loop): a timeout surfaces as `WouldBlock`/`TimedOut`
        // here, and leaving the READ_CHUNK zero-padding in `buf` would make the
        // next `try_parse` read a bogus all-zero header (`total == 0` →
        // `InvalidData`). Truncating back keeps the buffer at a clean frame
        // boundary so a subsequent `recv` resumes correctly.
        let (n, fds) = match recvmsg_with_fds(self.fd.as_raw_fd(), &mut self.buf[start..], flags) {
            Ok(v) => v,
            Err(e) => {
                self.buf.truncate(start);
                return Err(e);
            }
        };
        self.buf.truncate(start + n);
        for fd in fds {
            self.fds.push_back(fd);
        }
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "peer closed connection",
            ));
        }
        Ok(())
    }
}

impl AsRawFd for ImsgReader {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl AsFd for ImsgReader {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for ImsgWriter {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl AsFd for ImsgWriter {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for NonblockingImsgWriter {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl AsFd for NonblockingImsgWriter {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl ImsgWriter {
    pub fn new(fd: OwnedFd) -> Self {
        ImsgWriter { fd }
    }

    /// Encode and send a frame, passing `frame.fd` as `SCM_RIGHTS` if present.
    pub fn send(&mut self, frame: Frame) -> io::Result<()> {
        let mut buffer = ImsgWriteBuffer::new(frame);
        sendmsg_buffer(self.fd.as_raw_fd(), &mut buffer, 0)
    }
}

impl NonblockingImsgWriter {
    pub fn new(fd: OwnedFd) -> Self {
        Self::with_queue_limit(fd, DEFAULT_WRITE_QUEUE_LIMIT)
    }

    /// Construct a nonblocking writer with a private output queue high-water
    /// mark.
    pub fn with_queue_limit(fd: OwnedFd, max_pending_bytes: usize) -> Self {
        NonblockingImsgWriter {
            fd,
            pending: VecDeque::new(),
            pending_bytes: 0,
            max_pending_bytes,
        }
    }

    /// Whether one more frame may cross the queue's high-water mark.
    ///
    /// Callers stop producing after that frame, so pending output is bounded by
    /// the configured mark plus one protocol frame.
    pub(crate) fn is_below_high_water(&self) -> bool {
        self.pending_bytes < self.max_pending_bytes
    }

    fn try_queue_frame(&mut self, frame: Frame) -> Result<(), WriteQueueFull<Frame>> {
        if !self.is_below_high_water() {
            return Err(WriteQueueFull::new(frame));
        }
        let bytes = encode_bytes(&frame);
        let frame_bytes = bytes.len();

        let buffer = ImsgWriteBuffer {
            bytes,
            written: 0,
            fd: frame.fd,
        };
        self.pending_bytes += frame_bytes;
        self.pending.push_back(buffer);
        Ok(())
    }

    fn flush_queued(&mut self, flags: c_int) -> io::Result<()> {
        while !self.pending.is_empty() {
            let (result, written) = {
                let buffer = self
                    .pending
                    .front_mut()
                    .expect("pending queue is not empty");
                let before = buffer.remaining();
                let result = sendmsg_buffer(self.fd.as_raw_fd(), buffer, flags);
                (result, before - buffer.remaining())
            };
            self.pending_bytes -= written;

            match result {
                Ok(()) => {
                    self.pending.pop_front();
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}

// ---- raw syscall wrappers ---------------------------------------------

/// `recvmsg` collecting up to one fd's worth of `SCM_RIGHTS` ancillary data per
/// call (matching tmux's one-fd control buffer). Returns `(bytes_read, fds)`.
fn recvmsg_with_fds(fd: RawFd, buf: &mut [u8], flags: c_int) -> io::Result<(usize, Vec<OwnedFd>)> {
    // Space for one fd; padded per CMSG rules. A generous fixed buffer avoids a
    // heap alloc and still fits the single-fd control message.
    let cmsg_space = unsafe { libc::CMSG_SPACE(mem::size_of::<c_int>() as u32) } as usize;
    let mut cmsg_buf = vec![0u8; cmsg_space];

    let mut iov = libc::iovec {
        iov_base: buf.as_mut_ptr() as *mut c_void,
        iov_len: buf.len(),
    };
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut c_void;
    msg.msg_controllen = cmsg_buf.len() as _;

    let n = loop {
        let r = unsafe { libc::recvmsg(fd, &mut msg, flags) };
        if r < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e);
        }
        break r as usize;
    };

    let mut fds = Vec::new();
    unsafe {
        let mut cmsg = libc::CMSG_FIRSTHDR(&msg);
        while !cmsg.is_null() {
            if (*cmsg).cmsg_level == libc::SOL_SOCKET && (*cmsg).cmsg_type == libc::SCM_RIGHTS {
                let data = libc::CMSG_DATA(cmsg);
                let hdr_len = libc::CMSG_LEN(0) as usize;
                let count = ((*cmsg).cmsg_len as usize - hdr_len) / mem::size_of::<c_int>();
                for i in 0..count {
                    let mut raw: c_int = 0;
                    std::ptr::copy_nonoverlapping(
                        data.add(i * mem::size_of::<c_int>()),
                        &mut raw as *mut c_int as *mut u8,
                        mem::size_of::<c_int>(),
                    );
                    let flags = libc::fcntl(raw, libc::F_GETFD);
                    if flags < 0 || libc::fcntl(raw, libc::F_SETFD, flags | libc::FD_CLOEXEC) < 0 {
                        let error = io::Error::last_os_error();
                        libc::close(raw);
                        return Err(error);
                    }
                    fds.push(OwnedFd::from_raw_fd(raw));
                }
            }
            cmsg = libc::CMSG_NXTHDR(&msg, cmsg);
        }
    }

    Ok((n, fds))
}

/// Advance a prepared imsg buffer, attaching its descriptor to the first
/// successful write only.
fn sendmsg_buffer(fd: RawFd, buffer: &mut ImsgWriteBuffer, flags: c_int) -> io::Result<()> {
    let cmsg_space = unsafe { libc::CMSG_SPACE(mem::size_of::<c_int>() as u32) } as usize;
    let mut cmsg_buf = vec![0u8; cmsg_space];

    while buffer.written < buffer.bytes.len() {
        let mut iov = libc::iovec {
            iov_base: buffer.bytes[buffer.written..].as_ptr() as *mut c_void,
            iov_len: buffer.bytes.len() - buffer.written,
        };
        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;

        if let Some(raw) = buffer.fd.as_ref().map(AsRawFd::as_raw_fd) {
            cmsg_buf.fill(0);
            msg.msg_control = cmsg_buf.as_mut_ptr() as *mut c_void;
            msg.msg_controllen = cmsg_buf.len() as _;
            unsafe {
                let cmsg = libc::CMSG_FIRSTHDR(&msg);
                (*cmsg).cmsg_level = libc::SOL_SOCKET;
                (*cmsg).cmsg_type = libc::SCM_RIGHTS;
                (*cmsg).cmsg_len = libc::CMSG_LEN(mem::size_of::<c_int>() as u32) as _;
                std::ptr::copy_nonoverlapping(
                    &raw as *const c_int as *const u8,
                    libc::CMSG_DATA(cmsg),
                    mem::size_of::<c_int>(),
                );
            }
        }

        let r = unsafe { libc::sendmsg(fd, &msg, flags) };
        if r < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e);
        }
        if r == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "sendmsg wrote zero bytes",
            ));
        }

        // A descriptor attached to a successful stream write was transferred
        // with its first byte. Drop our copy before continuing with the suffix
        // so a retry cannot send it twice.
        buffer.fd.take();
        buffer.written += r as usize;
    }
    Ok(())
}

/// Duplicate a borrowed fd into an owned one (used to split a socket into
/// independent read/write handles for the two-thread model).
pub fn dup_fd(fd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    fd.try_clone_to_owned()
}

/// Split a connected Unix stream into independent framed read/write handles.
pub fn split_stream(stream: UnixStream) -> io::Result<(ImsgReader, ImsgWriter)> {
    let read_fd: OwnedFd = stream.into();
    let write_fd = dup_fd(read_fd.as_fd())?;
    Ok((ImsgReader::new(read_fd), ImsgWriter::new(write_fd)))
}

/// Split a connected Unix stream into independent nonblocking framed handles.
pub fn split_nonblocking_stream(
    stream: UnixStream,
) -> io::Result<(ImsgReader, NonblockingImsgWriter)> {
    split_nonblocking_stream_with_queue_limit(stream, DEFAULT_WRITE_QUEUE_LIMIT)
}

pub(crate) fn split_nonblocking_stream_with_queue_limit(
    stream: UnixStream,
    max_pending_bytes: usize,
) -> io::Result<(ImsgReader, NonblockingImsgWriter)> {
    let read_fd: OwnedFd = stream.into();
    let write_fd = dup_fd(read_fd.as_fd())?;
    Ok((
        ImsgReader::new(read_fd),
        NonblockingImsgWriter::with_queue_limit(write_fd, max_pending_bytes),
    ))
}

impl NonblockingFrameReader for ImsgReader {
    fn try_recv(&mut self) -> io::Result<Frame> {
        ImsgReader::try_recv(self)
    }
}

impl NonblockingFrameWriter for NonblockingImsgWriter {
    type Frame = Frame;

    fn try_queue(&mut self, frame: Self::Frame) -> Result<(), WriteQueueFull<Self::Frame>> {
        self.try_queue_frame(frame)
    }

    fn try_flush(&mut self) -> io::Result<()> {
        self.flush_queued(libc::MSG_DONTWAIT)
    }

    fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::message::{msgtype, Frame, Message, PROTOCOL_VERSION};
    use std::io::Write as _;

    /// Decode a frame's bytes back into `(type, version, payload)` for
    /// round-trip assertions (no fd path — that needs a real socket).
    fn decode_bytes(bytes: &[u8]) -> (u32, u8, Vec<u8>) {
        let type_ = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
        let raw_len = u32::from_ne_bytes(bytes[4..8].try_into().unwrap());
        let peerid = u32::from_ne_bytes(bytes[8..12].try_into().unwrap());
        let total = (raw_len & !IMSG_FD_MARK) as usize;
        (
            type_,
            (peerid & 0xff) as u8,
            bytes[HEADER_SIZE..total].to_vec(),
        )
    }

    /// `decode(encode(msg)) == msg`, with the version byte preserved.
    fn assert_roundtrip(msg: Message) {
        let frame = Frame::new(msg.clone());
        let bytes = encode_bytes(&frame);
        let (type_, version, payload) = decode_bytes(&bytes);
        assert_eq!(version, PROTOCOL_VERSION, "version byte preserved");
        let decoded = Message::decode(type_, &payload);
        assert_eq!(decoded, msg, "semantic round-trip");
    }

    #[test]
    fn roundtrip_every_modeled_variant() {
        let cases = vec![
            Message::IdentifyFlags(0x1234),
            Message::IdentifyLongFlags(0x1_0000_0002),
            Message::IdentifyTerm("screen-256color".into()),
            Message::IdentifyTtyName("/dev/pts/3".into()),
            Message::IdentifyCwd("/home/user".into()),
            Message::IdentifyEnviron("PATH=/usr/bin".into()),
            Message::IdentifyTerminfo("xterm".into()),
            Message::IdentifyFeatures(7),
            Message::IdentifyStdin,
            Message::IdentifyStdout,
            Message::IdentifyClientPid(4242),
            Message::IdentifyDone,
            Message::Command(vec!["new-session".into(), "-d".into()]),
            Message::Command(vec![]),
            Message::Detach(None),
            Message::Detach(Some("work".into())),
            Message::DetachKill(None),
            Message::DetachKill(Some("work".into())),
            Message::Exit(None),
            Message::Exit(Some(-1)),
            Message::Lock("lock -np".into()),
            Message::Unlock,
            Message::Suspend,
            Message::Wakeup,
            Message::Resize,
            Message::Shutdown,
            Message::Flags(0xdead_beef),
            Message::Ready,
            Message::Version,
            Message::Exited,
            Message::Exiting,
            Message::ReadOpen {
                stream: 1,
                fd: -1,
                path: b"/tmp/x\0".to_vec(),
            },
            Message::ReadOpen {
                stream: 1,
                fd: 0,
                path: Vec::new(),
            }, // pathless (stdio)
            Message::Read {
                stream: 2,
                data: b"read data".to_vec(),
            },
            Message::ReadDone {
                stream: 2,
                error: 0,
            },
            Message::ReadCancel { stream: 2 },
            Message::WriteOpen {
                stream: 3,
                fd: -1,
                flags: 0,
                path: b"/tmp/y\0".to_vec(),
            },
            Message::WriteOpen {
                stream: 1,
                fd: 1,
                flags: 0,
                path: Vec::new(),
            }, // pathless
            Message::Write {
                stream: 3,
                data: vec![1, 2, 3, 0, 255],
            },
            Message::WriteReady {
                stream: 3,
                error: 5,
            },
            Message::WriteClose { stream: 3 },
            Message::Unknown {
                type_: 999,
                payload: vec![9, 8, 7],
            },
        ];
        for c in cases {
            assert_roundtrip(c);
        }
    }

    #[test]
    fn suspend_lifecycle_messages_have_empty_wire_payloads() {
        for (message, expected_type) in [
            (Message::Suspend, msgtype::MSG_SUSPEND),
            (Message::Wakeup, msgtype::MSG_WAKEUP),
        ] {
            let bytes = encode_bytes(&Frame::new(message));
            let (type_, version, payload) = decode_bytes(&bytes);
            assert_eq!(type_, expected_type);
            assert_eq!(version, PROTOCOL_VERSION);
            assert!(payload.is_empty());
            assert_eq!(bytes.len(), HEADER_SIZE);
        }
    }

    #[test]
    fn command_packing_matches_cmd_pack_argv() {
        // argc (native i32) then each arg + NUL.
        let msg = Message::Command(vec!["list-sessions".into()]);
        let (type_, payload) = msg.encode();
        assert_eq!(type_, msgtype::MSG_COMMAND);
        let mut expected = 1i32.to_ne_bytes().to_vec();
        expected.extend_from_slice(b"list-sessions\0");
        assert_eq!(payload, expected);
    }

    #[test]
    fn unknown_forwarded_verbatim() {
        // A malformed EXIT (odd length) must survive as Unknown, byte-for-byte.
        let payload = vec![1u8, 2, 3];
        let decoded = Message::decode(msgtype::MSG_EXIT, &payload);
        assert!(matches!(decoded, Message::Unknown { .. }));
        let (type_, out) = decoded.encode();
        assert_eq!(type_, msgtype::MSG_EXIT);
        assert_eq!(out, payload);
    }

    /// Regression: a `recvmsg` that fails (e.g. `SO_RCVTIMEO` timeout →
    /// `EAGAIN`) must not leave the read buffer padded with zero bytes. Before
    /// the fix, `fill()` propagated the error *after* growing `buf` by
    /// `READ_CHUNK` and *before* truncating, so the next `try_parse` read an
    /// all-zero header (`total == 0`) and returned `InvalidData` — which broke
    /// the attach loop (the only reader that uses a recv timeout).
    #[test]
    fn timeout_does_not_corrupt_buffer_then_next_frame_decodes() {
        // A connected socketpair; read from `a`, write to `b`.
        let mut fds = [0 as libc::c_int; 2];
        let rc = unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "socketpair failed");
        let a = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let b = unsafe { OwnedFd::from_raw_fd(fds[1]) };

        // Make the read fail with no data pending. `SO_RCVTIMEO` is what the
        // attach loop uses, but its expiry and `O_NONBLOCK` reach `fill` as the
        // same `EAGAIN` from the same `recvmsg`, and only that error path is
        // under test — so take the one that does not spend a timeout waiting.
        let flags = unsafe { libc::fcntl(a.as_raw_fd(), libc::F_GETFL) };
        assert!(flags >= 0, "fcntl F_GETFL failed");
        let rc = unsafe { libc::fcntl(a.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) };
        assert_eq!(rc, 0, "fcntl F_SETFL failed");

        let mut reader = ImsgReader::new(a);

        // No data yet: the timeout must surface as WouldBlock/TimedOut, NOT as
        // an InvalidData from a corrupted buffer.
        let e = reader.recv().expect_err("should time out");
        assert!(
            matches!(
                e.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
            ),
            "expected a timeout error, got {:?}",
            e.kind()
        );

        // Now write a real frame from the peer and confirm the reader decodes it
        // cleanly — proving the buffer was rolled back to a clean boundary.
        let mut writer = ImsgWriter::new(b);
        let msg = Message::Command(vec!["list-sessions".into()]);
        writer.send(Frame::new(msg.clone())).expect("send");

        let frame = reader.recv().expect("decode after timeout");
        assert_eq!(frame.version, PROTOCOL_VERSION);
        assert_eq!(frame.msg, msg);
    }

    #[test]
    fn try_recv_reports_would_block_without_changing_socket_mode() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let mut reader = ImsgReader::new(receiver.into());

        let error = reader.try_recv().expect_err("no frame is available");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);

        let flags = unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(flags & libc::O_NONBLOCK, 0);

        drop(sender);
        let error = reader.try_recv().expect_err("closed peer is EOF");
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn try_recv_retains_partial_frame_until_complete() {
        let (mut sender, receiver) = UnixStream::pair().unwrap();
        let mut reader = ImsgReader::new(receiver.into());
        let message = Message::Command(vec!["list-sessions".into()]);
        let bytes = encode_bytes(&Frame::new(message.clone()));

        sender.write_all(&bytes[..HEADER_SIZE - 1]).unwrap();
        let error = reader.try_recv().expect_err("header is incomplete");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);

        sender.write_all(&bytes[HEADER_SIZE - 1..]).unwrap();
        let frame = reader.try_recv().expect("completed frame");
        assert_eq!(frame.version, PROTOCOL_VERSION);
        assert_eq!(frame.msg, message);
    }

    #[test]
    fn try_recv_drains_frames_already_buffered_in_userspace() {
        let (mut sender, receiver) = UnixStream::pair().unwrap();
        let mut reader = ImsgReader::new(receiver.into());
        let first = Message::Command(vec!["first".into()]);
        let second = Message::Command(vec!["second".into()]);
        let mut bytes = encode_bytes(&Frame::new(first.clone()));
        bytes.extend_from_slice(&encode_bytes(&Frame::new(second.clone())));
        sender.write_all(&bytes).unwrap();

        assert_eq!(reader.try_recv().unwrap().msg, first);
        assert_eq!(reader.try_recv().unwrap().msg, second);
        let error = reader.try_recv().expect_err("all frames were drained");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn try_recv_preserves_received_descriptor_association() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let mut writer = ImsgWriter::new(sender.into());
        let mut reader = ImsgReader::new(receiver.into());
        let descriptor: OwnedFd = std::fs::File::open("/dev/null").unwrap().into();

        writer
            .send(Frame::with_fd(Message::IdentifyStdin, descriptor))
            .unwrap();
        let frame = reader.try_recv().expect("frame with descriptor");

        assert_eq!(frame.msg, Message::IdentifyStdin);
        assert!(frame.fd.is_some());
    }

    #[test]
    fn try_queue_performs_no_io_and_flushes_in_order() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let (_sender_reader, mut writer) = split_nonblocking_stream(sender).unwrap();
        let mut reader = ImsgReader::new(receiver.into());
        let first = Message::Command(vec!["first".into()]);
        let second = Message::Command(vec!["second".into()]);

        writer.try_queue(Frame::new(first.clone())).unwrap();
        writer.try_queue(Frame::new(second.clone())).unwrap();
        assert!(writer.has_pending());
        let error = reader.try_recv().expect_err("queueing must not write");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);

        writer.try_flush().unwrap();
        assert!(!writer.has_pending());
        assert_eq!(reader.try_recv().unwrap().msg, first);
        assert_eq!(reader.try_recv().unwrap().msg, second);
    }

    #[test]
    fn try_queue_rejects_at_the_private_queue_limit() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let accepted = Message::Command(vec!["one".into()]);
        let rejected_message = Message::Command(vec!["two".into()]);
        let queue_limit = encode_bytes(&Frame::new(accepted.clone())).len();
        let mut writer = NonblockingImsgWriter::with_queue_limit(sender.into(), queue_limit);
        let mut reader = ImsgReader::new(receiver.into());

        writer.try_queue(Frame::new(accepted.clone())).unwrap();
        let rejected_frame = writer
            .try_queue(Frame::new(rejected_message.clone()))
            .expect_err("the queue is full")
            .into_frame();
        assert_eq!(rejected_frame.msg, rejected_message);
        assert!(writer.has_pending());
        let error = reader.try_recv().expect_err("queueing must not write");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);

        writer.try_flush().unwrap();
        assert_eq!(reader.try_recv().unwrap().msg, accepted);
        assert!(!writer.has_pending());

        writer.try_queue(rejected_frame).unwrap();
        writer.try_flush().unwrap();
        assert_eq!(reader.try_recv().unwrap().msg, rejected_message);
    }

    #[test]
    fn try_queue_accepts_one_frame_crossing_the_high_water_mark() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let first = Message::Command(vec!["one".into()]);
        let second = Message::Command(vec!["two".into()]);
        let third = Message::Command(vec!["three".into()]);
        let queue_limit = encode_bytes(&Frame::new(first.clone())).len() + 1;
        let mut writer = NonblockingImsgWriter::with_queue_limit(sender.into(), queue_limit);
        let mut reader = ImsgReader::new(receiver.into());

        writer.try_queue(Frame::new(first.clone())).unwrap();
        assert!(writer.is_below_high_water());
        writer.try_queue(Frame::new(second.clone())).unwrap();
        assert!(!writer.is_below_high_water());
        let rejected = writer
            .try_queue(Frame::new(third.clone()))
            .expect_err("the high-water mark has been reached")
            .into_frame();
        assert_eq!(rejected.msg, third);

        writer.try_flush().unwrap();
        assert_eq!(reader.try_recv().unwrap().msg, first);
        assert_eq!(reader.try_recv().unwrap().msg, second);
    }

    #[test]
    fn rejected_frame_preserves_its_descriptor_for_retry() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let queue_limit = encode_bytes(&Frame::new(Message::Ready)).len();
        let mut writer = NonblockingImsgWriter::with_queue_limit(sender.into(), queue_limit);
        let mut reader = ImsgReader::new(receiver.into());
        let descriptor: OwnedFd = std::fs::File::open("/dev/null").unwrap().into();
        let raw = descriptor.as_raw_fd();

        writer.try_queue(Frame::new(Message::Ready)).unwrap();
        let rejected = writer
            .try_queue(Frame::with_fd(Message::IdentifyStdin, descriptor))
            .expect_err("the queue is full");
        assert!(rejected.frame().fd.is_some());
        assert!(unsafe { libc::fcntl(raw, libc::F_GETFD) } >= 0);

        writer.try_flush().unwrap();
        writer.try_queue(rejected.into_frame()).unwrap();
        writer.try_flush().unwrap();

        assert_eq!(reader.try_recv().unwrap().msg, Message::Ready);
        let retried = reader.try_recv().unwrap();
        assert_eq!(retried.msg, Message::IdentifyStdin);
        assert!(retried.fd.is_some());
    }

    #[test]
    fn try_flush_retains_unwritten_frames_after_would_block() {
        const FRAME_COUNT: i32 = 64;

        let (sender, receiver) = UnixStream::pair().unwrap();
        let send_buffer_bytes: c_int = 4096;
        let result = unsafe {
            libc::setsockopt(
                sender.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_SNDBUF,
                &send_buffer_bytes as *const _ as *const c_void,
                mem::size_of_val(&send_buffer_bytes) as libc::socklen_t,
            )
        };
        assert_eq!(result, 0, "setsockopt failed");

        let mut writer = NonblockingImsgWriter::new(sender.into());
        let mut reader = ImsgReader::new(receiver.into());
        let payload_len = MAX_IMSGSIZE - HEADER_SIZE - mem::size_of::<i32>();
        for stream in 0..FRAME_COUNT {
            writer
                .try_queue(Frame::new(Message::Write {
                    stream,
                    data: vec![stream as u8; payload_len],
                }))
                .unwrap();
        }

        let error = writer
            .try_flush()
            .expect_err("the small socket buffer must apply backpressure");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(writer.has_pending());

        let mut received = 0;
        for _ in 0..4096 {
            loop {
                match reader.try_recv() {
                    Ok(frame) => {
                        let Message::Write { stream, data } = frame.msg else {
                            panic!("unexpected frame");
                        };
                        assert_eq!(stream, received);
                        assert_eq!(data.len(), payload_len);
                        assert!(data.iter().all(|byte| *byte == received as u8));
                        received += 1;
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                    Err(error) => panic!("receive failed: {error}"),
                }
            }

            if received == FRAME_COUNT && !writer.has_pending() {
                break;
            }

            match writer.try_flush() {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("flush failed: {error}"),
            }
        }

        assert_eq!(received, FRAME_COUNT);
        assert!(!writer.has_pending());
    }

    #[test]
    fn fd_mark_set_in_length_when_fd_present() {
        // Can't attach a real fd without a socket, but a stdin frame carrying a
        // borrowed fd sets IMSG_FD_MARK. Use the raw stdin fd via a dummy dup.
        let devnull = std::fs::File::open("/dev/null").unwrap();
        let owned: OwnedFd = devnull.into();
        let frame = Frame::with_fd(Message::IdentifyStdin, owned);
        let bytes = encode_bytes(&frame);
        let raw_len = u32::from_ne_bytes(bytes[4..8].try_into().unwrap());
        assert!(raw_len & IMSG_FD_MARK != 0, "fd mark set");
        assert_eq!((raw_len & !IMSG_FD_MARK) as usize, HEADER_SIZE);
    }

    #[test]
    fn received_fds_are_close_on_exec() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let mut writer = ImsgWriter::new(sender.into());
        let mut reader = ImsgReader::new(receiver.into());
        let devnull: OwnedFd = std::fs::File::open("/dev/null").unwrap().into();

        writer
            .send(Frame::with_fd(Message::IdentifyStdin, devnull))
            .unwrap();
        let received = reader.recv().unwrap().fd.expect("received descriptor");
        let flags = unsafe { libc::fcntl(received.as_raw_fd(), libc::F_GETFD) };

        assert!(flags >= 0);
        assert_ne!(flags & libc::FD_CLOEXEC, 0);
    }
}
