use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::Window as _;
use hmux2::src::window::{PaneOrder, Window};
use hmux2::src::window_pane::WindowPane as _;
use std::rc::Rc;

#[test]
fn order_swaps_preserve_weak_membership_within_and_between_windows() {
    unsafe {
        let left_owner = hmux2::src::shared::window::WindowRef::empty();
        let right_owner = hmux2::src::shared::window::WindowRef::empty();
        let first_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
        let second_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
        let third_owner = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            {
                let mut left = left_owner.borrow_pane_order_mut(order);
                left.push_back(Rc::downgrade(&first_owner));
                left.push_back(Rc::downgrade(&second_owner));
            }
            right_owner
                .borrow_pane_order_mut(order)
                .push_back(Rc::downgrade(&third_owner));
            // An aliased Rc must still take only one borrow of the Window.
            let same_window = left_owner.clone();
            left_owner.swap_pane_order(
                order,
                &Rc::downgrade(&first_owner),
                &same_window,
                &Rc::downgrade(&second_owner),
            );
            same_window.release(c"same-window ordering alias");
            assert!(Rc::ptr_eq(
                &left_owner.step_pane(order, None, false).unwrap(),
                &second_owner
            ));
            assert!(Rc::ptr_eq(
                &left_owner.step_pane(order, None, true).unwrap(),
                &first_owner
            ));
            left_owner.swap_pane_order(
                order,
                &Rc::downgrade(&second_owner),
                &right_owner,
                &Rc::downgrade(&third_owner),
            );
            assert!(Rc::ptr_eq(
                &left_owner.step_pane(order, None, false).unwrap(),
                &third_owner
            ));
            assert!(Rc::ptr_eq(
                &left_owner.step_pane(order, None, true).unwrap(),
                &first_owner
            ));
            assert!(Rc::ptr_eq(
                &right_owner.step_pane(order, None, false).unwrap(),
                &second_owner
            ));
        }
        for owner in [&first_owner, &second_owner, &third_owner] {
            assert_eq!(Rc::strong_count(owner), 1, "ordering must not own panes");
        }

        // These are ordering-only fixtures, with no pane registry membership.
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            left_owner.borrow_pane_order_mut(order).storage.clear();
            right_owner.borrow_pane_order_mut(order).storage.clear();
        }
        left_owner.release(c"test owner");
        right_owner.release(c"test owner");
    }
}
