use std::ffi::CString;

use crate::src::compat::stdio::CFile;
use crate::src::ffi::libc::{fgetc, fopen, ioctl, readlink, setenv, tcgetpgrp, unsetenv};
use crate::src::reactor::event_init;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::event::*;
use crate::src::shared::stdio::EOF;
use crate::src::shared::stdio::FILE;

pub const MAXPATHLEN: ::core::ffi::c_int = PATH_MAX;
pub const PATH_MAX: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;

pub const TIOCGSID: ::core::ffi::c_int = 0x5429 as ::core::ffi::c_int;
pub(crate) unsafe fn osdep_get_name_cstring(
    fd: ::core::ffi::c_int,
    _tty: *mut ::core::ffi::c_char,
) -> Option<CString> {
    let pgrp = tcgetpgrp(fd) as pid_t;
    if pgrp == -(1 as ::core::ffi::c_int) {
        return None;
    }
    let path = CString::new(format!("/proc/{pgrp}/cmdline")).unwrap();
    let f = fopen(
        path.as_ptr(),
        b"r\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if f.is_null() {
        return None;
    }
    let stream = CFile::from_raw(f).expect("fopen returned a non-null stream");
    let mut buf = Vec::new();
    loop {
        let ch = fgetc(stream.as_ptr());
        if ch == EOF || ch == 0 {
            break;
        }
        buf.push(ch as u8);
    }
    drop(stream);
    if buf.is_empty() {
        None
    } else {
        Some(CString::new(buf).expect("cmdline stops at the first NUL"))
    }
}
pub unsafe fn osdep_get_cwd(mut fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
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
pub unsafe fn osdep_event_init() -> *mut event_base {
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
