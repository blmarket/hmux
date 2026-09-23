use crate::src::ffi::libc::strcmp;
use crate::src::shared::abi::*;
pub use crate::src::shared::hyperlinks::{
    hyperlink_inner_entry, hyperlink_uri_entry, hyperlinks, hyperlinks_by_inner_tree,
    hyperlinks_by_uri_tree, hyperlinks_uri,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_OCTAL};
use crate::src::utf8::utf8_stravis_cstring;
use std::{collections::VecDeque, ffi::CString};

/// One retained reference to the mutable hyperlink table. The legacy table
/// remains behind the raw C API; this owner never creates an aliased Rust
/// reference to it. Clone/drop preserve the existing C retain/release contract.
pub(crate) struct HyperlinksRef(std::ptr::NonNull<hyperlinks>);

impl HyperlinksRef {
    pub(crate) unsafe fn new() -> Self {
        Self(std::ptr::NonNull::new(hyperlinks_init()).expect("allocated hyperlink table"))
    }
    pub(crate) fn as_ptr(&self) -> *mut hyperlinks {
        self.0.as_ptr()
    }
}
impl Clone for HyperlinksRef {
    fn clone(&self) -> Self {
        unsafe {
            hyperlinks_copy(self.as_ptr());
        }
        Self(self.0)
    }
}
impl Drop for HyperlinksRef {
    fn drop(&mut self) {
        unsafe {
            hyperlinks_free(self.as_ptr());
        }
    }
}

// The C-layout record is the first field so tree index pointers still point
// at hyperlinks_uri. The external ID remains valid until hyperlinks_remove.
#[repr(C)]
struct HyperlinkUriOwner {
    node: hyperlinks_uri,
    internal_id: CString,
    external_id: CString,
    uri: CString,
}

const _: () = assert!(std::mem::offset_of!(HyperlinkUriOwner, node) == 0);

pub const MAX_HYPERLINKS: ::core::ffi::c_int = 5000 as ::core::ffi::c_int;
pub const MAX_HYPERLINK_URI: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
static mut hyperlinks_next_external_id: ::core::ffi::c_longlong = 1 as ::core::ffi::c_longlong;
// This queue owns the records and their insertion order. Boxes keep each node
// at a stable address for the per-table URI and inner-ID indexes.
static mut GLOBAL_HYPERLINKS: VecDeque<Box<HyperlinkUriOwner>> = VecDeque::new();
unsafe extern "C" fn hyperlinks_by_uri_cmp(
    mut left: *mut hyperlinks_uri,
    mut right: *mut hyperlinks_uri,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    if *(*left).internal_id as ::core::ffi::c_int == '\0' as i32
        || *(*right).internal_id as ::core::ffi::c_int == '\0' as i32
    {
        if *(*left).internal_id as ::core::ffi::c_int != '\0' as i32 {
            return -(1 as ::core::ffi::c_int);
        }
        if *(*right).internal_id as ::core::ffi::c_int != '\0' as i32 {
            return 1 as ::core::ffi::c_int;
        }
        return (*left).inner.wrapping_sub((*right).inner) as ::core::ffi::c_int;
    }
    r = strcmp((*left).internal_id, (*right).internal_id);
    if r != 0 as ::core::ffi::c_int {
        return r;
    }
    return strcmp((*left).uri, (*right).uri);
}

unsafe extern "C" fn hyperlinks_by_inner_cmp(
    mut left: *mut hyperlinks_uri,
    mut right: *mut hyperlinks_uri,
) -> ::core::ffi::c_int {
    return (*left).inner.wrapping_sub((*right).inner) as ::core::ffi::c_int;
}

unsafe extern "C" fn hyperlinks_remove(mut hlu: *mut hyperlinks_uri) {
    let global_hyperlinks = std::ptr::addr_of_mut!(GLOBAL_HYPERLINKS);
    let index = (*global_hyperlinks)
        .iter()
        .position(|owner| std::ptr::addr_of!((**owner).node).cast_mut() == hlu)
        .expect("hyperlink record missing from global insertion order");
    let owner = (*global_hyperlinks)
        .remove(index)
        .expect("hyperlink owner missing from global insertion order");
    let hl = owner.node.tree;
    hyperlinks_by_inner_tree_remove(&raw mut (*hl).by_inner, hlu);
    hyperlinks_by_uri_tree_remove(&raw mut (*hl).by_uri, hlu);
    drop(owner);
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_put(
    mut hl: *mut hyperlinks,
    mut uri_in: *const ::core::ffi::c_char,
    mut internal_id_in: *const ::core::ffi::c_char,
) -> u_int {
    let mut find: hyperlinks_uri = hyperlinks_uri {
        tree: ::core::ptr::null_mut::<hyperlinks>(),
        inner: 0,
        internal_id: ::core::ptr::null::<::core::ffi::c_char>(),
        external_id: ::core::ptr::null::<::core::ffi::c_char>(),
        uri: ::core::ptr::null::<::core::ffi::c_char>(),
        by_inner_entry: hyperlink_inner_entry {
            owner: std::ptr::null_mut(),
        },
        by_uri_entry: hyperlink_uri_entry {
            owner: std::ptr::null_mut(),
        },
    };
    let mut hlu: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    if internal_id_in.is_null() {
        internal_id_in = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let uri = utf8_stravis_cstring(uri_in, VIS_OCTAL | VIS_CSTYLE);
    if uri.as_bytes().len() > MAX_HYPERLINK_URI as usize {
        return 0 as u_int;
    }
    let internal_id = utf8_stravis_cstring(internal_id_in, VIS_OCTAL | VIS_CSTYLE);
    if !internal_id.as_bytes().is_empty() {
        find.uri = uri.as_ptr();
        find.internal_id = internal_id.as_ptr();
        hlu = hyperlinks_by_uri_tree_find(&raw mut (*hl).by_uri, &raw mut find);
        if !hlu.is_null() {
            return (*hlu).inner;
        }
    }
    let fresh0 = hyperlinks_next_external_id;
    hyperlinks_next_external_id = hyperlinks_next_external_id + 1;
    let mut owner = Box::new(HyperlinkUriOwner {
        node: std::mem::zeroed(),
        internal_id,
        external_id: CString::new(format!("tmux{:X}", fresh0 as u64))
            .expect("generated hyperlink ID contains no NUL"),
        uri,
    });
    owner.node.internal_id = owner.internal_id.as_ptr();
    owner.node.external_id = owner.external_id.as_ptr();
    owner.node.uri = owner.uri.as_ptr();
    hlu = &mut owner.node;
    let fresh1 = (*hl).next_inner;
    (*hl).next_inner = (*hl).next_inner.wrapping_add(1);
    (*hlu).inner = fresh1;
    (*hlu).tree = hl;
    hyperlinks_by_uri_tree_insert(&raw mut (*hl).by_uri, hlu);
    hyperlinks_by_inner_tree_insert(&raw mut (*hl).by_inner, hlu);
    let global_hyperlinks = std::ptr::addr_of_mut!(GLOBAL_HYPERLINKS);
    (*global_hyperlinks).push_back(owner);
    if (*global_hyperlinks).len() == MAX_HYPERLINKS as usize {
        let oldest_owner = (*global_hyperlinks)
            .front()
            .expect("new hyperlink missing from global insertion order");
        let oldest = std::ptr::addr_of!((**oldest_owner).node).cast_mut();
        hyperlinks_remove(oldest);
    }
    return (*hlu).inner;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_get(
    mut hl: *mut hyperlinks,
    mut inner: u_int,
    mut uri_out: *mut *const ::core::ffi::c_char,
    mut internal_id_out: *mut *const ::core::ffi::c_char,
    mut external_id_out: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut find: hyperlinks_uri = hyperlinks_uri {
        tree: ::core::ptr::null_mut::<hyperlinks>(),
        inner: 0,
        internal_id: ::core::ptr::null::<::core::ffi::c_char>(),
        external_id: ::core::ptr::null::<::core::ffi::c_char>(),
        uri: ::core::ptr::null::<::core::ffi::c_char>(),
        by_inner_entry: hyperlink_inner_entry {
            owner: std::ptr::null_mut(),
        },
        by_uri_entry: hyperlink_uri_entry {
            owner: std::ptr::null_mut(),
        },
    };
    let mut hlu: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    find.inner = inner;
    hlu = hyperlinks_by_inner_tree_find(&raw mut (*hl).by_inner, &raw mut find);
    if hlu.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !internal_id_out.is_null() {
        *internal_id_out = (*hlu).internal_id;
    }
    if !external_id_out.is_null() {
        *external_id_out = (*hlu).external_id;
    }
    *uri_out = (*hlu).uri;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_init() -> *mut hyperlinks {
    let mut hl: *mut hyperlinks = ::core::ptr::null_mut::<hyperlinks>();
    hl = Box::into_raw(Box::new(::core::mem::zeroed::<hyperlinks>()));
    (*hl).next_inner = 1 as u_int;
    (*hl).by_uri.storage = std::ptr::null_mut();
    (*hl).by_inner.storage = std::ptr::null_mut();
    (*hl).references = 1 as u_int;
    return hl;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_copy(mut hl: *mut hyperlinks) -> *mut hyperlinks {
    (*hl).references = (*hl).references.wrapping_add(1);
    return hl;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_reset(mut hl: *mut hyperlinks) {
    let mut hlu: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut hlu1: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    hlu = hyperlinks_by_inner_tree_minmax(&raw mut (*hl).by_inner, RB_NEGINF);
    while !hlu.is_null() && {
        hlu1 = hyperlinks_by_inner_tree_next(hlu);
        1 as ::core::ffi::c_int != 0
    } {
        hyperlinks_remove(hlu);
        hlu = hlu1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_free(mut hl: *mut hyperlinks) {
    (*hl).references = (*hl).references.wrapping_sub(1);
    if (*hl).references == 0 as u_int {
        hyperlinks_reset(hl);
        drop(Box::from_raw(hl));
    }
}
unsafe fn hyperlinks_by_inner_tree_key(elm: *mut hyperlinks_uri) -> u32 {
    (*elm).inner
}
pub unsafe fn hyperlinks_by_inner_tree_find(
    head: *mut hyperlinks_by_inner_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_inner_tree_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn hyperlinks_by_inner_tree_nfind(
    head: *mut hyperlinks_by_inner_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_inner_tree_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn hyperlinks_by_inner_tree_insert(
    head: *mut hyperlinks_by_inner_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let key = hyperlinks_by_inner_tree_key(elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).by_inner_entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn hyperlinks_by_inner_tree_remove(
    head: *mut hyperlinks_by_inner_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = hyperlinks_by_inner_tree_key(elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).by_inner_entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn hyperlinks_by_inner_tree_minmax(
    head: *mut hyperlinks_by_inner_tree,
    direction: ::core::ffi::c_int,
) -> *mut hyperlinks_uri {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn hyperlinks_by_inner_tree_next(elm: *mut hyperlinks_uri) -> *mut hyperlinks_uri {
    let Some(map) = (*elm).by_inner_entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_inner_tree_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn hyperlinks_by_inner_tree_prev(elm: *mut hyperlinks_uri) -> *mut hyperlinks_uri {
    let Some(map) = (*elm).by_inner_entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_inner_tree_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

unsafe fn hyperlinks_by_uri_tree_key(elm: *mut hyperlinks_uri) -> (bool, Vec<u8>, Vec<u8>, u32) {
    {
        let id = std::ffi::CStr::from_ptr((*elm).internal_id).to_bytes();
        if id.is_empty() {
            (true, Vec::new(), Vec::new(), (*elm).inner)
        } else {
            (
                false,
                id.to_vec(),
                std::ffi::CStr::from_ptr((*elm).uri).to_bytes().to_vec(),
                0,
            )
        }
    }
}
pub unsafe fn hyperlinks_by_uri_tree_find(
    head: *mut hyperlinks_by_uri_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_uri_tree_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn hyperlinks_by_uri_tree_nfind(
    head: *mut hyperlinks_by_uri_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_uri_tree_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn hyperlinks_by_uri_tree_insert(
    head: *mut hyperlinks_by_uri_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let key = hyperlinks_by_uri_tree_key(elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).by_uri_entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn hyperlinks_by_uri_tree_remove(
    head: *mut hyperlinks_by_uri_tree,
    elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = hyperlinks_by_uri_tree_key(elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).by_uri_entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn hyperlinks_by_uri_tree_minmax(
    head: *mut hyperlinks_by_uri_tree,
    direction: ::core::ffi::c_int,
) -> *mut hyperlinks_uri {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn hyperlinks_by_uri_tree_next(elm: *mut hyperlinks_uri) -> *mut hyperlinks_uri {
    let Some(map) = (*elm).by_uri_entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_uri_tree_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn hyperlinks_by_uri_tree_prev(elm: *mut hyperlinks_uri) -> *mut hyperlinks_uri {
    let Some(map) = (*elm).by_uri_entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = hyperlinks_by_uri_tree_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
