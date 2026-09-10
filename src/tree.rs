use ::std::cell::{Ref, RefCell, RefMut};
use ::std::collections::{BTreeMap, VecDeque};
/// A process-owned, thread-confined ordered registry.
pub struct GlobalTree<K, V> {
    inner: RefCell<BTreeMap<K, V>>,
}

impl<K: Ord, V> GlobalTree<K, V> {
    pub const fn new() -> Self {
        GlobalTree {
            inner: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn map(&self) -> RefMut<'_, BTreeMap<K, V>> {
        self.inner.borrow_mut()
    }

    pub(crate) fn read(&self) -> Ref<'_, BTreeMap<K, V>> {
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

/// A process-owned, thread-confined queue replacing a transpiled `TAILQ_HEAD`.
pub struct GlobalQueue<T> {
    inner: RefCell<VecDeque<T>>,
}

impl<T> GlobalQueue<T> {
    pub const fn new() -> Self {
        GlobalQueue {
            inner: RefCell::new(VecDeque::new()),
        }
    }

    pub fn queue(&self) -> RefMut<'_, VecDeque<T>> {
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
