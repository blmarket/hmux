use hmux2::src::window::Window as _;
use hmux2::src::shared::pane::{
    pane_history_first, pane_history_next, pane_history_push, pane_history_remove, window_pane,
    window_pane_history, window_panes,
};
use hmux2::src::window::*;
use hmux2::src::window::{PaneOrder, Window};
use std::cell::UnsafeCell;
use std::rc::Rc;

fn pane_owner() -> Rc<UnsafeCell<window_pane>> {
    // Collection fixtures have no display resources requiring model cleanup.
    window_pane::new()
}

#[test]
fn removing_from_another_history_preserves_membership_and_cleanup() {
    let owner = pane_owner();
    let observer = Rc::downgrade(&owner);
    let mut source = window_pane_history::default();
    let mut destination = window_pane_history::default();
    pane_history_push(&mut source, observer.clone());
    unsafe {
        window_pane_stack_remove(&mut destination, Some(&owner));
        assert!(destination.is_empty());
        let retained = pane_history_first(&source)
            .expect("removing from another history preserves the source");
        assert!(Rc::ptr_eq(&retained, &owner));
        assert_eq!(
            Rc::strong_count(&owner),
            2,
            "histories store only weak entries"
        );
        window_pane_stack_remove(&mut source, Some(&owner));
        assert!(source.is_empty());

        // Cleanup follows membership alone and is safe to repeat.
        window_pane_stack_remove(&mut source, Some(&owner));
        pane_history_push(&mut source, observer.clone());
        window_pane_stack_remove(&mut source, Some(&owner));
        assert!(source.is_empty());
        drop(owner);
        assert!(
            observer.upgrade().is_some(),
            "the traversal result still retains the pane"
        );
        drop(retained);
        assert!(observer.upgrade().is_none());
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
    assert!(Rc::ptr_eq(
        &order.previous(&observers[3]).unwrap(),
        &owners[2]
    ));
    order.insert_before(&observers[2], observers[3].clone());
    order.swap(&observers[1], &observers[3]);
    assert!(Rc::ptr_eq(&order.next(&observers[0]).unwrap(), &owners[3]));
    assert!(order.remove(&observers[3]));
    assert!(Rc::ptr_eq(&order.next(&observers[0]).unwrap(), &owners[1]));

    let mut history = window_pane_history::default();
    pane_history_push(&mut history, observers[1].clone());
    pane_history_push(&mut history, observers[2].clone());
    pane_history_push(&mut history, observers[1].clone());
    assert!(Rc::ptr_eq(
        &pane_history_first(&history).unwrap(),
        &owners[1]
    ));
    assert!(Rc::ptr_eq(
        &pane_history_next(&history, &observers[1]).unwrap(),
        &owners[2]
    ));
    assert!(pane_history_remove(&mut history, &observers[2]));
    assert!(pane_history_next(&history, &observers[1]).is_none());
    drop(owners);
    assert!(observers
        .iter()
        .all(|observer| observer.upgrade().is_none()));
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
        pane_history_push(&mut history, observer.clone());
    }
    drop(first);
    drop(last);
    assert_eq!(order.position(&last_observer), Some(1));
    assert!(order.remove(&last_observer));
    assert!(pane_history_remove(&mut history, &last_observer));
    assert!(!order.remove(&last_observer));
    assert!(order.remove(&first_observer));
    assert!(pane_history_remove(&mut history, &first_observer));
    assert!(order.storage.is_empty());
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
        assert!(Rc::ptr_eq(
            &window_pane_find_by_id_str(c"%123").unwrap(),
            &owner
        ));
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
        let window = hmux2::src::shared::window::WindowRef::empty();
        let first = pane_owner();
        let last = pane_owner();
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .push_back(Rc::downgrade(&first));
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .push_back(Rc::downgrade(&last));
        let selected = window.pane_by_number(Some(&last), 3, false).unwrap();
        assert!(Rc::ptr_eq(&selected, &first));
        assert!(Rc::ptr_eq(
            &window.pane_by_number(Some(&first), 1, true).unwrap(),
            &last,
        ));
        assert!(Rc::ptr_eq(
            &window.pane_by_number(Some(&last), 0, false).unwrap(),
            &last,
        ));
        let observer = Rc::downgrade(&first);
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .storage
            .clear();
        drop(first);
        drop(last);
        assert!(observer.upgrade().is_some());
        assert!(window.pane_by_number(None, 1, false).is_none());
        drop(selected);
        assert!(observer.upgrade().is_none());
        window.release(c"test owner");
    }
}

#[test]
fn window_membership_uses_live_allocation_identity() {
    unsafe {
        let window = hmux2::src::shared::window::WindowRef::empty();
        let member = pane_owner();
        let unrelated = pane_owner();
        // Equal pane IDs do not make these the same allocation.
        (*member.get()).id = 42;
        (*unrelated.get()).id = 42;
        let observer = Rc::downgrade(&member);
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .push_back(observer.clone());
        assert!(window.contains_pane(&observer));
        assert!(!window.contains_pane(&Rc::downgrade(&unrelated)));
        drop(member);
        assert!(!window.contains_pane(&observer));
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .storage
            .clear();
        window.release(c"test owner");
    }
}
