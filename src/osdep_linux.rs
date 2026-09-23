use std::ffi::CString;

use crate::src::ffi::libc::{fclose, fgetc, fopen, ioctl, readlink, setenv, tcgetpgrp, unsetenv};
use crate::src::reactor::event_init;
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__off64_t, __off_t, ssize_t};
use crate::src::shared::event::*;
pub use crate::src::shared::stdio::EOF;
pub use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::xmalloc::xrealloc;

pub const MAXPATHLEN: ::core::ffi::c_int = PATH_MAX;
pub const PATH_MAX: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;

pub const TIOCGSID: ::core::ffi::c_int = 0x5429 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn osdep_get_name(
    mut fd: ::core::ffi::c_int,
    mut tty: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut ch: ::core::ffi::c_int = 0;
    let mut pgrp: pid_t = 0;
    pgrp = tcgetpgrp(fd) as pid_t;
    if pgrp == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let path = CString::new(format!("/proc/{pgrp}/cmdline")).unwrap();
    f = fopen(
        path.as_ptr(),
        b"r\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if f.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    len = 0 as size_t;
    buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        ch = fgetc(f);
        if !(ch != EOF) {
            break;
        }
        if ch == '\0' as i32 {
            break;
        }
        buf = xrealloc(
            buf as *mut ::core::ffi::c_void,
            len.wrapping_add(2 as size_t),
        ) as *mut ::core::ffi::c_char;
        let fresh0 = len;
        len = len.wrapping_add(1);
        *buf.offset(fresh0 as isize) = ch as ::core::ffi::c_char;
    }
    if !buf.is_null() {
        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
    fclose(f);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn osdep_get_cwd(mut fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    static mut target: [::core::ffi::c_char; 4097] = [0; 4097];
    let mut pgrp: pid_t = 0;
    let mut sid: pid_t = 0;
    let mut n: ssize_t = 0;
    pgrp = tcgetpgrp(fd) as pid_t;
    if pgrp == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let path = CString::new(format!("/proc/{pgrp}/cwd")).unwrap();
    n = readlink(
        path.as_ptr(),
        &raw mut target as *mut ::core::ffi::c_char,
        MAXPATHLEN as size_t,
    );
    if n == -(1 as ::core::ffi::c_int) as ssize_t
        && ioctl(fd, TIOCGSID as ::core::ffi::c_ulong, &raw mut sid) != -(1 as ::core::ffi::c_int)
    {
        let path = CString::new(format!("/proc/{sid}/cwd")).unwrap();
        n = readlink(
            path.as_ptr(),
            &raw mut target as *mut ::core::ffi::c_char,
            MAXPATHLEN as size_t,
        );
    }
    if n > 0 as ssize_t {
        target[n as usize] = '\0' as i32 as ::core::ffi::c_char;
        return &raw mut target as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn osdep_event_init() -> *mut event_base {
    let mut base: *mut event_base = ::core::ptr::null_mut::<event_base>();
    setenv(
        b"EVENT_NOEPOLL\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    base = event_init();
    unsetenv(b"EVENT_NOEPOLL\0" as *const u8 as *const ::core::ffi::c_char);
    return base;
}
