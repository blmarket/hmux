use super::*;
use std::cell::Cell;

#[test]
fn live_iteration_observes_changes_without_retaining_unvisited_entries() {
    let registry = HandleRegistry::new();
    let first = registry.register(1, Rc::new("first"));
    let second_value = Rc::new("second");
    let removed = Rc::downgrade(&second_value);
    let second = registry.register(2, second_value);
    let mut entries = registry.iter();
    let (key, value) = entries.next().unwrap();
    assert_eq!((key, *value), (1, "first"));
    drop(first);
    drop(second);
    assert!(removed.upgrade().is_none());
    let third = registry.register(3, Rc::new("third"));
    assert_eq!(*entries.next().unwrap().1, "third");
    assert!(entries.next().is_none());
    drop(third);
}

#[test]
fn old_registration_cannot_remove_its_replacement() {
    let registry = HandleRegistry::new();
    let old = registry.register(1, "old");
    let current = registry.register(1, "current");
    drop(old);
    assert_eq!(registry.get(1), Some("current"));
    drop(current);
    assert_eq!(registry.get(1), None);
}

#[test]
fn updating_a_registration_cannot_change_its_replacement() {
    let registry = HandleRegistry::new();
    let mut old = registry.register(1, "old");
    old.update("updated");
    assert_eq!(registry.get(1), Some("updated"));
    let current = registry.register(1, "current");
    old.update("stale");
    assert_eq!(registry.get(1), Some("current"));
    drop(old);
    assert_eq!(registry.get(1), Some("current"));
    drop(current);
    assert_eq!(registry.get(1), None);
}

#[test]
fn updating_an_observation_releases_the_borrow_before_dropping_values() {
    struct OnDrop(Box<dyn Fn()>);
    impl Drop for OnDrop {
        fn drop(&mut self) {
            (self.0)();
        }
    }
    let registry = HandleRegistry::new();
    let entries = Rc::clone(&registry.entries);
    let mut registration = registry.register(
        1,
        OnDrop(Box::new(move || {
            assert!(entries.borrow_mut().contains_key(&1));
        })),
    );
    registration.update(OnDrop(Box::new(|| {})));
    registry.remove(1);
    let entries = Rc::clone(&registry.entries);
    registration.update(OnDrop(Box::new(move || {
        assert!(entries.borrow_mut().is_empty());
    })));
}

#[test]
fn clearing_an_index_does_not_let_old_tokens_remove_new_entries() {
    let registry = HandleRegistry::new();
    let old = registry.register(1, "old");
    registry.clear();
    let current = registry.register(1, "new");
    drop(old);
    assert_eq!(registry.get(1), Some("new"));
    drop(current);
    assert_eq!(registry.get(1), None);
}

#[test]
fn registration_owns_cleanup_after_the_registry_owner_is_gone() {
    let registry = HandleRegistry::new();
    let allocation = Rc::downgrade(&registry.entries);
    let registration = registry.register(1, "retained");
    drop(registry);
    assert!(allocation.upgrade().is_some());
    drop(registration);
    assert!(allocation.upgrade().is_none());
}

#[test]
fn removing_an_observation_releases_the_borrow_before_its_destructor() {
    struct OnDrop(Box<dyn Fn()>);
    impl Drop for OnDrop {
        fn drop(&mut self) {
            (self.0)();
        }
    }
    let registry = HandleRegistry::new();
    let entries = Rc::clone(&registry.entries);
    let observed = Rc::new(Cell::new(false));
    let signal = Rc::clone(&observed);
    let registration = registry.register(
        1,
        OnDrop(Box::new(move || {
            assert!(entries.borrow_mut().is_empty());
            signal.set(true);
        })),
    );
    drop(registration);
    assert!(observed.get());
}
