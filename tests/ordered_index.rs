use hmux2::src::shared::tree::OrderedIndex;
use std::ptr::null_mut;

#[test]
fn ordering_duplicates_bounds_and_empty_storage_lifetime() {
    unsafe {
        let mut index = null_mut();
        let mut nodes = [0u32; 128];
        for i in (0..128).rev() {
            let node = &mut nodes[i] as *mut u32;
            assert!(OrderedIndex::insert(&mut index, i * 2, node).is_null());
        }
        let stable = index;
        let mut duplicate = 999;
        assert_eq!(
            OrderedIndex::insert(&mut index, 4, &mut duplicate),
            &mut nodes[2] as *mut _
        );
        assert_eq!(index, stable);
        assert_eq!(OrderedIndex::nfind(index, &3), &mut nodes[2] as *mut _);
        assert!(OrderedIndex::nfind(index, &255).is_null());
        assert!(OrderedIndex::neighbor(index, &0, false).is_null());
        assert!(OrderedIndex::neighbor(index, &254, true).is_null());
        assert_eq!(OrderedIndex::edge(index, false), &mut nodes[127] as *mut _);
        // Removing a different record with the same key must not corrupt the index.
        assert!(OrderedIndex::remove(&mut index, &4, &mut duplicate).is_null());
        for i in 0..128 {
            let node = &mut nodes[i] as *mut _;
            assert_eq!(OrderedIndex::edge(index, true), node);
            if i < 127 {
                assert_eq!(
                    OrderedIndex::neighbor(index, &(i * 2), true),
                    &mut nodes[i + 1] as *mut _
                );
            }
            assert_eq!(OrderedIndex::remove(&mut index, &(i * 2), node), node);
        }
        assert!(index.is_null());
        assert!(OrderedIndex::edge(index, true).is_null());
        assert!(OrderedIndex::insert(&mut index, 1, &mut duplicate).is_null());
        OrderedIndex::remove(&mut index, &1, &mut duplicate);
        assert!(index.is_null());
    }
}
