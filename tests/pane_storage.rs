use hmux2::src::shared::pane::{window_pane, window_pane_history, window_panes};
use hmux2::src::window::*;
use refbox::{BorrowError, RefBox};

#[test]
fn removing_from_another_history_preserves_membership_and_cleanup() {
    let owner = RefBox::new(window_pane::empty());
    let pane = owner.as_ptr() as *mut window_pane;
    let mut source = window_pane_history::default();
    let mut destination = window_pane_history::default();
    source.push_front(owner.downgrade());
    unsafe {
        (*pane).flags |= hmux2::src::shared::pane::PANE_VISITED;
        window_pane_stack_remove(&mut destination, pane);
        assert_ne!((*pane).flags & hmux2::src::shared::pane::PANE_VISITED, 0);
        window_pane_stack_remove(&mut source, pane);
        assert!(source.is_empty());
        assert_eq!((*pane).flags & hmux2::src::shared::pane::PANE_VISITED, 0);

        // Cleanup must also make progress if an entry has lost its flag.
        source.push_front(owner.downgrade());
        window_pane_stack_remove(&mut source, pane);
        assert!(source.is_empty());
    }
}

#[test]
fn pane_order_and_visit_history_preserve_stable_weak_entries() {
    let owners: Vec<RefBox<window_pane>> =
        (0..4).map(|_| RefBox::new(window_pane::empty())).collect();
    let panes: Vec<*mut window_pane> = owners
        .iter()
        .map(|owner| owner.as_ptr() as *mut window_pane)
        .collect();

    unsafe {
        let mut order = window_panes::default();
        for owner in &owners {
            order.push_back(owner.downgrade());
        }
        assert_eq!(order.first(), panes[0]);
        assert_eq!(order.next(panes[0]), panes[1]);
        assert_eq!(order.previous(panes[3]), panes[2]);

        order.insert_before(panes[2], owners[3].downgrade());
        assert_eq!(order.first(), panes[0]);
        assert_eq!(order.next(panes[0]), panes[1]);
        order.swap_ptrs(panes[1], panes[3]);
        assert_eq!(order.next(panes[0]), panes[3]);
        assert!(order.remove_ptr(panes[3]));
        assert_eq!(order.next(panes[0]), panes[1]);

        let mut history = window_pane_history::default();
        history.push_front(owners[1].downgrade());
        history.push_front(owners[2].downgrade());
        history.push_front(owners[1].downgrade());
        assert_eq!(history.first(), panes[1]);
        assert_eq!(history.next(panes[1]), panes[2]);
        assert!(history.remove_ptr(panes[2]));
        assert_eq!(history.next(panes[1]), std::ptr::null_mut());
    }

    let weak = owners[0].downgrade();
    drop(owners);
    assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
}

#[test]
fn global_lookup_preserves_pane_identity() {
    unsafe {
        let head = &raw mut all_window_panes;
        assert!((*head).storage.is_none());
        let mut pane = window_pane::empty();
        pane.id = 123;
        assert!(window_pane_tree_insert(head, &mut pane).is_null());
        assert_eq!(window_pane_find_by_id(123), &mut pane as *mut _);
        assert!(window_pane_find_by_id(124).is_null());
        assert_eq!(
            window_pane_tree_remove(head, &mut pane),
            &mut pane as *mut _
        );
        assert!(window_pane_find_by_id(123).is_null());
        assert!((*head).storage.is_none());
    }
}
