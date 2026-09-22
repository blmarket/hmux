use hmux2::src::window::*;
use std::ptr::null_mut;

#[test]
fn pane_ids_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = window_pane_tree {
            storage: null_mut(),
        };
        assert!(window_pane_tree_minmax(&mut head, -1).is_null());
        let ids = [u32::MAX, 0, 42, 0x8000_0000];
        let mut nodes: Vec<window_pane> = ids.iter().map(|_| std::mem::zeroed()).collect();
        for (node, id) in nodes.iter_mut().zip(ids) {
            node.id = id;
            assert!(window_pane_tree_insert(&mut head, node).is_null());
        }
        // Entry-only traversal must survive a moved tree head.
        let mut head = Box::new(head);
        let mut probe: window_pane = std::mem::zeroed();
        probe.id = 42;
        let existing = &mut nodes[2] as *mut window_pane;
        assert_eq!(window_pane_tree_insert(&mut *head, &mut probe), existing);
        assert!(probe.tree_entry.owner.is_null());
        assert!(window_pane_tree_remove(&mut *head, &mut probe).is_null());
        assert_eq!(window_pane_tree_find(&mut *head, &mut probe), existing);
        assert_eq!(window_pane_tree_nfind(&mut *head, &mut probe), existing);
        probe.id = 1;
        assert!(window_pane_tree_find(&mut *head, &mut probe).is_null());
        assert_eq!(window_pane_tree_nfind(&mut *head, &mut probe), existing);
        let mut node = window_pane_tree_minmax(&mut *head, -1);
        for id in [0, 42, 0x8000_0000, u32::MAX] {
            assert_eq!((*node).id, id);
            node = window_pane_tree_next(node);
        }
        assert!(node.is_null());
        node = window_pane_tree_minmax(&mut *head, 1);
        for id in [u32::MAX, 0x8000_0000, 42, 0] {
            assert_eq!((*node).id, id);
            node = window_pane_tree_prev(node);
        }
        assert!(node.is_null());
        node = window_pane_tree_minmax(&mut *head, -1);
        while !node.is_null() {
            let next = window_pane_tree_next(node);
            assert_eq!(window_pane_tree_remove(&mut *head, node), node);
            assert!((*node).tree_entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_null());
        assert!(window_pane_tree_nfind(&mut *head, &mut probe).is_null());
        assert!(window_pane_tree_insert(&mut *head, existing).is_null());
        assert_eq!(window_pane_tree_remove(&mut *head, existing), existing);
        assert!(head.storage.is_null());
    }
}

#[test]
fn global_lookup_preserves_pane_identity() {
    unsafe {
        let head = &raw mut all_window_panes;
        assert!((*head).storage.is_null());
        let mut pane: window_pane = std::mem::zeroed();
        pane.id = 123;
        assert!(window_pane_tree_insert(head, &mut pane).is_null());
        assert_eq!(window_pane_find_by_id(123), &mut pane as *mut _);
        assert!(window_pane_find_by_id(124).is_null());
        assert_eq!(
            window_pane_tree_remove(head, &mut pane),
            &mut pane as *mut _
        );
        assert!(window_pane_find_by_id(123).is_null());
        assert!((*head).storage.is_null());
    }
}
