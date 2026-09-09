use crate::ffi::{__errno_location, abort, close, readv, writev};
pub use crate::types::*;
use crate::{ControlMessage, ControlMessageHeader, ControlMessages, MessageHeader, SocketMessage};
use ::core::ffi::{c_int, c_uint};
#[cfg(test)]
use ::core::ptr::null_mut;
use ::core::ptr::write_bytes;
use bytes::{Buf as BytesBuf, BufMut as BytesBufMut, BytesMut};
use std::io::{IoSlice, IoSliceMut};
pub type scm_type = c_uint;
pub const SCM_RIGHTS: scm_type = 1;
/// A queue of buffers waiting to be written or read, front to back. The
/// queue owns them: whatever comes off it is the caller's to give up.
pub struct ibufqueue {
    pub bufs: std::collections::VecDeque<Box<ibuf>>,
}
pub type msgbuf_read_cb =
    Option<std::rc::Rc<dyn Fn(&mut ibuf, Option<uint32_t>, &mut c_int) -> Option<Box<ibuf>>>>;
#[repr(C)]
pub struct msgbuf {
    pub bufs: ibufqueue,
    pub rbufs: ibufqueue,
    pub rbuf: Option<Box<[u8]>>,
    /// The message being read in, held until the last of it has arrived.
    pub rpmsg: Option<Box<ibuf>>,
    pub readhdr: msgbuf_read_cb,
    pub read_limit: Option<uint32_t>,
    pub roff: size_t,
    pub hdrsize: size_t,
}

impl Drop for msgbuf {
    fn drop(&mut self) {
        unsafe {
            msgbuf_clear(self);
        }
    }
}
#[repr(C)]
#[derive(Default)]
struct ControlStorage {
    _alignment: [libc::cmsghdr; 0],
    buf: [u8; 24],
}

pub const SOL_SOCKET: c_int = 1 as c_int;
pub const __IOV_MAX: c_int = 1024 as c_int;
pub const IOV_MAX: c_int = __IOV_MAX;
pub use crate::consts::{EAGAIN, EBADMSG, EINTR, EINVAL, ERANGE, SIZE_MAX, UINT32_MAX};

pub const EMSGSIZE: c_int = 90 as c_int;
pub const ENOBUFS: c_int = 105 as c_int;

pub const IBUF_READ_SIZE: c_int = 65535 as c_int;

/// Sets the error number and answers the failure the caller hands back.
unsafe fn ibuf_fail<T>(errno: c_int, answer: T) -> T {
    unsafe {
        *__errno_location() = errno;
        answer
    }
}

/// Whether the buffer's bytes belong to somebody else, so that it may neither
/// grow nor be freed nor be queued.
fn ibuf_on_stack(buf: &ibuf) -> bool {
    buf.borrowed
}

/// An empty `ibuf`.
fn ibuf_empty() -> ibuf {
    ibuf::default()
}

const IBUF_BUF_MUT_CHUNK: usize = 64 * 1024;

impl BytesBuf for ibuf {
    fn remaining(&self) -> usize {
        self.wpos.saturating_sub(self.rpos)
    }

    fn chunk(&self) -> &[u8] {
        &self.buf[self.rpos..self.wpos]
    }

    fn advance(&mut self, count: usize) {
        assert!(count <= self.remaining());
        self.rpos += count;
    }
}

unsafe impl BytesBufMut for ibuf {
    fn remaining_mut(&self) -> usize {
        self.max.saturating_sub(self.wpos)
    }

    unsafe fn advance_mut(&mut self, count: usize) {
        assert!(count <= self.remaining_mut());
        let wpos = self.wpos + count;
        if wpos > self.size {
            assert!(wpos <= self.buf.capacity());
            unsafe { BytesBufMut::advance_mut(&mut self.buf, wpos - self.size) };
            self.size = wpos;
        }
        self.wpos = wpos;
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        let remaining = self.remaining_mut();
        if remaining == 0 {
            return bytes::buf::UninitSlice::new(&mut []);
        }
        let needed = self.wpos + remaining.min(IBUF_BUF_MUT_CHUNK);
        if needed > self.buf.capacity() {
            self.buf.reserve(needed - self.buf.len());
        }
        let initialized = self.buf.len();
        if self.wpos < initialized {
            let end = initialized.min(self.wpos + remaining);
            return bytes::buf::UninitSlice::new(&mut self.buf[self.wpos..end]);
        }
        let spare = &mut self.buf.spare_capacity_mut()[self.wpos - initialized..];
        let available = remaining.min(spare.len());
        bytes::buf::UninitSlice::uninit(&mut spare[..available])
    }
}

/// A fresh buffer holding `len` zeroed bytes, which is the caller's.
fn ibuf_alloc(len: size_t) -> Box<ibuf> {
    Box::new(ibuf {
        buf: BytesMut::zeroed(len),
        fd: -1,
        ..ibuf::default()
    })
}

pub fn ibuf_open(len: size_t) -> Option<Box<ibuf>> {
    let mut buf = ibuf_alloc(len);
    buf.max = len;
    buf.size = buf.max;
    Some(buf)
}

pub fn ibuf_dynamic(len: size_t, max: size_t) -> Option<Box<ibuf>> {
    unsafe {
        if max == 0 as size_t || max < len {
            return ibuf_fail(EINVAL, None);
        }
        let mut buf = ibuf_alloc(len);
        buf.size = len;
        buf.max = max;
        Some(buf)
    }
}

pub fn ibuf_reserve(buf: &mut ibuf, len: size_t) -> Option<&mut [u8]> {
    if len > (SIZE_MAX as size_t).wrapping_sub(buf.wpos) {
        return unsafe { ibuf_fail(ERANGE, None) };
    }
    if buf.borrowed {
        return unsafe { ibuf_fail(EINVAL, None) };
    }
    let want = buf.wpos.wrapping_add(len);
    if want > buf.size {
        if want > buf.max {
            return unsafe { ibuf_fail(ERANGE, None) };
        }
        let len = want.wrapping_sub(buf.size);
        buf.buf.reserve(len);
        unsafe { write_bytes(buf.buf.spare_capacity_mut().as_mut_ptr(), 0, len) };
        unsafe { BytesBufMut::advance_mut(&mut buf.buf, len) };
        buf.size = want;
    }
    let start = buf.wpos;
    buf.wpos = want;
    Some(&mut buf.buf[start..want])
}

pub unsafe fn ibuf_add(buf: &mut ibuf, data: &[u8]) -> c_int {
    if data.is_empty() {
        return 0 as c_int;
    }
    let Some(b) = ibuf_reserve(buf, data.len()) else {
        return -(1 as c_int);
    };
    b.copy_from_slice(data);
    0 as c_int
}

pub unsafe fn ibuf_add_ibuf(buf: &mut ibuf, from: &ibuf) -> c_int {
    unsafe { ibuf_add(buf, ibuf_data(from)) }
}

/// Whether a number fits the width the buffer keeps for it. Upstream refuses
/// one that does not with `EINVAL`, before touching the buffer at all.
fn ibuf_too_wide(value: uint64_t, max: uint64_t) -> bool {
    unsafe {
        if value > max {
            *__errno_location() = EINVAL;
            return true;
        }
        false
    }
}

pub unsafe fn ibuf_seek(buf: &mut ibuf, pos: size_t, len: size_t) -> Option<&mut [u8]> {
    let size = buf.wpos.wrapping_sub(buf.rpos);
    if size < pos || (SIZE_MAX as size_t).wrapping_sub(pos) < len || size < pos.wrapping_add(len) {
        return unsafe { ibuf_fail(ERANGE, None) };
    }
    let start = buf.rpos.wrapping_add(pos);
    let end = start.wrapping_add(len);
    Some(&mut buf.buf[start..end])
}

pub fn ibuf_set(buf: &mut ibuf, pos: size_t, data: &[u8]) -> c_int {
    unsafe {
        let Some(b) = ibuf_seek(buf, pos, data.len()) else {
            return -(1 as c_int);
        };
        b.copy_from_slice(data);
        0 as c_int
    }
}

/// Writes the bytes of a number over a place already inside the buffer.
unsafe fn ibuf_set_bytes(buf: &mut ibuf, pos: size_t, bytes: &[u8]) -> c_int {
    ibuf_set(buf, pos, bytes)
}

pub unsafe fn ibuf_set_h32(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
        return -(1 as c_int);
    }
    unsafe { ibuf_set_bytes(buf, pos, &(value as uint32_t).to_ne_bytes()) }
}

pub fn ibuf_data(buf: &ibuf) -> &[u8] {
    &buf.buf[buf.rpos..buf.wpos]
}

pub unsafe fn ibuf_size(buf: &ibuf) -> size_t {
    buf.wpos.wrapping_sub(buf.rpos)
}

pub unsafe fn ibuf_left(buf: &ibuf) -> size_t {
    {
        if ibuf_on_stack(buf) {
            return 0 as size_t;
        }
        buf.max.wrapping_sub(buf.wpos)
    }
}

pub unsafe fn ibuf_close(msgbuf: &mut msgbuf, buf: Box<ibuf>) {
    unsafe {
        ibufq_push(&mut msgbuf.bufs, buf);
    }
}

/// Replaces `buf` with an owned copy of a temporary byte range.
pub unsafe fn ibuf_from_buffer(buf: &mut ibuf, data: &[u8]) {
    let bytes = if data.is_empty() {
        BytesMut::new()
    } else {
        BytesMut::from(data)
    };
    *buf = ibuf {
        buf: bytes,
        size: data.len(),
        wpos: data.len(),
        borrowed: true,
        ..ibuf::default()
    };
}

pub unsafe fn ibuf_get(buf: &mut ibuf, data: &mut [u8]) -> c_int {
    unsafe {
        if ibuf_size(buf) < data.len() {
            return ibuf_fail(EBADMSG, -(1 as c_int));
        }
        data.copy_from_slice(&ibuf_data(buf)[..data.len()]);
        buf.rpos = buf.rpos.wrapping_add(data.len());
        0 as c_int
    }
}

pub unsafe fn ibuf_get_ibuf(buf: &mut ibuf, len: size_t) -> Option<ibuf> {
    unsafe {
        if ibuf_size(buf) < len {
            ibuf_fail(EBADMSG, -(1 as c_int));
            return None;
        }
        let data = &ibuf_data(buf)[..len];
        let mut new = ibuf::default();
        ibuf_from_buffer(&mut new, data);
        buf.rpos = buf.rpos.wrapping_add(len);
        Some(new)
    }
}

/// Gives up a buffer the caller owns, closing whatever descriptor it carries.
pub unsafe fn ibuf_free(mut buf: Box<ibuf>) {
    unsafe {
        let save_errno = *__errno_location();
        if ibuf_on_stack(&buf) {
            abort();
        }
        if buf.fd >= 0 as c_int {
            close(buf.fd);
        }
        buf.buf.fill(0);
        drop(buf);
        *__errno_location() = save_errno;
    }
}

pub fn ibuf_fd_avail(buf: &ibuf) -> c_int {
    (buf.fd >= 0 as c_int) as c_int
}

pub fn ibuf_fd_get(buf: &mut ibuf) -> c_int {
    if buf.fd < 0 as c_int {
        return -(1 as c_int);
    }
    let fd = buf.fd;
    buf.fd = -(1 as c_int);
    fd
}

pub unsafe fn ibuf_fd_set(buf: &mut ibuf, fd: c_int) {
    unsafe {
        if ibuf_on_stack(&*buf) {
            abort();
        }
        if buf.fd >= 0 as c_int {
            close(buf.fd);
        }
        buf.fd = -(1 as c_int);
        if fd >= 0 as c_int {
            buf.fd = fd;
        }
    }
}

pub fn msgbuf_new() -> Box<msgbuf> {
    Box::new(msgbuf {
        bufs: ibufqueue {
            bufs: std::collections::VecDeque::new(),
        },
        rbufs: ibufqueue {
            bufs: std::collections::VecDeque::new(),
        },
        rbuf: None,
        rpmsg: None,
        readhdr: None,
        read_limit: None,
        roff: 0,
        hdrsize: 0,
    })
}

pub unsafe fn msgbuf_new_reader(
    hdrsz: size_t,
    readhdr: msgbuf_read_cb,
    read_limit: Option<uint32_t>,
) -> Option<Box<msgbuf>> {
    unsafe {
        if hdrsz == 0 as size_t || hdrsz > (IBUF_READ_SIZE / 2 as c_int) as size_t {
            return ibuf_fail(EINVAL, None);
        }
        let mut msgbuf = msgbuf_new();
        msgbuf.rbuf = Some(vec![0; IBUF_READ_SIZE as usize].into_boxed_slice());
        msgbuf.hdrsize = hdrsz;
        msgbuf.readhdr = readhdr;
        msgbuf.read_limit = read_limit;
        Some(msgbuf)
    }
}

pub unsafe fn msgbuf_queuelen(msgbuf: &msgbuf) -> uint32_t {
    unsafe { ibufq_queuelen(&msgbuf.bufs) }
}

pub unsafe fn msgbuf_clear(msgbuf: &mut msgbuf) {
    unsafe {
        ibufq_flush(&mut msgbuf.bufs);
        ibufq_flush(&mut msgbuf.rbufs);
        msgbuf.roff = 0 as size_t;
        if let Some(rpmsg) = msgbuf.rpmsg.take() {
            ibuf_free(rpmsg);
        }
    }
}

pub unsafe fn msgbuf_get(msgbuf: &mut msgbuf) -> Option<Box<ibuf>> {
    unsafe { ibufq_pop(&mut msgbuf.rbufs) }
}

/// The borrowed ranges a write can hand the kernel, and the optional descriptor.
/// A message write stops before any descriptor after the first buffer.
fn ibufq_iovecs(bufq: &ibufqueue, one_fd_only: bool) -> (Vec<IoSlice<'_>>, Option<c_int>) {
    let mut iov = Vec::with_capacity(bufq.bufs.len().min(IOV_MAX as usize));
    let mut with_fd = None;
    for buf in bufq.bufs.iter().take(IOV_MAX as usize) {
        if one_fd_only && !iov.is_empty() && buf.fd != -1 {
            break;
        }
        iov.push(IoSlice::new(ibuf_data(buf)));
        if one_fd_only && buf.fd != -1 {
            with_fd = Some(buf.fd);
        }
    }
    (iov, with_fd)
}

/// What a failed write should answer: `None` to try again, or the value to
/// hand back. A socket with no room left is not an error — the queue keeps
/// what it could not send.
fn ibuf_retry_write() -> Option<c_int> {
    unsafe {
        match *__errno_location() {
            EINTR => None,
            EAGAIN | ENOBUFS => Some(0 as c_int),
            _ => Some(-(1 as c_int)),
        }
    }
}

/// The same for a failed read, which answers that the socket is still open
/// when there is nothing on it yet. Upstream does not read `ENOBUFS` here.
fn ibuf_retry_read() -> Option<c_int> {
    unsafe {
        match *__errno_location() {
            EINTR => None,
            EAGAIN => Some(1 as c_int),
            _ => Some(-(1 as c_int)),
        }
    }
}

pub unsafe fn ibuf_write(fd: c_int, msgbuf: &mut msgbuf) -> c_int {
    unsafe {
        let (iov, _) = ibufq_iovecs(&msgbuf.bufs, false);
        if iov.is_empty() {
            return 0 as c_int;
        }
        let n = loop {
            let n = writev(fd, iov.as_ptr().cast(), iov.len() as c_int);
            if n != -(1 as c_int) as ssize_t {
                break n;
            }
            if let Some(answer) = ibuf_retry_write() {
                return answer;
            }
        };
        drop(iov);
        msgbuf_drain(msgbuf, n as size_t);
        0 as c_int
    }
}

pub unsafe fn msgbuf_write(fd: c_int, msgbuf: &mut msgbuf) -> c_int {
    unsafe {
        let mut cmsgbuf = ControlStorage::default();
        let (iov, with_fd) = ibufq_iovecs(&msgbuf.bufs, true);
        if iov.is_empty() {
            return 0 as c_int;
        }
        if let Some(fd) = with_fd {
            let mut cmsg = ControlMessage::from_control_message_header(
                &mut cmsgbuf.buf,
                crate::CONTROL_MESSAGE_HEADER_SIZE + size_of::<c_int>(),
                SOL_SOCKET,
                SCM_RIGHTS as c_int,
            )
            .expect("storage for one descriptor");
            cmsg.control_message_data_mut()
                .copy_from_slice(&fd.to_ne_bytes());
        }
        let control = if with_fd.is_some() {
            &cmsgbuf.buf[..]
        } else {
            &[]
        };
        let n = loop {
            let n = crate::message_header::send_socket_message(fd, &iov, control, 0);
            if n != -(1 as c_int) as ssize_t {
                break n;
            }
            if let Some(answer) = ibuf_retry_write() {
                return answer;
            }
        };
        drop(iov);
        if let Some(fd) = with_fd {
            close(fd);
            msgbuf
                .bufs
                .bufs
                .front_mut()
                .expect("the transmitted buffer")
                .fd = -1;
        }
        msgbuf_drain(msgbuf, n as size_t);
        0 as c_int
    }
}

unsafe fn ibuf_read_process(msgbuf: &mut msgbuf, fd: c_int) -> c_int {
    unsafe {
        let mut fd = fd;
        let mut rbuf: ibuf = ibuf_empty();
        let mut msg: ibuf = ibuf_empty();
        let rdata = if msgbuf.roff == 0 {
            &[]
        } else {
            &msgbuf.rbuf.as_deref().expect("a reader has a buffer")[..msgbuf.roff]
        };
        ibuf_from_buffer(&mut rbuf, rdata);
        let taken = loop {
            if msgbuf.rpmsg.is_none() {
                if ibuf_size(&rbuf) < msgbuf.hdrsize {
                    break true;
                }
                let hdata = if msgbuf.hdrsize == 0 {
                    &[]
                } else {
                    &ibuf_data(&rbuf)[..msgbuf.hdrsize]
                };
                ibuf_from_buffer(&mut msg, hdata);
                let readhdr = msgbuf
                    .readhdr
                    .clone()
                    .expect("a message reader has a header callback");
                msgbuf.rpmsg = readhdr(&mut msg, msgbuf.read_limit, &mut fd);
                if msgbuf.rpmsg.is_none() {
                    break false;
                }
            }
            let rpmsg = msgbuf
                .rpmsg
                .as_deref_mut()
                .expect("a message being read into");
            let sz = ibuf_left(rpmsg).min(ibuf_size(&rbuf));
            let Some(new) = ibuf_get_ibuf(&mut rbuf, sz) else {
                break false;
            };
            msg = new;
            if ibuf_add_ibuf(rpmsg, &msg) == -(1 as c_int) {
                break false;
            }
            if ibuf_left(rpmsg) == 0 as size_t {
                let rpmsg = msgbuf.rpmsg.take().expect("the message just filled");
                ibufq_push(&mut msgbuf.rbufs, rpmsg);
            }
            if ibuf_size(&rbuf) == 0 as size_t {
                break true;
            }
        };
        if taken {
            if ibuf_size(&rbuf) > 0 as size_t {
                msgbuf.rbuf.as_deref_mut().expect("a reader has a buffer")[..ibuf_size(&rbuf)]
                    .copy_from_slice(ibuf_data(&rbuf));
            }
            msgbuf.roff = ibuf_size(&rbuf);
        }
        if fd != -(1 as c_int) {
            close(fd);
        }
        if taken { 1 as c_int } else { -(1 as c_int) }
    }
}

/// Where the next read goes and how much room is left for it.
fn msgbuf_room(msgbuf: &mut msgbuf) -> &mut [u8] {
    &mut msgbuf.rbuf.as_deref_mut().expect("a reader has a buffer")[msgbuf.roff..]
}

pub unsafe fn ibuf_read(fd: c_int, msgbuf: &mut msgbuf) -> c_int {
    unsafe {
        if msgbuf.rbuf.is_none() {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        let mut iov = IoSliceMut::new(msgbuf_room(msgbuf));
        let n = loop {
            let n = readv(fd, (&raw mut iov).cast(), 1 as c_int);
            if n != -(1 as c_int) as ssize_t {
                break n;
            }
            if let Some(answer) = ibuf_retry_read() {
                return answer;
            }
        };
        if n == 0 as ssize_t {
            return 0 as c_int;
        }
        msgbuf.roff = msgbuf.roff.wrapping_add(n as size_t);
        ibuf_read_process(msgbuf, -(1 as c_int))
    }
}

pub unsafe fn msgbuf_read(fd: c_int, msgbuf: &mut msgbuf) -> c_int {
    unsafe {
        if msgbuf.rbuf.is_none() {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        let mut cmsgbuf = ControlStorage::default();
        let mut iov = [IoSliceMut::new(msgbuf_room(msgbuf))];
        let mut msg = SocketMessage::from_message_header(None, &mut iov, &mut cmsgbuf.buf, 0);
        let n = loop {
            let n = msg.receive(fd, 0);
            if n != -(1 as c_int) as ssize_t {
                break n;
            }
            // A message too big for the room set aside for it is read again, the
            // same way an interrupted one is.
            if *__errno_location() == EMSGSIZE {
                continue;
            }
            if let Some(answer) = ibuf_retry_read() {
                return answer;
            }
        };
        let control_length = msg.message_control_length();
        if n == 0 as ssize_t {
            return 0 as c_int;
        }
        msgbuf.roff = msgbuf.roff.wrapping_add(n as size_t);
        let mut fdpass = -(1 as c_int);
        for cmsg in ControlMessages::new(&mut cmsgbuf.buf[..control_length]) {
            if cmsg.control_message_level() == SOL_SOCKET
                && cmsg.control_message_kind() == SCM_RIGHTS as c_int
            {
                for (i, data) in cmsg
                    .control_message_data()
                    .as_chunks::<{ size_of::<c_int>() }>()
                    .0
                    .iter()
                    .enumerate()
                {
                    let bytes: [u8; 4] = data[..].try_into().unwrap();
                    let f = c_int::from_ne_bytes(bytes);
                    if i == 0 {
                        fdpass = f;
                    } else {
                        close(f);
                    }
                }
            }
        }
        ibuf_read_process(msgbuf, fdpass)
    }
}

unsafe fn msgbuf_drain(msgbuf: &mut msgbuf, mut n: size_t) {
    unsafe {
        let bufq = &mut msgbuf.bufs;
        loop {
            let Some(buf) = bufq.bufs.front_mut() else {
                return;
            };
            let size = ibuf_size(buf);
            if n < size {
                buf.rpos = buf.rpos.wrapping_add(n);
                return;
            }
            n = n.wrapping_sub(size);
            let buf = bufq.bufs.pop_front().expect("the buffer just looked at");
            ibuf_free(buf);
        }
    }
}

pub unsafe fn ibufq_pop(bufq: &mut ibufqueue) -> Option<Box<ibuf>> {
    bufq.bufs.pop_front()
}

pub unsafe fn ibufq_push(bufq: &mut ibufqueue, buf: Box<ibuf>) {
    if ibuf_on_stack(&buf) {
        unsafe { abort() };
    }
    bufq.bufs.push_back(buf);
}

pub unsafe fn ibufq_queuelen(bufq: &ibufqueue) -> uint32_t {
    bufq.bufs.len() as uint32_t
}

pub unsafe fn ibufq_flush(bufq: &mut ibufqueue) {
    while let Some(buf) = bufq.bufs.pop_front() {
        unsafe { ibuf_free(buf) };
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_imsg_buffer.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::{
    ibuf_add_n8, ibuf_add_n16, ibuf_add_n32, ibuf_add_n64, ibuf_add_strbuf, ibuf_from_ibuf,
    ibuf_get_n16, ibuf_get_n32, ibuf_get_n64, ibuf_get_strbuf, ibuf_truncate, ibufq_free,
    ibufq_new,
};

#[cfg(test)]
pub(crate) use tests::ibuf_rewind;

#[cfg(test)]
pub const EOVERFLOW: c_int = 75 as c_int;
#[cfg(test)]
pub const UINT8_MAX: c_int = 255 as c_int;
#[cfg(test)]
pub const UINT16_MAX: c_int = 65535 as c_int;
