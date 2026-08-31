use crate::compat::htonll::htonll;
use crate::compat::ntohll::ntohll;
use crate::ffi::{__errno_location, abort, close, readv, recvmsg, sendmsg, strlcpy, writev};
pub use crate::types::*;
use ::core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};
use ::core::ptr::{copy_nonoverlapping, null_mut, write_bytes};
use ::std::ffi::CString;
use bytes::{Buf as BytesBuf, BufMut as BytesBufMut, BytesMut};
pub type __caddr_t = *mut c_char;
pub type caddr_t = __caddr_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msghdr {
    pub msg_name: *mut c_void,
    pub msg_namelen: socklen_t,
    pub msg_iov: *mut iovec,
    pub msg_iovlen: size_t,
    pub msg_control: *mut c_void,
    pub msg_controllen: size_t,
    pub msg_flags: c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmsghdr {
    pub cmsg_len: size_t,
    pub cmsg_level: c_int,
    pub cmsg_type: c_int,
    pub __cmsg_data: [c_uchar; 0],
}
pub type scm_type = c_uint;
pub const SCM_RIGHTS: scm_type = 1;
/// A queue of buffers waiting to be written or read, front to back. The
/// queue owns them: whatever comes off it is the caller's to give up.
pub struct ibufqueue {
    pub bufs: ::std::collections::VecDeque<Box<ibuf>>,
}
#[repr(C)]
pub struct msgbuf {
    pub bufs: ibufqueue,
    pub rbufs: ibufqueue,
    pub rbuf: Option<Box<[c_char]>>,
    /// The message being read in, held until the last of it has arrived.
    pub rpmsg: Option<Box<ibuf>>,
    pub readhdr: Option<unsafe fn(*mut ibuf, *mut imsgbuf, *mut c_int) -> Option<Box<ibuf>>>,
    pub rarg: *mut imsgbuf,
    pub roff: size_t,
    pub hdrsize: size_t,
}

impl msgbuf {
    /// Where the read buffer starts, or a null pointer when this is not a
    /// reader and has none.
    fn rbuf_ptr(&mut self) -> *mut c_char {
        match &mut self.rbuf {
            Some(rbuf) => rbuf.as_mut_ptr(),
            None => null_mut(),
        }
    }
}

impl Drop for msgbuf {
    fn drop(&mut self) {
        unsafe {
            msgbuf_clear(self);
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union cmsgbuf_storage {
    pub hdr: cmsghdr,
    pub buf: [c_char; 24],
}

/// How much room the control message the fd rides in takes.
const CMSGBUF_SIZE: usize = 24;

/// The expanded `CMSG_FIRSTHDR`: the first control message of `msg`, if the
/// room set aside for one is big enough to hold a header.
unsafe fn cmsg_firsthdr(msg: *const msghdr) -> *mut cmsghdr {
    unsafe {
        if (*msg).msg_controllen >= ::core::mem::size_of::<cmsghdr>() {
            (*msg).msg_control as *mut cmsghdr
        } else {
            null_mut::<cmsghdr>()
        }
    }
}

/// The expanded `CMSG_ALIGN`: a length rounded up to a word.
const fn cmsg_align(len: usize) -> usize {
    let word = ::core::mem::size_of::<size_t>();
    len.wrapping_add(word).wrapping_sub(1) & !word.wrapping_sub(1)
}

/// The expanded `CMSG_LEN`: the length a control message carrying `len` bytes
/// of data declares.
const fn cmsg_len(len: usize) -> usize {
    cmsg_align(::core::mem::size_of::<cmsghdr>()).wrapping_add(len)
}

/// The bytes of a control message, which is where a passed descriptor sits.
unsafe fn cmsg_data(cmsg: *mut cmsghdr) -> *mut c_int {
    unsafe { &raw mut (*cmsg).__cmsg_data as *mut c_uchar as *mut c_int }
}

#[inline]
unsafe fn __cmsg_nxthdr(mut __mhdr: *mut msghdr, mut __cmsg: *mut cmsghdr) -> *mut cmsghdr {
    unsafe {
        let mut __msg_control_ptr: *mut c_uchar = (*__mhdr).msg_control as *mut c_uchar;
        let mut __cmsg_ptr: *mut c_uchar = __cmsg as *mut c_uchar;
        let mut __size_needed: size_t = (::core::mem::size_of::<cmsghdr>() as size_t).wrapping_add(
            (::core::mem::size_of::<size_t>() as size_t).wrapping_sub(
                (*__cmsg).cmsg_len & (::core::mem::size_of::<size_t>() as size_t).wrapping_sub(1),
            ) & (::core::mem::size_of::<size_t>() as size_t).wrapping_sub(1),
        );
        if (*__cmsg).cmsg_len < ::core::mem::size_of::<cmsghdr>() {
            return null_mut::<cmsghdr>();
        }
        if (__msg_control_ptr
            .add((*__mhdr).msg_controllen)
            .offset_from(__cmsg_ptr) as ::core::ffi::c_long as size_t)
            < __size_needed
            || (__msg_control_ptr
                .add((*__mhdr).msg_controllen)
                .offset_from(__cmsg_ptr) as ::core::ffi::c_long as size_t)
                .wrapping_sub(__size_needed)
                < (*__cmsg).cmsg_len
        {
            return null_mut::<cmsghdr>();
        }
        __cmsg = (__cmsg as *mut c_uchar).add(cmsg_align((*__cmsg).cmsg_len)) as *mut cmsghdr;
        __cmsg
    }
}
pub const SOL_SOCKET: c_int = 1 as c_int;
pub const __IOV_MAX: c_int = 1024 as c_int;
pub const IOV_MAX: c_int = __IOV_MAX;
pub const EINTR: c_int = 4 as c_int;
pub const EAGAIN: c_int = 11 as c_int;
pub const EINVAL: c_int = 22 as c_int;
pub const ERANGE: c_int = 34 as c_int;
pub const EBADMSG: c_int = 74 as c_int;
pub const EOVERFLOW: c_int = 75 as c_int;
pub const EMSGSIZE: c_int = 90 as c_int;
pub const ENOBUFS: c_int = 105 as c_int;
pub const UINT8_MAX: c_int = 255 as c_int;
pub const UINT16_MAX: c_int = 65535 as c_int;
pub const UINT32_MAX: c_uint = 4294967295 as c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
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
unsafe fn ibuf_on_stack(buf: *const ibuf) -> bool {
    unsafe { (*buf).borrowed }
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
            return unsafe {
                bytes::buf::UninitSlice::from_raw_parts_mut(self.buf.as_mut_ptr(), 0)
            };
        }
        let needed = self.wpos + remaining.min(IBUF_BUF_MUT_CHUNK);
        if needed > self.buf.capacity() {
            self.buf.reserve(needed - self.buf.len());
        }
        let available = remaining.min(self.buf.capacity() - self.wpos);
        unsafe {
            bytes::buf::UninitSlice::from_raw_parts_mut(
                self.buf.as_mut_ptr().add(self.wpos),
                available,
            )
        }
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

pub unsafe fn ibuf_reserve(buf: *mut ibuf, len: size_t) -> *mut c_uchar {
    unsafe {
        if len > (SIZE_MAX as size_t).wrapping_sub((*buf).wpos) {
            return ibuf_fail(ERANGE, null_mut::<c_uchar>());
        }
        if ibuf_on_stack(buf) {
            return ibuf_fail(EINVAL, null_mut::<c_uchar>());
        }
        let want = (*buf).wpos.wrapping_add(len);
        if want > (*buf).size {
            if want > (*buf).max {
                return ibuf_fail(ERANGE, null_mut::<c_uchar>());
            }
            let len = want.wrapping_sub((*buf).size);
            (*buf).buf.reserve(len);
            write_bytes((*buf).buf.spare_capacity_mut().as_mut_ptr(), 0, len);
            BytesBufMut::advance_mut(&mut (*buf).buf, len);
            (*buf).size = want;
        }
        let b = (*buf).buf.as_mut_ptr().add((*buf).wpos);
        (*buf).wpos = want;
        b
    }
}

pub unsafe fn ibuf_add(buf: *mut ibuf, data: *const c_uchar, len: size_t) -> c_int {
    unsafe {
        if len == 0 as size_t {
            return 0 as c_int;
        }
        let b = ibuf_reserve(buf, len);
        if b.is_null() {
            return -(1 as c_int);
        }
        copy_nonoverlapping(data, b, len);
        0 as c_int
    }
}

pub unsafe fn ibuf_add_ibuf(buf: *mut ibuf, from: *const ibuf) -> c_int {
    unsafe { ibuf_add(buf, ibuf_data(from), ibuf_size(from)) }
}

/// Puts the bytes of a number at the end of the buffer.
unsafe fn ibuf_add_bytes(buf: *mut ibuf, bytes: &[u8]) -> c_int {
    unsafe { ibuf_add(buf, bytes.as_ptr(), bytes.len()) }
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

pub unsafe fn ibuf_add_n8(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT8_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint8_t).to_ne_bytes())
    }
}

pub unsafe fn ibuf_add_n16(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint16_t).swap_bytes().to_ne_bytes())
    }
}

pub unsafe fn ibuf_add_n32(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint32_t).swap_bytes().to_ne_bytes())
    }
}

pub unsafe fn ibuf_add_n64(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe { ibuf_add_bytes(buf, &htonll(value).to_ne_bytes()) }
}

pub unsafe fn ibuf_add_h16(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint16_t).to_ne_bytes())
    }
}

pub unsafe fn ibuf_add_h32(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint32_t).to_ne_bytes())
    }
}

pub unsafe fn ibuf_add_h64(buf: *mut ibuf, value: uint64_t) -> c_int {
    unsafe { ibuf_add_bytes(buf, &value.to_ne_bytes()) }
}

pub unsafe fn ibuf_add_zero(buf: *mut ibuf, len: size_t) -> c_int {
    unsafe {
        if len == 0 as size_t {
            return 0 as c_int;
        }
        let b = ibuf_reserve(buf, len);
        if b.is_null() {
            return -(1 as c_int);
        }
        write_bytes(b, 0, len);
        0 as c_int
    }
}

pub unsafe fn ibuf_add_strbuf(buf: *mut ibuf, str: *const c_char, len: size_t) -> c_int {
    unsafe {
        let b = ibuf_reserve(buf, len) as *mut c_char;
        if b.is_null() {
            return -(1 as c_int);
        }
        let n = strlcpy(b, str, len) as size_t;
        if n >= len {
            return ibuf_fail(EOVERFLOW, -(1 as c_int));
        }
        write_bytes(b.add(n) as *mut u8, 0, len.wrapping_sub(n));
        0 as c_int
    }
}

pub unsafe fn ibuf_seek(buf: *mut ibuf, pos: size_t, len: size_t) -> *mut c_uchar {
    unsafe {
        if ibuf_size(buf) < pos
            || (SIZE_MAX as size_t).wrapping_sub(pos) < len
            || ibuf_size(buf) < pos.wrapping_add(len)
        {
            return ibuf_fail(ERANGE, null_mut::<c_uchar>());
        }
        (*buf).buf.as_ptr().add((*buf).rpos).add(pos) as *mut c_uchar
    }
}

pub unsafe fn ibuf_set(buf: *mut ibuf, pos: size_t, data: *const c_uchar, len: size_t) -> c_int {
    unsafe {
        let b = ibuf_seek(buf, pos, len);
        if b.is_null() {
            return -(1 as c_int);
        }
        if len == 0 as size_t {
            return 0 as c_int;
        }
        copy_nonoverlapping(data, b, len);
        0 as c_int
    }
}

/// Writes the bytes of a number over a place already inside the buffer.
unsafe fn ibuf_set_bytes(buf: *mut ibuf, pos: size_t, bytes: &[u8]) -> c_int {
    unsafe { ibuf_set(buf, pos, bytes.as_ptr(), bytes.len()) }
}

pub unsafe fn ibuf_set_n8(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT8_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_set_bytes(buf, pos, &(value as uint8_t).to_ne_bytes())
    }
}

pub unsafe fn ibuf_set_n16(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_set_bytes(buf, pos, &(value as uint16_t).swap_bytes().to_ne_bytes())
    }
}

pub unsafe fn ibuf_set_n32(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_set_bytes(buf, pos, &(value as uint32_t).swap_bytes().to_ne_bytes())
    }
}

pub unsafe fn ibuf_set_n64(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe { ibuf_set_bytes(buf, pos, &htonll(value).to_ne_bytes()) }
}

pub unsafe fn ibuf_set_h16(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_set_bytes(buf, pos, &(value as uint16_t).to_ne_bytes())
    }
}

pub unsafe fn ibuf_set_h32(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_set_bytes(buf, pos, &(value as uint32_t).to_ne_bytes())
    }
}

pub unsafe fn ibuf_set_h64(buf: *mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe { ibuf_set_bytes(buf, pos, &value.to_ne_bytes()) }
}

pub unsafe fn ibuf_set_maxsize(buf: *mut ibuf, max: size_t) -> c_int {
    unsafe {
        if ibuf_on_stack(buf) {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        if max > (*buf).max {
            return ibuf_fail(ERANGE, -(1 as c_int));
        }
        (*buf).max = max;
        0 as c_int
    }
}

pub unsafe fn ibuf_data(buf: *const ibuf) -> *mut c_uchar {
    unsafe { (*buf).buf.as_ptr().add((*buf).rpos) as *mut c_uchar }
}

pub unsafe fn ibuf_size(buf: *const ibuf) -> size_t {
    unsafe { (*buf).wpos.wrapping_sub((*buf).rpos) }
}

pub unsafe fn ibuf_left(buf: *const ibuf) -> size_t {
    unsafe {
        if ibuf_on_stack(buf) {
            return 0 as size_t;
        }
        (*buf).max.wrapping_sub((*buf).wpos)
    }
}

pub unsafe fn ibuf_truncate(buf: *mut ibuf, len: size_t) -> c_int {
    unsafe {
        if ibuf_size(buf) >= len {
            (*buf).wpos = (*buf).rpos.wrapping_add(len);
            return 0 as c_int;
        }
        if ibuf_on_stack(buf) {
            return ibuf_fail(ERANGE, -(1 as c_int));
        }
        ibuf_add_zero(buf, len.wrapping_sub(ibuf_size(buf)))
    }
}

pub unsafe fn ibuf_rewind(buf: *mut ibuf) {
    unsafe {
        (*buf).rpos = 0 as size_t;
    }
}

pub unsafe fn ibuf_close(msgbuf: *mut msgbuf, buf: Box<ibuf>) {
    unsafe {
        ibufq_push(&raw mut (*msgbuf).bufs, buf);
    }
}

/// Replaces `buf` with an owned copy of a temporary byte range.
pub unsafe fn ibuf_from_buffer(buf: *mut ibuf, data: *mut c_uchar, len: size_t) {
    unsafe {
        let bytes = if len == 0 {
            BytesMut::new()
        } else {
            BytesMut::from(::core::slice::from_raw_parts(data, len))
        };
        *buf = ibuf {
            buf: bytes,
            size: len,
            wpos: len,
            borrowed: true,
            ..ibuf::default()
        };
    }
}

pub unsafe fn ibuf_from_ibuf(buf: *mut ibuf, from: *const ibuf) {
    unsafe {
        ibuf_from_buffer(buf, ibuf_data(from), ibuf_size(from));
    }
}

pub unsafe fn ibuf_get(buf: *mut ibuf, data: *mut c_uchar, len: size_t) -> c_int {
    unsafe {
        if ibuf_size(buf) < len {
            return ibuf_fail(EBADMSG, -(1 as c_int));
        }
        copy_nonoverlapping(ibuf_data(buf), data, len);
        (*buf).rpos = (*buf).rpos.wrapping_add(len);
        0 as c_int
    }
}

pub unsafe fn ibuf_get_ibuf(buf: *mut ibuf, len: size_t, new: *mut ibuf) -> c_int {
    unsafe {
        if ibuf_size(buf) < len {
            return ibuf_fail(EBADMSG, -(1 as c_int));
        }
        ibuf_from_buffer(new, ibuf_data(buf), len);
        (*buf).rpos = (*buf).rpos.wrapping_add(len);
        0 as c_int
    }
}

/// Takes a number out of the buffer as it lies there, without changing its
/// byte order.
unsafe fn ibuf_get_number<T>(buf: *mut ibuf, value: *mut T) -> c_int {
    unsafe {
        ibuf_get(
            buf,
            value as *mut c_uchar,
            ::core::mem::size_of::<T>() as size_t,
        )
    }
}

pub unsafe fn ibuf_get_h16(buf: *mut ibuf, value: *mut uint16_t) -> c_int {
    unsafe { ibuf_get_number(buf, value) }
}

pub unsafe fn ibuf_get_h32(buf: *mut ibuf, value: *mut uint32_t) -> c_int {
    unsafe { ibuf_get_number(buf, value) }
}

pub unsafe fn ibuf_get_h64(buf: *mut ibuf, value: *mut uint64_t) -> c_int {
    unsafe { ibuf_get_number(buf, value) }
}

pub unsafe fn ibuf_get_n8(buf: *mut ibuf, value: *mut uint8_t) -> c_int {
    unsafe { ibuf_get_number(buf, value) }
}

pub unsafe fn ibuf_get_n16(buf: *mut ibuf, value: *mut uint16_t) -> c_int {
    unsafe {
        let rv = ibuf_get_number(buf, value);
        // Upstream turns the bytes round whether or not there were any to read,
        // so a failed read still writes over what the caller had.
        *value = (*value).swap_bytes();
        rv
    }
}

pub unsafe fn ibuf_get_n32(buf: *mut ibuf, value: *mut uint32_t) -> c_int {
    unsafe {
        let rv = ibuf_get_number(buf, value);
        *value = (*value).swap_bytes();
        rv
    }
}

pub unsafe fn ibuf_get_n64(buf: *mut ibuf, value: *mut uint64_t) -> c_int {
    unsafe {
        let rv = ibuf_get_number(buf, value);
        *value = ntohll(*value);
        rv
    }
}

pub unsafe fn ibuf_get_string(buf: *mut ibuf, len: size_t) -> Option<CString> {
    unsafe {
        if ibuf_size(buf) < len {
            ibuf_fail(EBADMSG, null_mut::<c_char>());
            return None;
        }
        let bytes = ::core::slice::from_raw_parts(ibuf_data(buf) as *const u8, len);
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        let string = CString::new(&bytes[..end]).expect("ibuf string has no NUL");
        (*buf).rpos = (*buf).rpos.wrapping_add(len);
        Some(string)
    }
}

pub unsafe fn ibuf_get_strbuf(buf: *mut ibuf, str: *mut c_char, len: size_t) -> c_int {
    unsafe {
        if len == 0 as size_t {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        if ibuf_get(buf, str as *mut c_uchar, len) == -(1 as c_int) {
            return -(1 as c_int);
        }
        let last = str.add(len.wrapping_sub(1));
        if *last as c_int != '\0' as i32 {
            *last = '\0' as i32 as c_char;
            return ibuf_fail(EOVERFLOW, -(1 as c_int));
        }
        0 as c_int
    }
}

pub unsafe fn ibuf_skip(buf: *mut ibuf, len: size_t) -> c_int {
    unsafe {
        if ibuf_size(buf) < len {
            return ibuf_fail(EBADMSG, -(1 as c_int));
        }
        (*buf).rpos = (*buf).rpos.wrapping_add(len);
        0 as c_int
    }
}

/// Gives up a buffer the caller owns, closing whatever descriptor it carries.
pub unsafe fn ibuf_free(mut buf: Box<ibuf>) {
    unsafe {
        let save_errno = *__errno_location();
        if ibuf_on_stack(&raw const *buf) {
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

pub unsafe fn ibuf_fd_avail(buf: *mut ibuf) -> c_int {
    unsafe { ((*buf).fd >= 0 as c_int) as c_int }
}

pub unsafe fn ibuf_fd_get(buf: *mut ibuf) -> c_int {
    unsafe {
        if (*buf).fd < 0 as c_int {
            return -(1 as c_int);
        }
        let fd = (*buf).fd;
        (*buf).fd = -(1 as c_int);
        fd
    }
}

pub unsafe fn ibuf_fd_set(buf: *mut ibuf, fd: c_int) {
    unsafe {
        if ibuf_on_stack(buf) {
            abort();
        }
        if (*buf).fd >= 0 as c_int {
            close((*buf).fd);
        }
        (*buf).fd = -(1 as c_int);
        if fd >= 0 as c_int {
            (*buf).fd = fd;
        }
    }
}

pub fn msgbuf_new() -> Box<msgbuf> {
    Box::new(msgbuf {
        bufs: ibufqueue {
            bufs: ::std::collections::VecDeque::new(),
        },
        rbufs: ibufqueue {
            bufs: ::std::collections::VecDeque::new(),
        },
        rbuf: None,
        rpmsg: None,
        readhdr: None,
        rarg: null_mut(),
        roff: 0,
        hdrsize: 0,
    })
}

pub unsafe fn msgbuf_new_reader(
    hdrsz: size_t,
    readhdr: Option<unsafe fn(*mut ibuf, *mut imsgbuf, *mut c_int) -> Option<Box<ibuf>>>,
    arg: *mut imsgbuf,
) -> Option<Box<msgbuf>> {
    unsafe {
        if hdrsz == 0 as size_t || hdrsz > (IBUF_READ_SIZE / 2 as c_int) as size_t {
            return ibuf_fail(EINVAL, None);
        }
        let mut msgbuf = msgbuf_new();
        msgbuf.rbuf = Some(vec![0; IBUF_READ_SIZE as usize].into_boxed_slice());
        msgbuf.hdrsize = hdrsz;
        msgbuf.readhdr = readhdr;
        msgbuf.rarg = arg;
        Some(msgbuf)
    }
}

pub unsafe fn msgbuf_queuelen(msgbuf: *mut msgbuf) -> uint32_t {
    unsafe { ibufq_queuelen(&raw mut (*msgbuf).bufs) }
}

pub unsafe fn msgbuf_clear(msgbuf: *mut msgbuf) {
    unsafe {
        ibufq_flush(&raw mut (*msgbuf).bufs);
        ibufq_flush(&raw mut (*msgbuf).rbufs);
        (*msgbuf).roff = 0 as size_t;
        if let Some(rpmsg) = (*msgbuf).rpmsg.take() {
            ibuf_free(rpmsg);
        }
    }
}

pub unsafe fn msgbuf_get(msgbuf: *mut msgbuf) -> Option<Box<ibuf>> {
    unsafe { ibufq_pop(&raw mut (*msgbuf).rbufs) }
}

pub unsafe fn msgbuf_concat(msgbuf: *mut msgbuf, from: *mut ibufqueue) {
    unsafe {
        ibufq_concat(&raw mut (*msgbuf).bufs, from);
    }
}

/// Every buffer waiting in the queue, front to back.
fn ibufq_iter(bufq: *const ibufqueue) -> impl Iterator<Item = *mut ibuf> {
    let bufs: Vec<*mut ibuf> = unsafe {
        (*bufq)
            .bufs
            .iter()
            .map(|buf| &raw const **buf as *mut ibuf)
            .collect()
    };
    bufs.into_iter()
}

/// What a write of the queue may hand the kernel in one call, and how much of
/// the queue it covers. Upstream stops at the iovec limit either way; a
/// message write also stops at the second buffer carrying a descriptor,
/// because only one rides along.
unsafe fn ibufq_iovecs(
    bufq: *const ibufqueue,
    iov: &mut [iovec; IOV_MAX as usize],
    one_fd_only: bool,
) -> (usize, *mut ibuf) {
    unsafe {
        let mut i = 0;
        let mut with_fd = null_mut::<ibuf>();
        for buf in ibufq_iter(bufq) {
            if i >= IOV_MAX as usize {
                break;
            }
            if one_fd_only && i > 0 && (*buf).fd != -(1 as c_int) {
                break;
            }
            iov[i] = iovec {
                iov_base: ibuf_data(buf) as *mut c_void,
                iov_len: ibuf_size(buf),
            };
            i += 1;
            if one_fd_only && (*buf).fd != -(1 as c_int) {
                with_fd = buf;
            }
        }
        (i, with_fd)
    }
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

pub unsafe fn ibuf_write(fd: c_int, msgbuf: *mut msgbuf) -> c_int {
    unsafe {
        let mut iov: [iovec; IOV_MAX as usize] = [iovec {
            iov_base: null_mut::<c_void>(),
            iov_len: 0,
        }; IOV_MAX as usize];
        let (i, _) = ibufq_iovecs(&raw const (*msgbuf).bufs, &mut iov, false);
        if i == 0 {
            return 0 as c_int;
        }
        let n = loop {
            let n = writev(fd, iov.as_mut_ptr(), i as c_int);
            if n != -(1 as c_int) as ssize_t {
                break n;
            }
            if let Some(answer) = ibuf_retry_write() {
                return answer;
            }
        };
        msgbuf_drain(msgbuf, n as size_t);
        0 as c_int
    }
}

pub unsafe fn msgbuf_write(fd: c_int, msgbuf: *mut msgbuf) -> c_int {
    unsafe {
        let mut iov: [iovec; IOV_MAX as usize] = [iovec {
            iov_base: null_mut::<c_void>(),
            iov_len: 0,
        }; IOV_MAX as usize];
        let mut cmsgbuf: cmsgbuf_storage = ::core::mem::zeroed();
        let (i, buf0) = ibufq_iovecs(&raw const (*msgbuf).bufs, &mut iov, true);
        if i == 0 {
            return 0 as c_int;
        }
        let mut msg: msghdr = ::core::mem::zeroed();
        msg.msg_iov = iov.as_mut_ptr();
        msg.msg_iovlen = i as size_t;
        if !buf0.is_null() {
            msg.msg_control = &raw mut cmsgbuf.buf as caddr_t as *mut c_void;
            msg.msg_controllen = CMSGBUF_SIZE as size_t;
            let cmsg = cmsg_firsthdr(&raw const msg);
            (*cmsg).cmsg_len = cmsg_len(::core::mem::size_of::<c_int>()) as size_t;
            (*cmsg).cmsg_level = SOL_SOCKET;
            (*cmsg).cmsg_type = SCM_RIGHTS as c_int;
            *cmsg_data(cmsg) = (*buf0).fd;
        }
        let n = loop {
            let n = sendmsg(fd, &raw const msg, 0 as c_int);
            if n != -(1 as c_int) as ssize_t {
                break n;
            }
            if let Some(answer) = ibuf_retry_write() {
                return answer;
            }
        };
        if !buf0.is_null() {
            close((*buf0).fd);
            (*buf0).fd = -(1 as c_int);
        }
        msgbuf_drain(msgbuf, n as size_t);
        0 as c_int
    }
}

unsafe fn ibuf_read_process(msgbuf: *mut msgbuf, fd: c_int) -> c_int {
    unsafe {
        let mut fd = fd;
        let mut rbuf: ibuf = ibuf_empty();
        let mut msg: ibuf = ibuf_empty();
        ibuf_from_buffer(
            &raw mut rbuf,
            (*msgbuf).rbuf_ptr() as *mut c_uchar,
            (*msgbuf).roff,
        );
        let taken = loop {
            if (*msgbuf).rpmsg.is_none() {
                if ibuf_size(&raw mut rbuf) < (*msgbuf).hdrsize {
                    break true;
                }
                ibuf_from_buffer(&raw mut msg, ibuf_data(&raw mut rbuf), (*msgbuf).hdrsize);
                (*msgbuf).rpmsg = (*msgbuf).readhdr.expect("non-null function pointer")(
                    &raw mut msg,
                    (*msgbuf).rarg,
                    &raw mut fd,
                );
                if (*msgbuf).rpmsg.is_none() {
                    break false;
                }
            }
            let rpmsg = (*msgbuf)
                .rpmsg
                .as_deref_mut()
                .map(|rpmsg| &raw mut *rpmsg)
                .expect("a message being read into");
            let sz = ibuf_left(rpmsg).min(ibuf_size(&raw mut rbuf));
            if ibuf_get_ibuf(&raw mut rbuf, sz, &raw mut msg) == -(1 as c_int)
                || ibuf_add_ibuf(rpmsg, &raw mut msg) == -(1 as c_int)
            {
                break false;
            }
            if ibuf_left(rpmsg) == 0 as size_t {
                let rpmsg = (*msgbuf).rpmsg.take().expect("the message just filled");
                ibufq_push(&raw mut (*msgbuf).rbufs, rpmsg);
            }
            if ibuf_size(&raw mut rbuf) == 0 as size_t {
                break true;
            }
        };
        if taken {
            if ibuf_size(&raw mut rbuf) > 0 as size_t {
                ::core::ptr::copy(
                    ibuf_data(&raw mut rbuf) as *const u8,
                    (*msgbuf).rbuf_ptr() as *mut u8,
                    ibuf_size(&raw mut rbuf),
                );
            }
            (*msgbuf).roff = ibuf_size(&raw mut rbuf);
        }
        if fd != -(1 as c_int) {
            close(fd);
        }
        if taken { 1 as c_int } else { -(1 as c_int) }
    }
}

/// Where the next read goes and how much room is left for it.
unsafe fn msgbuf_room(msgbuf: *mut msgbuf) -> iovec {
    unsafe {
        iovec {
            iov_base: (*msgbuf).rbuf_ptr().add((*msgbuf).roff) as *mut c_void,
            iov_len: (IBUF_READ_SIZE as size_t).wrapping_sub((*msgbuf).roff),
        }
    }
}

pub unsafe fn ibuf_read(fd: c_int, msgbuf: *mut msgbuf) -> c_int {
    unsafe {
        if (*msgbuf).rbuf.is_none() {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        let mut iov = msgbuf_room(msgbuf);
        let n = loop {
            let n = readv(fd, &raw mut iov, 1 as c_int);
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
        (*msgbuf).roff = (*msgbuf).roff.wrapping_add(n as size_t);
        ibuf_read_process(msgbuf, -(1 as c_int))
    }
}

pub unsafe fn msgbuf_read(fd: c_int, msgbuf: *mut msgbuf) -> c_int {
    unsafe {
        if (*msgbuf).rbuf.is_none() {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        let mut cmsgbuf: cmsgbuf_storage = ::core::mem::zeroed();
        let mut iov = msgbuf_room(msgbuf);
        let mut msg: msghdr = ::core::mem::zeroed();
        msg.msg_iov = &raw mut iov;
        msg.msg_iovlen = 1 as size_t;
        msg.msg_control = &raw mut cmsgbuf.buf as *mut c_void;
        msg.msg_controllen = CMSGBUF_SIZE as size_t;
        let n = loop {
            let n = recvmsg(fd, &raw mut msg, 0 as c_int);
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
        if n == 0 as ssize_t {
            return 0 as c_int;
        }
        (*msgbuf).roff = (*msgbuf).roff.wrapping_add(n as size_t);
        let mut fdpass = -(1 as c_int);
        let mut cmsg = cmsg_firsthdr(&raw const msg);
        while !cmsg.is_null() {
            if (*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SCM_RIGHTS as c_int {
                let data = cmsg_data(cmsg);
                let j = ((cmsg as *mut c_char)
                    .add((*cmsg).cmsg_len)
                    .offset_from(data as *mut c_char) as size_t)
                    .wrapping_div(::core::mem::size_of::<c_int>()) as c_int;
                // Only the first descriptor is kept; anything else that came with
                // the message is closed.
                for i in 0..j {
                    let f = *data.offset(i as isize);
                    if i == 0 as c_int {
                        fdpass = f;
                    } else {
                        close(f);
                    }
                }
            }
            cmsg = __cmsg_nxthdr(&raw mut msg, cmsg);
        }
        ibuf_read_process(msgbuf, fdpass)
    }
}

unsafe fn msgbuf_drain(msgbuf: *mut msgbuf, mut n: size_t) {
    unsafe {
        let bufq = &raw mut (*msgbuf).bufs;
        loop {
            let Some(buf) = (*bufq).bufs.front_mut() else {
                return;
            };
            let size = ibuf_size(&raw mut **buf);
            if n < size {
                buf.rpos = buf.rpos.wrapping_add(n);
                return;
            }
            n = n.wrapping_sub(size);
            let buf = (*bufq).bufs.pop_front().expect("the buffer just looked at");
            ibuf_free(buf);
        }
    }
}

pub fn ibufq_new() -> Box<ibufqueue> {
    Box::new(ibufqueue {
        bufs: ::std::collections::VecDeque::new(),
    })
}

pub unsafe fn ibufq_free(mut bufq: Box<ibufqueue>) {
    unsafe {
        ibufq_flush(&raw mut *bufq);
        drop(bufq);
    }
}

pub unsafe fn ibufq_pop(bufq: *mut ibufqueue) -> Option<Box<ibuf>> {
    unsafe { (*bufq).bufs.pop_front() }
}

pub unsafe fn ibufq_push(bufq: *mut ibufqueue, buf: Box<ibuf>) {
    unsafe {
        if ibuf_on_stack(&raw const *buf) {
            abort();
        }
        (*bufq).bufs.push_back(buf);
    }
}

pub unsafe fn ibufq_queuelen(bufq: *mut ibufqueue) -> uint32_t {
    unsafe { (*bufq).bufs.len() as uint32_t }
}

pub unsafe fn ibufq_concat(to: *mut ibufqueue, from: *mut ibufqueue) {
    unsafe {
        let moved = ::core::mem::take(&mut (*from).bufs);
        (*to).bufs.extend(moved);
    }
}

pub unsafe fn ibufq_flush(bufq: *mut ibufqueue) {
    unsafe {
        while let Some(buf) = (*bufq).bufs.pop_front() {
            ibuf_free(buf);
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_imsg_buffer.rs"]
mod tests;
