use crate::src::ffi::libc::{getpid, glob, globfree, snprintf};
use crate::src::log::fatal;
pub use crate::src::shared::posix_io::{dirent, glob_t, stat};
pub use crate::src::shared::abi::{__size_t};
use crate::src::shared::abi::*;

#[no_mangle]
pub unsafe extern "C" fn getdtablecount() -> ::core::ffi::c_int {
    let mut path: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut g: glob_t = glob_t {
        gl_pathc: 0,
        gl_pathv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        gl_offs: 0,
        gl_flags: 0,
        gl_closedir: None,
        gl_readdir: None,
        gl_opendir: None,
        gl_lstat: None,
        gl_stat: None,
    };
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
    if glob(
        &raw mut path as *mut ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        None,
        &raw mut g,
    ) == 0 as ::core::ffi::c_int
    {
        n = g.gl_pathc as ::core::ffi::c_int;
    }
    globfree(&raw mut g);
    return n;
}
