use hmux2::src::shared::session::session;
use hmux2::src::shared::window::window;
use hmux2::src::window::{
    window_winlinks_first, window_winlinks_next, winlink_add, winlink_remove, winlink_set_window,
    winlinks_reindex,
};
use refbox::BorrowError;

unsafe fn window_indices(w: *mut window) -> Vec<i32> {
    let mut result = Vec::new();
    let mut link = window_winlinks_first(w);
    while !link.is_null() {
        result.push((*link).idx);
        link = window_winlinks_next(w, link);
    }
    result
}

#[test]
fn window_winlinks_keep_association_order_and_stable_session_owned_links() {
    unsafe {
        let mut owner = Box::new(session::empty());
        let first_owner =
            hmux2::src::shared::rc::take(hmux2::src::shared::rc::new(window::default()));
        let second_owner =
            hmux2::src::shared::rc::take(hmux2::src::shared::rc::new(window::default()));
        let first_window = hmux2::src::shared::rc::as_ptr(&first_owner);
        let second_window = hmux2::src::shared::rc::as_ptr(&second_owner);
        (*first_window).entry.owner = None;
        (*second_window).entry.owner = None;
        // These synthetic windows retain one external reference so moving the
        // test links never destroys a window before the assertions finish.

        let first = winlink_add(&raw mut owner.windows, 12);
        let second = winlink_add(&raw mut owner.windows, 3);
        let third = winlink_add(&raw mut owner.windows, 18);
        let initial = [first, second, third];
        for link in initial {
            (*link).session = &mut *owner;
            winlink_set_window(link, first_window);
        }
        assert_eq!(window_indices(first_window), [12, 3, 18]);

        let weak = owner
            .windows
            .storage
            .as_ref()
            .unwrap()
            .try_borrow_mut()
            .unwrap()
            .get(&12)
            .unwrap()
            .downgrade();
        let weak_second = owner
            .windows
            .storage
            .as_ref()
            .unwrap()
            .try_borrow_mut()
            .unwrap()
            .get(&3)
            .unwrap()
            .downgrade();
        let stable_first = first as *const _;
        let mut added = vec![first, second, third];
        for idx in 100..228 {
            let link = winlink_add(&raw mut owner.windows, idx);
            (*link).session = &mut *owner;
            winlink_set_window(link, first_window);
            added.push(link);
        }

        assert_eq!(weak.as_ptr(), stable_first);
        assert_eq!(window_winlinks_first(first_window), first);
        assert_eq!(window_indices(first_window).len(), added.len());
        assert_eq!(window_indices(first_window)[..3], [12, 3, 18]);

        // Reindexing changes the session lookup key without changing either
        // the handle address or its position in the window's association order.
        winlinks_reindex(&raw mut owner.windows, second, 30);
        assert_eq!(weak_second.as_ptr(), second as *const _);
        assert_eq!(window_indices(first_window)[..3], [12, 30, 18]);

        // Moving a link removes it from the old owner's ordered handles. Moving
        // it back appends it, matching the former TAILQ insertion behavior.
        winlink_set_window(second, second_window);
        assert_eq!(window_indices(first_window)[..2], [12, 18]);
        assert_eq!(window_indices(second_window), [30]);
        winlink_set_window(second, first_window);
        assert_eq!(window_indices(first_window).last(), Some(&30));

        for link in added {
            winlink_remove(&raw mut owner.windows, link);
        }
        assert!(window_winlinks_first(first_window).is_null());
        assert!(window_winlinks_first(second_window).is_null());
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
    }
}

#[test]
fn close_notification_can_retain_the_last_window_reference() {
    use hmux2::src::events::{events_add_sink, events_remove_sink};
    use hmux2::src::shared::rc;
    use hmux2::src::window::{window_add_ref, window_remove_ref};
    use std::{cell::Cell, rc::Rc};

    unsafe {
        let w = rc::new(window::default());
        let weak = rc::downgrade(w);
        let notified = Rc::new(Cell::new(false));
        let observed = notified.clone();
        let sink = events_add_sink(
            c"window-closed",
            Rc::new(move |_, _| {
                observed.set(true);
                window_add_ref(w, c"close callback".as_ptr());
            }),
        );
        window_remove_ref(w, c"original owner".as_ptr());
        assert!(notified.get());
        assert!(weak.upgrade().is_some());
        assert_eq!(rc::strong_count(w), 1);
        events_remove_sink(sink);
        window_remove_ref(w, c"callback owner".as_ptr());
        assert!(weak.upgrade().is_none());
    }
}
