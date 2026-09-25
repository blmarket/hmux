//! Authoritative json model declarations.

use super::abi::int64_t;
use std::collections::BTreeMap;
use std::collections::HashMap;

#[repr(C)]
pub struct json_node {
    pub type_0: json_node_type,
    pub key: Option<std::ffi::CString>,
    pub parent: *mut json_node,
    pub c2rust_unnamed: json_node_c2rust_unnamed,
    // The tagged union is still Copy and zero-initialized by parser code.
    // It borrows this string and the array storage below until node teardown;
    // move these into the variants when migrating the union itself.
    pub(crate) string: Option<std::ffi::CString>,
    pub(crate) members: Option<Box<json_members_storage>>,
}

impl json_node {
    pub fn empty() -> Self {
        Self {
            type_0: unsafe { ::core::mem::zeroed() },
            key: Default::default(),
            parent: unsafe { ::core::mem::zeroed() },
            c2rust_unnamed: unsafe { ::core::mem::zeroed() },
            string: Default::default(),
            members: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union json_node_c2rust_unnamed {
    pub str_0: *mut ::core::ffi::c_char,
    pub num: int64_t,
    pub boolean: ::core::ffi::c_int,
    pub fields: json_fields,
    pub members: json_members,
}

#[derive(Copy, Clone)]
#[repr(C)]
/// Handle to array ordering storage owned by the array node.
pub struct json_members {
    pub storage: *mut json_members_storage,
}

/// Array order belongs to the array node. Child nodes do not need intrusive links.
#[derive(Default)]
pub struct json_members_storage {
    pub(crate) members: Vec<*mut json_node>,
    pub(crate) indices: HashMap<*mut json_node, usize>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_fields {
    pub entries: *mut json_fields_storage,
}

/// Rust-owned ordering storage for an object's Box-owned field nodes.
#[derive(Default)]
pub struct json_fields_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut json_node>,
}

pub type json_node_type = ::core::ffi::c_uint;
