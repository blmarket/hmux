use hmux2::src::shared::pane::{window_pane, window_pane_history, window_panes};
use hmux2::src::shared::rc;
use hmux2::src::window::*;
use std::cell::UnsafeCell;
use std::rc::Rc;

fn pane_owner() -> Rc<UnsafeCell<window_pane>> {
    // Collection fixtures have no display resources requiring model cleanup.
    window_pane::new()
}

#[test]
fn removing_from_another_history_preserves_membership_and_cleanup() {
    let owner = pane_owner();
    let pane = rc::as_ptr(&owner);
    let mut source = window_pane_history::default();
    let mut destination = window_pane_history::default();
    source.push_front(Rc::downgrade(&owner));
    unsafe {
        (*pane).flags |= hmux2::src::shared::pane::PANE_VISITED;
        window_pane_stack_remove(&mut destination, pane);
        assert_ne!((*pane).flags & hmux2::src::shared::pane::PANE_VISITED, 0);
        window_pane_stack_remove(&mut source, pane);
        assert!(source.is_empty());
        assert_eq!((*pane).flags & hmux2::src::shared::pane::PANE_VISITED, 0);

        // Cleanup must also make progress if an entry has lost its flag.
        source.push_front(Rc::downgrade(&owner));
        window_pane_stack_remove(&mut source, pane);
        assert!(source.is_empty());
    }
}

#[test]
fn pane_order_and_visit_history_preserve_stable_weak_entries() {
    let owners: Vec<_> = (0..4).map(|_| pane_owner()).collect();
    let observers: Vec<_> = owners.iter().map(Rc::downgrade).collect();
    let mut order = window_panes::default();
    for observer in &observers {
        order.push_back(observer.clone());
    }
    assert!(Rc::ptr_eq(&order.first().unwrap(), &owners[0]));
    assert!(Rc::ptr_eq(&order.next(&observers[0]).unwrap(), &owners[1]));
    assert!(Rc::ptr_eq(&order.previous(&observers[3]).unwrap(), &owners[2]));
    order.insert_before(&observers[2], observers[3].clone());
    order.swap(&observers[1], &observers[3]);
    assert!(Rc::ptr_eq(&order.next(&observers[0]).unwrap(), &owners[3]));
    assert!(order.remove(&observers[3]));
    assert!(Rc::ptr_eq(&order.next(&observers[0]).unwrap(), &owners[1]));

    let mut history = window_pane_history::default();
    history.push_front(observers[1].clone());
    history.push_front(observers[2].clone());
    history.push_front(observers[1].clone());
    assert!(Rc::ptr_eq(&history.first().unwrap(), &owners[1]));
    assert!(Rc::ptr_eq(&history.next(&observers[1]).unwrap(), &owners[2]));
    assert!(history.remove(&observers[2]));
    assert!(history.next(&observers[1]).is_none());
    drop(owners);
    assert!(observers.iter().all(|observer| observer.upgrade().is_none()));
}

#[test]
fn expired_observers_can_be_removed_without_upgrading_any_pane() {
    let first = pane_owner();
    let last = pane_owner();
    let first_observer = Rc::downgrade(&first);
    let last_observer = Rc::downgrade(&last);
    let mut order = window_panes::default();
    let mut history = window_pane_history::default();
    for observer in [&first_observer, &last_observer] {
        order.push_back(observer.clone());
        history.push_front(observer.clone());
    }
    drop(first);
    drop(last);
    assert_eq!(order.position(&last_observer), Some(1));
    assert!(order.remove(&last_observer));
    assert!(history.remove(&last_observer));
    assert!(!order.remove(&last_observer));
    assert!(order.remove(&first_observer));
    assert!(history.remove(&first_observer));
    assert!(order.storage.is_none());
    assert!(history.is_empty());
}

#[test]
fn global_lookup_preserves_pane_identity_and_retains_removed_pane() {
    unsafe {
        let head = &raw mut all_window_panes;
        assert!((*head).storage.is_none());
        let owner = window_pane::new();
        (*owner.get()).id = 123;
        let observer = Rc::downgrade(&owner);
        assert!(window_pane_tree_insert(&mut *head, owner.clone()).is_none());
        let retained = window_pane_find_by_id(123).unwrap();
        assert!(Rc::ptr_eq(&retained, &owner));
        assert!(Rc::ptr_eq(&window_pane_find_by_id_str(c"%123").unwrap(), &owner));
        for invalid in [c"123", c"%", c"%x", c"%-1", c"%4294967296"] {
            assert!(window_pane_find_by_id_str(invalid).is_none());
        }
        assert!(window_pane_find_by_id(124).is_none());
        drop(window_pane_tree_remove(&mut *head, &mut *owner.get()));
        assert!(window_pane_find_by_id(123).is_none());
        assert!((*head).storage.is_none());
        drop(owner);
        assert!(observer.upgrade().is_some());
        assert_eq!((*retained.get()).id, 123);
        drop(retained);
        assert!(observer.upgrade().is_none());
    }
}

#[test]
fn ordering_lookup_retains_a_detached_pane() {
    let owner = pane_owner();
    let weak = Rc::downgrade(&owner);
    let mut order = window_panes::default();
    order.push_back(weak.clone());
    unsafe {
        let retained = order.first().unwrap();
        assert!(order.remove(&Rc::downgrade(&retained)));
        drop(owner);
        assert!(weak.upgrade().is_some());
        drop(retained);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn index_traversal_results_retain_panes_after_index_removal() {
    unsafe {
        let mut head = hmux2::src::shared::pane::window_pane_tree { storage: None };
        let first = pane_owner();
        let second = pane_owner();
        (*first.get()).id = 1;
        (*second.get()).id = 2;
        let first_observer = Rc::downgrade(&first);
        let second_observer = Rc::downgrade(&second);
        assert!(window_pane_tree_insert(&mut head, first.clone()).is_none());
        assert!(window_pane_tree_insert(&mut head, second.clone()).is_none());
        let current = window_pane_tree_minmax(&head).unwrap();
        let successor = window_pane_tree_next(&*current.get()).unwrap();
        assert!(Rc::ptr_eq(&current, &first));
        assert!(Rc::ptr_eq(&successor, &second));
        drop(window_pane_tree_remove(&mut head, &mut *first.get()));
        drop(window_pane_tree_remove(&mut head, &mut *second.get()));
        drop(first);
        drop(second);
        assert!(head.storage.is_none());
        assert!(window_pane_tree_minmax(&head).is_none());
        assert!(window_pane_tree_next(&*current.get()).is_none());
        assert!(first_observer.upgrade().is_some());
        assert!(second_observer.upgrade().is_some());
        drop(current);
        drop(successor);
        assert!(first_observer.upgrade().is_none());
        assert!(second_observer.upgrade().is_none());
    }
}

#[test]
fn relative_pane_selection_wraps_and_retains_its_result() {
    unsafe {
        let window = hmux2::src::shared::window::window::new();
        let first = pane_owner();
        let last = pane_owner();
        (*window.get()).panes.push_back(Rc::downgrade(&first));
        (*window.get()).panes.push_back(Rc::downgrade(&last));
        let selected = window_pane_next_by_number(&*window.get(), Some(&last), 3).unwrap();
        assert!(Rc::ptr_eq(&selected, &first));
        assert!(Rc::ptr_eq(
            &window_pane_previous_by_number(&*window.get(), Some(&first), 1).unwrap(),
            &last,
        ));
        assert!(Rc::ptr_eq(
            &window_pane_next_by_number(&*window.get(), Some(&last), 0).unwrap(),
            &last,
        ));
        let observer = Rc::downgrade(&first);
        (*window.get()).panes.storage = None;
        drop(first);
        drop(last);
        assert!(observer.upgrade().is_some());
        assert!(window_pane_next_by_number(&*window.get(), None, 1).is_none());
        drop(selected);
        assert!(observer.upgrade().is_none());
        hmux2::src::window::window_remove_ref(window, c"test owner".as_ptr());
    }
}

#[test]
fn window_membership_uses_live_allocation_identity() {
    unsafe {
        let window = hmux2::src::shared::window::window::new();
        let member = pane_owner();
        let unrelated = pane_owner();
        // Equal pane IDs do not make these the same allocation.
        (*member.get()).id = 42;
        (*unrelated.get()).id = 42;
        let observer = Rc::downgrade(&member);
        (*window.get()).panes.push_back(observer.clone());
        assert!(window_has_pane(&*window.get(), &observer));
        assert!(!window_has_pane(&*window.get(), &Rc::downgrade(&unrelated)));
        drop(member);
        assert!(!window_has_pane(&*window.get(), &observer));
        (*window.get()).panes.storage = None;
        hmux2::src::window::window_remove_ref(window, c"test owner".as_ptr());
    }
}
