use ::core::cell::UnsafeCell;
use ::std::collections::{BTreeMap, VecDeque};

/// A [`BTreeMap`] that lives in a `static`, replacing a transpiled `RB_HEAD`
/// global. The server is single-threaded, so access is unsynchronised.
pub struct GlobalTree<K, V> {
    inner: UnsafeCell<BTreeMap<K, V>>,
}

unsafe impl<K, V> Sync for GlobalTree<K, V> {}

impl<K: Ord, V> GlobalTree<K, V> {
    pub const fn new() -> Self {
        GlobalTree {
            inner: UnsafeCell::new(BTreeMap::new()),
        }
    }

    /// Borrow the map. Callers must not hold a borrow across a call that
    /// mutates the same tree.
    #[allow(clippy::mut_from_ref)]
    pub fn map(&self) -> &mut BTreeMap<K, V> {
        unsafe { &mut *self.inner.get() }
    }
}

impl<K: Ord, V> Default for GlobalTree<K, V> {
    fn default() -> Self {
        GlobalTree::new()
    }
}

/// A [`VecDeque`] that lives in a `static`, replacing a transpiled `TAILQ_HEAD`
/// global whose entries the queued struct no longer carries. The server is
/// single-threaded, so access is unsynchronised.
pub struct GlobalQueue<T> {
    inner: UnsafeCell<VecDeque<T>>,
}

unsafe impl<T> Sync for GlobalQueue<T> {}

impl<T> GlobalQueue<T> {
    pub const fn new() -> Self {
        GlobalQueue {
            inner: UnsafeCell::new(VecDeque::new()),
        }
    }

    /// Borrow the queue. Callers must not hold a borrow across a call that
    /// mutates the same queue.
    #[allow(clippy::mut_from_ref)]
    pub fn queue(&self) -> &mut VecDeque<T> {
        unsafe { &mut *self.inner.get() }
    }
}

impl<T> Default for GlobalQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tests/test_tree.rs"]
mod tests;
