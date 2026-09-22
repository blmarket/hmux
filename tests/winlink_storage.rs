use hmux2::src::window::*;
use std::ptr::null_mut;

#[test]
fn winlink_indexes_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = winlinks {
            storage: null_mut(),
        };
        assert!(winlinks_minmax(&mut head, -1).is_null());
        let ids = [i32::MAX, 0, 42, i32::MIN];
        let mut nodes: Vec<winlink> = ids.iter().map(|_| std::mem::zeroed()).collect();
        for (node, id) in nodes.iter_mut().zip(ids) {
            node.idx = id;
            assert!(winlinks_insert(&mut head, node).is_null());
        }
        // Entry-only traversal must survive a moved tree head.
        let mut head = Box::new(head);
        let mut probe: winlink = std::mem::zeroed();
        probe.idx = 42;
        let existing = &mut nodes[2] as *mut winlink;
        assert_eq!(winlinks_insert(&mut *head, &mut probe), existing);
        assert!(probe.entry.owner.is_null());
        assert!(winlinks_remove(&mut *head, &mut probe).is_null());
        assert_eq!(winlinks_find(&mut *head, &mut probe), existing);
        assert_eq!(winlinks_nfind(&mut *head, &mut probe), existing);
        probe.idx = 1;
        assert!(winlinks_find(&mut *head, &mut probe).is_null());
        assert_eq!(winlinks_nfind(&mut *head, &mut probe), existing);
        let mut node = winlinks_minmax(&mut *head, -1);
        for id in [i32::MIN, 0, 42, i32::MAX] {
            assert_eq!((*node).idx, id);
            node = winlinks_next(node);
        }
        assert!(node.is_null());
        node = winlinks_minmax(&mut *head, 1);
        for id in [i32::MAX, 42, 0, i32::MIN] {
            assert_eq!((*node).idx, id);
            node = winlinks_prev(node);
        }
        assert!(node.is_null());
        node = winlinks_minmax(&mut *head, -1);
        while !node.is_null() {
            let next = winlinks_next(node);
            assert_eq!(winlinks_remove(&mut *head, node), node);
            assert!((*node).entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_null());
        assert!(winlinks_nfind(&mut *head, &mut probe).is_null());
        assert!(winlinks_insert(&mut *head, existing).is_null());
        assert_eq!(winlinks_remove(&mut *head, existing), existing);
        assert!(head.storage.is_null());
    }
}

#[test]
fn moved_head_and_reindexed_nodes_keep_independent_storage() {
    unsafe {
        let mut head: winlinks = std::mem::zeroed();
        let mut nodes: [winlink; 3] = std::mem::zeroed();
        for (idx, node) in nodes.iter_mut().enumerate() {
            node.idx = idx as i32;
            assert!(winlinks_insert(&mut head, node).is_null());
        }
        // Session synchronization transfers the head and clears the original.
        let mut old = head;
        head.storage = null_mut();
        let mut replacement: winlink = std::mem::zeroed();
        replacement.idx = 1;
        assert!(winlinks_insert(&mut head, &mut replacement).is_null());
        assert_eq!(winlinks_next(&mut nodes[0]), &mut nodes[1] as *mut _);
        assert!(winlinks_next(&mut replacement).is_null());
        // Shuffling removes a node before changing its key and reinserting it.
        assert_eq!(
            winlinks_remove(&mut old, &mut nodes[1]),
            &mut nodes[1] as *mut _
        );
        nodes[1].idx = 5;
        assert!(winlinks_insert(&mut old, &mut nodes[1]).is_null());
        assert_eq!(winlink_find_by_index(&mut old, 5), &mut nodes[1] as *mut _);
        assert!(winlink_find_by_index(&mut old, 1).is_null());
        assert_eq!(winlinks_next(&mut nodes[2]), &mut nodes[1] as *mut _);
        for node in &mut nodes {
            assert_eq!(winlinks_remove(&mut old, node), node as *mut _);
        }
        assert!(old.storage.is_null());
        assert_eq!(winlink_count(&mut head), 1);
        assert_eq!(
            winlinks_remove(&mut head, &mut replacement),
            &mut replacement as *mut _
        );
        assert!(head.storage.is_null());
    }
}
