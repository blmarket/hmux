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
    let owners: Vec<Rc<UnsafeCell<window_pane>>> = (0..4).map(|_| pane_owner()).collect();
    let panes: Vec<*mut window_pane> = owners.iter().map(rc::as_ptr).collect();

    unsafe {
        let mut order = window_panes::default();
        for owner in &owners {
            order.push_back(Rc::downgrade(owner));
        }
        assert_eq!(order.first().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[0]);
        assert_eq!(order.next(panes[0]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[1]);
        assert_eq!(order.previous(panes[3]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[2]);

        order.insert_before(panes[2], Rc::downgrade(&owners[3]));
        assert_eq!(order.first().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[0]);
        assert_eq!(order.next(panes[0]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[1]);
        order.swap_ptrs(panes[1], panes[3]);
        assert_eq!(order.next(panes[0]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[3]);
        assert!(order.remove_ptr(panes[3]));
        assert_eq!(order.next(panes[0]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[1]);

        let mut history = window_pane_history::default();
        history.push_front(Rc::downgrade(&owners[1]));
        history.push_front(Rc::downgrade(&owners[2]));
        history.push_front(Rc::downgrade(&owners[1]));
        assert_eq!(history.first().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[1]);
        assert_eq!(history.next(panes[1]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), panes[2]);
        assert!(history.remove_ptr(panes[2]));
        assert_eq!(history.next(panes[1]).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), std::ptr::null_mut());
    }

    let weak = Rc::downgrade(&owners[0]);
    drop(owners);
    assert!(weak.upgrade().is_none());
}

#[test]
fn global_lookup_preserves_pane_identity() {
    unsafe {
        let head = &raw mut all_window_panes;
        assert!((*head).storage.is_none());
        let owner = window_pane::new();
        let pane = &mut *rc::as_ptr(&owner);
        pane.id = 123;
        assert!(window_pane_tree_insert(head, owner.clone()).is_null());
        assert_eq!(window_pane_find_by_id(123), pane as *mut _);
        assert!(window_pane_find_by_id(124).is_null());
        assert_eq!(
            rc::as_ptr(&window_pane_tree_remove(head, pane).unwrap()),
            pane as *mut _
        );
        assert!(window_pane_find_by_id(123).is_null());
        assert!((*head).storage.is_none());
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
        assert!(order.remove_ptr(retained.get()));
        drop(owner);
        assert!(weak.upgrade().is_some());
        drop(retained);
        assert!(weak.upgrade().is_none());
    }
}
