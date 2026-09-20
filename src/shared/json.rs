//! Authoritative json model declarations.

use super::abi::int64_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_node {
    pub type_0: json_node_type,
    pub key: *mut ::core::ffi::c_char,
    pub parent: *mut json_node,
    pub c2rust_unnamed: json_node_c2rust_unnamed,
    pub oentry: json_node_oentry,
    pub aentry: json_node_aentry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_node_aentry {
    pub tqe_next: *mut json_node,
    pub tqe_prev: *mut *mut json_node,
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
pub struct json_members {
    pub tqh_first: *mut json_node,
    pub tqh_last: *mut *mut json_node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_fields {
    pub rbh_root: *mut json_node,
}

pub type json_node_type = ::core::ffi::c_uint;
