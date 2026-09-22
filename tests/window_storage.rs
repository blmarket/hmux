use hmux2::src::window as w;
use std::ptr::null_mut;

#[test]
fn global_ids_and_moved_winlink_heads_keep_stable_iteration() {
    unsafe {
        let ids = [u32::MAX, 0, 42];
        let mut windows: Vec<w::window> = ids.iter().map(|_| std::mem::zeroed()).collect();
        let mut panes: Vec<w::window_pane> = ids.iter().map(|_| std::mem::zeroed()).collect();
        for (i, id) in ids.into_iter().enumerate() {
            windows[i].id = id;
            panes[i].id = id;
            w::windows_insert(&raw mut w::windows, &mut windows[i]);
            w::window_pane_tree_insert(&raw mut w::all_window_panes, &mut panes[i]);
            assert_eq!(w::window_find_by_id(id), &mut windows[i] as *mut _);
            assert_eq!(w::window_pane_find_by_id(id), &mut panes[i] as *mut _);
        }
        assert_eq!((*w::windows_minmax(&raw mut w::windows, -1)).id, 0);
        assert_eq!(
            (*w::window_pane_tree_minmax(&raw mut w::all_window_panes, 1)).id,
            u32::MAX
        );
        for i in 0..ids.len() {
            w::windows_remove(&raw mut w::windows, &mut windows[i]);
            w::window_pane_tree_remove(&raw mut w::all_window_panes, &mut panes[i]);
        }
        assert!(w::windows.storage.is_null());
        assert!(w::all_window_panes.storage.is_null());

        let mut links = w::winlinks {
            storage: null_mut(),
        };
        let high = w::winlink_add(&mut links, i32::MAX);
        let first = w::winlink_add(&mut links, 0);
        let middle = w::winlink_add(&mut links, 9);
        assert!(w::winlink_add(&mut links, 9).is_null());
        // Session synchronization/renumbering copies the head and clears the old slot.
        let mut moved = links;
        links.storage = null_mut();
        let new_first = w::winlink_add(&mut links, 0);
        assert_eq!(w::winlink_next(first), middle);
        assert_eq!(w::winlink_previous(high), middle);
        assert_eq!(w::winlink_find_by_index(&mut moved, 9), middle);
        let automatic = w::winlink_add(&mut moved, -1);
        assert_eq!((*automatic).idx, 1);
        assert_eq!(w::winlink_count(&mut moved), 4);
        let mut node = w::winlinks_minmax(&mut moved, -1);
        for expected in [0, 1, 9, i32::MAX] {
            assert_eq!((*node).idx, expected);
            let next = w::winlink_next(node);
            w::winlink_remove(&mut moved, node);
            node = next;
        }
        assert!(node.is_null() && moved.storage.is_null());
        assert_eq!(w::winlink_find_by_index(&mut links, 0), new_first);
        w::winlink_remove(&mut links, new_first);
        assert!(links.storage.is_null());
    }
}
