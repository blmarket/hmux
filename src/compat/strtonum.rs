extern "C" {
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn strtoll(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct errval {
    pub errstr: *const ::core::ffi::c_char,
    pub err: ::core::ffi::c_int,
}
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const LLONG_MAX: ::core::ffi::c_longlong = __LONG_LONG_MAX__;
pub const LLONG_MIN: ::core::ffi::c_longlong = -__LONG_LONG_MAX__ - 1 as ::core::ffi::c_longlong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INVALID: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const TOOSMALL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TOOLARGE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn strtonum(
    mut numstr: *const ::core::ffi::c_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut errstrp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut ll: ::core::ffi::c_longlong = 0 as ::core::ffi::c_longlong;
    let mut ep: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ev: [errval; 4] = [
        errval {
            errstr: ::core::ptr::null::<::core::ffi::c_char>(),
            err: 0 as ::core::ffi::c_int,
        },
        errval {
            errstr: b"invalid\0" as *const u8 as *const ::core::ffi::c_char,
            err: EINVAL,
        },
        errval {
            errstr: b"too small\0" as *const u8 as *const ::core::ffi::c_char,
            err: ERANGE,
        },
        errval {
            errstr: b"too large\0" as *const u8 as *const ::core::ffi::c_char,
            err: ERANGE,
        },
    ];
    ev[0 as ::core::ffi::c_int as usize].err = *__errno_location();
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
    if !errstrp.is_null() {
        *errstrp = ev[error as usize].errstr;
    }
    *__errno_location() = ev[error as usize].err;
    if error != 0 {
        ll = 0 as ::core::ffi::c_longlong;
    }
    return ll;
}
pub const __LONG_LONG_MAX__: ::core::ffi::c_longlong =
    9223372036854775807 as ::core::ffi::c_longlong;
