use crate::compat::recallocarray;
use crate::ffi::{__errno_location, calloc, malloc, reallocarray, strerror};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_into};
use crate::log::fatalx;
pub use crate::types::*;
use ::core::ffi::{c_char, c_int, c_void};
use ::std::ffi::CString;

pub fn xmalloc(size: size_t) -> *mut u8 {
    unsafe {
        if size == 0 {
            fatalx(c"xmalloc: zero size".as_ptr(), fmt_args![]);
        }
        let ptr = malloc(size);
        if ptr.is_null() {
            fatalx(
                c"xmalloc: allocating %zu bytes: %s".as_ptr(),
                fmt_args![size, strerror(*__errno_location())],
            );
        }
        ptr as *mut u8
    }
}

pub fn xcalloc(nmemb: size_t, size: size_t) -> *mut u8 {
    unsafe {
        if size == 0 || nmemb == 0 {
            fatalx(c"xcalloc: zero size".as_ptr(), fmt_args![]);
        }
        let ptr = calloc(nmemb, size);
        if ptr.is_null() {
            fatalx(
                c"xcalloc: allocating %zu * %zu bytes: %s".as_ptr(),
                fmt_args![nmemb, size, strerror(*__errno_location())],
            );
        }
        ptr as *mut u8
    }
}

pub unsafe fn xrealloc(ptr: *mut u8, size: size_t) -> *mut u8 {
    unsafe { xreallocarray(ptr, 1, size) }
}

pub unsafe fn xreallocarray(ptr: *mut u8, nmemb: size_t, size: size_t) -> *mut u8 {
    unsafe {
        if nmemb == 0 || size == 0 {
            fatalx(c"xreallocarray: zero size".as_ptr(), fmt_args![]);
        }
        let new_ptr = reallocarray(ptr as *mut c_void, nmemb, size);
        if new_ptr.is_null() {
            fatalx(
                c"xreallocarray: allocating %zu * %zu bytes: %s".as_ptr(),
                fmt_args![nmemb, size, strerror(*__errno_location())],
            );
        }
        new_ptr as *mut u8
    }
}

pub unsafe fn xrecallocarray(
    ptr: *mut u8,
    oldnmemb: size_t,
    nmemb: size_t,
    size: size_t,
) -> *mut u8 {
    unsafe {
        if nmemb == 0 || size == 0 {
            fatalx(c"xrecallocarray: zero size".as_ptr(), fmt_args![]);
        }
        let new_ptr = recallocarray(ptr, oldnmemb, nmemb, size);
        if new_ptr.is_null() {
            fatalx(
                c"xrecallocarray: allocating %zu * %zu bytes: %s".as_ptr(),
                fmt_args![nmemb, size, strerror(*__errno_location())],
            );
        }
        new_ptr
    }
}

pub unsafe fn xasprintf(fmt: *const c_char, args: &[FmtArg]) -> CString {
    unsafe { format_alloc(fmt, args) }
}

pub unsafe fn xsnprintf(
    str: *mut c_char,
    len: size_t,
    fmt: *const c_char,
    args: &[FmtArg],
) -> c_int {
    unsafe {
        if len > c_int::MAX as size_t {
            fatalx(c"xsnprintf: len > INT_MAX".as_ptr(), fmt_args![]);
        }
        let i = format_into(str, len, fmt, args);
        if i < 0 || i >= len as c_int {
            fatalx(c"xsnprintf: overflow".as_ptr(), fmt_args![]);
        }
        i
    }
}

#[cfg(test)]
#[path = "tests/test_xmalloc.rs"]
mod tests;
