use crate::compat::imsg_buffer::{
    ibuf_add, ibuf_close, ibuf_dynamic, ibuf_fd_avail, ibuf_fd_get, ibuf_fd_set, ibuf_free,
    ibuf_get, ibuf_open, ibuf_read, ibuf_set_h32, ibuf_size, ibuf_write, msgbuf_get,
    msgbuf_new_reader, msgbuf_queuelen, msgbuf_read, msgbuf_write,
};

pub use crate::consts::{ERANGE, IMSG_HEADER_SIZE, MAX_IMSGSIZE};
use crate::ffi::{__errno_location, getpid};
pub use crate::types::*;
use ::core::ffi::{c_int, c_uint};

pub const IMSG_ALLOW_FDPASS: c_int = 0x1 as c_int;

/// The top bit of a header's length field, set when a descriptor rides along
/// with the message. It is masked off before the length is read.
pub const IMSG_FD_MARK: c_uint = 0x80000000 as c_uint;

fn imsgbuf_msgbuf(imsgbuf: &mut imsgbuf) -> &mut msgbuf {
    imsgbuf
        .w
        .as_deref_mut()
        .expect("an initialized message buffer")
}

/// Sets the error number and answers the failure the caller hands back.
unsafe fn imsg_fail<T>(errno: c_int, answer: T) -> T {
    unsafe {
        *__errno_location() = errno;
        answer
    }
}

/// Puts `hdr` at the end of `buf`.
fn imsg_add_hdr(buf: &mut ibuf, hdr: &imsg_hdr) -> c_int {
    ibuf_add(buf, &hdr.to_ne_bytes())
}

/// The buffer a message was read into, as the borrowed view the reading
/// calls take.
fn imsg_buf(imsg: &mut imsg) -> &mut ibuf {
    imsg.buf.as_deref_mut().unwrap()
}

/// Reads a header off the front of `buf`.
unsafe fn imsg_get_hdr(buf: &mut ibuf) -> Option<imsg_hdr> {
    unsafe {
        let mut bytes = [0; IMSG_HEADER_SIZE];
        if ibuf_get(buf, &mut bytes) == -(1 as c_int) {
            None
        } else {
            Some(imsg_hdr::from_ne_bytes(bytes))
        }
    }
}

/// Takes the header off `buf` and builds the message around what is left. The
/// buffer becomes the message's, whether this works or not.
unsafe fn imsg_from_ibuf(mut buf: Box<ibuf>) -> Option<imsg> {
    unsafe {
        let mut m = imsg::default();
        if let Some(hdr) = imsg_get_hdr(&mut buf) {
            m.hdr = hdr;
        } else {
            ibuf_free(buf);
            return None;
        }
        m.body_range = buf.rpos..buf.wpos;
        m.buf = Some(buf);
        m.hdr.len = (m.hdr.len as c_uint & !IMSG_FD_MARK) as uint32_t;
        Some(m)
    }
}

pub unsafe fn imsgbuf_init(imsgbuf: &mut imsgbuf, fd: c_int) -> c_int {
    unsafe {
        let Some(w) = msgbuf_new_reader(
            IMSG_HEADER_SIZE,
            Some(std::rc::Rc::new(imsg_parse_hdr)),
            Some(MAX_IMSGSIZE as uint32_t),
        ) else {
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

pub fn imsgbuf_allow_fdpass(imsgbuf: &mut imsgbuf) {
    imsgbuf.flags |= IMSG_ALLOW_FDPASS;
}

/// Whether this buffer's messages may carry a descriptor, which decides
/// whether it goes over the socket as a plain write or as a message.
fn imsg_fdpass(imsgbuf: &imsgbuf) -> bool {
    imsgbuf.flags & IMSG_ALLOW_FDPASS != 0
}

pub unsafe fn imsgbuf_read(imsgbuf: &mut imsgbuf) -> c_int {
    unsafe {
        let fd = imsgbuf.fd;
        let fdpass = imsg_fdpass(imsgbuf);
        let maxsize = imsgbuf.maxsize;
        let reader = imsgbuf_msgbuf(imsgbuf);
        reader.read_limit = Some(maxsize);
        if fdpass {
            msgbuf_read(fd, reader)
        } else {
            ibuf_read(fd, reader)
        }
    }
}

pub unsafe fn imsgbuf_write(imsgbuf: &mut imsgbuf) -> c_int {
    unsafe {
        let fdpass = imsg_fdpass(imsgbuf);
        let Some(w) = imsgbuf.w.as_deref_mut() else {
            return -(1 as c_int);
        };
        if fdpass {
            msgbuf_write(imsgbuf.fd, w)
        } else {
            ibuf_write(imsgbuf.fd, w)
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

pub fn imsgbuf_clear(imsgbuf: &mut imsgbuf) {
    let _ = imsgbuf.w.take();
}

pub fn imsgbuf_queuelen(imsgbuf: &imsgbuf) -> uint32_t {
    msgbuf_queuelen(imsgbuf.w.as_deref().expect("an initialized message buffer"))
}

pub unsafe fn imsgbuf_get(imsgbuf: &mut imsgbuf) -> Result<Option<imsg>, ()> {
    unsafe {
        let Some(buf) = msgbuf_get(imsgbuf_msgbuf(imsgbuf)) else {
            return Ok(None);
        };
        let Some(m) = imsg_from_ibuf(buf) else {
            return Err(());
        };
        Ok(Some(m))
    }
}

#[derive(Debug)]
pub struct InvalidImsg;

pub unsafe fn imsg_get(imsgbuf: &mut imsgbuf) -> Result<Option<(imsg, size_t)>, InvalidImsg> {
    unsafe {
        let Some(m) = (match imsgbuf_get(imsgbuf) {
            Ok(m) => m,
            Err(()) => return Err(InvalidImsg),
        }) else {
            return Ok(None);
        };
        let len = imsg_get_len(&m).wrapping_add(IMSG_HEADER_SIZE);
        Ok(Some((m, len)))
    }
}

pub fn imsg_get_fd(imsg: &mut imsg) -> c_int {
    ibuf_fd_get(imsg_buf(imsg))
}

pub fn imsg_get_len(imsg: &imsg) -> size_t {
    ibuf_size(imsg.buf.as_deref().unwrap())
}

#[cfg(test)]
pub fn imsg_get_type(imsg: &imsg) -> uint32_t {
    imsg.hdr.type_0
}

pub unsafe fn imsg_compose(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    fd: c_int,
    data: &[u8],
) -> c_int {
    unsafe {
        let Some(mut wbuf) = imsg_create(imsgbuf, type_0, id, pid, data.len()) else {
            return -(1 as c_int);
        };
        if ibuf_add(&mut wbuf, data) != -(1 as c_int) {
            ibuf_fd_set(&mut wbuf, fd);
            imsg_close(imsgbuf, wbuf);
            return 1 as c_int;
        }
        ibuf_free(wbuf);
        -(1 as c_int)
    }
}

/// The header a message about to be sent carries. A message with no process
/// of its own is sent as this one's.
fn imsg_make_hdr(
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
        if imsg_add_hdr(&mut wbuf, &hdr) != -(1 as c_int) {
            return Some(wbuf);
        }
        ibuf_free(wbuf);
        None
    }
}

pub unsafe fn imsg_close(imsgbuf: &mut imsgbuf, mut msg: Box<ibuf>) {
    unsafe {
        let mut len = ibuf_size(&msg) as uint32_t;
        if ibuf_fd_avail(&msg) != 0 {
            len = (len as c_uint | IMSG_FD_MARK) as uint32_t;
        }
        ibuf_set_h32(&mut msg, 4 as size_t, len as uint64_t);
        ibuf_close(imsgbuf_msgbuf(imsgbuf), msg);
    }
}

pub unsafe fn imsg_free(mut imsg: imsg) {
    unsafe {
        if let Some(buf) = imsg.buf.take() {
            ibuf_free(buf);
        }
    }
}

/// The reader the message buffer calls with each header as it arrives: it
/// answers a buffer sized for the whole message, and takes over the descriptor
/// that came with it when the header says one did.
fn imsg_parse_hdr(buf: &mut ibuf, maxsize: Option<uint32_t>, fd: &mut c_int) -> Option<Box<ibuf>> {
    unsafe {
        let maxsize = maxsize.expect("imsg reader without a message-size limit");
        let hdr = imsg_get_hdr(buf)?;
        let len = hdr.len & !(IMSG_FD_MARK as uint32_t);
        if (len as usize) < IMSG_HEADER_SIZE || len > maxsize {
            return imsg_fail(ERANGE, None);
        }
        let mut b = ibuf_open(len as size_t)?;
        if hdr.len & IMSG_FD_MARK as uint32_t != 0 {
            ibuf_fd_set(&mut b, *fd);
            *fd = -(1 as c_int);
        }
        Some(b)
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_imsg.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::imsg_get_buf;

#[cfg(test)]
pub use crate::consts::{EBADMSG, EINVAL, UINT32_MAX};
