//! Authoritative hyperlinks declarations, shared by the C translation units.

use super::abi::u_int;
#[repr(C)]
/// Box-owned from `hyperlinks_init` through the final `hyperlinks_free`.
/// URI nodes borrow this address.
pub struct hyperlinks {
    pub next_inner: u_int,
    pub by_inner: hyperlinks_by_inner_tree,
    pub by_uri: hyperlinks_by_uri_tree,
    pub references: u_int,
}
impl hyperlinks {
    pub fn empty() -> Self {
        Self {
            next_inner: 0,
            by_inner: hyperlinks_by_inner_tree { storage: None },
            by_uri: hyperlinks_by_uri_tree { storage: None },
            references: 0,
        }
    }
}

#[repr(C)]
pub struct hyperlinks_by_uri_tree {
    pub storage: Option<
        refbox::RefBox<
            std::collections::BTreeMap<(bool, Vec<u8>, Vec<u8>, u32), *mut hyperlinks_uri>,
        >,
    >,
}
#[repr(C)]
pub struct hyperlinks_uri {
    pub tree: *mut hyperlinks,
    pub inner: u_int,
    pub internal_id: std::ffi::CString,
    pub external_id: std::ffi::CString,
    pub uri: std::ffi::CString,
    pub by_inner_entry: hyperlink_inner_entry,
    pub by_uri_entry: hyperlink_uri_entry,
}

impl hyperlinks_uri {
    pub fn empty() -> Self {
        Self {
            tree: Default::default(),
            inner: Default::default(),
            internal_id: Default::default(),
            external_id: Default::default(),
            uri: Default::default(),
            by_inner_entry: hyperlink_inner_entry { owner: None },
            by_uri_entry: hyperlink_uri_entry { owner: None },
        }
    }
}
#[repr(C)]
pub struct hyperlink_uri_entry {
    /// Weak traversal handle into the URI index.
    pub owner: Option<
        refbox::Weak<
            std::collections::BTreeMap<(bool, Vec<u8>, Vec<u8>, u32), *mut hyperlinks_uri>,
        >,
    >,
}
#[repr(C)]
pub struct hyperlink_inner_entry {
    /// Weak traversal handle into the inner-ID index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<u32, *mut hyperlinks_uri>>>,
}
#[repr(C)]
pub struct hyperlinks_by_inner_tree {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<u32, *mut hyperlinks_uri>>>,
}
