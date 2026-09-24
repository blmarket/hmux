//! Authoritative paste model declarations.

use super::abi::{size_t, time_t, u_int};

#[repr(C)]
pub struct paste_buffer {
    pub data: Option<Box<[u8]>>,
    pub size: size_t,
    pub name: std::ffi::CString,
    pub created: time_t,
    pub automatic: ::core::ffi::c_int,
    pub order: u_int,
    pub name_entry: paste_buffer_name_entry,
    pub time_entry: paste_buffer_time_entry,
}

impl paste_buffer {
    pub fn empty() -> Self {
        Self {
            data: Default::default(),
            size: unsafe { ::core::mem::zeroed() },
            name: Default::default(),
            created: unsafe { ::core::mem::zeroed() },
            automatic: unsafe { ::core::mem::zeroed() },
            order: unsafe { ::core::mem::zeroed() },
            name_entry: unsafe { ::core::mem::zeroed() },
            time_entry: unsafe { ::core::mem::zeroed() },
        }
    }
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
