use hmux2::src::shared::session::session;
use hmux2::src::window::*;
use refbox::BorrowError;

#[test]
fn owned_winlinks_preserve_duplicates_bounds_and_traversal() {
    unsafe {
        let mut head = winlinks { storage: None };
        assert!(winlinks_minmax(&head, -1).is_null());
        let ids = [i32::MAX, 0, 42, 7];
        let nodes: Vec<_> = ids.iter().map(|&id| winlink_add(&mut head, id)).collect();
        // Entry-only traversal must survive a moved tree head.
        let mut head = Box::new(head);
        let existing = nodes[2];
        let weak = head
            .storage
            .as_ref()
            .unwrap()
            .try_borrow_mut()
            .unwrap()
            .get(&42)
            .unwrap()
            .downgrade();
        assert!(winlink_add(&mut *head, 42).is_null());
        assert_eq!(weak.as_ptr(), existing as *const winlink);
        assert!(weak.is_alive());
        let mut probe: winlink = std::mem::zeroed();
        probe.idx = 42;
        probe.entry.owner = None;
        assert_eq!(winlinks_find(&*head, &probe), existing);
        assert_eq!(winlinks_nfind(&*head, &probe), existing);
        probe.idx = 8;
        assert!(winlinks_find(&*head, &probe).is_null());
        assert_eq!(winlinks_nfind(&*head, &probe), existing);
        let mut node = winlinks_minmax(&*head, -1);
        for id in [0, 7, 42, i32::MAX] {
            assert_eq!((*node).idx, id);
            node = winlinks_next(&*node);
        }
        assert!(node.is_null());
        node = winlinks_minmax(&*head, 1);
        for id in [i32::MAX, 42, 7, 0] {
            assert_eq!((*node).idx, id);
            node = winlinks_prev(&*node);
        }
        assert!(node.is_null());
        node = winlinks_minmax(&*head, -1);
        while !node.is_null() {
            let next = winlinks_next(&*node);
            winlink_remove(&mut *head, node);
            node = next;
        }
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
        assert!(head.storage.is_none());
        assert!(winlinks_nfind(&*head, &probe).is_null());
        let replacement = winlink_add(&mut *head, 42);
        assert!(!replacement.is_null());
        winlink_remove(&mut *head, replacement);
        assert!(head.storage.is_none());
    }
}

#[test]
fn moved_map_and_reindexed_owner_keep_identity_through_growth() {
    unsafe {
        let mut head = winlinks { storage: None };
        let first = winlink_add(&mut head, 0);
        let weak = head
            .storage
            .as_ref()
            .unwrap()
            .try_borrow_mut()
            .unwrap()
            .get(&0)
            .unwrap()
            .downgrade();
        let mut old = std::mem::replace(&mut head, winlinks { storage: None });
        let replacement = winlink_add(&mut head, 0);
        winlinks_reindex(&mut old, first, 5);
        assert_eq!(winlink_find_by_index(&mut old, 5), first);
        assert!(winlink_find_by_index(&mut old, 0).is_null());
        assert_eq!(weak.try_access_mut(|link| link.idx).unwrap(), 5);
        for idx in 6..134 {
            assert!(!winlink_add(&mut old, idx).is_null());
        }
        assert_eq!(weak.as_ptr(), first as *const winlink);
        assert_eq!(winlinks_next(&*first), winlink_find_by_index(&mut old, 6));
        assert!(winlinks_next(&*replacement).is_null());
        while old.storage.is_some() {
            let node = winlinks_minmax(&old, -1);
            winlink_remove(&mut old, node);
        }
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
        assert_eq!(winlink_count(&mut head), 1);
        winlink_remove(&mut head, replacement);
        assert!(head.storage.is_none());
    }
}

#[test]
fn shuffle_moves_owners_without_losing_history_and_removal_clears_observers() {
    unsafe {
        let mut session = Box::new(session::empty());
        let mut nodes = Vec::new();
        for idx in 1..=3 {
            let node = winlink_add(&raw mut session.windows, idx);
            (*node).session = &mut *session;
            nodes.push(node);
        }
        winlink_stack_push(&raw mut session.lastw, nodes[0]);
        winlink_stack_push(&raw mut session.lastw, nodes[1]);
        let weak = session
            .windows
            .storage
            .as_ref()
            .unwrap()
            .try_borrow_mut()
            .unwrap()
            .get(&1)
            .unwrap()
            .downgrade();
        assert_eq!(winlink_shuffle_up(&mut *session, nodes[0], 1), 1);
        assert!(winlink_find_by_index(&raw mut session.windows, 1).is_null());
        for (node, idx) in nodes.iter().zip(2..=4) {
            assert_eq!(winlink_find_by_index(&raw mut session.windows, idx), *node);
        }
        assert_eq!(winlink_stack_indices(&session.lastw), [3, 2]);
        assert_eq!(weak.as_ptr(), nodes[0] as *const winlink);
        for node in nodes {
            winlink_remove(&raw mut session.windows, node);
        }
        assert!(winlink_stack_indices(&session.lastw).is_empty());
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
        assert!(session.windows.storage.is_none());
    }
}
