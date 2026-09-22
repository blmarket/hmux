//! Authoritative tree declarations, shared by the C translation units.

pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

/// Rust-owned ordered index. Most records remain externally allocated;
/// winlinks are owned by `RefBox` entries in `owners`. Empty trees are null,
/// and removing the last record releases the index. The heap address remains
/// stable when a containing C tree head is moved.
pub struct OrderedIndex<K, T> {
    entries: std::collections::BTreeMap<K, *mut T>,
    /// Only winlinks use this during their ownership migration. Other indices
    /// continue to index externally allocated records.
    owners: std::collections::HashMap<usize, refbox::RefBox<T>>,
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

    pub unsafe fn neighbor<Q: Ord + ?Sized>(index: *mut Self, key: &Q, next: bool) -> *mut T
    where
        K: std::borrow::Borrow<Q>,
    {
        use std::ops::Bound::{Excluded, Unbounded};
        if index.is_null() {
            return std::ptr::null_mut();
        }
        let pair = if next {
            (*index)
                .entries
                .range::<Q, _>((Excluded(key), Unbounded))
                .next()
        } else {
            (*index)
                .entries
                .range::<Q, _>((Unbounded, Excluded(key)))
                .next_back()
        };
        pair.map_or(std::ptr::null_mut(), |(_, &p)| p)
    }

    /// Returns the existing record on duplicate keys without replacing it.
    pub unsafe fn insert(slot: *mut *mut Self, key: K, node: *mut T) -> *mut T {
        if (*slot).is_null() {
            *slot = Box::into_raw(Box::new(Self {
                entries: Default::default(),
                owners: Default::default(),
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
        if (**slot).entries.is_empty() && (**slot).owners.is_empty() {
            drop(Box::from_raw(*slot));
            *slot = std::ptr::null_mut();
        }
        node
    }

    pub unsafe fn insert_owned(slot: *mut *mut Self, key: K, owner: refbox::RefBox<T>) -> *mut T {
        let node = owner.as_ptr() as *mut T;
        let found = Self::insert(slot, key, node);
        if found.is_null() {
            (**slot).owners.insert(node as usize, owner);
        }
        found
    }

    pub unsafe fn downgrade(index: *mut Self, node: *mut T) -> Option<refbox::Weak<T>> {
        if index.is_null() {
            return None;
        }
        (*index).owners.get(&(node as usize)).map(refbox::RefBox::downgrade)
    }

    /// Transfer ownership before removing the node from the ordered index.
    pub unsafe fn take_owned(index: *mut Self, node: *mut T) -> Option<refbox::RefBox<T>> {
        if index.is_null() {
            return None;
        }
        (*index).owners.remove(&(node as usize))
    }
}
