use crate::src::ffi::libc::{__errno_location, strtoll};
use crate::src::shared::errno::{EINVAL, ERANGE};
use crate::src::shared::limits::__LONG_LONG_MAX__;
use std::ffi::CStr;

pub const LLONG_MAX: ::core::ffi::c_longlong = __LONG_LONG_MAX__;
pub const LLONG_MIN: ::core::ffi::c_longlong = -__LONG_LONG_MAX__ - 1 as ::core::ffi::c_longlong;
pub const INVALID: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const TOOSMALL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TOOLARGE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub unsafe fn strtonum(
    mut numstr: *const ::core::ffi::c_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut errstrp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut ll: ::core::ffi::c_longlong = 0 as ::core::ffi::c_longlong;
    let mut ep: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let saved_errno = *__errno_location();
    *__errno_location() = 0 as ::core::ffi::c_int;
    if minval > maxval {
        error = INVALID;
    } else {
        ll = strtoll(numstr, &raw mut ep, 10 as ::core::ffi::c_int);
        if numstr == ep as *const ::core::ffi::c_char || *ep as ::core::ffi::c_int != '\0' as i32 {
            error = INVALID;
        } else if ll == LLONG_MIN && *__errno_location() == ERANGE || ll < minval {
            error = TOOSMALL;
        } else if ll == LLONG_MAX && *__errno_location() == ERANGE || ll > maxval {
            error = TOOLARGE;
        }
    }
    let message = match error {
        INVALID => Some(c"invalid"),
        TOOSMALL => Some(c"too small"),
        TOOLARGE => Some(c"too large"),
        _ => None,
    };
    if !errstrp.is_null() {
        *errstrp = message.map_or(std::ptr::null(), CStr::as_ptr);
    }
    *__errno_location() = match error {
        INVALID => EINVAL,
        TOOSMALL | TOOLARGE => ERANGE,
        _ => saved_errno,
    };
    if error != 0 {
        ll = 0 as ::core::ffi::c_longlong;
    }
    return ll;
}
