//! Authoritative environment declarations.

use std::collections::BTreeMap;
use std::ffi::CString;

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ {
    pub entries: *mut environ_storage,
}

#[repr(C)]
pub struct environ_entry {
    pub name: std::ffi::CString,
    pub value: Option<std::ffi::CString>,
    pub flags: ::core::ffi::c_int,
    pub owner: *mut environ,
}

impl environ_entry {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            value: Default::default(),
            flags: unsafe { ::core::mem::zeroed() },
            owner: unsafe { ::core::mem::zeroed() },
        }
    }
}

/// Rust-owned index and entry storage. Boxed entries keep exported pointers stable.
#[derive(Default)]
pub struct environ_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, Box<environ_entry>>,
}
