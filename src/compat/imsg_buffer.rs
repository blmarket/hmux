use crate::src::compat::freezero::freezero;
use crate::src::compat::htonll::htonll;
use crate::src::compat::ntohll::ntohll;
use crate::src::ffi::libc::{
    __errno_location, abort, calloc, close, free, malloc, memcpy, memmove, memset, readv,
    realloc, recvmsg, sendmsg, strlcpy, strndup, writev,
};
pub use crate::src::ffi::libc::msghdr;
pub use crate::src::shared::posix_io::{iovec};
pub use crate::src::shared::message::{ibuf, ibuf_entry, ibufqueue, ibufqueue_bufs, msgbuf};
pub use crate::src::shared::errno::{EAGAIN, EBADMSG, EINTR, EINVAL, ERANGE};
pub use crate::src::shared::limits::{SIZE_MAX, UINT32_MAX};
pub use crate::src::shared::socket::SOL_SOCKET;
pub use crate::src::shared::abi::{
    __socklen_t, __uint16_t, __uint32_t, socklen_t, ssize_t, uint16_t, uint32_t,
};
use crate::src::shared::abi::*;

pub type __caddr_t = *mut ::core::ffi::c_char;
pub type caddr_t = __caddr_t;

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
unsafe extern "C" fn __bswap_16(mut __bsx: __uint16_t) -> __uint16_t {
    return (__bsx as ::core::ffi::c_int >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as __uint16_t;
}
#[inline]
unsafe extern "C" fn __bswap_32(mut __bsx: __uint32_t) -> __uint32_t {
    return (__bsx & 0xff000000 as __uint32_t) >> 24 as ::core::ffi::c_int
        | (__bsx & 0xff0000 as __uint32_t) >> 8 as ::core::ffi::c_int
        | (__bsx & 0xff00 as __uint32_t) << 8 as ::core::ffi::c_int
        | (__bsx & 0xff as __uint32_t) << 24 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn __cmsg_nxthdr(
    mut __mhdr: *mut msghdr,
    mut __cmsg: *mut cmsghdr,
) -> *mut cmsghdr {
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
pub const IBUF_FD_MARK_ON_STACK: ::core::ffi::c_int = -(2 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn ibuf_open(mut len: size_t) -> *mut ibuf {
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    buf = calloc(1 as size_t, ::core::mem::size_of::<ibuf>() as size_t) as *mut ibuf;
    if buf.is_null() {
        return ::core::ptr::null_mut::<ibuf>();
    }
    if len > 0 as size_t {
        (*buf).buf = calloc(len, 1 as size_t) as *mut ::core::ffi::c_uchar;
        if (*buf).buf.is_null() {
            free(buf as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<ibuf>();
        }
    }
    (*buf).max = len;
    (*buf).size = (*buf).max;
    (*buf).fd = -(1 as ::core::ffi::c_int);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_dynamic(mut len: size_t, mut max: size_t) -> *mut ibuf {
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    if max == 0 as size_t || max < len {
        *__errno_location() = EINVAL;
        return ::core::ptr::null_mut::<ibuf>();
    }
    buf = calloc(1 as size_t, ::core::mem::size_of::<ibuf>() as size_t) as *mut ibuf;
    if buf.is_null() {
        return ::core::ptr::null_mut::<ibuf>();
    }
    if len > 0 as size_t {
        (*buf).buf = calloc(len, 1 as size_t) as *mut ::core::ffi::c_uchar;
        if (*buf).buf.is_null() {
            free(buf as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<ibuf>();
        }
    }
    (*buf).size = len;
    (*buf).max = max;
    (*buf).fd = -(1 as ::core::ffi::c_int);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_reserve(
    mut buf: *mut ibuf,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    let mut b: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if len > (SIZE_MAX as size_t).wrapping_sub((*buf).wpos) {
        *__errno_location() = ERANGE;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
        *__errno_location() = EINVAL;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*buf).wpos.wrapping_add(len) > (*buf).size {
        let mut nb: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
        if (*buf).wpos.wrapping_add(len) > (*buf).max {
            *__errno_location() = ERANGE;
            return ::core::ptr::null_mut::<::core::ffi::c_void>();
        }
        nb = realloc(
            (*buf).buf as *mut ::core::ffi::c_void,
            (*buf).wpos.wrapping_add(len),
        ) as *mut ::core::ffi::c_uchar;
        if nb.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_void>();
        }
        memset(
            nb.offset((*buf).size as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            (*buf).wpos.wrapping_add(len).wrapping_sub((*buf).size),
        );
        (*buf).buf = nb;
        (*buf).size = (*buf).wpos.wrapping_add(len);
    }
    b = (*buf).buf.offset((*buf).wpos as isize) as *mut ::core::ffi::c_void;
    (*buf).wpos = (*buf).wpos.wrapping_add(len);
    return b;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add(
    mut buf: *mut ibuf,
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
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_ibuf(
    mut buf: *mut ibuf,
    mut from: *const ibuf,
) -> ::core::ffi::c_int {
    return ibuf_add(buf, ibuf_data(from), ibuf_size(from));
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_n8(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint8_t = 0;
    if value > UINT8_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = value as uint8_t;
    return ibuf_add(
        buf,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint8_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_n16(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint16_t = 0;
    if value > UINT16_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = __bswap_16(value as __uint16_t) as uint16_t;
    return ibuf_add(
        buf,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint16_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_n32(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint32_t = 0;
    if value > UINT32_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = __bswap_32(value as __uint32_t) as uint32_t;
    return ibuf_add(
        buf,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint32_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_n64(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    value = htonll(value);
    return ibuf_add(
        buf,
        &raw mut value as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_h16(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint16_t = 0;
    if value > UINT16_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = value as uint16_t;
    return ibuf_add(
        buf,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint16_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_h32(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint32_t = 0;
    if value > UINT32_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = value as uint32_t;
    return ibuf_add(
        buf,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint32_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_h64(
    mut buf: *mut ibuf,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    return ibuf_add(
        buf,
        &raw mut value as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_zero(mut buf: *mut ibuf, mut len: size_t) -> ::core::ffi::c_int {
    let mut b: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    b = ibuf_reserve(buf, len);
    if b.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    memset(b, 0 as ::core::ffi::c_int, len);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_add_strbuf(
    mut buf: *mut ibuf,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut b: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    b = ibuf_reserve(buf, len) as *mut ::core::ffi::c_char;
    if b.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    n = strlcpy(b, str, len) as size_t;
    if n >= len {
        *__errno_location() = EOVERFLOW;
        return -(1 as ::core::ffi::c_int);
    }
    memset(
        b.offset(n as isize) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        len.wrapping_sub(n),
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_seek(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    if ibuf_size(buf) < pos
        || (SIZE_MAX as size_t).wrapping_sub(pos) < len
        || ibuf_size(buf) < pos.wrapping_add(len)
    {
        *__errno_location() = ERANGE;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*buf).buf.offset((*buf).rpos as isize).offset(pos as isize)
        as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set(
    mut buf: *mut ibuf,
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
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_n8(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint8_t = 0;
    if value > UINT8_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = value as uint8_t;
    return ibuf_set(
        buf,
        pos,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint8_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_n16(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint16_t = 0;
    if value > UINT16_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = __bswap_16(value as __uint16_t) as uint16_t;
    return ibuf_set(
        buf,
        pos,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint16_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_n32(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint32_t = 0;
    if value > UINT32_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = __bswap_32(value as __uint32_t) as uint32_t;
    return ibuf_set(
        buf,
        pos,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint32_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_n64(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    value = htonll(value);
    return ibuf_set(
        buf,
        pos,
        &raw mut value as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_h16(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    let mut v: uint16_t = 0;
    if value > UINT16_MAX as uint64_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    v = value as uint16_t;
    return ibuf_set(
        buf,
        pos,
        &raw mut v as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint16_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_h32(
    mut buf: *mut ibuf,
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
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_h64(
    mut buf: *mut ibuf,
    mut pos: size_t,
    mut value: uint64_t,
) -> ::core::ffi::c_int {
    return ibuf_set(
        buf,
        pos,
        &raw mut value as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_set_maxsize(
    mut buf: *mut ibuf,
    mut max: size_t,
) -> ::core::ffi::c_int {
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    if max > (*buf).max {
        *__errno_location() = ERANGE;
        return -(1 as ::core::ffi::c_int);
    }
    (*buf).max = max;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_data(mut buf: *const ibuf) -> *mut ::core::ffi::c_void {
    return (*buf).buf.offset((*buf).rpos as isize) as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_size(mut buf: *const ibuf) -> size_t {
    return (*buf).wpos.wrapping_sub((*buf).rpos);
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_left(mut buf: *const ibuf) -> size_t {
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
        return 0 as size_t;
    }
    return (*buf).max.wrapping_sub((*buf).wpos);
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_truncate(mut buf: *mut ibuf, mut len: size_t) -> ::core::ffi::c_int {
    if ibuf_size(buf) >= len {
        (*buf).wpos = (*buf).rpos.wrapping_add(len);
        return 0 as ::core::ffi::c_int;
    }
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
        *__errno_location() = ERANGE;
        return -(1 as ::core::ffi::c_int);
    }
    return ibuf_add_zero(buf, len.wrapping_sub(ibuf_size(buf)));
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_rewind(mut buf: *mut ibuf) {
    (*buf).rpos = 0 as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_close(mut msgbuf: *mut msgbuf, mut buf: *mut ibuf) {
    ibufq_push(&raw mut (*msgbuf).bufs, buf);
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_from_buffer(
    mut buf: *mut ibuf,
    mut data: *mut ::core::ffi::c_void,
    mut len: size_t,
) {
    memset(
        buf as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<ibuf>() as size_t,
    );
    (*buf).buf = data as *mut ::core::ffi::c_uchar;
    (*buf).wpos = len;
    (*buf).size = (*buf).wpos;
    (*buf).fd = IBUF_FD_MARK_ON_STACK;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_from_ibuf(mut buf: *mut ibuf, mut from: *const ibuf) {
    ibuf_from_buffer(buf, ibuf_data(from), ibuf_size(from));
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get(
    mut buf: *mut ibuf,
    mut data: *mut ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if ibuf_size(buf) < len {
        *__errno_location() = EBADMSG;
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(data, ibuf_data(buf), len);
    (*buf).rpos = (*buf).rpos.wrapping_add(len);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_ibuf(
    mut buf: *mut ibuf,
    mut len: size_t,
    mut new: *mut ibuf,
) -> ::core::ffi::c_int {
    if ibuf_size(buf) < len {
        *__errno_location() = EBADMSG;
        return -(1 as ::core::ffi::c_int);
    }
    ibuf_from_buffer(new, ibuf_data(buf), len);
    (*buf).rpos = (*buf).rpos.wrapping_add(len);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_h16(
    mut buf: *mut ibuf,
    mut value: *mut uint16_t,
) -> ::core::ffi::c_int {
    return ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint16_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_h32(
    mut buf: *mut ibuf,
    mut value: *mut uint32_t,
) -> ::core::ffi::c_int {
    return ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint32_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_h64(
    mut buf: *mut ibuf,
    mut value: *mut uint64_t,
) -> ::core::ffi::c_int {
    return ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_n8(
    mut buf: *mut ibuf,
    mut value: *mut uint8_t,
) -> ::core::ffi::c_int {
    return ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint8_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_n16(
    mut buf: *mut ibuf,
    mut value: *mut uint16_t,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    rv = ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint16_t>() as size_t,
    );
    *value = __bswap_16(*value) as uint16_t;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_n32(
    mut buf: *mut ibuf,
    mut value: *mut uint32_t,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    rv = ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint32_t>() as size_t,
    );
    *value = __bswap_32(*value) as uint32_t;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_n64(
    mut buf: *mut ibuf,
    mut value: *mut uint64_t,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    rv = ibuf_get(
        buf,
        value as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
    *value = ntohll(*value);
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_string(
    mut buf: *mut ibuf,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if ibuf_size(buf) < len {
        *__errno_location() = EBADMSG;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    str = strndup(ibuf_data(buf) as *const ::core::ffi::c_char, len);
    if str.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*buf).rpos = (*buf).rpos.wrapping_add(len);
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_get_strbuf(
    mut buf: *mut ibuf,
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if len == 0 as size_t {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    if ibuf_get(buf, str as *mut ::core::ffi::c_void, len) == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    if *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int != '\0' as i32 {
        *str.offset(len.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
        *__errno_location() = EOVERFLOW;
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_skip(mut buf: *mut ibuf, mut len: size_t) -> ::core::ffi::c_int {
    if ibuf_size(buf) < len {
        *__errno_location() = EBADMSG;
        return -(1 as ::core::ffi::c_int);
    }
    (*buf).rpos = (*buf).rpos.wrapping_add(len);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_free(mut buf: *mut ibuf) {
    let mut save_errno: ::core::ffi::c_int = *__errno_location();
    if buf.is_null() {
        return;
    }
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
        abort();
    }
    if (*buf).fd >= 0 as ::core::ffi::c_int {
        close((*buf).fd);
    }
    freezero((*buf).buf as *mut ::core::ffi::c_void, (*buf).size);
    free(buf as *mut ::core::ffi::c_void);
    *__errno_location() = save_errno;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_fd_avail(mut buf: *mut ibuf) -> ::core::ffi::c_int {
    return ((*buf).fd >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_fd_get(mut buf: *mut ibuf) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = 0;
    if (*buf).fd < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    fd = (*buf).fd;
    (*buf).fd = -(1 as ::core::ffi::c_int);
    return fd;
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_fd_set(mut buf: *mut ibuf, mut fd: ::core::ffi::c_int) {
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
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
#[no_mangle]
pub unsafe extern "C" fn msgbuf_new() -> *mut msgbuf {
    let mut msgbuf: *mut msgbuf = ::core::ptr::null_mut::<msgbuf>();
    msgbuf = calloc(1 as size_t, ::core::mem::size_of::<msgbuf>() as size_t) as *mut msgbuf;
    if msgbuf.is_null() {
        return ::core::ptr::null_mut::<msgbuf>();
    }
    ibufq_init(&raw mut (*msgbuf).bufs);
    ibufq_init(&raw mut (*msgbuf).rbufs);
    return msgbuf;
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_new_reader(
    mut hdrsz: size_t,
    mut readhdr: Option<
        unsafe extern "C" fn(
            *mut ibuf,
            *mut ::core::ffi::c_void,
            *mut ::core::ffi::c_int,
        ) -> *mut ibuf,
    >,
    mut arg: *mut ::core::ffi::c_void,
) -> *mut msgbuf {
    let mut msgbuf: *mut msgbuf = ::core::ptr::null_mut::<msgbuf>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if hdrsz == 0 as size_t || hdrsz > (IBUF_READ_SIZE / 2 as ::core::ffi::c_int) as size_t {
        *__errno_location() = EINVAL;
        return ::core::ptr::null_mut::<msgbuf>();
    }
    buf = malloc(IBUF_READ_SIZE as size_t) as *mut ::core::ffi::c_char;
    if buf.is_null() {
        return ::core::ptr::null_mut::<msgbuf>();
    }
    msgbuf = msgbuf_new();
    if msgbuf.is_null() {
        free(buf as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<msgbuf>();
    }
    (*msgbuf).rbuf = buf;
    (*msgbuf).hdrsize = hdrsz;
    (*msgbuf).readhdr = readhdr;
    (*msgbuf).rarg = arg;
    return msgbuf;
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_free(mut msgbuf: *mut msgbuf) {
    if msgbuf.is_null() {
        return;
    }
    msgbuf_clear(msgbuf);
    free((*msgbuf).rbuf as *mut ::core::ffi::c_void);
    free(msgbuf as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_queuelen(mut msgbuf: *mut msgbuf) -> uint32_t {
    return ibufq_queuelen(&raw mut (*msgbuf).bufs);
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_clear(mut msgbuf: *mut msgbuf) {
    ibufq_flush(&raw mut (*msgbuf).bufs);
    ibufq_flush(&raw mut (*msgbuf).rbufs);
    (*msgbuf).roff = 0 as size_t;
    ibuf_free((*msgbuf).rpmsg);
    (*msgbuf).rpmsg = ::core::ptr::null_mut::<ibuf>();
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_get(mut msgbuf: *mut msgbuf) -> *mut ibuf {
    return ibufq_pop(&raw mut (*msgbuf).rbufs);
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_concat(mut msgbuf: *mut msgbuf, mut from: *mut ibufqueue) {
    ibufq_concat(&raw mut (*msgbuf).bufs, from);
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_write(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> ::core::ffi::c_int {
    let mut iov: [iovec; 1024] = [iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    }; 1024];
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut n: ssize_t = 0;
    memset(
        &raw mut iov as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[iovec; 1024]>() as size_t,
    );
    buf = (*msgbuf).bufs.bufs.tqh_first;
    while !buf.is_null() {
        if i >= IOV_MAX as ::core::ffi::c_uint {
            break;
        }
        iov[i as usize].iov_base = ibuf_data(buf);
        iov[i as usize].iov_len = ibuf_size(buf);
        i = i.wrapping_add(1);
        buf = (*buf).entry.tqe_next;
    }
    if i == 0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    's_68: {
        loop {
            n = writev(fd, &raw mut iov as *mut iovec, i as ::core::ffi::c_int);
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
#[no_mangle]
pub unsafe extern "C" fn msgbuf_write(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> ::core::ffi::c_int {
    let mut iov: [iovec; 1024] = [iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    }; 1024];
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    let mut buf0: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut n: ssize_t = 0;
    let mut msg: msghdr = msghdr {
        msg_name: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        msg_namelen: 0,
        msg_iov: ::core::ptr::null_mut::<iovec>(),
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
        ::core::mem::size_of::<[iovec; 1024]>() as size_t,
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
    buf = (*msgbuf).bufs.bufs.tqh_first;
    while !buf.is_null() {
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
        buf = (*buf).entry.tqe_next;
    }
    if i == 0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    msg.msg_iov = &raw mut iov as *mut iovec;
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
unsafe extern "C" fn ibuf_read_process(
    mut msgbuf: *mut msgbuf,
    mut fd: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut rbuf: ibuf = ibuf {
        entry: ibuf_entry {
            tqe_next: ::core::ptr::null_mut::<ibuf>(),
            tqe_prev: ::core::ptr::null_mut::<*mut ibuf>(),
        },
        buf: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        size: 0,
        max: 0,
        wpos: 0,
        rpos: 0,
        fd: 0,
    };
    let mut msg: ibuf = ibuf {
        entry: ibuf_entry {
            tqe_next: ::core::ptr::null_mut::<ibuf>(),
            tqe_prev: ::core::ptr::null_mut::<*mut ibuf>(),
        },
        buf: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        size: 0,
        max: 0,
        wpos: 0,
        rpos: 0,
        fd: 0,
    };
    let mut sz: ssize_t = 0;
    ibuf_from_buffer(
        &raw mut rbuf,
        (*msgbuf).rbuf as *mut ::core::ffi::c_void,
        (*msgbuf).roff,
    );
    loop {
        if (*msgbuf).rpmsg.is_null() {
            if ibuf_size(&raw mut rbuf) < (*msgbuf).hdrsize {
                current_block = 13586036798005543211;
                break;
            }
            ibuf_from_buffer(&raw mut msg, ibuf_data(&raw mut rbuf), (*msgbuf).hdrsize);
            (*msgbuf).rpmsg = (*msgbuf).readhdr.expect("non-null function pointer")(
                &raw mut msg,
                (*msgbuf).rarg,
                &raw mut fd,
            );
            if (*msgbuf).rpmsg.is_null() {
                current_block = 12397102733025307875;
                break;
            }
        }
        if ibuf_left((*msgbuf).rpmsg) <= ibuf_size(&raw mut rbuf) {
            sz = ibuf_left((*msgbuf).rpmsg) as ssize_t;
        } else {
            sz = ibuf_size(&raw mut rbuf) as ssize_t;
        }
        if ibuf_get_ibuf(&raw mut rbuf, sz as size_t, &raw mut msg) == -(1 as ::core::ffi::c_int)
            || ibuf_add_ibuf((*msgbuf).rpmsg, &raw mut msg) == -(1 as ::core::ffi::c_int)
        {
            current_block = 12397102733025307875;
            break;
        }
        if ibuf_left((*msgbuf).rpmsg) == 0 as size_t {
            ibufq_push(&raw mut (*msgbuf).rbufs, (*msgbuf).rpmsg);
            (*msgbuf).rpmsg = ::core::ptr::null_mut::<ibuf>();
        }
        if !(ibuf_size(&raw mut rbuf) > 0 as size_t) {
            current_block = 13586036798005543211;
            break;
        }
    }
    match current_block {
        12397102733025307875 => {
            if fd != -(1 as ::core::ffi::c_int) {
                close(fd);
            }
            return -(1 as ::core::ffi::c_int);
        }
        _ => {
            if ibuf_size(&raw mut rbuf) > 0 as size_t {
                memmove(
                    (*msgbuf).rbuf as *mut ::core::ffi::c_void,
                    ibuf_data(&raw mut rbuf),
                    ibuf_size(&raw mut rbuf),
                );
            }
            (*msgbuf).roff = ibuf_size(&raw mut rbuf);
            if fd != -(1 as ::core::ffi::c_int) {
                close(fd);
            }
            return 1 as ::core::ffi::c_int;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ibuf_read(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> ::core::ffi::c_int {
    let mut iov: iovec = iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    };
    let mut n: ssize_t = 0;
    if (*msgbuf).rbuf.is_null() {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
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
                    return 1 as ::core::ffi::c_int;
                }
                return -(1 as ::core::ffi::c_int);
            } else {
                break 's_45;
            }
        }
    }
    if n == 0 as ssize_t {
        return 0 as ::core::ffi::c_int;
    }
    (*msgbuf).roff = (*msgbuf).roff.wrapping_add(n as size_t);
    return ibuf_read_process(msgbuf, -(1 as ::core::ffi::c_int));
}
#[no_mangle]
pub unsafe extern "C" fn msgbuf_read(
    mut fd: ::core::ffi::c_int,
    mut msgbuf: *mut msgbuf,
) -> ::core::ffi::c_int {
    let mut msg: msghdr = msghdr {
        msg_name: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        msg_namelen: 0,
        msg_iov: ::core::ptr::null_mut::<iovec>(),
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
    let mut iov: iovec = iovec {
        iov_base: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        iov_len: 0,
    };
    let mut n: ssize_t = 0;
    let mut fdpass: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if (*msgbuf).rbuf.is_null() {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
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
                return 1 as ::core::ffi::c_int;
            }
            return -(1 as ::core::ffi::c_int);
        } else {
            if n == 0 as ssize_t {
                return 0 as ::core::ffi::c_int;
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
unsafe extern "C" fn msgbuf_drain(mut msgbuf: *mut msgbuf, mut n: size_t) {
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    loop {
        buf = (*msgbuf).bufs.bufs.tqh_first;
        if buf.is_null() {
            break;
        }
        if n >= ibuf_size(buf) {
            n = n.wrapping_sub(ibuf_size(buf));
            if !(*buf).entry.tqe_next.is_null() {
                (*(*buf).entry.tqe_next).entry.tqe_prev = (*buf).entry.tqe_prev;
            } else {
                (*msgbuf).bufs.bufs.tqh_last = (*buf).entry.tqe_prev;
            }
            *(*buf).entry.tqe_prev = (*buf).entry.tqe_next;
            (*msgbuf).bufs.queued = (*msgbuf).bufs.queued.wrapping_sub(1);
            ibuf_free(buf);
        } else {
            (*buf).rpos = (*buf).rpos.wrapping_add(n);
            return;
        }
    }
}
unsafe extern "C" fn ibufq_init(mut bufq: *mut ibufqueue) {
    (*bufq).bufs.tqh_first = ::core::ptr::null_mut::<ibuf>();
    (*bufq).bufs.tqh_last = &raw mut (*bufq).bufs.tqh_first;
    (*bufq).queued = 0 as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_new() -> *mut ibufqueue {
    let mut bufq: *mut ibufqueue = ::core::ptr::null_mut::<ibufqueue>();
    bufq = calloc(1 as size_t, ::core::mem::size_of::<ibufqueue>() as size_t) as *mut ibufqueue;
    if bufq.is_null() {
        return ::core::ptr::null_mut::<ibufqueue>();
    }
    ibufq_init(bufq);
    return bufq;
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_free(mut bufq: *mut ibufqueue) {
    if bufq.is_null() {
        return;
    }
    ibufq_flush(bufq);
    free(bufq as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_pop(mut bufq: *mut ibufqueue) -> *mut ibuf {
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    buf = (*bufq).bufs.tqh_first;
    if buf.is_null() {
        return ::core::ptr::null_mut::<ibuf>();
    }
    if !(*buf).entry.tqe_next.is_null() {
        (*(*buf).entry.tqe_next).entry.tqe_prev = (*buf).entry.tqe_prev;
    } else {
        (*bufq).bufs.tqh_last = (*buf).entry.tqe_prev;
    }
    *(*buf).entry.tqe_prev = (*buf).entry.tqe_next;
    (*bufq).queued = (*bufq).queued.wrapping_sub(1);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_push(mut bufq: *mut ibufqueue, mut buf: *mut ibuf) {
    if (*buf).fd == IBUF_FD_MARK_ON_STACK {
        abort();
    }
    (*buf).entry.tqe_next = ::core::ptr::null_mut::<ibuf>();
    (*buf).entry.tqe_prev = (*bufq).bufs.tqh_last;
    *(*bufq).bufs.tqh_last = buf;
    (*bufq).bufs.tqh_last = &raw mut (*buf).entry.tqe_next;
    (*bufq).queued = (*bufq).queued.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_queuelen(mut bufq: *mut ibufqueue) -> uint32_t {
    return (*bufq).queued;
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_concat(mut to: *mut ibufqueue, mut from: *mut ibufqueue) {
    (*to).queued = (*to).queued.wrapping_add((*from).queued);
    if !(*from).bufs.tqh_first.is_null() {
        *(*to).bufs.tqh_last = (*from).bufs.tqh_first;
        (*(*from).bufs.tqh_first).entry.tqe_prev = (*to).bufs.tqh_last;
        (*to).bufs.tqh_last = (*from).bufs.tqh_last;
        (*from).bufs.tqh_first = ::core::ptr::null_mut::<ibuf>();
        (*from).bufs.tqh_last = &raw mut (*from).bufs.tqh_first;
    }
    (*from).queued = 0 as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn ibufq_flush(mut bufq: *mut ibufqueue) {
    let mut buf: *mut ibuf = ::core::ptr::null_mut::<ibuf>();
    loop {
        buf = (*bufq).bufs.tqh_first;
        if buf.is_null() {
            break;
        }
        if !(*buf).entry.tqe_next.is_null() {
            (*(*buf).entry.tqe_next).entry.tqe_prev = (*buf).entry.tqe_prev;
        } else {
            (*bufq).bufs.tqh_last = (*buf).entry.tqe_prev;
        }
        *(*buf).entry.tqe_prev = (*buf).entry.tqe_next;
        ibuf_free(buf);
    }
    (*bufq).queued = 0 as uint32_t;
}
