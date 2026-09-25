//! Authoritative paste model declarations.

use super::abi::{size_t, time_t, u_int};

pub struct paste_buffer {
    pub data: Option<Box<[u8]>>,
    pub size: size_t,
    pub name: std::ffi::CString,
    pub created: time_t,
    pub automatic: ::core::ffi::c_int,
    pub order: u_int,
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
        }
    }
}
