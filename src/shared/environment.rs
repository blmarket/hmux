//! Authoritative environment declarations.

use std::collections::BTreeMap;
use std::ffi::CString;

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

/// Owns the ABI-visible entry and its stable name and optional value buffers.
pub(crate) struct EnvironEntryOwner {
    pub(crate) entry: environ_entry,
    pub(crate) name: CString,
    pub(crate) value: Option<CString>,
}

/// Rust-owned index and entry storage. Boxed entries keep exported pointers stable.
#[derive(Default)]
pub struct environ_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, Box<EnvironEntryOwner>>,
}
