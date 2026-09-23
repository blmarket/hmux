use hmux2::src::shared::session::session;
use hmux2::src::shared::window::window;
use hmux2::src::window::{
    winlink_add, winlink_remove, winlink_set_window, winlinks_reindex, window_winlinks_first,
    window_winlinks_next,
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
        let mut owner = Box::new(std::mem::zeroed::<session>());
        let mut first_window = Box::new(std::mem::zeroed::<window>());
        let mut second_window = Box::new(std::mem::zeroed::<window>());
        // These synthetic windows retain one external reference so moving the
        // test links never runs the global window destruction path.
        first_window.references = 1;
        second_window.references = 1;

        let first = winlink_add(&raw mut owner.windows, 12);
        let second = winlink_add(&raw mut owner.windows, 3);
        let third = winlink_add(&raw mut owner.windows, 18);
        let initial = [first, second, third];
        for link in initial {
            (*link).session = &mut *owner;
            winlink_set_window(link, &mut *first_window);
        }
        assert_eq!(window_indices(&mut *first_window), [12, 3, 18]);

        let weak = owner
            .windows
            .storage
            .as_ref()
            .unwrap()
            .get(&12)
            .unwrap()
            .downgrade();
        let weak_second = owner
            .windows
            .storage
            .as_ref()
            .unwrap()
            .get(&3)
            .unwrap()
            .downgrade();
        let stable_first = first as *const _;
        let mut added = vec![first, second, third];
        for idx in 100..228 {
            let link = winlink_add(&raw mut owner.windows, idx);
            (*link).session = &mut *owner;
            winlink_set_window(link, &mut *first_window);
            added.push(link);
        }

        assert_eq!(weak.as_ptr(), stable_first);
        assert_eq!(window_winlinks_first(&mut *first_window), first);
        assert_eq!(window_indices(&mut *first_window).len(), added.len());
        assert_eq!(window_indices(&mut *first_window)[..3], [12, 3, 18]);

        // Reindexing changes the session lookup key without changing either
        // the handle address or its position in the window's association order.
        winlinks_reindex(&raw mut owner.windows, second, 30);
        assert_eq!(weak_second.as_ptr(), second as *const _);
        assert_eq!(window_indices(&mut *first_window)[..3], [12, 30, 18]);

        // Moving a link removes it from the old owner's ordered handles. Moving
        // it back appends it, matching the former TAILQ insertion behavior.
        winlink_set_window(second, &mut *second_window);
        assert_eq!(window_indices(&mut *first_window)[..2], [12, 18]);
        assert_eq!(window_indices(&mut *second_window), [30]);
        winlink_set_window(second, &mut *first_window);
        assert_eq!(window_indices(&mut *first_window).last(), Some(&30));

        for link in added {
            winlink_remove(&raw mut owner.windows, link);
        }
        assert!(window_winlinks_first(&mut *first_window).is_null());
        assert!(window_winlinks_first(&mut *second_window).is_null());
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
    }
}
