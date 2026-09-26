//! Authoritative environment declarations.

use std::collections::BTreeMap;

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// Ordered environment storage. Boxes keep entry addresses stable for UI IDs.
#[derive(Default, Clone)]
#[repr(C)]
pub struct environ {
    pub(crate) entries: BTreeMap<Vec<u8>, Box<environ_entry>>,
}

#[derive(Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: std::ffi::CString,
    pub value: Option<std::ffi::CString>,
    pub flags: ::core::ffi::c_int,
}
