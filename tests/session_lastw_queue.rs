use hmux2::src::shared::session::session;
use hmux2::src::shared::window::{winlink_stack, winlinks};
use hmux2::src::window::{
    winlink_add, winlink_remove, winlink_stack_clear, winlink_stack_first, winlink_stack_indices,
    winlink_stack_next, winlink_stack_push, winlink_stack_remove, winlinks_reindex,
};
use refbox::BorrowError;

#[test]
fn visit_order_uses_weak_links_and_survives_index_change() {
    unsafe {
        let mut links = winlinks { storage: None };
        let mut stack = winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        };
        let first = winlink_add(&raw mut links, 1);
        let second = winlink_add(&raw mut links, 2);
        let old_second = links.storage.as_ref().unwrap().get(&2).unwrap().downgrade();

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

        winlinks_reindex(&raw mut links, first, 3);
        assert_eq!(winlink_stack_indices(&raw const stack), [3]);
        assert_eq!(winlink_stack_first(&raw const stack, &raw mut links), first);

        winlink_stack_remove(&raw mut stack, first);
        winlink_remove(&raw mut links, first);
        winlink_stack_clear(&raw mut stack);
        assert!(links.storage.is_none());
    }
}

#[test]
fn history_entry_is_removed_before_its_owner() {
    unsafe {
        let mut links = winlinks { storage: None };
        let mut stack = winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        };
        let link = winlink_add(&raw mut links, 7);
        winlink_stack_push(&raw mut stack, link);
        winlink_stack_remove(&raw mut stack, link);
        winlink_remove(&raw mut links, link);

        assert!(winlink_stack_first(&raw const stack, &raw mut links).is_null());
        assert!(winlink_stack_indices(&raw const stack).is_empty());
        winlink_stack_clear(&raw mut stack);
    }
}

#[test]
fn boxed_session_drops_its_winlink_owner() {
    unsafe {
        let mut owner = Box::new(session::empty());
        let link = winlink_add(&raw mut owner.windows, 9);
        let weak = owner
            .windows
            .storage
            .as_ref()
            .unwrap()
            .get(&9)
            .unwrap()
            .downgrade();
        winlink_stack_push(&raw mut owner.lastw, link);
        winlink_stack_clear(&raw mut owner.lastw);
        drop(owner);
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
    }
}

#[test]
#[should_panic(expected = "visited winlink owner was dropped before observer teardown")]
fn expired_history_observer_is_an_invariant_violation() {
    unsafe {
        let mut links = winlinks { storage: None };
        let mut stack = winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        };
        let link = winlink_add(&raw mut links, 1);
        winlink_stack_push(&raw mut stack, link);
        // Deliberately violate teardown order without dereferencing the dead pointer.
        drop(links);
        winlink_stack_indices(&raw const stack);
    }
}
