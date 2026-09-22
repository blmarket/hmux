use hmux2::src::shared::tree::OrderedIndex;
use hmux2::src::window::*;
use std::mem::size_of;

#[test]
fn boxed_index_slot_keeps_pointer_layout() {
    assert_eq!(
        size_of::<windows>(),
        size_of::<*mut OrderedIndex<u32, window>>()
    );
}

#[test]
fn window_ids_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = windows { storage: None };
        assert!(windows_minmax(&mut head, -1).is_null());
        let ids = [u32::MAX, 0, 42, 0x8000_0000];
        let mut nodes: Vec<window> = ids.iter().map(|_| std::mem::zeroed()).collect();
        for (node, id) in nodes.iter_mut().zip(ids) {
            node.id = id;
            assert!(windows_insert(&mut head, node).is_null());
        }
        // Entry-only traversal must survive a moved tree head.
        let mut head = Box::new(head);
        let mut probe: window = std::mem::zeroed();
        probe.id = 42;
        let existing = &mut nodes[2] as *mut window;
        assert_eq!(windows_insert(&mut *head, &mut probe), existing);
        assert!(probe.entry.owner.is_null());
        assert!(windows_remove(&mut *head, &mut probe).is_null());
        assert_eq!(windows_find(&mut *head, &mut probe), existing);
        assert_eq!(windows_nfind(&mut *head, &mut probe), existing);
        probe.id = 1;
        assert!(windows_find(&mut *head, &mut probe).is_null());
        assert_eq!(windows_nfind(&mut *head, &mut probe), existing);
        let mut node = windows_minmax(&mut *head, -1);
        for id in [0, 42, 0x8000_0000, u32::MAX] {
            assert_eq!((*node).id, id);
            node = windows_next(node);
        }
        assert!(node.is_null());
        node = windows_minmax(&mut *head, 1);
        for id in [u32::MAX, 0x8000_0000, 42, 0] {
            assert_eq!((*node).id, id);
            node = windows_prev(node);
        }
        assert!(node.is_null());
        node = windows_minmax(&mut *head, -1);
        while !node.is_null() {
            let next = windows_next(node);
            assert_eq!(windows_remove(&mut *head, node), node);
            assert!((*node).entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_none());
        assert!(windows_nfind(&mut *head, &mut probe).is_null());
        assert!(windows_insert(&mut *head, existing).is_null());
        assert_eq!(windows_remove(&mut *head, existing), existing);
        assert!(head.storage.is_none());
    }
}

#[test]
fn global_lookup_preserves_window_identity() {
    unsafe {
        let head = &raw mut windows;
        assert!((*head).storage.is_none());
        let mut window: window = std::mem::zeroed();
        window.id = 123;
        assert!(windows_insert(head, &mut window).is_null());
        assert_eq!(window_find_by_id(123), &mut window as *mut _);
        assert!(window_find_by_id(124).is_null());
        assert_eq!(windows_remove(head, &mut window), &mut window as *mut _);
        assert!(window_find_by_id(123).is_null());
        assert!((*head).storage.is_none());
    }
}
