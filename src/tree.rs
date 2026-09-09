use ::std::cell::{Ref, RefCell, RefMut};
use ::std::collections::{BTreeMap, VecDeque};
use std::sync::OnceLock;
use std::thread::ThreadId;

struct CollectionThread(OnceLock<ThreadId>);

impl CollectionThread {
    const fn new() -> Self {
        Self(OnceLock::new())
    }

    fn check(&self) {
        // Unit tests hand the server's global state between worker threads
        // under the fixture lock. Production still has a single owner.
        #[cfg(test)]
        if crate::tests::test_fixtures::globals_held() {
            return;
        }
        let current = std::thread::current().id();
        assert_eq!(
            *self.0.get_or_init(|| current),
            current,
            "collection belongs to another thread",
        );
    }
}

/// A [`BTreeMap`] that lives in a `static`, replacing a transpiled `RB_HEAD`
/// global. The first access binds the collection to that thread; access from
/// another thread panics before inspecting the collection.
pub struct GlobalTree<K, V> {
    owner: CollectionThread,
    inner: RefCell<BTreeMap<K, V>>,
}

/// The synchronized owner check excludes other threads before collection access.
unsafe impl<K, V> Sync for GlobalTree<K, V> {}

impl<K: Ord, V> GlobalTree<K, V> {
    pub const fn new() -> Self {
        GlobalTree {
            owner: CollectionThread::new(),
            inner: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn map(&self) -> RefMut<'_, BTreeMap<K, V>> {
        self.owner.check();
        self.inner.borrow_mut()
    }

    pub(crate) fn read(&self) -> Ref<'_, BTreeMap<K, V>> {
        self.owner.check();
        self.inner.borrow()
    }

    /// Saves the successor before yielding each value, allowing the body to remove it.
    /// Only the current value and its successor are retained, as in RB_FOREACH_SAFE.
    pub(crate) fn walk_safe(&self) -> impl Iterator<Item = V> + '_
    where
        K: Clone,
        V: Clone,
    {
        let mut next = self
            .read()
            .first_key_value()
            .map(|(key, value)| (key.clone(), value.clone()));
        std::iter::from_fn(move || {
            let (key, value) = next.take()?;
            next = self
                .read()
                .range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
                .next()
                .map(|(key, value)| (key.clone(), value.clone()));
            Some(value)
        })
    }
}

impl<K: Ord, V> Default for GlobalTree<K, V> {
    fn default() -> Self {
        GlobalTree::new()
    }
}

/// A [`VecDeque`] that lives in a `static`, replacing a transpiled `TAILQ_HEAD`
/// global whose entries the queued struct no longer carries. The first access
/// binds the collection to that thread; access from another thread panics
/// before inspecting the collection.
pub struct GlobalQueue<T> {
    owner: CollectionThread,
    inner: RefCell<VecDeque<T>>,
}

/// The synchronized owner check excludes other threads before collection access.
unsafe impl<T> Sync for GlobalQueue<T> {}

impl<T> GlobalQueue<T> {
    pub const fn new() -> Self {
        GlobalQueue {
            owner: CollectionThread::new(),
            inner: RefCell::new(VecDeque::new()),
        }
    }

    pub fn queue(&self) -> RefMut<'_, VecDeque<T>> {
        self.owner.check();
        self.inner.borrow_mut()
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
