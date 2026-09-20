use crate::src::compat::vis::stravis;
use crate::src::ffi::libc::{
    __errno_location, exit, fclose, fflush, fopen, fprintf, free, getpid, gettimeofday, setvbuf,
    snprintf, strerror, vasprintf,
};
use crate::src::ffi::libevent::event_set_log_callback;
pub use crate::src::ffi::libevent::event_log_cb;
use crate::src::xmalloc::xasprintf;
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{__off64_t, __off_t};
use crate::src::shared::abi::*;

pub const _IOLBF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

static mut log_file: *mut FILE = ::core::ptr::null::<FILE>() as *mut FILE;
static mut log_level: ::core::ffi::c_int = 0;
unsafe extern "C" fn log_event_cb(
    mut severity: ::core::ffi::c_int,
    mut msg: *const ::core::ffi::c_char,
) {
    log_debug(b"%s\0" as *const u8 as *const ::core::ffi::c_char, msg);
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
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if log_level == 0 as ::core::ffi::c_int {
        return;
    }
    log_close();
    xasprintf(
        &raw mut path,
        b"tmux-%s-%ld.log\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        getpid() as ::core::ffi::c_long,
    );
    log_file = fopen(path, b"a\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    free(path as *mut ::core::ffi::c_void);
    if log_file.is_null() {
        return;
    }
    setvbuf(
        log_file,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        _IOLBF,
        0 as size_t,
    );
    event_set_log_callback(Some(
        log_event_cb as unsafe extern "C" fn(::core::ffi::c_int, *const ::core::ffi::c_char) -> (),
    ));
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
    if !log_file.is_null() {
        fclose(log_file);
    }
    log_file = ::core::ptr::null_mut::<FILE>();
    event_set_log_callback(None);
}
unsafe extern "C" fn log_vwrite(
    mut msg: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if log_file.is_null() {
        return;
    }
    if vasprintf(&raw mut s, msg, ap) == -(1 as ::core::ffi::c_int) {
        return;
    }
    if stravis(&raw mut out, s, VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL)
        == -(1 as ::core::ffi::c_int)
    {
        free(s as *mut ::core::ffi::c_void);
        return;
    }
    free(s as *mut ::core::ffi::c_void);
    gettimeofday(&raw mut tv, NULL);
    if fprintf(
        log_file,
        b"%lld.%06d %s%s\n\0" as *const u8 as *const ::core::ffi::c_char,
        tv.tv_sec as ::core::ffi::c_longlong,
        tv.tv_usec as ::core::ffi::c_int,
        prefix,
        out,
    ) != -(1 as ::core::ffi::c_int)
    {
        fflush(log_file);
    }
    free(out as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn log_debug(mut msg: *const ::core::ffi::c_char, mut args: ...) {
    let mut ap: ::core::ffi::VaList;
    if log_file.is_null() {
        return;
    }
    ap = args.clone();
    log_vwrite(
        msg,
        ap,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
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
    log_vwrite(
        msg,
        ap,
        &raw mut tmp as *mut ::core::ffi::c_char,
    );
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
