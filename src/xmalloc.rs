use crate::src::ffi::libc::{
    calloc, free, vasprintf, vsnprintf,
};
use crate::src::log::{fatal, fatalx};
use crate::src::shared::abi::*;
use crate::src::shared::limits::{__INT_MAX__, INT_MAX, SIZE_MAX};
use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
use std::ffi::{CStr, CString};

#[no_mangle]
pub unsafe extern "C" fn xcalloc(mut nmemb: size_t, mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut ptr: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if size == 0 as size_t || nmemb == 0 as size_t {
        fatalx(b"xcalloc: zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (SIZE_MAX as size_t).wrapping_div(nmemb) < size {
        fatalx(b"xcalloc: nmemb * size > SIZE_MAX\0" as *const u8 as *const ::core::ffi::c_char);
    }
    ptr = calloc(nmemb, size);
    if ptr.is_null() {
        fatal(
            b"xcalloc: allocating %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
            size.wrapping_mul(nmemb),
        );
    }
    return ptr;
}
#[no_mangle]
pub unsafe extern "C" fn xasprintf(
    mut ret: *mut *mut ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    let mut ap: ::core::ffi::VaList;
    let mut i: ::core::ffi::c_int = 0;
    ap = args.clone();
    i = xvasprintf(ret, fmt, ap);
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn xvasprintf(
    mut ret: *mut *mut ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = vasprintf(ret, fmt, ap);
    if i == -(1 as ::core::ffi::c_int) || (*ret).is_null() {
        fatal(b"xvasprintf\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return i;
}

/// Format into a Rust-owned string while keeping the existing C allocator
/// contract at the variadic boundary.
///
/// `vasprintf` allocates with libc's allocator, so its result cannot be
/// adopted with `CString::from_raw`. Copying the C string first lets the
/// caller use ordinary `CString` ownership while this adapter releases the
/// matching C allocation. The C string is intentionally copied through
/// `CStr`, preserving arbitrary non-UTF-8 bytes and the same first-NUL view
/// exposed to the existing `%s` consumers.
pub(crate) unsafe fn xvasprintf_cstring(
    fmt: *const ::core::ffi::c_char,
    ap: ::core::ffi::VaList,
) -> CString {
    let mut raw = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xvasprintf(&raw mut raw, fmt, ap);
    let value = CStr::from_ptr(raw).to_owned();
    free(raw as *mut ::core::ffi::c_void);
    value
}

/// Fallible variant for callers whose existing `vasprintf` failure path
/// returns without aborting. The returned C-string view ends at the first NUL.
pub(crate) unsafe fn try_vasprintf_cstring(
    fmt: *const ::core::ffi::c_char,
    ap: ::core::ffi::VaList,
) -> Option<CString> {
    let mut raw = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if vasprintf(&raw mut raw, fmt, ap) == -1 || raw.is_null() {
        if !raw.is_null() {
            free(raw.cast());
        }
        return None;
    }
    let value = CStr::from_ptr(raw).to_owned();
    free(raw.cast());
    Some(value)
}

#[no_mangle]
pub unsafe extern "C" fn xsnprintf(
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    let mut ap: ::core::ffi::VaList;
    let mut i: ::core::ffi::c_int = 0;
    ap = args.clone();
    i = xvsnprintf(str, len, fmt, ap);
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn xvsnprintf(
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    if len > INT_MAX as size_t {
        fatalx(b"xsnprintf: len > INT_MAX\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = vsnprintf(str, len, fmt, ap);
    if i < 0 as ::core::ffi::c_int || i >= len as ::core::ffi::c_int {
        fatalx(b"xsnprintf: overflow\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return i;
}

#[cfg(test)]
mod tests {
    use super::xvasprintf_cstring;
    use std::ffi::CString;

    unsafe extern "C" fn format_to_raw(
        fmt: *const ::core::ffi::c_char,
        mut args: ...
    ) -> *mut ::core::ffi::c_char {
        CString::into_raw(xvasprintf_cstring(fmt, args.clone()))
    }

    #[test]
    fn c_allocator_bridge_preserves_bytes_and_c_termination() {
        unsafe {
            let value = CString::from_raw(format_to_raw(
                c"%s:%c%s".as_ptr(),
                CString::new(vec![b'\xff', b'x']).unwrap().as_ptr(),
                0 as ::core::ffi::c_int,
                c"tail".as_ptr(),
            ));
            assert_eq!(value.as_bytes(), &[0xff, b'x', b':']);
        }
    }
}
