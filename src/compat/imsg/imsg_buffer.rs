use crate::src::ffi::libc::{__errno_location, readv, recvmsg, sendmsg, writev};
use crate::src::shared::abi::*;
use crate::src::shared::abi::uint32_t;
use ::libc::{cmsghdr, msghdr};
pub use crate::src::shared::errno::{EAGAIN, EBADMSG, EINTR, EINVAL, ENOMEM, ERANGE};
use crate::src::shared::limits::{SIZE_MAX, UINT32_MAX};
use super::message::{ibuf, msgbuf, OwnedIbuf, IbufStorage};
use crate::src::shared::socket::SOL_SOCKET;
use std::collections::VecDeque;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

pub(crate) struct ibufqueue {
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

    fn iter(&self) -> impl Iterator<Item = &OwnedIbuf> + '_ {
        self.entries.iter().map(Box::as_ref)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn push_back_owned(&mut self, buf: Box<OwnedIbuf>) -> Result<(), ::core::ffi::c_int> {
        if !buf.is_owned() {
            return Err(EINVAL);
        }
        self.entries.try_reserve(1).map_err(|_| ENOMEM)?;
        self.entries.push_back(buf);
        Ok(())
    }

    fn pop_front_owned(&mut self) -> Option<Box<OwnedIbuf>> {
        self.entries.pop_front()
    }

    fn front(&self) -> Option<&OwnedIbuf> {
        self.entries.front().map(Box::as_ref)
    }

    fn front_mut(&mut self) -> Option<&mut OwnedIbuf> {
        self.entries.front_mut().map(Box::as_mut)
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

pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SCM_PIDFD: C2RustUnnamed = 4;
pub const SCM_SECURITY: C2RustUnnamed = 3;
pub const SCM_CREDENTIALS: C2RustUnnamed = 2;
pub const SCM_RIGHTS: C2RustUnnamed = 1;

const IMSG_CMSG_FD_BUFFER_SIZE: usize = unsafe {
    ::libc::CMSG_SPACE(::core::mem::size_of::<::core::ffi::c_int>() as ::libc::c_uint) as usize
};

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub hdr: cmsghdr,
    pub buf: [::core::ffi::c_char; IMSG_CMSG_FD_BUFFER_SIZE],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_3 {
    pub hdr: cmsghdr,
    pub buf: [::core::ffi::c_char; IMSG_CMSG_FD_BUFFER_SIZE],
}

pub const __IOV_MAX: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const IOV_MAX: ::core::ffi::c_int = __IOV_MAX;

pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const EMSGSIZE: ::core::ffi::c_int = 90 as ::core::ffi::c_int;
pub const ENOBUFS: ::core::ffi::c_int = 105 as ::core::ffi::c_int;
pub const UINT8_MAX: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const UINT16_MAX: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;

pub const IBUF_READ_SIZE: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;

fn try_zeroed_vec(len: usize) -> Result<Vec<u8>, std::collections::TryReserveError> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len)?;
    bytes.resize(len, 0);
    Ok(bytes)
}

pub fn ibuf_open(len: size_t) -> Result<Box<OwnedIbuf>, ::core::ffi::c_int> {
    let Ok(mut buf) = Box::try_new(ibuf {
        storage: IbufStorage::Owned(Vec::new()),
        max: len,
        wpos: 0,
        rpos: 0,
        fd: None,
    }) else {
        return Err(ENOMEM);
    };
    if len > 0 as size_t {
        match try_zeroed_vec(len) {
            Ok(bytes) => {
                buf.storage = IbufStorage::Owned(bytes);
            }
            Err(_) => {
                return Err(ENOMEM);
            }
        }
    }
    Ok(buf)
}
pub fn ibuf_dynamic(len: size_t, max: size_t) -> Result<Box<OwnedIbuf>, ::core::ffi::c_int> {
    if max == 0 as size_t || max < len {
        return Err(EINVAL);
    }
    let Ok(mut buf) = Box::try_new(ibuf {
        storage: IbufStorage::Owned(Vec::new()),
        max,
        wpos: 0,
        rpos: 0,
        fd: None,
    }) else {
        return Err(ENOMEM);
    };
    if len > 0 as size_t {
        match try_zeroed_vec(len) {
            Ok(bytes) => {
                buf.storage = IbufStorage::Owned(bytes);
            }
            Err(_) => {
                return Err(ENOMEM);
            }
        }
    }
    Ok(buf)
}
/// Reserve writable space at the end of an owned ibuf.
///
pub fn ibuf_reserve(
    buf: &mut OwnedIbuf,
    len: size_t,
) -> Result<&mut [u8], ::core::ffi::c_int> {
    let Some(new_wpos) = buf.wpos.checked_add(len) else {
        return Err(ERANGE);
    };
    if new_wpos > SIZE_MAX as usize {
        return Err(ERANGE);
    }
    if !buf.is_owned() {
        return Err(EINVAL);
    }
    if new_wpos > buf.storage_len() {
        if new_wpos > buf.max {
            return Err(ERANGE);
        }
        let old_size = buf.storage_len();
        let mut bytes = try_zeroed_vec(new_wpos).map_err(|_| ENOMEM)?;
        if old_size > 0 {
            bytes[..old_size].copy_from_slice(&buf.storage.as_slice()[..old_size]);
        }
        buf.replace_owned(bytes);
    }
    let start = buf.wpos;
    buf.wpos = new_wpos;
    buf.storage
        .as_mut_slice()
        .ok_or(EINVAL)?
        .get_mut(start..new_wpos)
        .ok_or(ERANGE)
}
pub fn ibuf_add(buf: &mut OwnedIbuf, data: &[u8]) -> Result<(), ::core::ffi::c_int> {
    if !buf.is_owned() {
        return Err(EINVAL);
    }
    if data.is_empty() {
        return Ok(());
    }
    let target = ibuf_reserve(buf, data.len())?;
    target.copy_from_slice(data);
    Ok(())
}
pub fn ibuf_seek(
    buf: &mut OwnedIbuf,
    pos: size_t,
    len: size_t,
) -> Result<&mut [u8], ::core::ffi::c_int> {
    if !buf.is_owned() {
        return Err(EINVAL);
    }
    let Some(end) = pos.checked_add(len) else {
        return Err(ERANGE);
    };
    if end > ibuf_size(buf) {
        return Err(ERANGE);
    }
    let Some(start) = buf.rpos.checked_add(pos) else {
        return Err(ERANGE);
    };
    let Some(end) = buf.rpos.checked_add(end) else {
        return Err(ERANGE);
    };
    buf.storage
        .as_mut_slice()
        .ok_or(EINVAL)?
        .get_mut(start..end)
        .ok_or(ERANGE)
}
pub fn ibuf_set(
    buf: &mut OwnedIbuf,
    pos: size_t,
    data: &[u8],
) -> Result<(), ::core::ffi::c_int> {
    let target = ibuf_seek(buf, pos, data.len())?;
    target.copy_from_slice(data);
    Ok(())
}
pub fn ibuf_set_h32(
    buf: &mut OwnedIbuf,
    pos: size_t,
    value: uint64_t,
) -> Result<(), ::core::ffi::c_int> {
    if value > UINT32_MAX as uint64_t {
        return Err(EINVAL);
    }
    let bytes = (value as uint32_t).to_ne_bytes();
    ibuf_set(buf, pos, &bytes)
}
pub fn ibuf_data(buf: &OwnedIbuf) -> &[u8] {
    buf.unread()
}
pub fn ibuf_size(buf: &OwnedIbuf) -> size_t {
    buf.size()
}
pub fn ibuf_left(buf: &OwnedIbuf) -> size_t {
    if !buf.is_owned() {
        return 0 as size_t;
    }
    buf.max.saturating_sub(buf.wpos)
}
pub fn ibuf_close(
    msgbuf: &mut msgbuf,
    buf: Box<OwnedIbuf>,
) -> Result<(), ::core::ffi::c_int> {
    msgbuf.bufs.bufs.push_back_owned(buf)
}
pub fn ibuf_get(buf: &mut OwnedIbuf, data: &mut [u8]) -> Result<(), ::core::ffi::c_int> {
    if ibuf_size(buf) < data.len() {
        return Err(EBADMSG);
    }
    if data.is_empty() {
        return Ok(());
    }
    let len = data.len();
    data.copy_from_slice(&ibuf_data(buf)[..len]);
    if !buf.skip(len) {
        return Err(EBADMSG);
    }
    Ok(())
}
pub fn ibuf_fd_avail(buf: &OwnedIbuf) -> bool {
    buf.fd.is_some()
}
pub fn ibuf_fd_get(buf: &mut OwnedIbuf) -> Option<OwnedFd> {
    buf.fd.take()
}
pub fn ibuf_fd_set(
    buf: &mut OwnedIbuf,
    fd: Option<OwnedFd>,
) -> Result<(), ::core::ffi::c_int> {
    if !buf.is_owned() {
        return Err(EINVAL);
    }
    buf.fd = fd;
    Ok(())
}
pub fn msgbuf_new() -> Result<Box<msgbuf>, ::core::ffi::c_int> {
    let Ok(msgbuf) = Box::try_new(msgbuf {
        bufs: ibufqueue::new(),
        rbufs: ibufqueue::new(),
        rbuf: Vec::new(),
        rpmsg: None,
        readhdr: None,
        roff: 0,
        hdrsize: 0,
    }) else {
        return Err(ENOMEM);
    };
    Ok(msgbuf)
}

fn msgbuf_new_reader_with(
    hdrsz: size_t,
    readhdr: Option<
        Box<
            dyn FnMut(
                &[u8],
                Option<OwnedFd>,
            ) -> Result<(Box<OwnedIbuf>, Option<OwnedFd>), ::core::ffi::c_int>,
        >,
    >,
) -> Result<Box<msgbuf>, ::core::ffi::c_int> {
    if hdrsz == 0 || hdrsz > (IBUF_READ_SIZE / 2) as size_t {
        return Err(EINVAL);
    }
    let scratch = try_zeroed_vec(IBUF_READ_SIZE as usize).map_err(|_| ENOMEM)?;
    let mut msgbuf = msgbuf_new()?;
    msgbuf.rbuf = scratch;
    msgbuf.hdrsize = hdrsz;
    msgbuf.readhdr = readhdr;
    Ok(msgbuf)
}

pub(crate) fn msgbuf_new_reader_owned(
    hdrsz: size_t,
    callback: impl FnMut(
            &[u8],
            Option<OwnedFd>,
        ) -> Result<(Box<OwnedIbuf>, Option<OwnedFd>), ::core::ffi::c_int>
        + 'static,
) -> Result<Box<msgbuf>, ::core::ffi::c_int> {
    msgbuf_new_reader_with(hdrsz, Some(Box::new(callback)))
}

pub fn msgbuf_queuelen(msgbuf: &msgbuf) -> uint32_t {
    ibufq_queuelen(&msgbuf.bufs) as uint32_t
}

pub fn msgbuf_clear(msgbuf: &mut msgbuf) {
    ibufq_flush(&mut msgbuf.bufs);
    ibufq_flush(&mut msgbuf.rbufs);
    msgbuf.roff = 0;
    msgbuf.rpmsg = None;
}

pub fn msgbuf_get(msgbuf: &mut msgbuf) -> Option<Box<OwnedIbuf>> {
    ibufq_pop(&mut msgbuf.rbufs)
}

pub fn ibuf_write(
    fd: ::core::ffi::c_int,
    msgbuf: &mut msgbuf,
) -> Result<(), ::core::ffi::c_int> {
    let mut iov: [libc::iovec; 1024] = [libc::iovec {
        iov_base: ::core::ptr::null_mut(),
        iov_len: 0,
    }; 1024];
    let mut i = 0usize;
    for queued_buf in msgbuf.bufs.bufs.iter().take(IOV_MAX as usize) {
        let data = ibuf_data(queued_buf);
        iov[i].iov_base = data.as_ptr().cast_mut().cast();
        iov[i].iov_len = data.len();
        i += 1;
    }
    if i == 0 {
        return Ok(());
    }
    let n = loop {
        let n = unsafe { writev(fd, iov.as_mut_ptr(), i as ::core::ffi::c_int) };
        if n == -1 {
            let error = unsafe { *__errno_location() };
            if error == EINTR {
                continue;
            }
            if error == EAGAIN || error == ENOBUFS {
                return Ok(());
            }
            return Err(error);
        }
        break n;
    };
    msgbuf_drain(msgbuf, n as size_t);
    Ok(())
}

pub fn msgbuf_write(
    fd: ::core::ffi::c_int,
    msgbuf: &mut msgbuf,
) -> Result<(), ::core::ffi::c_int> {
    let mut iov: [libc::iovec; 1024] = [libc::iovec {
        iov_base: ::core::ptr::null_mut(),
        iov_len: 0,
    }; 1024];
    let mut fd_buf_index = None;
    let mut i = 0usize;
    for (index, queued_buf) in msgbuf.bufs.bufs.entries.iter().enumerate() {
        if i >= IOV_MAX as usize {
            break;
        }
        if i > 0 && queued_buf.fd.is_some() {
            break;
        }
        let data = ibuf_data(queued_buf);
        iov[i].iov_base = data.as_ptr().cast_mut().cast();
        iov[i].iov_len = data.len();
        i += 1;
        if queued_buf.fd.is_some() {
            fd_buf_index = Some(index);
        }
    }
    if i == 0 {
        return Ok(());
    }

    let mut msg: msghdr = unsafe { ::core::mem::zeroed() };
    let mut cmsgbuf = C2RustUnnamed_2 {
        buf: [0; IMSG_CMSG_FD_BUFFER_SIZE],
    };
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = i as size_t;
    if let Some(index) = fd_buf_index {
        let Some(fd) = msgbuf.bufs.bufs.entries[index].fd.as_ref() else {
            return Err(EINVAL);
        };
        msg.msg_control = (&mut cmsgbuf as *mut C2RustUnnamed_2).cast();
        msg.msg_controllen = IMSG_CMSG_FD_BUFFER_SIZE;
        let cmsg = unsafe { ::libc::CMSG_FIRSTHDR(&msg) };
        if cmsg.is_null() {
            return Err(EINVAL);
        }
        unsafe {
            (*cmsg).cmsg_len = ::libc::CMSG_LEN(
                ::core::mem::size_of::<::core::ffi::c_int>() as ::libc::c_uint,
            ) as size_t;
            (*cmsg).cmsg_level = SOL_SOCKET;
            (*cmsg).cmsg_type = SCM_RIGHTS as ::core::ffi::c_int;
            ::libc::CMSG_DATA(cmsg)
                .cast::<::core::ffi::c_int>()
                .write(fd.as_raw_fd());
        }
    }

    let n = loop {
        let n = unsafe { sendmsg(fd, &mut msg, 0) };
        if n == -1 {
            let error = unsafe { *__errno_location() };
            if error == EINTR {
                continue;
            }
            if error == EAGAIN || error == ENOBUFS {
                return Ok(());
            }
            return Err(error);
        }
        break n;
    };
    if let Some(index) = fd_buf_index {
        drop(msgbuf.bufs.bufs.entries[index].fd.take());
    }
    msgbuf_drain(msgbuf, n as size_t);
    Ok(())
}

fn ibuf_read_process(
    msgbuf: &mut msgbuf,
    mut fd: Option<OwnedFd>,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    let read_len = msgbuf.roff;
    let mut cursor = 0usize;
    loop {
        if msgbuf.rpmsg.is_none() {
            if read_len.saturating_sub(cursor) < msgbuf.hdrsize {
                break;
            }
            let header_end = cursor + msgbuf.hdrsize;
            let header = &msgbuf.rbuf[cursor..header_end];
            let (message, remaining_fd) = msgbuf
                .readhdr
                .as_mut()
                .expect("reader has a header callback")(header, fd.take())?;
            fd = remaining_fd;
            if !message.is_owned() {
                return Err(EINVAL);
            }
            msgbuf.rpmsg = Some(message);
        }

        let available = read_len.saturating_sub(cursor);
        let copy_len = ibuf_left(msgbuf.rpmsg.as_ref().unwrap()).min(available);
        let chunk_end = cursor + copy_len;
        let chunk = &msgbuf.rbuf[cursor..chunk_end];
        let left = {
            let current = msgbuf.rpmsg.as_mut().unwrap();
            ibuf_add(current, chunk)?;
            ibuf_left(current)
        };
        cursor += copy_len;
        if left == 0 {
            ibufq_push(&mut msgbuf.rbufs, msgbuf.rpmsg.take().unwrap())?;
        }
        if cursor >= read_len {
            break;
        }
    }
    let remaining = read_len.saturating_sub(cursor);
    if remaining > 0 {
        msgbuf.rbuf.copy_within(cursor..read_len, 0);
    }
    msgbuf.roff = remaining;
    drop(fd);
    Ok(1)
}

pub fn ibuf_read(
    fd: ::core::ffi::c_int,
    msgbuf: &mut msgbuf,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    if msgbuf.rbuf.is_empty() {
        return Err(EINVAL);
    }
    let mut iov = libc::iovec {
        iov_base: unsafe { msgbuf.rbuf.as_mut_ptr().add(msgbuf.roff).cast() },
        iov_len: msgbuf.rbuf.len().saturating_sub(msgbuf.roff),
    };
    let n = loop {
        let n = unsafe { readv(fd, &mut iov, 1) };
        if n == -1 {
            let error = unsafe { *__errno_location() };
            if error == EINTR {
                continue;
            }
            if error == EAGAIN {
                return Ok(1);
            }
            return Err(error);
        }
        break n;
    };
    if n == 0 {
        return Ok(0);
    }
    msgbuf.roff += n as usize;
    ibuf_read_process(msgbuf, None)
}

pub fn msgbuf_read(
    fd: ::core::ffi::c_int,
    msgbuf: &mut msgbuf,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    if msgbuf.rbuf.is_empty() {
        return Err(EINVAL);
    }
    let mut cmsgbuf = C2RustUnnamed_3 {
        buf: [0; IMSG_CMSG_FD_BUFFER_SIZE],
    };
    let mut iov = libc::iovec {
        iov_base: unsafe { msgbuf.rbuf.as_mut_ptr().add(msgbuf.roff).cast() },
        iov_len: msgbuf.rbuf.len().saturating_sub(msgbuf.roff),
    };
    let mut msg: msghdr = unsafe { ::core::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = (&mut cmsgbuf as *mut C2RustUnnamed_3).cast();
    msg.msg_controllen = IMSG_CMSG_FD_BUFFER_SIZE;

    let n = loop {
        let n = unsafe { recvmsg(fd, &mut msg, 0) };
        if n == -1 {
            let error = unsafe { *__errno_location() };
            if error == EINTR || error == EMSGSIZE {
                continue;
            }
            if error == EAGAIN {
                return Ok(1);
            }
            return Err(error);
        }
        break n;
    };
    if n == 0 {
        return Ok(0);
    }
    msgbuf.roff += n as usize;

    let mut fdpass = None;
    let mut cmsg = unsafe { ::libc::CMSG_FIRSTHDR(&msg) };
    while !cmsg.is_null() {
        unsafe {
            if (*cmsg).cmsg_level == SOL_SOCKET && (*cmsg).cmsg_type == SCM_RIGHTS as i32 {
                let data = ::libc::CMSG_DATA(cmsg).cast::<i32>();
                let data_bytes = ((*cmsg).cmsg_len as usize)
                    .saturating_sub(::libc::CMSG_LEN(0) as usize);
                let count = data_bytes / ::core::mem::size_of::<i32>();
                for index in 0..count {
                    let raw_fd = *data.add(index);
                    let owned_fd = OwnedFd::from_raw_fd(raw_fd);
                    if fdpass.is_none() {
                        fdpass = Some(owned_fd);
                    } else {
                        drop(owned_fd);
                    }
                }
            }
            cmsg = ::libc::CMSG_NXTHDR(&msg, cmsg);
        }
    }
    ibuf_read_process(msgbuf, fdpass)
}

fn msgbuf_drain(msgbuf: &mut msgbuf, mut n: size_t) {
    loop {
        let Some(buf) = msgbuf.bufs.bufs.front() else {
            return;
        };
        let size = ibuf_size(buf);
        if n >= size {
            n -= size;
            drop(msgbuf.bufs.bufs.pop_front_owned());
        } else {
            let Some(buf) = msgbuf.bufs.bufs.front_mut() else {
                return;
            };
            buf.rpos += n;
            return;
        }
    }
}

pub fn ibufq_pop(bufq: &mut ibufqueue) -> Option<Box<OwnedIbuf>> {
    bufq.bufs.pop_front_owned()
}

pub fn ibufq_push(
    bufq: &mut ibufqueue,
    buf: Box<OwnedIbuf>,
) -> Result<(), ::core::ffi::c_int> {
    bufq.bufs.push_back_owned(buf)
}

pub fn ibufq_queuelen(bufq: &ibufqueue) -> usize {
    bufq.bufs.len()
}

pub fn ibufq_flush(bufq: &mut ibufqueue) {
    bufq.bufs.clear();
}
