mod imsg_buffer;
mod message;

pub use imsg_buffer::ibufqueue;
pub use message::{
    ibuf, ibuf_from_buffer, ibuf_from_ibuf, ibuf_get_ibuf, imsg, imsg_hdr, imsgbuf,
    msg_command, msgbuf, msgtype, OwnedIbuf, IMSG_HEADER_SIZE, MAX_IMSGSIZE, PROTOCOL_VERSION,
    MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING,
    MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE,
    MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY,
    MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP,
    MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_DONE, MSG_WRITE_OPEN, MSG_WRITE_READY,
};

use imsg_buffer::{
    ibuf_add, ibuf_close, ibuf_data, ibuf_dynamic, ibuf_fd_avail, ibuf_fd_get, ibuf_fd_set,
    ibuf_free, ibuf_get, ibuf_open, ibuf_read, ibuf_set_h32, ibuf_size, ibuf_write, msgbuf_free,
    msgbuf_get, msgbuf_new_reader_owned, msgbuf_queuelen, msgbuf_read, msgbuf_write, IbufView,
};
use crate::src::ffi::libc::{__errno_location, getpid, memset};
use crate::src::shared::abi::*;
use crate::src::shared::abi::uint32_t;
use crate::src::shared::errno::ERANGE;
use std::os::fd::{IntoRawFd, OwnedFd};

pub const IMSG_ALLOW_FDPASS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const IMSG_FD_MARK: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
pub unsafe fn imsgbuf_init(
    mut imsgbuf: *mut imsgbuf,
    mut fd: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let maxsize = MAX_IMSGSIZE as uint32_t;
    (*imsgbuf).w = msgbuf_new_reader_owned(IMSG_HEADER_SIZE, move |header, fd| unsafe {
        imsg_parse_hdr(header, maxsize, fd)
    });
    if (*imsgbuf).w.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*imsgbuf).pid = getpid() as pid_t;
    (*imsgbuf).maxsize = maxsize;
    (*imsgbuf).fd = fd;
    (*imsgbuf).flags = 0 as ::core::ffi::c_int;
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn imsgbuf_allow_fdpass(mut imsgbuf: *mut imsgbuf) {
    (*imsgbuf).flags |= IMSG_ALLOW_FDPASS;
}
pub unsafe fn imsgbuf_read(
    mut imsgbuf: *mut imsgbuf,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    if (*imsgbuf).flags & IMSG_ALLOW_FDPASS != 0 {
        return msgbuf_read((*imsgbuf).fd, (*imsgbuf).w);
    } else {
        return ibuf_read((*imsgbuf).fd, (*imsgbuf).w);
    };
}
pub unsafe fn imsgbuf_write(mut imsgbuf: *mut imsgbuf) -> ::core::ffi::c_int {
    if (*imsgbuf).flags & IMSG_ALLOW_FDPASS != 0 {
        return msgbuf_write((*imsgbuf).fd, (*imsgbuf).w);
    } else {
        return ibuf_write((*imsgbuf).fd, (*imsgbuf).w);
    };
}
pub unsafe fn imsgbuf_flush(mut imsgbuf: *mut imsgbuf) -> ::core::ffi::c_int {
    while imsgbuf_queuelen(imsgbuf) > 0 as uint32_t {
        if imsgbuf_write(imsgbuf) == -(1 as ::core::ffi::c_int) {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn imsgbuf_clear(mut imsgbuf: *mut imsgbuf) {
    msgbuf_free((*imsgbuf).w);
    (*imsgbuf).w = ::core::ptr::null_mut::<msgbuf>();
}
pub unsafe fn imsgbuf_queuelen(mut imsgbuf: *mut imsgbuf) -> uint32_t {
    return msgbuf_queuelen((*imsgbuf).w);
}
pub unsafe fn imsgbuf_get(mut imsgbuf: *mut imsgbuf, mut imsg: *mut imsg) -> ::core::ffi::c_int {
    let mut m: imsg = imsg {
        hdr: imsg_hdr {
            type_0: 0,
            len: 0,
            peerid: 0,
            pid: 0,
        },
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        buf: ::core::ptr::null_mut::<OwnedIbuf>(),
    };
    let mut buf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    buf = msgbuf_get((*imsgbuf).w);
    if buf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if ibuf_get(
        buf,
        &raw mut m.hdr as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<imsg_hdr>() as size_t,
    ) == -(1 as ::core::ffi::c_int)
    {
        ibuf_free(buf);
        return -(1 as ::core::ffi::c_int);
    }
    if ibuf_size(buf) != 0 {
        m.data = ibuf_data(buf);
    } else {
        m.data = NULL;
    }
    m.buf = buf;
    m.hdr.len = (m.hdr.len as ::core::ffi::c_uint & !IMSG_FD_MARK) as uint32_t;
    *imsg = m;
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn imsg_get_fd(mut imsg: *mut imsg) -> ::core::ffi::c_int {
    return ibuf_fd_get((*imsg).buf);
}
pub unsafe fn imsg_compose(
    mut imsgbuf: *mut imsgbuf,
    mut type_0: uint32_t,
    mut id: uint32_t,
    mut pid: pid_t,
    mut fd: ::core::ffi::c_int,
    mut data: *const ::core::ffi::c_void,
    mut datalen: size_t,
) -> ::core::ffi::c_int {
    let Some(mut wbuf) = imsg_create(imsgbuf, type_0, id, pid, datalen) else {
        return -(1 as ::core::ffi::c_int);
    };
    let wbuf_ptr = &mut *wbuf as *mut OwnedIbuf;
    if ibuf_add(wbuf_ptr, data, datalen) == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    ibuf_fd_set(wbuf_ptr, fd);
    imsg_close(imsgbuf, wbuf);
    1 as ::core::ffi::c_int
}
pub unsafe fn imsg_create(
    mut imsgbuf: *mut imsgbuf,
    mut type_0: uint32_t,
    mut id: uint32_t,
    mut pid: pid_t,
    mut datalen: size_t,
) -> Option<Box<OwnedIbuf>> {
    let mut hdr: imsg_hdr = imsg_hdr {
        type_0: 0,
        len: 0,
        peerid: 0,
        pid: 0,
    };
    datalen = (datalen as ::core::ffi::c_ulong)
        .wrapping_add(IMSG_HEADER_SIZE as ::core::ffi::c_ulong) as size_t as size_t;
    if datalen > (*imsgbuf).maxsize as size_t {
        *__errno_location() = ERANGE;
        return None;
    }
    hdr.len = 0 as uint32_t;
    hdr.type_0 = type_0;
    hdr.peerid = id;
    hdr.pid = pid as uint32_t;
    if hdr.pid == 0 as uint32_t {
        hdr.pid = (*imsgbuf).pid as uint32_t;
    }
    let mut wbuf = ibuf_dynamic(datalen, (*imsgbuf).maxsize as size_t)?;
    if ibuf_add(
        &mut *wbuf as *mut OwnedIbuf,
        &raw mut hdr as *const ::core::ffi::c_void,
        ::core::mem::size_of::<imsg_hdr>() as size_t,
    ) == -(1 as ::core::ffi::c_int)
    {
        return None;
    }
    Some(wbuf)
}
pub unsafe fn imsg_close(mut imsgbuf: *mut imsgbuf, mut msg: Box<OwnedIbuf>) {
    let mut len: uint32_t = 0;
    let msg_ptr = &mut *msg as *mut OwnedIbuf;
    len = ibuf_size(msg_ptr) as uint32_t;
    if ibuf_fd_avail(msg_ptr) != 0 {
        len = (len as ::core::ffi::c_uint | IMSG_FD_MARK) as uint32_t;
    }
    ibuf_set_h32(msg_ptr, 4 as size_t, len as uint64_t);
    ibuf_close((*imsgbuf).w, msg);
}
unsafe fn imsg_parse_hdr(
    header: &[u8],
    maxsize: uint32_t,
    mut fd: Option<OwnedFd>,
) -> (Option<Box<OwnedIbuf>>, Option<OwnedFd>) {
    let mut view = IbufView::new(header);
    let mut hdr: imsg_hdr = imsg_hdr {
        type_0: 0,
        len: 0,
        peerid: 0,
        pid: 0,
    };
    let mut len: uint32_t = 0;
    let header_view = unsafe { view.as_ibuf_ptr() };
    if ibuf_get(
        header_view,
        &raw mut hdr as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<imsg_hdr>() as size_t,
    ) == -(1 as ::core::ffi::c_int)
    {
        return (None, fd);
    }
    len = hdr.len & !(IMSG_FD_MARK as uint32_t);
    if (len as usize) < IMSG_HEADER_SIZE || len > maxsize {
        *__errno_location() = ERANGE;
        return (None, fd);
    }
    let Some(mut b) = ibuf_open(len as size_t) else {
        return (None, fd);
    };
    if hdr.len & IMSG_FD_MARK as uint32_t != 0 {
        let raw_fd = fd.take().map(IntoRawFd::into_raw_fd).unwrap_or(-1);
        ibuf_fd_set(&mut *b as *mut OwnedIbuf, raw_fd);
    }
    (Some(b), fd)
}
