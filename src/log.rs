use crate::src::compat::stdio::CFile;
use crate::src::compat::vis::strvis;
use crate::src::ffi::libc::{
    __errno_location, exit, fflush, fopen, fprintf, getpid, gettimeofday, setvbuf, snprintf,
    strerror,
};
use crate::src::shared::abi::*;
use crate::src::shared::stdio::FILE;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::xmalloc::try_vasprintf_cstring;
use std::ffi::{CStr, CString};

pub const _IOLBF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

static mut log_file: Option<CFile> = None;
static mut log_level: ::core::ffi::c_int = 0;

unsafe fn log_file_ptr() -> *mut FILE {
    (*(&raw const log_file))
        .as_ref()
        .map_or(::core::ptr::null_mut::<FILE>(), CFile::as_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn log_add_level() {
    log_level += 1;
}
#[no_mangle]
pub unsafe extern "C" fn log_get_level() -> ::core::ffi::c_int {
    return log_level;
}
#[no_mangle]
pub unsafe extern "C" fn log_open(mut name: *const ::core::ffi::c_char) {
    if log_level == 0 as ::core::ffi::c_int {
        return;
    }
    log_close();
    let pid = (getpid() as ::core::ffi::c_long).to_string();
    let mut path = b"tmux-".to_vec();
    path.extend_from_slice(CStr::from_ptr(name).to_bytes());
    path.push(b'-');
    path.extend_from_slice(pid.as_bytes());
    path.extend_from_slice(b".log");
    let path = CString::new(path).expect("log filename components contain no interior NUL");
    let file = fopen(
        path.as_ptr(),
        b"a\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    let Some(file) = CFile::from_raw(file) else {
        return;
    };
    log_file = Some(file);
    setvbuf(
        log_file_ptr(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        _IOLBF,
        0 as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn log_toggle(mut name: *const ::core::ffi::c_char) {
    if log_level == 0 as ::core::ffi::c_int {
        log_level = 1 as ::core::ffi::c_int;
        log_open(name);
        log_debug(b"log opened\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        log_debug(b"log closed\0" as *const u8 as *const ::core::ffi::c_char);
        log_level = 0 as ::core::ffi::c_int;
        log_close();
    };
}
#[no_mangle]
pub unsafe extern "C" fn log_close() {
    log_file = None;
}
unsafe extern "C" fn log_vwrite(
    mut msg: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let file = log_file_ptr();
    if file.is_null() {
        return;
    }
    let Some(s) = try_vasprintf_cstring(msg, ap) else {
        return;
    };
    // strvis writes at most four bytes per input byte plus the terminator.
    let Some(capacity) = s
        .as_bytes()
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(4))
    else {
        return;
    };
    let mut out = Vec::<u8>::new();
    if out.try_reserve_exact(capacity).is_err() {
        return;
    }
    out.resize(capacity, 0);
    strvis(
        out.as_mut_ptr().cast(),
        s.as_ptr(),
        VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
    );
    drop(s);
    gettimeofday(&raw mut tv, NULL);
    if fprintf(
        file,
        b"%lld.%06d %s%s\n\0" as *const u8 as *const ::core::ffi::c_char,
        tv.tv_sec as ::core::ffi::c_longlong,
        tv.tv_usec as ::core::ffi::c_int,
        prefix,
        out.as_ptr().cast::<::core::ffi::c_char>(),
    ) != -(1 as ::core::ffi::c_int)
    {
        fflush(file);
    }
}
#[no_mangle]
pub unsafe extern "C" fn log_debug(mut msg: *const ::core::ffi::c_char, mut args: ...) {
    let mut ap: ::core::ffi::VaList;
    if log_file_ptr().is_null() {
        return;
    }
    ap = args.clone();
    log_vwrite(msg, ap, b"\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn fatal(mut msg: *const ::core::ffi::c_char, mut args: ...) -> ! {
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut ap: ::core::ffi::VaList;
    if snprintf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        b"fatal: %s: \0" as *const u8 as *const ::core::ffi::c_char,
        strerror(*__errno_location()),
    ) < 0 as ::core::ffi::c_int
    {
        exit(1 as ::core::ffi::c_int);
    }
    ap = args.clone();
    log_vwrite(msg, ap, &raw mut tmp as *mut ::core::ffi::c_char);
    exit(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn fatalx(mut msg: *const ::core::ffi::c_char, mut args: ...) -> ! {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    log_vwrite(
        msg,
        ap,
        b"fatal: \0" as *const u8 as *const ::core::ffi::c_char,
    );
    exit(1 as ::core::ffi::c_int);
}
