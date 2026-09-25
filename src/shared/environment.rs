//! Authoritative environment declarations.

use std::collections::BTreeMap;

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

#[repr(C)]
pub struct environ {
    pub entries: refbox::RefBox<environ_storage>,
}

impl Default for environ {
    fn default() -> Self {
        Self {
            entries: refbox::RefBox::default(),
        }
    }
}

#[repr(C)]
pub struct environ_entry {
    pub name: std::ffi::CString,
    pub value: Option<std::ffi::CString>,
    pub flags: ::core::ffi::c_int,
    pub owner: Option<refbox::Weak<environ_storage>>,
}

impl environ_entry {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            value: Default::default(),
            flags: unsafe { ::core::mem::zeroed() },
            owner: None,
        }
    }
}

/// Rust-owned index and entry storage. Boxed entries keep exported pointers stable.
#[derive(Default)]
pub struct environ_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, Box<environ_entry>>,
}
