//! Authoritative hyperlinks declarations, shared by the C translation units.

use super::abi::u_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks {
    pub next_inner: u_int,
    pub by_inner: hyperlinks_by_inner_tree,
    pub by_uri: hyperlinks_by_uri_tree,
    pub references: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_by_uri_tree {
    pub rbh_root: *mut hyperlinks_uri,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_uri {
    pub tree: *mut hyperlinks,
    pub inner: u_int,
    pub internal_id: *const ::core::ffi::c_char,
    pub external_id: *const ::core::ffi::c_char,
    pub uri: *const ::core::ffi::c_char,
    pub list_entry: hyperlink_list_entry,
    pub by_inner_entry: hyperlink_inner_entry,
    pub by_uri_entry: hyperlink_uri_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlink_uri_entry {
    pub rbe_left: *mut hyperlinks_uri,
    pub rbe_right: *mut hyperlinks_uri,
    pub rbe_parent: *mut hyperlinks_uri,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlink_inner_entry {
    pub rbe_left: *mut hyperlinks_uri,
    pub rbe_right: *mut hyperlinks_uri,
    pub rbe_parent: *mut hyperlinks_uri,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlink_list_entry {
    pub tqe_next: *mut hyperlinks_uri,
    pub tqe_prev: *mut *mut hyperlinks_uri,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_by_inner_tree {
    pub rbh_root: *mut hyperlinks_uri,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_list {
    pub tqh_first: *mut hyperlinks_uri,
    pub tqh_last: *mut *mut hyperlinks_uri,
}
