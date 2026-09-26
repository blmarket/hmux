use crate::src::ffi::libc::msghdr;
use crate::src::ffi::libc::{
    __errno_location, abort, close, memcpy, memset, readv, recvmsg, sendmsg, writev,
};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{ssize_t, uint32_t};
pub use crate::src::shared::errno::{EAGAIN, EBADMSG, EINTR, EINVAL, ENOMEM, ERANGE};
use crate::src::shared::limits::{SIZE_MAX, UINT32_MAX};
use super::message::IbufStorage;
use super::message::{ibuf, msgbuf, OwnedIbuf};
use crate::src::shared::socket::SOL_SOCKET;
use std::collections::VecDeque;
use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd};
use std::ptr::slice_from_raw_parts_mut;

pub type __caddr_t = *mut ::core::ffi::c_char;
pub type caddr_t = __caddr_t;

#[repr(C)]
pub struct ibufqueue {
    bufs: ibufqueue_bufs,
}

struct ibufqueue_bufs {
    entries: VecDeque<Box<OwnedIbuf>>,
}

impl ibufqueue_bufs {
    fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = *mut OwnedIbuf> + '_ {
        self.entries
            .iter()
            .map(|buf| (&**buf as *const OwnedIbuf).cast_mut())
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    unsafe fn push_back_raw(&mut self, buf: *mut OwnedIbuf) -> bool {
        let Some(buf) = OwnedIbuf::from_raw_owned(buf) else {
            return false;
        };
        self.entries.push_back(buf);
        true
    }

    fn push_back_owned(&mut self, buf: Box<OwnedIbuf>) -> bool {
        if !buf.is_owned() {
            return false;
        }
        self.entries.push_back(buf);
        true
    }

    fn pop_front_raw(&mut self) -> Option<*mut OwnedIbuf> {
        self.entries.pop_front().map(Box::into_raw)
    }

    fn pop_front_owned(&mut self) -> Option<Box<OwnedIbuf>> {
        self.entries.pop_front()
    }

    fn front(&self) -> Option<*mut OwnedIbuf> {
        self.entries
            .front()
            .map(|buf| (&**buf as *const OwnedIbuf).cast_mut())
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

impl ibufqueue {
    fn new() -> Self {
        Self {
            bufs: ibufqueue_bufs::new(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmsghdr {
    pub cmsg_len: size_t,
    pub cmsg_level: ::core::ffi::c_int,
    pub cmsg_type: ::core::ffi::c_int,
    pub __cmsg_data: [::core::ffi::c_uchar; 0],
}
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SCM_PIDFD: C2RustUnnamed = 4;
pub const SCM_SECURITY: C2RustUnnamed = 3;
pub const SCM_CREDENTIALS: C2RustUnnamed = 2;
pub const SCM_RIGHTS: C2RustUnnamed = 1;

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub hdr: cmsghdr,
    pub buf: [::core::ffi::c_char; 24],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_3 {
    pub hdr: cmsghdr,
    pub buf: [::core::ffi::c_char; 24],
}
#[inline]
unsafe fn __cmsg_nxthdr(mut __mhdr: *mut msghdr, mut __cmsg: *mut cmsghdr) -> *mut cmsghdr {
    let mut __msg_control_ptr: *mut ::core::ffi::c_uchar =
        (*__mhdr).msg_control as *mut ::core::ffi::c_uchar;
    let mut __cmsg_ptr: *mut ::core::ffi::c_uchar = __cmsg as *mut ::core::ffi::c_uchar;
    let mut __size_needed: size_t = (::core::mem::size_of::<cmsghdr>() as size_t).wrapping_add(
        (::core::mem::size_of::<size_t>() as size_t).wrapping_sub(
            (*__cmsg).cmsg_len
                & (::core::mem::size_of::<size_t>() as size_t).wrapping_sub(1 as size_t),
        ) & (::core::mem::size_of::<size_t>() as size_t).wrapping_sub(1 as size_t),
    );
    if (*__cmsg).cmsg_len < ::core::mem::size_of::<cmsghdr>() as usize {
        return ::core::ptr::null_mut::<cmsghdr>();
    }
    if (__msg_control_ptr
        .offset((*__mhdr).msg_controllen as isize)
        .offset_from(__cmsg_ptr) as ::core::ffi::c_long as size_t)
        < __size_needed
        || (__msg_control_ptr
            .offset((*__mhdr).msg_controllen as isize)
            .offset_from(__cmsg_ptr) as ::core::ffi::c_long as size_t)
            .wrapping_sub(__size_needed)
            < (*__cmsg).cmsg_len
    {
        return ::core::ptr::null_mut::<cmsghdr>();
    }
    __cmsg = (__cmsg as *mut ::core::ffi::c_uchar).offset(
        ((*__cmsg)
            .cmsg_len
            .wrapping_add(::core::mem::size_of::<size_t>() as size_t)
            .wrapping_sub(1 as size_t)
            & !(::core::mem::size_of::<size_t>() as usize).wrapping_sub(1 as usize))
            as isize,
    ) as *mut cmsghdr;
    return __cmsg;
}

pub const __IOV_MAX: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const IOV_MAX: ::core::ffi::c_int = __IOV_MAX;

pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const EMSGSIZE: ::core::ffi::c_int = 90 as ::core::ffi::c_int;
pub const ENOBUFS: ::core::ffi::c_int = 105 as ::core::ffi::c_int;
pub const UINT8_MAX: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const UINT16_MAX: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;

pub const IBUF_READ_SIZE: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;

fn try_zeroed_boxed_slice(len: usize) -> Result<Box<[u8]>, std::collections::TryReserveError> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len)?;
    bytes.resize(len, 0);
    Ok(bytes.into_boxed_slice())
}

unsafe fn raw_boxed_bytes(buf: *mut u8, len: size_t) -> Box<[u8]> {
    Box::from_raw(slice_from_raw_parts_mut(buf, len))
}

/// A temporary ibuf-compatible view over a borrowed header span.
pub(crate) struct IbufView<'a> {
    record: ibuf<'a>,
}

impl<'a> IbufView<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            record: ibuf::borrowed(bytes),
        }
    }

    /// Adapt this borrow to the legacy callback ABI.
    ///
    /// # Safety
    /// The pointer must not be retained after the callback returns. The
    /// callback must treat borrowed storage as read-only and must not free the
    /// view.
    pub(crate) unsafe fn as_ibuf_ptr(&mut self) -> *mut OwnedIbuf {
        (&raw mut self.record).cast::<OwnedIbuf>()
    }

    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let start = self.record.rpos;
        let end = start.checked_add(len)?;
        let IbufStorage::Borrowed(bytes) = &self.record.storage else {
            return None;
        };
        let bytes = bytes.get(start..end)?;
        self.record.rpos = end;
        Some(bytes)
    }
}
pub unsafe fn ibuf_open(mut len: size_t) -> Option<Box<OwnedIbuf>> {
    let Ok(mut buf) = Box::try_new(ibuf {
        storage: IbufStorage::Owned(Vec::new().into_boxed_slice()),
        max: len,
        wpos: 0,
        rpos: 0,
        fd: -(1 as ::core::ffi::c_int),
    }) else {
        *__errno_location() = ENOMEM;
        return None;
    };
    if len > 0 as size_t {
        match try_zeroed_boxed_slice(len) {
            Ok(bytes) => {
                buf.storage = IbufStorage::Owned(bytes);
            }
            Err(_) => {
                *__errno_location() = ENOMEM;
                return None;
            }
        }
    }
    Some(buf)
}
pub unsafe fn ibuf_dynamic(mut len: size_t, mut max: size_t) -> Option<Box<OwnedIbuf>> {
    if max == 0 as size_t || max < len {
        *__errno_location() = EINVAL;
        return None;
    }
    let Ok(mut buf) = Box::try_new(ibuf {
        storage: IbufStorage::Owned(Vec::new().into_boxed_slice()),
        max,
        wpos: 0,
        rpos: 0,
        fd: -(1 as ::core::ffi::c_int),
    }) else {
        *__errno_location() = ENOMEM;
        return None;
    };
    if len > 0 as size_t {
        match try_zeroed_boxed_slice(len) {
            Ok(bytes) => {
                buf.storage = IbufStorage::Owned(bytes);
            }
            Err(_) => {
                *__errno_location() = ENOMEM;
                return None;
            }
        }
    }
    Some(buf)
}
/// Reserve writable space at the end of an owned ibuf.
///
/// # Safety
/// `buf` must point to a live, exclusively accessed owned ibuf. The returned
/// pointer is writable for the reserved length and remains valid only until
/// the next ibuf mutation or drop.
pub unsafe fn ibuf_reserve(mut buf: *mut OwnedIbuf, mut len: size_t) -> *mut ::core::ffi::c_void {
    let mut b: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if len > (SIZE_MAX as size_t).wrapping_sub((*buf).wpos) {
        *__errno_location() = ERANGE;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*buf).is_owned() {
        *__errno_location() = EINVAL;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*buf).wpos.wrapping_add(len) > (*buf).storage_len() {
        let new_size = (*buf).wpos.wrapping_add(len);
        if (*buf).wpos.wrapping_add(len) > (*buf).max {
            *__errno_location() = ERANGE;
            return ::core::ptr::null_mut::<::core::ffi::c_void>();
        }
        let old_size = (*buf).storage_len();
        let mut bytes = match try_zeroed_boxed_slice(new_size) {
            Ok(bytes) => bytes,
            Err(_) => {
                *__errno_location() = ENOMEM;
                return ::core::ptr::null_mut::<::core::ffi::c_void>();
            }
        };
        if old_size > 0 {
            ::core::ptr::copy_nonoverlapping(
                (*buf).storage.as_slice().as_ptr(),
                bytes.as_mut_ptr(),
                old_size,
            );
        }
        (*buf).replace_owned(bytes);
    }
    b = if (*buf).storage_len() == 0 {
        ::core::ptr::null_mut()
    } else {
        (*buf)
            .storage
            .as_mut_slice()
            .unwrap()
            .as_mut_ptr()
            .add((*buf).wpos) as *mut ::core::ffi::c_void
    };
    (*buf).wpos = (*buf).wpos.wrapping_add(len);
    return b;
}
pub unsafe fn ibuf_add(
    mut buf: *mut OwnedIbuf,
    mut data: *const ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut b: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    b = ibuf_reserve(buf, len);
    if b.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(b, data, len);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn ibuf_seek(
    mut buf: *mut OwnedIbuf,
    mut pos: size_t,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    if !(*buf).is_owned() {
        *__errno_location() = EINVAL;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if ibuf_size(buf) < pos
        || (SIZE_MAX as size_t).wrapping_sub(pos) < len
        || ibuf_size(buf) < pos.wrapping_add(len)
    {
        *__errno_location() = ERANGE;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    let Some(bytes) = (*buf).storage.as_mut_slice() else {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    };
    let data = bytes.as_mut_ptr().add((*buf).rpos);
    return data.add(pos) as *mut ::core::ffi::c_void;
}
pub unsafe fn ibuf_set(
    mut buf: *mut OwnedIbuf,
    mut pos: size_t,
    mut data: *const ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut b: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    b = ibuf_seek(buf, pos, len);
    if b.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(b, data, len);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn ibuf_set_h32(
    mut buf: *mut OwnedIbuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint32_t = 0;
    if value > UINT32_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = value as uint32_t;
    return ibuf_set(
        buf,
        pos,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint32_t>() as size_t,
    );
}
/// Return the unread bytes through the legacy raw-pointer interface.
///
/// # Safety
/// `buf` must point to a live ibuf. The pointer remains valid only while its
/// backing storage does; for borrowed storage it must be used read-only and
/// must not outlive the source borrow.
pub unsafe fn ibuf_data(mut buf: *const OwnedIbuf) -> *mut ::core::ffi::c_void {
    if (*buf).storage_len() == 0 {
        return ::core::ptr::null_mut();
    }
    return (*buf).storage.as_slice().as_ptr().add((*buf).rpos) as *mut ::core::ffi::c_void;
}
pub unsafe fn ibuf_size(mut buf: *const OwnedIbuf) -> size_t {
    return (*buf).wpos.wrapping_sub((*buf).rpos);
}
pub unsafe fn ibuf_left(mut buf: *const OwnedIbuf) -> size_t {
    if !(*buf).is_owned() {
        return 0 as size_t;
    }
    return (*buf).max.wrapping_sub((*buf).wpos);
}
pub unsafe fn ibuf_close(mut msgbuf: *mut msgbuf, buf: Box<OwnedIbuf>) {
    if !(*msgbuf).bufs.bufs.push_back_owned(buf) {
        abort();
    }
}
/// Read bytes from an ibuf through the raw compatibility API.
///
/// # Safety
/// `buf` must point to a live, exclusively accessed ibuf and `data` must be
/// writable for `len` bytes. Its unread bytes must remain valid for the call.
pub unsafe fn ibuf_get(
    mut buf: *mut OwnedIbuf,
    mut data: *mut ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if ibuf_size(buf) < len {
        *__errno_location() = EBADMSG;
        return -(1 as ::core::ffi::c_int);
    }
    if len == 0 {
        return 0;
    }
    memcpy(data, ibuf_data(buf), len);
    (*buf).rpos = (*buf).rpos.wrapping_add(len);
    return 0 as ::core::ffi::c_int;
}
/// Release a live owned ibuf allocation received through the raw-pointer API.
///
/// # Safety
/// `buf` must be null or a live owned ibuf pointer that has not been freed or
/// transferred to another owner.
pub unsafe fn ibuf_free(mut buf: *mut OwnedIbuf) {
    if buf.is_null() {
        return;
    }
    if !(*buf).is_owned() {
        abort();
    }
    drop(Box::from_raw(buf));
}
pub unsafe fn ibuf_fd_avail(mut buf: *mut OwnedIbuf) -> ::core::ffi::c_int {
    return ((*buf).fd >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
pub unsafe fn ibuf_fd_get(mut buf: *mut OwnedIbuf) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = 0;
    if (*buf).fd < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    fd = (*buf).fd;
    (*buf).fd = -(1 as ::core::ffi::c_int);
    return fd;
}
pub unsafe fn ibuf_fd_set(mut buf: *mut OwnedIbuf, mut fd: ::core::ffi::c_int) {
    if !(*buf).is_owned() {
        abort();
    }
    if (*buf).fd >= 0 as ::core::ffi::c_int {
        close((*buf).fd);
    }
    (*buf).fd = -(1 as ::core::ffi::c_int);
    if fd >= 0 as ::core::ffi::c_int {
        (*buf).fd = fd;
    }
}
pub unsafe fn msgbuf_new() -> *mut msgbuf {
    let Ok(msgbuf) = Box::try_new(msgbuf {
        bufs: ibufqueue::new(),
        rbufs: ibufqueue::new(),
        rbuf: ::core::ptr::null_mut(),
        rpmsg: ::core::ptr::null_mut(),
        readhdr: None,
        roff: 0,
        hdrsize: 0,
    }) else {
        *__errno_location() = ENOMEM;
        return ::core::ptr::null_mut();
    };
    Box::into_raw(msgbuf)
}

unsafe fn msgbuf_new_reader_with(
    hdrsz: size_t,
    readhdr: Option<
        Box<dyn FnMut(&[u8], Option<OwnedFd>) -> (Option<Box<OwnedIbuf>>, Option<OwnedFd>)>,
    >,
) -> *mut msgbuf {
    if hdrsz == 0 as size_t || hdrsz > (IBUF_READ_SIZE / 2 as ::core::ffi::c_int) as size_t {
        *__errno_location() = EINVAL;
        return ::core::ptr::null_mut::<msgbuf>();
    }
    let scratch = match try_zeroed_boxed_slice(IBUF_READ_SIZE as usize) {
        Ok(scratch) => scratch,
        Err(_) => {
            *__errno_location() = ENOMEM;
            return ::core::ptr::null_mut::<msgbuf>();
        }
    };
    let buf = Box::into_raw(scratch) as *mut ::core::ffi::c_char;
    let msgbuf = msgbuf_new();
    if msgbuf.is_null() {
        drop(raw_boxed_bytes(buf as *mut u8, IBUF_READ_SIZE as size_t));
        return ::core::ptr::null_mut::<msgbuf>();
    }
    (*msgbuf).rbuf = buf;
    (*msgbuf).hdrsize = hdrsz;
    (*msgbuf).readhdr = readhdr;
    return msgbuf;
}

pub(crate) unsafe fn msgbuf_new_reader_owned(
    hdrsz: size_t,
    callback: impl FnMut(&[u8], Option<OwnedFd>) -> (Option<Box<OwnedIbuf>>, Option<OwnedFd>) + 'static,
) -> *mut msgbuf {
    msgbuf_new_reader_with(hdrsz, Some(Box::new(callback)))
}
pub unsafe fn msgbuf_free(mut msgbuf: *mut msgbuf) {
    if msgbuf.is_null() {
        return;
    }
    msgbuf_clear(msgbuf);
    if !(*msgbuf).rbuf.is_null() {
        drop(raw_boxed_bytes(
            (*msgbuf).rbuf as *mut u8,
            IBUF_READ_SIZE as size_t,
        ));
    }
    drop(Box::from_raw(msgbuf));
}
pub unsafe fn msgbuf_queuelen(mut msgbuf: *mut msgbuf) -> uint32_t {
    return ibufq_queuelen(&raw mut (*msgbuf).bufs);
}
pub unsafe fn msgbuf_clear(mut msgbuf: *mut msgbuf) {
    ibufq_flush(&raw mut (*msgbuf).bufs);
    ibufq_flush(&raw mut (*msgbuf).rbufs);
    (*msgbuf).roff = 0 as size_t;
    ibuf_free((*msgbuf).rpmsg);
    (*msgbuf).rpmsg = ::core::ptr::null_mut::<OwnedIbuf>();
}
pub unsafe fn msgbuf_get(mut msgbuf: *mut msgbuf) -> *mut OwnedIbuf {
    return ibufq_pop(&raw mut (*msgbuf).rbufs);
}
pub unsafe fn ibuf_write(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> ::core::ffi::c_int {
    let mut iov: [libc::iovec; 1024] = [libc::iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    }; 1024];
    let mut buf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut n: ssize_t = 0;
    memset(
        &raw mut iov as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[libc::iovec; 1024]>() as size_t,
    );
    for queued_buf in (*msgbuf).bufs.bufs.iter() {
        buf = queued_buf;
        if i >= IOV_MAX as ::core::ffi::c_uint {
            break;
        }
        iov[i as usize].iov_base = ibuf_data(buf);
        iov[i as usize].iov_len = ibuf_size(buf);
        i = i.wrapping_add(1);
    }
    if i == 0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    's_68: {
        loop {
            n = writev(fd, &raw mut iov as *mut libc::iovec, i as ::core::ffi::c_int);
            if n == -(1 as ::core::ffi::c_int) as ssize_t {
                if *__errno_location() == EINTR {
                    continue;
                }
                if *__errno_location() == EAGAIN || *__errno_location() == ENOBUFS {
                    return 0 as ::core::ffi::c_int;
                }
                return -(1 as ::core::ffi::c_int);
            } else {
                break 's_68;
            }
        }
    }
    msgbuf_drain(msgbuf, n as size_t);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn msgbuf_write(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> ::core::ffi::c_int {
    let mut iov: [libc::iovec; 1024] = [libc::iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    }; 1024];
    let mut buf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    let mut buf0: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut n: ssize_t = 0;
    let mut msg: msghdr = msghdr {
        msg_name: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        msg_namelen: 0,
        msg_iov: ::core::ptr::null_mut::<libc::iovec>(),
        msg_iovlen: 0,
        msg_control: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        msg_controllen: 0,
        msg_flags: 0,
    };
    let mut cmsg: *mut cmsghdr = ::core::ptr::null_mut::<cmsghdr>();
    let mut cmsgbuf: C2RustUnnamed_2 = C2RustUnnamed_2 {
        hdr: cmsghdr {
            cmsg_len: 0,
            cmsg_level: 0,
            cmsg_type: 0,
            __cmsg_data: [],
        },
    };
    memset(
        &raw mut iov as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[libc::iovec; 1024]>() as size_t,
    );
    memset(
        &raw mut msg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<msghdr>() as size_t,
    );
    memset(
        &raw mut cmsgbuf as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<C2RustUnnamed_2>() as size_t,
    );
    for queued_buf in (*msgbuf).bufs.bufs.iter() {
        buf = queued_buf;
        if i >= IOV_MAX as ::core::ffi::c_uint {
            break;
        }
        if i > 0 as ::core::ffi::c_uint && (*buf).fd != -(1 as ::core::ffi::c_int) {
            break;
        }
        iov[i as usize].iov_base = ibuf_data(buf);
        iov[i as usize].iov_len = ibuf_size(buf);
        i = i.wrapping_add(1);
        if (*buf).fd != -(1 as ::core::ffi::c_int) {
            buf0 = buf;
        }
    }
    if i == 0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    msg.msg_iov = &raw mut iov as *mut libc::iovec;
    msg.msg_iovlen = i as size_t;
    if !buf0.is_null() {
        msg.msg_control = &raw mut cmsgbuf.buf as caddr_t as *mut ::core::ffi::c_void;
        msg.msg_controllen = ::core::mem::size_of::<[::core::ffi::c_char; 24]>() as usize as size_t;
        cmsg = if msg.msg_controllen >= ::core::mem::size_of::<cmsghdr>() as usize {
            msg.msg_control as *mut cmsghdr
        } else {
            ::core::ptr::null_mut::<cmsghdr>()
        };
        (*cmsg).cmsg_len = ((::core::mem::size_of::<cmsghdr>() as usize)
            .wrapping_add(::core::mem::size_of::<size_t>() as usize)
            .wrapping_sub(1 as usize)
            & !(::core::mem::size_of::<size_t>() as usize).wrapping_sub(1 as usize))
        .wrapping_add(::core::mem::size_of::<::core::ffi::c_int>() as usize)
            as size_t;
        (*cmsg).cmsg_level = SOL_SOCKET;
        (*cmsg).cmsg_type = SCM_RIGHTS as ::core::ffi::c_int;
        *(&raw mut (*cmsg).__cmsg_data as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_int) =
            (*buf0).fd;
    }
    's_129: {
        loop {
            n = sendmsg(fd, &raw mut msg, 0 as ::core::ffi::c_int);
            if n == -(1 as ::core::ffi::c_int) as ssize_t {
                if *__errno_location() == EINTR {
                    continue;
                }
                if *__errno_location() == EAGAIN || *__errno_location() == ENOBUFS {
                    return 0 as ::core::ffi::c_int;
                }
                return -(1 as ::core::ffi::c_int);
            } else {
                break 's_129;
            }
        }
    }
    if !buf0.is_null() {
        close((*buf0).fd);
        (*buf0).fd = -(1 as ::core::ffi::c_int);
    }
    msgbuf_drain(msgbuf, n as size_t);
    return 0 as ::core::ffi::c_int;
}
unsafe fn ibuf_read_process(
    mut msgbuf: *mut msgbuf,
    mut fd: ::core::ffi::c_int,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    let mut sz: ssize_t = 0;
    let scratch = (*msgbuf).rbuf as *mut u8;
    let read_len = (*msgbuf).roff;
    let mut cursor = 0usize;
    let mut failed = false;
    let mut error = 0 as ::core::ffi::c_int;
    'parse: loop {
        if (*msgbuf).rpmsg.is_null() {
            if read_len.wrapping_sub(cursor) < (*msgbuf).hdrsize {
                break;
            }
            let header = std::slice::from_raw_parts(scratch.add(cursor), (*msgbuf).hdrsize);
            let input_fd = if fd >= 0 {
                Some(OwnedFd::from_raw_fd(fd))
            } else {
                None
            };
            fd = -1;
            let (message, remaining_fd) =
                (*msgbuf)
                    .readhdr
                    .as_mut()
                    .expect("non-null header callback")(header, input_fd);
            fd = remaining_fd.map_or(-1, IntoRawFd::into_raw_fd);
            let Some(message) = message else {
                failed = true;
                break;
            };
            if !(*message).is_owned() {
                failed = true;
                break;
            }
            (*msgbuf).rpmsg = Box::into_raw(message);
        }
        let available = read_len.wrapping_sub(cursor);
        if ibuf_left((*msgbuf).rpmsg) <= available {
            sz = ibuf_left((*msgbuf).rpmsg) as ssize_t;
        } else {
            sz = available as ssize_t;
        }
        let copy_len = sz as size_t;
        let chunk = {
            let input = std::slice::from_raw_parts(scratch.add(cursor), available);
            let mut view = IbufView::new(input);
            match view.take(copy_len) {
                Some(chunk) => chunk,
                None => {
                    failed = true;
                    break 'parse;
                }
            }
        };
        if ibuf_add(
            (*msgbuf).rpmsg,
            chunk.as_ptr() as *const ::core::ffi::c_void,
            copy_len,
        ) == -(1 as ::core::ffi::c_int)
        {
            failed = true;
            error = *__errno_location();
            break;
        }
        cursor = cursor.wrapping_add(copy_len);
        if ibuf_left((*msgbuf).rpmsg) == 0 as size_t {
            ibufq_push(&raw mut (*msgbuf).rbufs, (*msgbuf).rpmsg);
            (*msgbuf).rpmsg = ::core::ptr::null_mut::<OwnedIbuf>();
        }
        if cursor >= read_len {
            break;
        }
    }
    if failed {
        if fd != -(1 as ::core::ffi::c_int) {
            close(fd);
        }
        return Err(error);
    }
    let remaining = read_len.wrapping_sub(cursor);
    if remaining > 0 {
        let scratch =
            std::slice::from_raw_parts_mut((*msgbuf).rbuf as *mut u8, IBUF_READ_SIZE as usize);
        scratch.copy_within(cursor..read_len, 0);
    }
    (*msgbuf).roff = remaining;
    if fd != -(1 as ::core::ffi::c_int) {
        close(fd);
    };
    Ok(1 as ::core::ffi::c_int)
}
pub unsafe fn ibuf_read(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    let mut iov: libc::iovec = libc::iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    };
    let mut n: ssize_t = 0;
    if (*msgbuf).rbuf.is_null() {
        *__errno_location() = EINVAL;
        return Err(EINVAL);
    }
    iov.iov_base = (*msgbuf).rbuf.offset((*msgbuf).roff as isize) as *mut ::core::ffi::c_void;
    iov.iov_len = (IBUF_READ_SIZE as size_t).wrapping_sub((*msgbuf).roff);
    's_45: {
        loop {
            n = readv(fd, &raw mut iov, 1 as ::core::ffi::c_int);
            if n == -(1 as ::core::ffi::c_int) as ssize_t {
                if *__errno_location() == EINTR {
                    continue;
                }
                if *__errno_location() == EAGAIN {
                    return Ok(1 as ::core::ffi::c_int);
                }
                return Err(*__errno_location());
            } else {
                break 's_45;
            }
        }
    }
    if n == 0 as ssize_t {
        return Ok(0 as ::core::ffi::c_int);
    }
    (*msgbuf).roff = (*msgbuf).roff.wrapping_add(n as size_t);
    return ibuf_read_process(msgbuf, -(1 as ::core::ffi::c_int));
}
pub unsafe fn msgbuf_read(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    let mut msg: msghdr = msghdr {
        msg_name: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        msg_namelen: 0,
        msg_iov: ::core::ptr::null_mut::<libc::iovec>(),
        msg_iovlen: 0,
        msg_control: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        msg_controllen: 0,
        msg_flags: 0,
    };
    let mut cmsg: *mut cmsghdr = ::core::ptr::null_mut::<cmsghdr>();
    let mut cmsgbuf: C2RustUnnamed_3 = C2RustUnnamed_3 {
        hdr: cmsghdr {
            cmsg_len: 0,
            cmsg_level: 0,
            cmsg_type: 0,
            __cmsg_data: [],
        },
    };
    let mut iov: libc::iovec = libc::iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    };
    let mut n: ssize_t = 0;
    let mut fdpass: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if (*msgbuf).rbuf.is_null() {
        *__errno_location() = EINVAL;
        return Err(EINVAL);
    }
    memset(
        &raw mut msg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<msghdr>() as size_t,
    );
    memset(
        &raw mut cmsgbuf as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<C2RustUnnamed_3>() as size_t,
    );
    iov.iov_base = (*msgbuf).rbuf.offset((*msgbuf).roff as isize) as *mut ::core::ffi::c_void;
    iov.iov_len = (IBUF_READ_SIZE as size_t).wrapping_sub((*msgbuf).roff);
    msg.msg_iov = &raw mut iov;
    msg.msg_iovlen = 1 as size_t;
    msg.msg_control = &raw mut cmsgbuf.buf as *mut ::core::ffi::c_void;
    msg.msg_controllen = ::core::mem::size_of::<[::core::ffi::c_char; 24]>() as usize as size_t;
    loop {
        n = recvmsg(fd, &raw mut msg, 0 as ::core::ffi::c_int);
        if n == -(1 as ::core::ffi::c_int) as ssize_t {
            if *__errno_location() == EINTR {
                continue;
            }
            if *__errno_location() == EMSGSIZE {
                continue;
            }
            if *__errno_location() == EAGAIN {
                return Ok(1 as ::core::ffi::c_int);
            }
            return Err(*__errno_location());
        } else {
            if n == 0 as ssize_t {
                return Ok(0 as ::core::ffi::c_int);
            }
            (*msgbuf).roff = (*msgbuf).roff.wrapping_add(n as size_t);
            cmsg = if msg.msg_controllen >= ::core::mem::size_of::<cmsghdr>() as usize {
                msg.msg_control as *mut cmsghdr
            } else {
                ::core::ptr::null_mut::<cmsghdr>()
            };
            while !cmsg.is_null() {
                if (*cmsg).cmsg_level == SOL_SOCKET
                    && (*cmsg).cmsg_type == SCM_RIGHTS as ::core::ffi::c_int
                {
                    let mut i: ::core::ffi::c_int = 0;
                    let mut j: ::core::ffi::c_int = 0;
                    let mut f: ::core::ffi::c_int = 0;
                    j = ((cmsg as *mut ::core::ffi::c_char)
                        .offset((*cmsg).cmsg_len as isize)
                        .offset_from(
                            &raw mut (*cmsg).__cmsg_data as *mut ::core::ffi::c_uchar
                                as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_long as usize)
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
                        as ::core::ffi::c_int;
                    i = 0 as ::core::ffi::c_int;
                    while i < j {
                        f = *(&raw mut (*cmsg).__cmsg_data as *mut ::core::ffi::c_uchar
                            as *mut ::core::ffi::c_int)
                            .offset(i as isize);
                        if i == 0 as ::core::ffi::c_int {
                            fdpass = f;
                        } else {
                            close(f);
                        }
                        i += 1;
                    }
                }
                cmsg = __cmsg_nxthdr(&raw mut msg, cmsg);
            }
            return ibuf_read_process(msgbuf, fdpass);
        }
    }
}
unsafe fn msgbuf_drain(mut msgbuf: *mut msgbuf, mut n: size_t) {
    let mut buf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    loop {
        buf = match (*msgbuf).bufs.bufs.front() {
            Some(buf) => buf,
            None => return,
        };
        if n >= ibuf_size(buf) {
            n = n.wrapping_sub(ibuf_size(buf));
            drop((*msgbuf).bufs.bufs.pop_front_owned());
        } else {
            (*buf).rpos = (*buf).rpos.wrapping_add(n);
            return;
        }
    }
}
pub unsafe fn ibufq_pop(mut bufq: *mut ibufqueue) -> *mut OwnedIbuf {
    return (*bufq)
        .bufs
        .pop_front_raw()
        .unwrap_or(::core::ptr::null_mut());
}
/// Transfer an owned ibuf allocation into a queue.
///
/// # Safety
/// `bufq` must be live and `buf` must point to a uniquely owned heap ibuf
/// allocation that has not already been queued or freed.
pub unsafe fn ibufq_push(mut bufq: *mut ibufqueue, mut buf: *mut OwnedIbuf) {
    if !(*bufq).bufs.push_back_raw(buf) {
        abort();
    }
}
pub unsafe fn ibufq_queuelen(mut bufq: *mut ibufqueue) -> uint32_t {
    return (*bufq).bufs.len() as uint32_t;
}
pub unsafe fn ibufq_flush(mut bufq: *mut ibufqueue) {
    (*bufq).bufs.clear();
}
