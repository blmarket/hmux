//! Authoritative account declarations from the translated Linux C ABI.
use super::abi::{__gid_t, __uid_t};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct passwd {
    pub pw_name: *mut ::core::ffi::c_char,
    pub pw_passwd: *mut ::core::ffi::c_char,
    pub pw_uid: __uid_t,
    pub pw_gid: __gid_t,
    pub pw_gecos: *mut ::core::ffi::c_char,
    pub pw_dir: *mut ::core::ffi::c_char,
    pub pw_shell: *mut ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct group {
    pub gr_name: *mut ::core::ffi::c_char,
    pub gr_passwd: *mut ::core::ffi::c_char,
    pub gr_gid: __gid_t,
    pub gr_mem: *mut *mut ::core::ffi::c_char,
}
