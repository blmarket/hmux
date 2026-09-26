use crate::src::compat::imsg_buffer::{
    ibuf_add, ibuf_add_ibuf, ibuf_close, ibuf_data, ibuf_dynamic, ibuf_fd_avail, ibuf_fd_get,
    ibuf_fd_set, ibuf_free, ibuf_get, ibuf_get_ibuf, ibuf_get_strbuf, ibuf_open, ibuf_read,
    ibuf_rewind, ibuf_set_h32, ibuf_set_maxsize, ibuf_size, ibuf_skip, ibuf_write, ibufq_pop,
    ibufq_push, msgbuf_free, msgbuf_get, msgbuf_new_reader_owned, msgbuf_queuelen, msgbuf_read,
    msgbuf_write, IbufView,
};
use crate::src::ffi::libc::{__errno_location, getpid, memset};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{ssize_t, uint32_t};
use crate::src::shared::errno::{EBADMSG, EINVAL, ERANGE};
use crate::src::shared::limits::UINT32_MAX;
pub use crate::src::shared::message::{ibuf, ibufqueue, imsg, imsgbuf, msgbuf, OwnedIbuf};
use crate::src::shared::message::{imsg_hdr, IMSG_HEADER_SIZE, MAX_IMSGSIZE};
use crate::src::shared::posix_io::iovec;
use std::os::fd::{IntoRawFd, OwnedFd};

pub const IMSG_ALLOW_FDPASS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const IMSG_FD_MARK: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_init(
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
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_allow_fdpass(mut imsgbuf: *mut imsgbuf) {
    (*imsgbuf).flags |= IMSG_ALLOW_FDPASS;
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_set_maxsize(
    mut imsgbuf: *mut imsgbuf,
    mut max: uint32_t,
) -> ::core::ffi::c_int {
    if max as usize > (UINT32_MAX as usize).wrapping_sub(IMSG_HEADER_SIZE) {
        *__errno_location() = ERANGE;
        return -(1 as ::core::ffi::c_int);
    }
    max = (max as ::core::ffi::c_ulong).wrapping_add(IMSG_HEADER_SIZE as ::core::ffi::c_ulong)
        as uint32_t as uint32_t;
    if max & IMSG_FD_MARK as uint32_t != 0 {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    (*imsgbuf).maxsize = max;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_read(mut imsgbuf: *mut imsgbuf) -> ::core::ffi::c_int {
    if (*imsgbuf).flags & IMSG_ALLOW_FDPASS != 0 {
        return msgbuf_read((*imsgbuf).fd, (*imsgbuf).w);
    } else {
        return ibuf_read((*imsgbuf).fd, (*imsgbuf).w);
    };
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_write(mut imsgbuf: *mut imsgbuf) -> ::core::ffi::c_int {
    if (*imsgbuf).flags & IMSG_ALLOW_FDPASS != 0 {
        return msgbuf_write((*imsgbuf).fd, (*imsgbuf).w);
    } else {
        return ibuf_write((*imsgbuf).fd, (*imsgbuf).w);
    };
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_flush(mut imsgbuf: *mut imsgbuf) -> ::core::ffi::c_int {
    while imsgbuf_queuelen(imsgbuf) > 0 as uint32_t {
        if imsgbuf_write(imsgbuf) == -(1 as ::core::ffi::c_int) {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_clear(mut imsgbuf: *mut imsgbuf) {
    msgbuf_free((*imsgbuf).w);
    (*imsgbuf).w = ::core::ptr::null_mut::<msgbuf>();
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_queuelen(mut imsgbuf: *mut imsgbuf) -> uint32_t {
    return msgbuf_queuelen((*imsgbuf).w);
}
#[no_mangle]
pub unsafe extern "C" fn imsgbuf_get(
    mut imsgbuf: *mut imsgbuf,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
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
#[no_mangle]
pub unsafe extern "C" fn imsg_get(mut imsgbuf: *mut imsgbuf, mut imsg: *mut imsg) -> ssize_t {
    let mut rv: ::core::ffi::c_int = 0;
    rv = imsgbuf_get(imsgbuf, imsg);
    if rv != 1 as ::core::ffi::c_int {
        return rv as ssize_t;
    }
    return imsg_get_len(imsg).wrapping_add(IMSG_HEADER_SIZE) as ssize_t;
}
#[no_mangle]
pub unsafe extern "C" fn imsg_ibufq_pop(
    mut bufq: *mut ibufqueue,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
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
    buf = ibufq_pop(bufq);
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
#[no_mangle]
pub unsafe extern "C" fn imsg_ibufq_push(mut bufq: *mut ibufqueue, mut imsg: *mut imsg) {
    ibuf_rewind((*imsg).buf);
    ibufq_push(bufq, (*imsg).buf);
    memset(
        imsg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<imsg>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_ibuf(
    mut imsg: *mut imsg,
    mut ibuf: *mut OwnedIbuf,
) -> ::core::ffi::c_int {
    if ibuf_size((*imsg).buf) == 0 as size_t {
        *__errno_location() = EBADMSG;
        return -(1 as ::core::ffi::c_int);
    }
    return ibuf_get_ibuf((*imsg).buf, ibuf_size((*imsg).buf), ibuf);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_data(
    mut imsg: *mut imsg,
    mut data: *mut ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if len == 0 as size_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    if ibuf_size((*imsg).buf) != len {
        *__errno_location() = EBADMSG;
        return -(1 as ::core::ffi::c_int);
    }
    return ibuf_get((*imsg).buf, data, len);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_buf(
    mut imsg: *mut imsg,
    mut data: *mut ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    return ibuf_get((*imsg).buf, data, len);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_strbuf(
    mut imsg: *mut imsg,
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    return ibuf_get_strbuf((*imsg).buf, str, len);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_fd(mut imsg: *mut imsg) -> ::core::ffi::c_int {
    return ibuf_fd_get((*imsg).buf);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_id(mut imsg: *mut imsg) -> uint32_t {
    return (*imsg).hdr.peerid;
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_len(mut imsg: *mut imsg) -> size_t {
    return ibuf_size((*imsg).buf);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_pid(mut imsg: *mut imsg) -> pid_t {
    return (*imsg).hdr.pid as pid_t;
}
#[no_mangle]
pub unsafe extern "C" fn imsg_get_type(mut imsg: *mut imsg) -> uint32_t {
    return (*imsg).hdr.type_0;
}
#[no_mangle]
pub unsafe extern "C" fn imsg_compose(
    mut imsgbuf: *mut imsgbuf,
    mut type_0: uint32_t,
    mut id: uint32_t,
    mut pid: pid_t,
    mut fd: ::core::ffi::c_int,
    mut data: *const ::core::ffi::c_void,
    mut datalen: size_t,
) -> ::core::ffi::c_int {
    let mut wbuf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    wbuf = imsg_create(imsgbuf, type_0, id, pid, datalen);
    if !wbuf.is_null() {
        if !(ibuf_add(wbuf, data, datalen) == -(1 as ::core::ffi::c_int)) {
            ibuf_fd_set(wbuf, fd);
            imsg_close(imsgbuf, wbuf);
            return 1 as ::core::ffi::c_int;
        }
    }
    ibuf_free(wbuf);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_composev(
    mut imsgbuf: *mut imsgbuf,
    mut type_0: uint32_t,
    mut id: uint32_t,
    mut pid: pid_t,
    mut fd: ::core::ffi::c_int,
    mut iov: *const iovec,
    mut iovcnt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut wbuf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    let mut i: ::core::ffi::c_int = 0;
    let mut datalen: size_t = 0 as size_t;
    i = 0 as ::core::ffi::c_int;
    while i < iovcnt {
        datalen = datalen.wrapping_add((*iov.offset(i as isize)).iov_len);
        i += 1;
    }
    wbuf = imsg_create(imsgbuf, type_0, id, pid, datalen);
    if !wbuf.is_null() {
        i = 0 as ::core::ffi::c_int;
        loop {
            if !(i < iovcnt) {
                current_block = 6937071982253665452;
                break;
            }
            if ibuf_add(
                wbuf,
                (*iov.offset(i as isize)).iov_base,
                (*iov.offset(i as isize)).iov_len,
            ) == -(1 as ::core::ffi::c_int)
            {
                current_block = 7614701788242082905;
                break;
            }
            i += 1;
        }
        match current_block {
            7614701788242082905 => {}
            _ => {
                ibuf_fd_set(wbuf, fd);
                imsg_close(imsgbuf, wbuf);
                return 1 as ::core::ffi::c_int;
            }
        }
    }
    ibuf_free(wbuf);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
/// Compose a message by consuming an owned ibuf.
///
/// # Safety
/// `imsgbuf` must be initialized. `buf` must be null or a uniquely owned heap
/// allocation containing owned storage; successful calls transfer it to the
/// outgoing queue, while a rejected owned buffer is freed.
pub unsafe extern "C" fn imsg_compose_ibuf(
    mut imsgbuf: *mut imsgbuf,
    mut type_0: uint32_t,
    mut id: uint32_t,
    mut pid: pid_t,
    mut buf: *mut OwnedIbuf,
) -> ::core::ffi::c_int {
    let mut hdrbuf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    let mut hdr: imsg_hdr = imsg_hdr {
        type_0: 0,
        len: 0,
        peerid: 0,
        pid: 0,
    };
    if buf.is_null() || !(*buf).is_owned() {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    if ibuf_size(buf).wrapping_add(IMSG_HEADER_SIZE) > (*imsgbuf).maxsize as size_t {
        *__errno_location() = ERANGE;
    } else {
        hdr.type_0 = type_0;
        hdr.len = ibuf_size(buf).wrapping_add(IMSG_HEADER_SIZE) as uint32_t;
        hdr.peerid = id;
        hdr.pid = pid as uint32_t;
        if hdr.pid == 0 as uint32_t {
            hdr.pid = (*imsgbuf).pid as uint32_t;
        }
        hdrbuf = ibuf_open(IMSG_HEADER_SIZE);
        if !hdrbuf.is_null() {
            if !(ibuf_add(
                hdrbuf,
                &raw mut hdr as *const ::core::ffi::c_void,
                ::core::mem::size_of::<imsg_hdr>() as size_t,
            ) == -(1 as ::core::ffi::c_int))
            {
                ibuf_close((*imsgbuf).w, hdrbuf);
                ibuf_close((*imsgbuf).w, buf);
                return 1 as ::core::ffi::c_int;
            }
        }
    }
    ibuf_free(buf);
    ibuf_free(hdrbuf);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_forward(
    mut imsgbuf: *mut imsgbuf,
    mut msg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut wbuf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
    let mut len: size_t = 0;
    ibuf_rewind((*msg).buf);
    ibuf_skip((*msg).buf, ::core::mem::size_of::<imsg_hdr>() as size_t);
    len = ibuf_size((*msg).buf);
    wbuf = imsg_create(
        imsgbuf,
        (*msg).hdr.type_0,
        (*msg).hdr.peerid,
        (*msg).hdr.pid as pid_t,
        len,
    );
    if wbuf.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if len != 0 as size_t {
        if ibuf_add_ibuf(wbuf, (*msg).buf) == -(1 as ::core::ffi::c_int) {
            ibuf_free(wbuf);
            return -(1 as ::core::ffi::c_int);
        }
    }
    imsg_close(imsgbuf, wbuf);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn imsg_create(
    mut imsgbuf: *mut imsgbuf,
    mut type_0: uint32_t,
    mut id: uint32_t,
    mut pid: pid_t,
    mut datalen: size_t,
) -> *mut OwnedIbuf {
    let mut wbuf: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
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
        return ::core::ptr::null_mut::<OwnedIbuf>();
    }
    hdr.len = 0 as uint32_t;
    hdr.type_0 = type_0;
    hdr.peerid = id;
    hdr.pid = pid as uint32_t;
    if hdr.pid == 0 as uint32_t {
        hdr.pid = (*imsgbuf).pid as uint32_t;
    }
    wbuf = ibuf_dynamic(datalen, (*imsgbuf).maxsize as size_t);
    if !wbuf.is_null() {
        if !(ibuf_add(
            wbuf,
            &raw mut hdr as *const ::core::ffi::c_void,
            ::core::mem::size_of::<imsg_hdr>() as size_t,
        ) == -(1 as ::core::ffi::c_int))
        {
            return wbuf;
        }
    }
    ibuf_free(wbuf);
    return ::core::ptr::null_mut::<OwnedIbuf>();
}
#[no_mangle]
pub unsafe extern "C" fn imsg_add(
    mut msg: *mut OwnedIbuf,
    mut data: *const ::core::ffi::c_void,
    mut datalen: size_t,
) -> ::core::ffi::c_int {
    if datalen != 0 {
        if ibuf_add(msg, data, datalen) == -(1 as ::core::ffi::c_int) {
            ibuf_free(msg);
            return -(1 as ::core::ffi::c_int);
        }
    }
    return datalen as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn imsg_close(mut imsgbuf: *mut imsgbuf, mut msg: *mut OwnedIbuf) {
    let mut len: uint32_t = 0;
    len = ibuf_size(msg) as uint32_t;
    if ibuf_fd_avail(msg) != 0 {
        len = (len as ::core::ffi::c_uint | IMSG_FD_MARK) as uint32_t;
    }
    ibuf_set_h32(msg, 4 as size_t, len as uint64_t);
    ibuf_close((*imsgbuf).w, msg);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_free(mut imsg: *mut imsg) {
    ibuf_free((*imsg).buf);
}
#[no_mangle]
pub unsafe extern "C" fn imsg_set_maxsize(
    mut msg: *mut OwnedIbuf,
    mut max: size_t,
) -> ::core::ffi::c_int {
    if max > (UINT32_MAX as usize).wrapping_sub(IMSG_HEADER_SIZE) {
        *__errno_location() = ERANGE;
        return -(1 as ::core::ffi::c_int);
    }
    return ibuf_set_maxsize(msg, max.wrapping_add(IMSG_HEADER_SIZE));
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
    let mut b: *mut OwnedIbuf = ::core::ptr::null_mut::<OwnedIbuf>();
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
    b = ibuf_open(len as size_t);
    if b.is_null() {
        return (None, fd);
    }
    if hdr.len & IMSG_FD_MARK as uint32_t != 0 {
        let raw_fd = fd.take().map(IntoRawFd::into_raw_fd).unwrap_or(-1);
        ibuf_fd_set(b, raw_fd);
    }
    let Some(b) = OwnedIbuf::from_raw_owned(b) else {
        return (None, fd);
    };
    (Some(b), fd)
}
