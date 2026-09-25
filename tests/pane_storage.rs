use hmux2::src::shared::pane::window_pane_tree;
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
fn pane_ids_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = window_pane_tree { storage: None };
        assert!(window_pane_tree_minmax(&head, -1).is_null());
        let ids = [u32::MAX, 0, 42, 0x8000_0000];
        let mut nodes: Vec<window_pane> = ids.iter().map(|_| window_pane::empty()).collect();
        for (node, id) in nodes.iter_mut().zip(ids) {
            node.id = id;
            assert!(window_pane_tree_insert(&mut head, node).is_null());
        }
        // Entry-only traversal must survive a moved tree head.
        let mut head = Box::new(head);
        let mut probe = window_pane::empty();
        probe.id = 42;
        let existing = &mut nodes[2] as *mut window_pane;
        assert_eq!(window_pane_tree_insert(&mut *head, &mut probe), existing);
        assert!(probe.tree_entry.owner.is_null());
        assert!(window_pane_tree_remove(&mut *head, &mut probe).is_null());
        assert_eq!(window_pane_tree_find(&*head, &probe), existing);
        assert_eq!(window_pane_tree_nfind(&*head, &probe), existing);
        probe.id = 1;
        assert!(window_pane_tree_find(&*head, &probe).is_null());
        assert_eq!(window_pane_tree_nfind(&*head, &probe), existing);
        let mut node = window_pane_tree_minmax(&*head, -1);
        for id in [0, 42, 0x8000_0000, u32::MAX] {
            assert_eq!((*node).id, id);
            node = window_pane_tree_next(&*node);
        }
        assert!(node.is_null());
        node = window_pane_tree_minmax(&*head, 1);
        for id in [u32::MAX, 0x8000_0000, 42, 0] {
            assert_eq!((*node).id, id);
            node = window_pane_tree_prev(&*node);
        }
        assert!(node.is_null());
        node = window_pane_tree_minmax(&*head, -1);
        while !node.is_null() {
            let next = window_pane_tree_next(&*node);
            assert_eq!(window_pane_tree_remove(&mut *head, node), node);
            assert!((*node).tree_entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_none());
        assert!(window_pane_tree_nfind(&*head, &probe).is_null());
        assert!(window_pane_tree_insert(&mut *head, existing).is_null());
        assert_eq!(window_pane_tree_remove(&mut *head, existing), existing);
        assert!(head.storage.is_none());
    }
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
