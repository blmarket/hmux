//! Authoritative posix io declarations.
use super::abi::{
    __blkcnt_t, __blksize_t, __dev_t, __gid_t, __ino_t, __mode_t, __nlink_t, __off_t, __size_t,
    __syscall_slong_t, __uid_t,
};
use super::time::timespec;
pub use crate::src::ffi::libc::dirent;
pub const WNOHANG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const WAIT_ANY: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);

pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;

pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;

pub const _PATH_BSHELL: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"/bin/sh\0") };

pub const _PATH_DEVNULL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"/dev/null\0") };

pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;

pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;

pub const O_NONBLOCK: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;

pub const FNM_CASEFOLD: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;

pub const __S_IREAD: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;

pub const __S_IWRITE: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;

pub const __S_IEXEC: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;

pub const S_IRWXU: ::core::ffi::c_int = __S_IREAD | __S_IWRITE | __S_IEXEC;

pub const X_OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct stat {
    pub st_dev: __dev_t,
    pub st_ino: __ino_t,
    pub st_nlink: __nlink_t,
    pub st_mode: __mode_t,
    pub st_uid: __uid_t,
    pub st_gid: __gid_t,
    pub __pad0: ::core::ffi::c_int,
    pub st_rdev: __dev_t,
    pub st_size: __off_t,
    pub st_blksize: __blksize_t,
    pub st_blocks: __blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [__syscall_slong_t; 3],
}

#[derive(Copy, Clone, Default)]
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
