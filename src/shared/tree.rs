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
    /// Return the stable address of an index stored in an `Option<Box<_>>`.
    /// The pointer is only a compatibility view; the option remains its owner.
    pub fn boxed_ptr(slot: &Option<Box<Self>>) -> *mut Self {
        slot.as_deref().map_or(std::ptr::null_mut(), |index| {
            index as *const Self as *mut Self
        })
    }

    /// Insert into an index whose allocation is owned by an `Option<Box<_>>`.
    pub fn insert_boxed(slot: &mut Option<Box<Self>>, key: K, node: *mut T) -> *mut T {
        if slot.is_none() {
            *slot = Some(Box::new(Self {
                entries: Default::default(),
                owners: Default::default(),
            }));
        }
        match slot
            .as_mut()
            .expect("boxed index was just initialized")
            .entries
            .entry(key)
        {
            std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(node);
                std::ptr::null_mut()
            }
        }
    }

    /// Remove from an index whose allocation is owned by an `Option<Box<_>>`.
    pub fn remove_boxed(slot: &mut Option<Box<Self>>, key: &K, node: *mut T) -> *mut T {
        Self::remove_boxed_with(slot, key, node, |_| {})
    }

    /// Remove from a boxed index and run an invalidation hook before the last
    /// index allocation is released. Compatibility pointers embedded in
    /// externally owned records can therefore be cleared while the index is
    /// still alive.
    pub fn remove_boxed_with<F>(
        slot: &mut Option<Box<Self>>,
        key: &K,
        node: *mut T,
        invalidate: F,
    ) -> *mut T
    where
        F: FnOnce(*mut T),
    {
        let (removed, empty) = {
            let Some(index) = slot.as_mut() else {
                return std::ptr::null_mut();
            };
            if index.entries.get(key).copied() != Some(node) || node.is_null() {
                return std::ptr::null_mut();
            }
            index.entries.remove(key);
            (node, index.entries.is_empty() && index.owners.is_empty())
        };
        invalidate(removed);
        if empty {
            *slot = None;
        }
        removed
    }

    /// Insert an owned node into an index whose allocation is owned by an
    /// `Option<Box<_>>`.
    pub fn insert_owned_boxed(
        slot: &mut Option<Box<Self>>,
        key: K,
        owner: refbox::RefBox<T>,
    ) -> *mut T {
        let node = owner.as_ptr() as *mut T;
        let found = Self::insert_boxed(slot, key, node);
        if found.is_null() {
            slot.as_mut()
                .expect("boxed index was just initialized")
                .owners
                .insert(node as usize, owner);
        }
        found
    }

    /// Transfer an owned node before removing it from a boxed index.
    pub fn take_owned_boxed(
        slot: &mut Option<Box<Self>>,
        node: *mut T,
    ) -> Option<refbox::RefBox<T>> {
        slot.as_mut()
            .and_then(|index| index.owners.remove(&(node as usize)))
    }

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
        (*index)
            .owners
            .get(&(node as usize))
            .map(refbox::RefBox::downgrade)
    }

    /// Transfer ownership before removing the node from the ordered index.
    pub unsafe fn take_owned(index: *mut Self, node: *mut T) -> Option<refbox::RefBox<T>> {
        if index.is_null() {
            return None;
        }
        (*index).owners.remove(&(node as usize))
    }
}
