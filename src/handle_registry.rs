use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A thread-confined index of observations, with cleanup owned by registrations.
#[derive(Clone)]
pub(crate) struct HandleRegistry<T> {
    entries: Rc<RefCell<BTreeMap<usize, Entry<T>>>>,
}

struct Entry<T> {
    identity: Rc<()>,
    value: T,
}

/// Retains the index through cleanup and removes only its own registration.
pub(crate) struct HandleRegistration<T> {
    entries: Rc<RefCell<BTreeMap<usize, Entry<T>>>>,
    key: usize,
    identity: Rc<()>,
}

impl<T> HandleRegistry<T> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub(crate) fn register(&self, key: usize, value: T) -> HandleRegistration<T> {
        let identity = Rc::new(());
        let old = self.entries.borrow_mut().insert(
            key,
            Entry {
                identity: Rc::clone(&identity),
                value,
            },
        );
        drop(old);
        HandleRegistration {
            entries: Rc::clone(&self.entries),
            key,
            identity,
        }
    }

    pub(crate) fn get(&self, key: usize) -> Option<T>
    where
        T: Clone,
    {
        self.entries
            .borrow()
            .get(&key)
            .map(|entry| entry.value.clone())
    }

    pub(crate) fn keys(&self) -> Vec<usize> {
        self.entries.borrow().keys().copied().collect()
    }

    /// Reads each observation from the live index in key order.
    /// Registry borrows end before yielding, so callers may register or remove entries.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, T)> + '_
    where
        T: Clone,
    {
        let mut last = None;
        std::iter::from_fn(move || {
            let lower = last.map_or(std::ops::Bound::Unbounded, std::ops::Bound::Excluded);
            let next = self
                .entries
                .borrow()
                .range((lower, std::ops::Bound::Unbounded))
                .next()
                .map(|(key, entry)| (*key, entry.value.clone()));
            last = next.as_ref().map(|(key, _)| *key);
            next
        })
        .fuse()
    }

    pub(crate) fn remove(&self, key: usize) {
        let removed = self.entries.borrow_mut().remove(&key);
        drop(removed);
    }

    #[cfg(test)]
    pub(crate) fn clear(&self) {
        let removed = std::mem::take(&mut *self.entries.borrow_mut());
        drop(removed);
    }
}

impl<T> HandleRegistration<T> {
    /// Updates this registration without changing a replacement's observation.
    #[cfg(test)]
    pub(crate) fn update(&mut self, value: T) {
        let mut value = Some(value);
        let old = {
            let mut entries = self.entries.borrow_mut();
            entries.get_mut(&self.key).and_then(|entry| {
                Rc::ptr_eq(&entry.identity, &self.identity)
                    .then(|| std::mem::replace(&mut entry.value, value.take().unwrap()))
            })
        };
        drop(old);
    }
}

impl<T> Drop for HandleRegistration<T> {
    fn drop(&mut self) {
        let removed = {
            let mut entries = self.entries.borrow_mut();
            if entries
                .get(&self.key)
                .is_some_and(|entry| Rc::ptr_eq(&entry.identity, &self.identity))
            {
                entries.remove(&self.key)
            } else {
                None
            }
        };
        drop(removed);
    }
}

#[cfg(test)]
#[path = "tests/test_handle_registry.rs"]
mod tests;
