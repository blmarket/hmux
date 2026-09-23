//! Authoritative json model declarations.

use super::abi::int64_t;
use std::collections::BTreeMap;
use std::collections::HashMap;

#[repr(C)]
/// Box-owned with its key by the parser until recursive `json_destroy_node`.
/// Parent pointers and owner-side collections borrow stable node addresses.
pub struct json_node {
    pub type_0: json_node_type,
    pub key: *mut ::core::ffi::c_char,
    pub parent: *mut json_node,
    pub c2rust_unnamed: json_node_c2rust_unnamed,
    pub oentry: json_node_oentry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_node_oentry {
    pub rbe_left: *mut json_node,
    pub rbe_right: *mut json_node,
    pub rbe_parent: *mut json_node,
    pub rbe_color: ::core::ffi::c_int,
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
