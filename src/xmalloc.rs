use crate::src::compat::recallocarray::recallocarray;
use crate::src::ffi::libc::{
    calloc, malloc, memcpy, reallocarray, strdup, strndup, vasprintf, vsnprintf,
};
use crate::src::log::{fatal, fatalx};
use crate::src::shared::abi::*;
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX, SIZE_MAX};
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};

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
