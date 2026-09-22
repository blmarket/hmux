//! Authoritative tree declarations, shared by the C translation units.

pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

/// Rust-owned ordered index for C-allocated records. Empty trees are null, and
/// removing the last record releases the index. Nodes are never freed here.
/// The heap address remains stable when a containing C tree head is moved.
///
/// Raw operations require a null or live index pointer, exclusive access during
/// mutation, and records whose addresses and keys stay fixed while indexed.
/// Each live index has one owning head; moving that head must clear the old slot.
/// Entry links borrow the index and are cleared when their record is removed.
pub struct OrderedIndex<K, T> {
    entries: std::collections::BTreeMap<K, *mut T>,
}

impl<K: Ord, T> OrderedIndex<K, T> {
    pub unsafe fn find(index: *mut Self, key: &K) -> *mut T {
        if index.is_null() {
            return std::ptr::null_mut();
        }
        (*index)
            .entries
            .get(key)
            .copied()
            .unwrap_or(std::ptr::null_mut())
    }

    pub unsafe fn nfind(index: *mut Self, key: &K) -> *mut T {
        if index.is_null() {
            return std::ptr::null_mut();
        }
        (*index)
            .entries
            .range(key..)
            .next()
            .map_or(std::ptr::null_mut(), |(_, &p)| p)
    }

    pub unsafe fn edge(index: *mut Self, first: bool) -> *mut T {
        if index.is_null() {
            return std::ptr::null_mut();
        }
        let pair = if first {
            (*index).entries.first_key_value()
        } else {
            (*index).entries.last_key_value()
        };
        pair.map_or(std::ptr::null_mut(), |(_, &p)| p)
    }

    pub unsafe fn neighbor(index: *mut Self, key: &K, next: bool) -> *mut T {
        use std::ops::Bound::{Excluded, Unbounded};
        if index.is_null() {
            return std::ptr::null_mut();
        }
        let pair = if next {
            (*index).entries.range((Excluded(key), Unbounded)).next()
        } else {
            (*index)
                .entries
                .range((Unbounded, Excluded(key)))
                .next_back()
        };
        pair.map_or(std::ptr::null_mut(), |(_, &p)| p)
    }

    /// Returns the existing record on duplicate keys without replacing it.
    pub unsafe fn insert(slot: *mut *mut Self, key: K, node: *mut T) -> *mut T {
        if (*slot).is_null() {
            *slot = Box::into_raw(Box::new(Self {
                entries: Default::default(),
            }));
        }
        match (**slot).entries.entry(key) {
            std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(node);
                std::ptr::null_mut()
            }
        }
    }

    pub unsafe fn remove(slot: *mut *mut Self, key: &K, node: *mut T) -> *mut T {
        if Self::find(*slot, key) != node || node.is_null() {
            return std::ptr::null_mut();
        }
        (**slot).entries.remove(key);
        if (**slot).entries.is_empty() {
            drop(Box::from_raw(*slot));
            *slot = std::ptr::null_mut();
        }
        node
    }
}
