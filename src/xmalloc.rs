use crate::src::compat::recallocarray::recallocarray;
use crate::src::ffi::libc::{
    calloc, free, malloc, memcpy, reallocarray, strdup, strndup, vasprintf, vsnprintf,
};
use crate::src::log::{fatal, fatalx};
use crate::src::shared::abi::*;
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX, SIZE_MAX};
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
use std::ffi::{CStr, CString};

#[no_mangle]
pub unsafe extern "C" fn xmalloc(mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut ptr: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if size == 0 as size_t {
        fatalx(b"xmalloc: zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    ptr = malloc(size);
    if ptr.is_null() {
        fatal(
            b"xmalloc: allocating %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
            size,
        );
    }
    return ptr;
}
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
pub unsafe extern "C" fn xrealloc(
    mut ptr: *mut ::core::ffi::c_void,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    return xreallocarray(ptr, 1 as size_t, size);
}
#[no_mangle]
pub unsafe extern "C" fn xreallocarray(
    mut ptr: *mut ::core::ffi::c_void,
    mut nmemb: size_t,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    let mut new_ptr: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if nmemb == 0 as size_t || size == 0 as size_t {
        fatalx(b"xreallocarray: zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    new_ptr = reallocarray(ptr, nmemb, size);
    if new_ptr.is_null() {
        fatal(
            b"xreallocarray: allocating %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
            size.wrapping_mul(nmemb),
        );
    }
    return new_ptr;
}
#[no_mangle]
pub unsafe extern "C" fn xrecallocarray(
    mut ptr: *mut ::core::ffi::c_void,
    mut oldnmemb: size_t,
    mut nmemb: size_t,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    let mut new_ptr: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if nmemb == 0 as size_t || size == 0 as size_t {
        fatalx(b"xrecallocarray: zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    new_ptr = recallocarray(ptr, oldnmemb, nmemb, size);
    if new_ptr.is_null() {
        fatal(
            b"xrecallocarray: allocating %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
            size.wrapping_mul(nmemb),
        );
    }
    return new_ptr;
}
#[no_mangle]
pub unsafe extern "C" fn xstrdup(mut str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    cp = strdup(str);
    if cp.is_null() {
        fatal(b"xstrdup\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return cp;
}
#[no_mangle]
pub unsafe extern "C" fn xstrndup(
    mut str: *const ::core::ffi::c_char,
    mut maxlen: size_t,
) -> *mut ::core::ffi::c_char {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    cp = strndup(str, maxlen);
    if cp.is_null() {
        fatal(b"xstrndup\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return cp;
}
#[no_mangle]
pub unsafe extern "C" fn xmemdup(
    mut ptr: *const ::core::ffi::c_void,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    cp = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    if len != 0 as size_t {
        memcpy(cp as *mut ::core::ffi::c_void, ptr, len);
    }
    *cp.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    return cp;
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

/// Format into Rust-owned bytes using the length returned by `vasprintf`.
///
/// Unlike `xvasprintf_cstring`, this keeps bytes after an embedded NUL. The
/// allocation belongs to libc, so copy it before calling the matching `free`.
pub(crate) unsafe fn xvasprintf_bytes(
    fmt: *const ::core::ffi::c_char,
    ap: ::core::ffi::VaList,
) -> Vec<u8> {
    let mut raw = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let len = xvasprintf(&raw mut raw, fmt, ap);
    let value = ::std::slice::from_raw_parts(raw.cast::<u8>(), len as usize).to_vec();
    free(raw.cast::<::core::ffi::c_void>());
    value
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
    use super::{xvasprintf_bytes, xvasprintf_cstring};
    use std::ffi::CString;

    unsafe extern "C" fn format_to_raw(
        fmt: *const ::core::ffi::c_char,
        mut args: ...
    ) -> *mut ::core::ffi::c_char {
        CString::into_raw(xvasprintf_cstring(fmt, args.clone()))
    }

    unsafe extern "C" fn format_to_bytes(
        out: *mut Vec<u8>,
        fmt: *const ::core::ffi::c_char,
        mut args: ...
    ) {
        *out = xvasprintf_bytes(fmt, args.clone());
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

    #[test]
    fn byte_bridge_preserves_bytes_after_embedded_nul() {
        unsafe {
            let mut value = Vec::new();
            format_to_bytes(
                &raw mut value,
                c"%s:%c%s".as_ptr(),
                CString::new(vec![b'\xff', b'x']).unwrap().as_ptr(),
                0 as ::core::ffi::c_int,
                c"tail".as_ptr(),
            );
            assert_eq!(value, b"\xffx:\0tail");
        }
    }
}
