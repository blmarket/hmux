//! Authoritative environment declarations.

use std::collections::BTreeMap;

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ {
    pub entries: *mut environ_storage,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    /// Owning environment, used by the entry-only successor API.
    pub owner: *mut environ,
}

/// Rust-owned index; entries and their strings retain their C allocation contract.
#[derive(Default)]
pub struct environ_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut environ_entry>,
}
