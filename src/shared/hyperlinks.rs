//! Shared hyperlink tables and immutable, heap-allocated entries.

use super::abi::u_int;
use std::{collections::BTreeMap, ffi::CString, rc::Rc};

pub(crate) type HyperlinkKey = (bool, Vec<u8>, Vec<u8>, u32);
pub type HyperlinkRef = Rc<hyperlinks_uri>;

/// The table, index records, and entries remain heap allocated.
/// Inner IDs own entries; the URI index stores IDs into the same table.
pub struct hyperlinks {
    pub(crate) next_inner: u_int,
    pub(crate) by_inner: Option<Box<BTreeMap<u_int, HyperlinkRef>>>,
    pub(crate) by_uri: Option<Box<BTreeMap<HyperlinkKey, u_int>>>,
}

impl hyperlinks {
    pub fn empty() -> Self {
        Self {
            next_inner: 0,
            by_inner: None,
            by_uri: None,
        }
    }
}

pub struct hyperlinks_uri {
    pub inner: u_int,
    pub internal_id: CString,
    pub external_id: CString,
    pub uri: CString,
}
