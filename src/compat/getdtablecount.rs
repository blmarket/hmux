use crate::src::compat::glob::GlobResult;
use crate::src::ffi::libc::{getpid, snprintf};
use crate::src::log::fatal;
pub use crate::src::shared::abi::__size_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::posix_io::{dirent, glob_t, stat};
use std::ffi::CStr;

#[no_mangle]
pub unsafe extern "C" fn getdtablecount() -> ::core::ffi::c_int {
    let mut path: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if snprintf(
        &raw mut path as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t,
        b"/proc/%ld/fd/*\0" as *const u8 as *const ::core::ffi::c_char,
        getpid() as ::core::ffi::c_long,
    ) < 0 as ::core::ffi::c_int
    {
        fatal(b"snprintf overflow\0" as *const u8 as *const ::core::ffi::c_char);
    }
    let (matches, status) = GlobResult::run(CStr::from_ptr(path.as_ptr()));
    if status == 0 {
        n = matches.len() as ::core::ffi::c_int;
    }
    return n;
}
