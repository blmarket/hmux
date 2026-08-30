use crate::compat::imsg_buffer::{
    ibuf_add, ibuf_add_ibuf, ibuf_close, ibuf_data, ibuf_dynamic, ibuf_fd_avail, ibuf_fd_get,
    ibuf_fd_set, ibuf_free, ibuf_get, ibuf_get_ibuf, ibuf_get_strbuf, ibuf_open, ibuf_read,
    ibuf_rewind, ibuf_set_h32, ibuf_set_maxsize, ibuf_size, ibuf_skip, ibuf_write, ibufq_pop,
    ibufq_push, msgbuf_get, msgbuf_new_reader, msgbuf_queuelen, msgbuf_read, msgbuf_write,
};
use crate::ffi::{__errno_location, getpid};
pub use crate::types::*;
use ::core::ffi::{c_char, c_int, c_uchar, c_uint};
use ::core::ptr::null_mut;
pub const EINVAL: c_int = 22 as c_int;
pub const ERANGE: c_int = 34 as c_int;
pub const EBADMSG: c_int = 74 as c_int;
pub const UINT32_MAX: c_uint = 4294967295 as c_uint;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: c_int = 16384 as c_int;
pub const IMSG_ALLOW_FDPASS: c_int = 0x1 as c_int;

/// The top bit of a header's length field, set when a descriptor rides along
/// with the message. It is masked off before the length is read.
pub const IMSG_FD_MARK: c_uint = 0x80000000 as c_uint;

unsafe fn imsgbuf_msgbuf(imsgbuf: &mut imsgbuf) -> *mut msgbuf {
    imsgbuf
        .w
        .as_mut()
        .map(|w| w.as_mut() as *mut msgbuf)
        .unwrap_or(null_mut::<msgbuf>())
}

/// Sets the error number and answers the failure the caller hands back.
unsafe fn imsg_fail<T>(errno: c_int, answer: T) -> T {
    unsafe {
        *__errno_location() = errno;
        answer
    }
}

/// A zeroed header, the way every message starts out.
fn imsg_blank_hdr() -> imsg_hdr {
    imsg_hdr {
        type_0: 0,
        len: 0,
        peerid: 0,
        pid: 0,
    }
}

/// Puts `hdr` at the end of `buf`.
unsafe fn imsg_add_hdr(buf: *mut ibuf, hdr: *const imsg_hdr) -> c_int {
    unsafe {
        ibuf_add(
            buf,
            hdr as *const c_uchar,
            ::core::mem::size_of::<imsg_hdr>() as size_t,
        )
    }
}

/// The buffer a message was read into, as the borrowed view the reading
/// calls take.
unsafe fn imsg_buf(imsg: *mut imsg) -> *mut ibuf {
    unsafe {
        (*imsg)
            .buf
            .as_deref_mut()
            .map_or(null_mut::<ibuf>(), |buf| &raw mut *buf)
    }
}

/// Reads a header off the front of `buf`.
unsafe fn imsg_get_hdr(buf: *mut ibuf, hdr: *mut imsg_hdr) -> c_int {
    unsafe {
        ibuf_get(
            buf,
            hdr as *mut c_uchar,
            ::core::mem::size_of::<imsg_hdr>() as size_t,
        )
    }
}

/// Takes the header off `buf` and builds the message around what is left. The
/// buffer becomes the message's, whether this works or not.
unsafe fn imsg_from_ibuf(mut buf: Box<ibuf>, imsg: *mut imsg) -> c_int {
    unsafe {
        let mut m = imsg::default();
        let buf_ptr = &raw mut *buf;
        if imsg_get_hdr(buf_ptr, &raw mut m.hdr) == -(1 as c_int) {
            ibuf_free(buf);
            return -(1 as c_int);
        }
        if ibuf_size(buf_ptr) != 0 {
            m.data = ibuf_data(buf_ptr);
        }
        m.buf = Some(buf);
        m.hdr.len = (m.hdr.len as c_uint & !IMSG_FD_MARK) as uint32_t;
        *imsg = m;
        1 as c_int
    }
}

pub unsafe fn imsgbuf_init(imsgbuf: &mut imsgbuf, fd: c_int) -> c_int {
    unsafe {
        let Some(w) = msgbuf_new_reader(IMSG_HEADER_SIZE, Some(imsg_parse_hdr), &raw mut *imsgbuf)
        else {
            return -(1 as c_int);
        };
        imsgbuf.w = Some(w);
        imsgbuf.pid = getpid() as pid_t;
        imsgbuf.maxsize = MAX_IMSGSIZE as uint32_t;
        imsgbuf.fd = fd;
        imsgbuf.flags = 0 as c_int;
        0 as c_int
    }
}

pub unsafe fn imsgbuf_allow_fdpass(imsgbuf: &mut imsgbuf) {
    imsgbuf.flags |= IMSG_ALLOW_FDPASS;
}

/// Whether this buffer's messages may carry a descriptor, which decides
/// whether it goes over the socket as a plain write or as a message.
fn imsg_fdpass(imsgbuf: &mut imsgbuf) -> bool {
    imsgbuf.flags & IMSG_ALLOW_FDPASS != 0
}

pub unsafe fn imsgbuf_set_maxsize(imsgbuf: &mut imsgbuf, max: uint32_t) -> c_int {
    unsafe {
        if max as usize > (UINT32_MAX as usize).wrapping_sub(IMSG_HEADER_SIZE) {
            return imsg_fail(ERANGE, -(1 as c_int));
        }
        let max = (max as ::core::ffi::c_ulong)
            .wrapping_add(IMSG_HEADER_SIZE as ::core::ffi::c_ulong) as uint32_t;
        if max & IMSG_FD_MARK as uint32_t != 0 {
            return imsg_fail(EINVAL, -(1 as c_int));
        }
        imsgbuf.maxsize = max;
        0 as c_int
    }
}

pub unsafe fn imsgbuf_read(imsgbuf: &mut imsgbuf) -> c_int {
    unsafe {
        if imsg_fdpass(imsgbuf) {
            msgbuf_read(imsgbuf.fd, imsgbuf_msgbuf(imsgbuf))
        } else {
            ibuf_read(imsgbuf.fd, imsgbuf_msgbuf(imsgbuf))
        }
    }
}

pub unsafe fn imsgbuf_write(imsgbuf: &mut imsgbuf) -> c_int {
    unsafe {
        if imsg_fdpass(imsgbuf) {
            msgbuf_write(imsgbuf.fd, imsgbuf_msgbuf(imsgbuf))
        } else {
            ibuf_write(imsgbuf.fd, imsgbuf_msgbuf(imsgbuf))
        }
    }
}

pub unsafe fn imsgbuf_flush(imsgbuf: &mut imsgbuf) -> c_int {
    unsafe {
        while imsgbuf_queuelen(imsgbuf) > 0 as uint32_t {
            if imsgbuf_write(imsgbuf) == -(1 as c_int) {
                return -(1 as c_int);
            }
        }
        0 as c_int
    }
}

pub unsafe fn imsgbuf_clear(imsgbuf: &mut imsgbuf) {
    let _ = imsgbuf.w.take();
}

pub unsafe fn imsgbuf_queuelen(imsgbuf: &mut imsgbuf) -> uint32_t {
    unsafe { msgbuf_queuelen(imsgbuf_msgbuf(imsgbuf)) }
}

pub unsafe fn imsgbuf_get(imsgbuf: &mut imsgbuf, imsg: *mut imsg) -> c_int {
    unsafe {
        let Some(buf) = msgbuf_get(imsgbuf_msgbuf(imsgbuf)) else {
            return 0 as c_int;
        };
        imsg_from_ibuf(buf, imsg)
    }
}

pub unsafe fn imsg_get(imsgbuf: &mut imsgbuf, imsg: *mut imsg) -> ssize_t {
    unsafe {
        let rv = imsgbuf_get(imsgbuf, imsg);
        if rv != 1 as c_int {
            return rv as ssize_t;
        }
        imsg_get_len(imsg).wrapping_add(IMSG_HEADER_SIZE) as ssize_t
    }
}

pub unsafe fn imsg_ibufq_pop(bufq: *mut ibufqueue, imsg: *mut imsg) -> c_int {
    unsafe {
        let Some(buf) = ibufq_pop(bufq) else {
            return 0 as c_int;
        };
        imsg_from_ibuf(buf, imsg)
    }
}

pub unsafe fn imsg_ibufq_push(bufq: *mut ibufqueue, imsg: *mut imsg) {
    unsafe {
        if let Some(buf) = (*imsg).buf.take() {
            let buf_ptr = &raw const *buf as *mut ibuf;
            ibuf_rewind(buf_ptr);
            ibufq_push(bufq, buf);
        }
        *imsg = self::imsg::default();
    }
}

pub unsafe fn imsg_get_ibuf(imsg: *mut imsg, ibuf: *mut ibuf) -> c_int {
    unsafe {
        if ibuf_size(imsg_buf(imsg)) == 0 as size_t {
            return imsg_fail(EBADMSG, -(1 as c_int));
        }
        ibuf_get_ibuf(imsg_buf(imsg), ibuf_size(imsg_buf(imsg)), ibuf)
    }
}

pub unsafe fn imsg_get_data(imsg: *mut imsg, data: *mut c_uchar, len: size_t) -> c_int {
    unsafe {
        if len == 0 as size_t {
            return imsg_fail(EINVAL, -(1 as c_int));
        }
        if ibuf_size(imsg_buf(imsg)) != len {
            return imsg_fail(EBADMSG, -(1 as c_int));
        }
        ibuf_get(imsg_buf(imsg), data, len)
    }
}

pub unsafe fn imsg_get_buf(imsg: *mut imsg, data: *mut c_uchar, len: size_t) -> c_int {
    unsafe { ibuf_get(imsg_buf(imsg), data, len) }
}

pub unsafe fn imsg_get_strbuf(imsg: *mut imsg, str: *mut c_char, len: size_t) -> c_int {
    unsafe { ibuf_get_strbuf(imsg_buf(imsg), str, len) }
}

pub unsafe fn imsg_get_fd(imsg: *mut imsg) -> c_int {
    unsafe { ibuf_fd_get(imsg_buf(imsg)) }
}

pub unsafe fn imsg_get_id(imsg: *mut imsg) -> uint32_t {
    unsafe { (*imsg).hdr.peerid }
}

pub unsafe fn imsg_get_len(imsg: *mut imsg) -> size_t {
    unsafe { ibuf_size(imsg_buf(imsg)) }
}

pub unsafe fn imsg_get_pid(imsg: *mut imsg) -> pid_t {
    unsafe { (*imsg).hdr.pid as pid_t }
}

pub unsafe fn imsg_get_type(imsg: *mut imsg) -> uint32_t {
    unsafe { (*imsg).hdr.type_0 }
}

pub unsafe fn imsg_compose(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    fd: c_int,
    data: *const c_uchar,
    datalen: size_t,
) -> c_int {
    unsafe {
        let Some(mut wbuf) = imsg_create(imsgbuf, type_0, id, pid, datalen) else {
            return -(1 as c_int);
        };
        if ibuf_add(&raw mut *wbuf, data, datalen) != -(1 as c_int) {
            ibuf_fd_set(&raw mut *wbuf, fd);
            imsg_close(imsgbuf, wbuf);
            return 1 as c_int;
        }
        ibuf_free(wbuf);
        -(1 as c_int)
    }
}

pub unsafe fn imsg_composev(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
) -> c_int {
    unsafe {
        let pieces = if iovcnt > 0 as c_int {
            ::core::slice::from_raw_parts(iov, iovcnt as usize)
        } else {
            &[]
        };
        let datalen = pieces
            .iter()
            .fold(0 as size_t, |sum, piece| sum.wrapping_add(piece.iov_len));
        let Some(mut wbuf) = imsg_create(imsgbuf, type_0, id, pid, datalen) else {
            return -(1 as c_int);
        };
        let wbuf_ptr = &raw mut *wbuf;
        if pieces.iter().all(|piece| {
            ibuf_add(wbuf_ptr, piece.iov_base as *const c_uchar, piece.iov_len) != -(1 as c_int)
        }) {
            ibuf_fd_set(wbuf_ptr, fd);
            imsg_close(imsgbuf, wbuf);
            return 1 as c_int;
        }
        ibuf_free(wbuf);
        -(1 as c_int)
    }
}

/// The header a message about to be sent carries. A message with no process
/// of its own is sent as this one's.
unsafe fn imsg_make_hdr(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    len: uint32_t,
) -> imsg_hdr {
    imsg_hdr {
        type_0,
        len,
        peerid: id,
        pid: if pid as uint32_t == 0 as uint32_t {
            imsgbuf.pid as uint32_t
        } else {
            pid as uint32_t
        },
    }
}

pub unsafe fn imsg_compose_ibuf(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    mut buf: Box<ibuf>,
) -> c_int {
    unsafe {
        let mut hdrbuf: Option<Box<ibuf>> = None;
        let len = ibuf_size(&raw mut *buf).wrapping_add(IMSG_HEADER_SIZE);
        if len > imsgbuf.maxsize as size_t {
            *__errno_location() = ERANGE;
        } else {
            let hdr = imsg_make_hdr(imsgbuf, type_0, id, pid, len as uint32_t);
            hdrbuf = ibuf_open(IMSG_HEADER_SIZE);
            if let Some(mut hdrbuf_box) = hdrbuf.take() {
                if imsg_add_hdr(&raw mut *hdrbuf_box, &raw const hdr) != -(1 as c_int) {
                    ibuf_close(imsgbuf_msgbuf(imsgbuf), hdrbuf_box);
                    ibuf_close(imsgbuf_msgbuf(imsgbuf), buf);
                    return 1 as c_int;
                }
                hdrbuf = Some(hdrbuf_box);
            }
        }
        ibuf_free(buf);
        if let Some(hdrbuf) = hdrbuf {
            ibuf_free(hdrbuf);
        }
        -(1 as c_int)
    }
}

pub unsafe fn imsg_forward(imsgbuf: &mut imsgbuf, msg: *mut imsg) -> c_int {
    unsafe {
        ibuf_rewind(imsg_buf(msg));
        ibuf_skip(imsg_buf(msg), ::core::mem::size_of::<imsg_hdr>() as size_t);
        let len = ibuf_size(imsg_buf(msg));
        let wbuf = imsg_create(
            imsgbuf,
            (*msg).hdr.type_0,
            (*msg).hdr.peerid,
            (*msg).hdr.pid as pid_t,
            len,
        );
        let Some(mut wbuf) = wbuf else {
            return -(1 as c_int);
        };
        if len != 0 as size_t && ibuf_add_ibuf(&raw mut *wbuf, imsg_buf(msg)) == -(1 as c_int) {
            ibuf_free(wbuf);
            return -(1 as c_int);
        }
        imsg_close(imsgbuf, wbuf);
        1 as c_int
    }
}

pub unsafe fn imsg_create(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    datalen: size_t,
) -> Option<Box<ibuf>> {
    unsafe {
        let datalen = datalen.wrapping_add(IMSG_HEADER_SIZE);
        if datalen > imsgbuf.maxsize as size_t {
            return imsg_fail(ERANGE, None);
        }
        // The length goes in when the message is closed, once it is known.
        let hdr = imsg_make_hdr(imsgbuf, type_0, id, pid, 0 as uint32_t);
        let mut wbuf = ibuf_dynamic(datalen, imsgbuf.maxsize as size_t)?;
        if imsg_add_hdr(&raw mut *wbuf, &raw const hdr) != -(1 as c_int) {
            return Some(wbuf);
        }
        ibuf_free(wbuf);
        None
    }
}

/// Adds a piece to the message. A piece that does not fit takes the message
/// with it: `msg` is left empty and the answer is -1.
pub unsafe fn imsg_add(
    msg: &mut Option<Box<ibuf>>,
    data: *const c_uchar,
    datalen: size_t,
) -> c_int {
    unsafe {
        let Some(buf) = msg.as_deref_mut() else {
            return -(1 as c_int);
        };
        if datalen != 0 && ibuf_add(&raw mut *buf, data, datalen) == -(1 as c_int) {
            ibuf_free(msg.take().expect("the message just looked at"));
            return -(1 as c_int);
        }
        datalen as c_int
    }
}

pub unsafe fn imsg_close(imsgbuf: &mut imsgbuf, mut msg: Box<ibuf>) {
    unsafe {
        let msg_ptr = &raw mut *msg;
        let mut len = ibuf_size(msg_ptr) as uint32_t;
        if ibuf_fd_avail(msg_ptr) != 0 {
            len = (len as c_uint | IMSG_FD_MARK) as uint32_t;
        }
        ibuf_set_h32(msg_ptr, 4 as size_t, len as uint64_t);
        ibuf_close(imsgbuf_msgbuf(imsgbuf), msg);
    }
}

pub unsafe fn imsg_free(imsg: *mut imsg) {
    unsafe {
        if let Some(buf) = (*imsg).buf.take() {
            ibuf_free(buf);
        }
    }
}

pub unsafe fn imsg_set_maxsize(msg: *mut ibuf, max: size_t) -> c_int {
    unsafe {
        if max > (UINT32_MAX as usize).wrapping_sub(IMSG_HEADER_SIZE) {
            return imsg_fail(ERANGE, -(1 as c_int));
        }
        ibuf_set_maxsize(msg, max.wrapping_add(IMSG_HEADER_SIZE))
    }
}

/// The reader the message buffer calls with each header as it arrives: it
/// answers a buffer sized for the whole message, and takes over the descriptor
/// that came with it when the header says one did.
unsafe fn imsg_parse_hdr(
    buf: *mut ibuf,
    imsgbuf: *mut imsgbuf,
    fd: *mut c_int,
) -> Option<Box<ibuf>> {
    unsafe {
        let mut hdr = imsg_blank_hdr();
        if imsg_get_hdr(buf, &raw mut hdr) == -(1 as c_int) {
            return None;
        }
        let len = hdr.len & !(IMSG_FD_MARK as uint32_t);
        if (len as usize) < IMSG_HEADER_SIZE || len > (*imsgbuf).maxsize {
            return imsg_fail(ERANGE, None);
        }
        let mut b = ibuf_open(len as size_t)?;
        if hdr.len & IMSG_FD_MARK as uint32_t != 0 {
            ibuf_fd_set(&raw mut *b, *fd);
            *fd = -(1 as c_int);
        }
        Some(b)
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_imsg.rs"]
mod tests;
