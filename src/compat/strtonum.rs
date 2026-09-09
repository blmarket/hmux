pub use crate::consts::{__LONG_LONG_MAX__, EINVAL, ERANGE};
use crate::ffi::{__errno_location, strtoll};

pub const LLONG_MAX: core::ffi::c_longlong = __LONG_LONG_MAX__;
pub const LLONG_MIN: core::ffi::c_longlong = -__LONG_LONG_MAX__ - 1 as core::ffi::c_longlong;

/// Why a string was not acceptable as a number, carrying the message and
/// `errno` value OpenBSD's `strtonum` reports for it.
#[derive(Clone, Copy)]
enum NumError {
    Invalid,
    TooSmall,
    TooLarge,
}

impl NumError {
    fn errstr(self) -> &'static core::ffi::CStr {
        match self {
            NumError::Invalid => c"invalid",
            NumError::TooSmall => c"too small",
            NumError::TooLarge => c"too large",
        }
    }

    fn errno(self) -> core::ffi::c_int {
        match self {
            NumError::Invalid => EINVAL,
            NumError::TooSmall | NumError::TooLarge => ERANGE,
        }
    }
}

/// Parse the whole of `numstr` as a base-10 number and require it to land in
/// `[minval, maxval]`. Clears `errno` around the underlying `strtoll` so its
/// `ERANGE` report can be told apart from a leftover value.
fn parse(
    numstr: &core::ffi::CStr,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
) -> Result<core::ffi::c_longlong, NumError> {
    if minval > maxval {
        return Err(NumError::Invalid);
    }
    let mut ep: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();
    let (ll, erange) = unsafe {
        *__errno_location() = 0;
        let ll = strtoll(numstr.as_ptr(), &raw mut ep, 10);
        (ll, *__errno_location() == ERANGE)
    };
    let terminator = core::ptr::from_ref(numstr.to_bytes_with_nul().last().unwrap()).cast();
    if core::ptr::eq(ep, numstr.as_ptr()) || !core::ptr::eq(ep, terminator) {
        return Err(NumError::Invalid);
    }
    if ll == LLONG_MIN && erange || ll < minval {
        return Err(NumError::TooSmall);
    }
    if ll == LLONG_MAX && erange || ll > maxval {
        return Err(NumError::TooLarge);
    }
    Ok(ll)
}

/// `numstr` as a base-10 number in `[minval, maxval]`, or the message
/// OpenBSD's `strtonum` reports for what is wrong with it. `errno` is left as
/// that same function leaves it: untouched when the number is good, and set to
/// `EINVAL` or `ERANGE` when it is not.
pub unsafe fn strtonum(
    numstr: &core::ffi::CStr,
    minval: core::ffi::c_longlong,
    maxval: core::ffi::c_longlong,
) -> Result<core::ffi::c_longlong, &'static core::ffi::CStr> {
    unsafe {
        let saved = *__errno_location();
        let result = parse(numstr, minval, maxval);
        *__errno_location() = match result {
            Ok(_) => saved,
            Err(e) => e.errno(),
        };
        result.map_err(NumError::errstr)
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_strtonum.rs"]
mod tests;
