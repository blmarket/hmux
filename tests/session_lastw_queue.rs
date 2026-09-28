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
        let mut stack = winlink_stack { storage: None };
        let first = winlink_add(&raw mut links, 1);
        let second = winlink_add(&raw mut links, 2);
        let old_second = links
            .storage
            .as_ref()
            .unwrap()
            .try_borrow_mut()
            .unwrap()
            .get(&2)
            .unwrap()
            .downgrade();

        winlink_stack_push(&raw mut stack, (first).clone());
        winlink_stack_push(&raw mut stack, (second).clone());
        assert_eq!(winlink_stack_indices(&stack), [2, 1]);
        assert_eq!(winlink_stack_first(&stack), second);
        assert_eq!(winlink_stack_next(&stack, (second).clone()), first);

        winlink_stack_remove(&raw mut stack, (second).clone());
        winlink_remove(&raw mut links, (second).clone());
        assert_eq!(
            old_second.try_borrow_mut().err(),
            Some(BorrowError::Dropped)
        );
        assert_eq!(winlink_stack_first(&stack), first);

        winlinks_reindex(&raw mut links, (first).clone(), 3);
        assert_eq!(winlink_stack_indices(&stack), [3]);
        assert_eq!(winlink_stack_first(&stack), first);

        winlink_stack_remove(&raw mut stack, (first).clone());
        winlink_remove(&raw mut links, (first).clone());
        winlink_stack_clear(&mut stack);
        assert!(links.storage.is_none());
    }
}

#[test]
fn history_entry_is_removed_before_its_owner() {
    unsafe {
        let mut links = winlinks { storage: None };
        let mut stack = winlink_stack { storage: None };
        let link = winlink_add(&raw mut links, 7);
        winlink_stack_push(&raw mut stack, link.clone());
        winlink_stack_remove(&raw mut stack, link.clone());
        winlink_remove(&raw mut links, link.clone());

        assert!(!winlink_stack_first(&stack).is_alive());
        assert!(winlink_stack_indices(&stack).is_empty());
        winlink_stack_clear(&mut stack);
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
            .try_borrow_mut()
            .unwrap()
            .get(&9)
            .unwrap()
            .downgrade();
        winlink_stack_push(&raw mut owner.lastw, link.clone());
        winlink_stack_clear(&mut owner.lastw);
        drop(owner);
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
    }
}

#[test]
#[should_panic(expected = "visited winlink owner was dropped before observer teardown")]
fn expired_history_observer_is_an_invariant_violation() {
    unsafe {
        let mut links = winlinks { storage: None };
        let mut stack = winlink_stack { storage: None };
        let link = winlink_add(&raw mut links, 1);
        winlink_stack_push(&raw mut stack, link.clone());
        // Deliberately violate teardown order without dereferencing the dead pointer.
        drop(links);
        winlink_stack_indices(&stack);
    }
}
