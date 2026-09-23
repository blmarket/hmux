//! Authoritative paste model declarations.

use super::abi::{size_t, time_t, u_int};

#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned from `paste_add` or `paste_set` until `paste_free`. The name and
/// order indexes borrow its stable address; data and name retain libc owners.
pub struct paste_buffer {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub name: *mut ::core::ffi::c_char,
    pub created: time_t,
    pub automatic: ::core::ffi::c_int,
    pub order: u_int,
    pub name_entry: paste_buffer_name_entry,
    pub time_entry: paste_buffer_time_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct paste_buffer_time_entry {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct paste_buffer_name_entry {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
