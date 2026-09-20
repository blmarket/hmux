use crate::src::shared::abi::*;
extern "C" {
    pub type stat;
    pub type dirent;
    fn glob(
        __pattern: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
        __errfunc: Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                ::core::ffi::c_int,
            ) -> ::core::ffi::c_int,
        >,
        __pglob: *mut glob_t,
    ) -> ::core::ffi::c_int;
    fn globfree(__pglob: *mut glob_t);
    fn getpid() -> __pid_t;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fatal(_: *const ::core::ffi::c_char, ...);
}
pub type __size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct glob_t {
    pub gl_pathc: __size_t,
    pub gl_pathv: *mut *mut ::core::ffi::c_char,
    pub gl_offs: __size_t,
    pub gl_flags: ::core::ffi::c_int,
    pub gl_closedir: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub gl_readdir: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut dirent>,
    pub gl_opendir:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> *mut ::core::ffi::c_void>,
    pub gl_lstat:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char, *mut stat) -> ::core::ffi::c_int>,
    pub gl_stat:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char, *mut stat) -> ::core::ffi::c_int>,
}
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
