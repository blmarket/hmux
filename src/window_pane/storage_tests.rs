//! Storage assertions stay beside the pane implementation.
#[cfg(test)]
use crate::src::window::WindowFixture as _;
use super::*;
use crate::src::shared::pane::{
    pane_history_first, pane_history_next, pane_history_push, pane_history_remove, window_pane,
    window_pane_history, window_panes,
};
use crate::src::window::Window as _;
use crate::src::window::*;
use crate::src::window::{PaneOrder, Window};
use crate::src::window_pane::WindowPane as _;
use std::cell::UnsafeCell;
use std::rc::Rc;

fn pane_owner() -> Rc<UnsafeCell<window_pane>> {
    // Collection fixtures have no display resources requiring model cleanup.
    std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate()
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
        let owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
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
        let mut head = crate::src::shared::pane::window_pane_tree { storage: None };
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
        let window = crate::src::shared::window::WindowRef::empty();
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
        (window).release(std::ffi::CStr::from_ptr(c"test owner".as_ptr()));
    }
}

#[test]
fn window_membership_uses_live_allocation_identity() {
    unsafe {
        let window = crate::src::shared::window::WindowRef::empty();
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
        (window).release(std::ffi::CStr::from_ptr(c"test owner".as_ptr()));
    }
}

#[cfg(test)]
mod collection_index_tests {
    use super::*;

    #[test]
    fn pane_rc_keeps_weak_observers_alive_until_the_last_release() {
        unsafe {
            // No display resources in this fixture; exercise the actual pane
            // retain/release functions with ordinary field drop.
            let wp_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let wp = wp_owner.get();
            let weak = window_pane_weak(&*(wp));
            let callback = window_pane_add_ref(
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                c"callback".as_ptr(),
            );
            window_pane_remove_ref(wp_owner, c"pane shutdown".as_ptr());
            let retained = weak.upgrade().expect("callback keeps the pane alive");
            assert_eq!(retained.get(), wp);
            window_pane_remove_ref(callback, c"callback complete".as_ptr());
            assert!(
                weak.upgrade().is_some(),
                "upgraded Rc independently owns the pane"
            );
            drop(retained);
            assert!(weak.upgrade().is_none());
        }
    }

    #[test]
    fn pane_index_observers_clear_on_removal_and_expire_with_the_owner() {
        unsafe {
            let mut head = window_pane_tree { storage: None };
            let mut other = window_pane_tree { storage: None };
            let first_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let first = first_owner.get();
            (*first).id = 1;
            let second_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let second = second_owner.get();
            (*second).id = 2;
            let duplicate_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let duplicate = duplicate_owner.get();
            (*duplicate).id = 1;

            assert!(window_pane_tree_insert(&mut head, first_owner.clone()).is_none());
            assert!(window_pane_tree_insert(&mut head, second_owner.clone()).is_none());
            let index_observer = (*first).owner.clone();
            assert!(Rc::ptr_eq(
                &window_pane_tree_insert(&mut head, duplicate_owner.clone()).unwrap(),
                &first_owner
            ));
            assert!((*duplicate).owner.is_empty());
            assert!(window_pane_tree_remove(&mut other, &mut *first).is_none());
            assert!(!(*first).owner.is_empty());

            let mut moved = head;
            assert!(Rc::ptr_eq(
                &window_pane_tree_next(&*first).unwrap(),
                &second_owner
            ));
            assert_eq!(
                window_pane_tree_remove(&mut moved, &mut *first).unwrap().get(),
                first
            );
            assert!((*first).owner.is_empty());
            assert!(window_pane_tree_next(&*first).is_none());
            assert_eq!(
                window_pane_tree_remove(&mut moved, &mut *second).unwrap().get(),
                second
            );

            drop(first_owner);
            drop(second_owner);
            drop(duplicate_owner);
            drop(moved);
            assert!(matches!(
                index_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
        }
    }
}

#[test]
fn border_status_ranges_return_independent_values() {
    unsafe {
        let window = crate::src::shared::window::WindowRef::fixture_with_options();
        window.with_options_mut(|options| {
            let definition = crate::src::options_table::options_table
                .iter()
                .find(|entry| entry.name == Some(c"pane-border-status"))
                .unwrap();
            crate::src::options::options_default(options, definition);
            crate::src::options::options_set_number(options, c"pane-border-status".as_ptr(), 1);
        });
        let pane = Rc::<UnsafeCell<window_pane>>::allocate();
        (*pane.get()).window = Rc::downgrade(&window);
        (*pane.get()).yoff = 3;
        (*pane.get()).border_status_line.ranges.push(Box::new(
            crate::src::shared::style::style_range {
                type_0: crate::src::shared::style::STYLE_RANGE_CONTROL,
                argument: 4,
                string: [0; 16],
                start: 1,
                end: 3,
            },
        ));
        let range = window_pane_status_get_range(&pane, 3, 2).unwrap();
        assert!(window_pane_status_get_range(&pane, 2, 2).is_none());
        assert!(window_pane_status_get_range(&pane, 5, 2).is_none());
        assert!(window_pane_status_get_range(&pane, 3, 3).is_none());
        (*pane.get()).border_status_line.ranges.clear();
        assert_eq!(range.argument, 4);
        assert!(window_pane_status_get_range(&pane, 3, 2).is_none());
        pane.release(c"border status fixture");
        window.release(c"border status fixture");
    }
}

#[test]
fn directional_selection_without_a_source_returns_none() {
    unsafe {
        assert!(window_pane_find_right(None).is_none());
    }
}
