//! Authoritative hyperlinks declarations, shared by the C translation units.

use super::abi::u_int;
#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned from `hyperlinks_init` through the final `hyperlinks_free`.
/// Copies retain the explicit reference count; URI nodes borrow this address.
pub struct hyperlinks {
    pub next_inner: u_int,
    pub by_inner: hyperlinks_by_inner_tree,
    pub by_uri: hyperlinks_by_uri_tree,
    pub references: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_by_uri_tree {
    pub storage:
        *mut std::collections::BTreeMap<(bool, Vec<u8>, Vec<u8>, u32), *mut hyperlinks_uri>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_uri {
    pub tree: *mut hyperlinks,
    pub inner: u_int,
    pub internal_id: *const ::core::ffi::c_char,
    pub external_id: *const ::core::ffi::c_char,
    pub uri: *const ::core::ffi::c_char,
    pub by_inner_entry: hyperlink_inner_entry,
    pub by_uri_entry: hyperlink_uri_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlink_uri_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<(bool, Vec<u8>, Vec<u8>, u32), *mut hyperlinks_uri>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlink_inner_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<u32, *mut hyperlinks_uri>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hyperlinks_by_inner_tree {
    pub storage: *mut std::collections::BTreeMap<u32, *mut hyperlinks_uri>,
}
