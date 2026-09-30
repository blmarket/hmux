use hmux2::src::window::Window as _;
use hmux2::src::session::Session as _;
use hmux2::src::session::SessionIndex as _;
use hmux2::src::shared::session::session;
use hmux2::src::shared::window::{window, WindowRef};
use hmux2::src::window::{
    winlink_add, winlink_remove, winlink_set_window, winlinks_reindex, Window,
};
use refbox::BorrowError;

unsafe fn window_indices(w: &WindowRef) -> Vec<i32> {
    let mut result = Vec::new();
    let mut link = w.next_winlink(None);
    while link.is_alive() {
        result.push(link.get_unchecked().idx);
        link = w.next_winlink(Some(link.clone()));
    }
    result
}

#[test]
fn window_winlinks_keep_association_order_and_stable_session_owned_links() {
    unsafe {
        let session_owner = hmux2::src::shared::session::SessionRef::allocate();
        let mut links = Default::default();
        let first_owner = hmux2::src::shared::window::WindowRef::empty();
        let second_owner = hmux2::src::shared::window::WindowRef::empty();
        // These synthetic windows retain one external reference so moving the
        // test links never destroys a window before the assertions finish.

        let first = winlink_add(&raw mut links, 12);
        let second = winlink_add(&raw mut links, 3);
        let third = winlink_add(&raw mut links, 18);
        let initial = [first.clone(), second.clone(), third.clone()];
        for mut link in initial {
            link.get_mut_unchecked().session = std::rc::Rc::downgrade(&session_owner);
            winlink_set_window(link.clone(), &first_owner);
        }
        assert_eq!(window_indices(&first_owner), [12, 3, 18]);

        let weak = links
            .try_borrow_mut()
            .unwrap()
            .get(&12)
            .unwrap()
            .downgrade();
        let weak_second = links
            .try_borrow_mut()
            .unwrap()
            .get(&3)
            .unwrap()
            .downgrade();
        let stable_first = first.clone();
        let mut added = vec![first.clone(), second.clone(), third.clone()];
        for idx in 100..228 {
            let mut link = winlink_add(&raw mut links, idx);
            link.get_mut_unchecked().session = std::rc::Rc::downgrade(&session_owner);
            winlink_set_window(link.clone(), &first_owner);
            added.push(link);
        }

        assert_eq!(weak, stable_first);
        assert_eq!(first_owner.next_winlink(None), first);
        assert_eq!(window_indices(&first_owner).len(), added.len());
        assert_eq!(window_indices(&first_owner)[..3], [12, 3, 18]);

        // Reindexing changes the session lookup key without changing either
        // the handle address or its position in the window's association order.
        winlinks_reindex(&raw mut links, (second).clone(), 30);
        assert_eq!(weak_second, second);
        assert_eq!(window_indices(&first_owner)[..3], [12, 30, 18]);

        // Moving a link removes it from the old owner's ordered handles. Moving
        // it back appends it, matching the former TAILQ insertion behavior.
        winlink_set_window((second).clone(), &second_owner);
        assert_eq!(window_indices(&first_owner)[..2], [12, 18]);
        assert_eq!(window_indices(&second_owner), [30]);
        assert!(!first_owner.next_winlink(Some(second.clone())).is_alive());
        assert_eq!(first_owner.next_winlink(Some(first.clone())), third);
        winlink_set_window((second).clone(), &first_owner);
        assert_eq!(window_indices(&first_owner).last(), Some(&30));

        for link in added {
            winlink_remove(&raw mut links, link.clone());
        }
        assert!(!first_owner.next_winlink(None).is_alive());
        assert!(!second_owner.next_winlink(None).is_alive());
        assert!(window_indices(&first_owner).is_empty());
        assert!(window_indices(&second_owner).is_empty());
        assert_eq!(weak.try_borrow_mut().err(), Some(BorrowError::Dropped));
        first_owner.release(c"test owner");
        second_owner.release(c"test owner");
    }
}

#[test]
fn close_notification_can_retain_the_last_window_reference() {
    use hmux2::src::events::{events_add_sink, events_remove_sink};

    use std::{cell::Cell, rc::Rc};

    unsafe {
        let w_owner = hmux2::src::shared::window::WindowRef::empty();
        let weak = Rc::downgrade(&w_owner);
        let notified = Rc::new(Cell::new(false));
        let observed = notified.clone();
        let retained = Rc::new(std::cell::RefCell::new(None));
        let retained_callback = retained.clone();
        let callback_window = weak.clone();
        let sink = events_add_sink(
            c"window-closed",
            Rc::new(move |_, _| {
                observed.set(true);
                *retained_callback.borrow_mut() = Some((&callback_window
                        .upgrade()
                        .expect("window lives through notification")).retain(c"close callback"));
            }),
        );
        w_owner.release(c"original owner");
        assert!(notified.get());
        assert!(weak.upgrade().is_some());
        assert_eq!(weak.strong_count(), 1);
        events_remove_sink(sink);
        let reference = retained.borrow_mut().take().unwrap();
        reference.release(c"callback owner");
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn removing_link_keeps_its_window_visible_during_close_notification() {
    use hmux2::src::events::{events_add_sink, events_remove_sink};

    use std::{cell::Cell, rc::Rc};

    unsafe {
        let mut links = Default::default();
        let link = winlink_add(&mut links, 0);
        let w_owner = hmux2::src::shared::window::WindowRef::empty();
        let weak = Rc::downgrade(&w_owner);
        winlink_set_window(link.clone(), &w_owner);
        w_owner.release(c"creator");
        assert_eq!(weak.strong_count(), 1);
        let count = Rc::new(Cell::new(0));
        let calls = count.clone();
        let retained = Rc::new(std::cell::RefCell::new(None));
        let retained_callback = retained.clone();
        let callback_window = weak.clone();
        let callback_link = link.clone();
        let sink = events_add_sink(
            c"window-closed",
            Rc::new(move |_, _| {
                assert!(
                    Rc::downgrade(callback_link.get_unchecked().window_handle().unwrap())
                        .ptr_eq(&callback_window)
                );
                calls.set(calls.get() + 1);
                *retained_callback.borrow_mut() = Some((&callback_window
                        .upgrade()
                        .expect("window lives through notification")).retain(c"close observer"));
            }),
        );
        winlink_remove(&mut links, link.clone());
        assert_eq!(count.get(), 1);
        assert!(hmux2::src::window::winlinks_is_empty(&links));
        assert_eq!(weak.strong_count(), 1);
        events_remove_sink(sink);
        let reference = retained.borrow_mut().take().unwrap();
        reference.release(c"close observer");
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn session_membership_uses_allocation_identity_and_tolerates_expired_back_references() {
    use hmux2::src::session::Session;
    use std::rc::Rc;
    unsafe {
        let first = hmux2::src::shared::session::SessionRef::allocate();
        let other = hmux2::src::shared::session::SessionRef::allocate();
        let window_owner = hmux2::src::shared::window::WindowRef::empty();
        let mut links = Default::default();
        let mut link = winlink_add(&mut links, 0);
        link.get_mut_unchecked().session = Rc::downgrade(&first);
        winlink_set_window(link.clone(), &window_owner);
        assert!(first.contains_window(&window_owner));
        assert!(!other.contains_window(&window_owner));
        let observer = Rc::downgrade(&first);
        drop(first);
        assert!(observer.upgrade().is_none());
        assert!(!other.contains_window(&window_owner));
        winlink_remove(&mut links, link.clone());
        assert!(!other.contains_window(&window_owner));
        window_owner.release(c"test owner");
    }
}

#[test]
fn unlink_guard_borrows_published_owner_without_adding_a_reference() {

    use std::rc::Rc;

    unsafe {
        let mut links = Default::default();
        let first = winlink_add(&mut links, 0);
        let window = hmux2::src::shared::window::WindowRef::empty();
        let observer = Rc::downgrade(&window);
        winlink_set_window(first.clone(), &window);
        window.release(c"unlink guard creator");
        // Just the published winlink owns the window. A getter that returns a
        // cloned Rc here would incorrectly permit unlinking its only link.
        assert_eq!(
            (hmux2::src::shared::session::SessionRef::window_linked_outside_group(None, first.get_unchecked().window_handle().unwrap()) as i32),
            0
        );
        assert_eq!(observer.strong_count(), 1);

        let second = winlink_add(&mut links, 1);
        winlink_set_window(
            second.clone(),
            first.get_unchecked().window_handle().unwrap(),
        );
        assert_eq!(
            (hmux2::src::shared::session::SessionRef::window_linked_outside_group(None, first.get_unchecked().window_handle().unwrap()) as i32),
            1
        );
        assert!(!first
            .get_unchecked()
            .window_handle()
            .unwrap()
            .is_linked_outside_group(2));
        winlink_remove(&mut links, second);
        assert_eq!(
            (hmux2::src::shared::session::SessionRef::window_linked_outside_group(None, first.get_unchecked().window_handle().unwrap()) as i32),
            0
        );
        winlink_remove(&mut links, first);
        assert!(observer.upgrade().is_none());
    }
}
