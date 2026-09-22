use hmux2::src::shared::tree::OrderedIndex;
use hmux2::src::shared::window::{winlink, winlink_stack, winlinks};
use hmux2::src::window::{
    winlink_add, winlink_remove, winlink_stack_clear, winlink_stack_first, winlink_stack_indices,
    winlink_stack_next, winlink_stack_push, winlink_stack_remove, winlinks_insert, winlinks_remove,
};
use refbox::BorrowError;

#[test]
fn visit_order_uses_weak_links_and_survives_index_change() {
    unsafe {
        let mut links = winlinks {
            storage: std::ptr::null_mut(),
        };
        let mut stack = winlink_stack {
            storage: std::ptr::null_mut(),
            reserved: std::ptr::null_mut(),
        };
        let first = winlink_add(&raw mut links, 1);
        let second = winlink_add(&raw mut links, 2);
        let old_second = OrderedIndex::<i32, winlink>::downgrade(links.storage, second).unwrap();

        winlink_stack_push(&raw mut stack, first);
        winlink_stack_push(&raw mut stack, second);
        assert_eq!(winlink_stack_indices(&raw const stack), [2, 1]);
        assert_eq!(
            winlink_stack_first(&raw const stack, &raw mut links),
            second
        );
        assert_eq!(
            winlink_stack_next(&raw const stack, &raw mut links, second),
            first
        );

        winlink_stack_remove(&raw mut stack, second);
        winlink_remove(&raw mut links, second);
        assert_eq!(
            old_second.try_borrow_mut().err(),
            Some(BorrowError::Dropped)
        );
        assert_eq!(winlink_stack_first(&raw const stack, &raw mut links), first);

        winlinks_remove(&raw mut links, first);
        (*first).idx = 3;
        winlinks_insert(&raw mut links, first);
        assert_eq!(winlink_stack_indices(&raw const stack), [3]);
        assert_eq!(winlink_stack_first(&raw const stack, &raw mut links), first);

        winlink_stack_remove(&raw mut stack, first);
        winlink_remove(&raw mut links, first);
        winlink_stack_clear(&raw mut stack);
        assert!(links.storage.is_null());
    }
}

#[test]
fn stale_history_entry_cannot_access_removed_link() {
    unsafe {
        let mut links = winlinks {
            storage: std::ptr::null_mut(),
        };
        let mut stack = winlink_stack {
            storage: std::ptr::null_mut(),
            reserved: std::ptr::null_mut(),
        };
        let link = winlink_add(&raw mut links, 7);
        winlink_stack_push(&raw mut stack, link);
        winlink_remove(&raw mut links, link);

        assert!(winlink_stack_first(&raw const stack, &raw mut links).is_null());
        assert!(winlink_stack_indices(&raw const stack).is_empty());
        winlink_stack_clear(&raw mut stack);
    }
}
