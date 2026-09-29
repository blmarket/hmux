use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{
    window_pane_list_insert_back, window_pane_swap_order,
    window_pane_z_insert_back, window_pane_z_swap_order,
};
use std::rc::Rc;

#[test]
fn order_swaps_preserve_weak_membership_within_and_between_windows() {
    unsafe {
        let left_owner = window::new();
        let right_owner = window::new();
        let first_owner = window_pane::new();
        let second_owner = window_pane::new();
        let third_owner = window_pane::new();
        let left = &mut *left_owner.get();
        let right = &mut *right_owner.get();
        let first = &*first_owner.get();
        let second = &*second_owner.get();
        let third = &*third_owner.get();
        for pane in [first, second] {
            window_pane_list_insert_back(left, pane);
            window_pane_z_insert_back(left, pane);
        }
        window_pane_list_insert_back(right, third);
        window_pane_z_insert_back(right, third);

        window_pane_swap_order(left, first, None, second);
        window_pane_z_swap_order(left, first, None, second);
        assert!(Rc::ptr_eq(&left.panes.first().unwrap(), &second_owner));
        assert!(Rc::ptr_eq(&left.panes.last().unwrap(), &first_owner));
        assert!(Rc::ptr_eq(&left.z_index.first().unwrap(), &second_owner));
        assert!(Rc::ptr_eq(&left.z_index.last().unwrap(), &first_owner));

        window_pane_swap_order(left, second, Some(right), third);
        window_pane_z_swap_order(left, second, Some(right), third);
        assert!(Rc::ptr_eq(&left.panes.first().unwrap(), &third_owner));
        assert!(Rc::ptr_eq(&left.panes.last().unwrap(), &first_owner));
        assert!(Rc::ptr_eq(&right.panes.first().unwrap(), &second_owner));
        assert!(Rc::ptr_eq(&left.z_index.first().unwrap(), &third_owner));
        assert!(Rc::ptr_eq(&left.z_index.last().unwrap(), &first_owner));
        assert!(Rc::ptr_eq(&right.z_index.first().unwrap(), &second_owner));
        for owner in [&first_owner, &second_owner, &third_owner] {
            assert_eq!(Rc::strong_count(owner), 1, "ordering must not own panes");
        }

        // These are ordering-only fixtures, with no pane registry membership.
        left.panes.storage.clear();
        left.z_index.storage.clear();
        right.panes.storage.clear();
        right.z_index.storage.clear();
        hmux2::src::window::window_remove_ref(left_owner, c"test owner".as_ptr());
        hmux2::src::window::window_remove_ref(right_owner, c"test owner".as_ptr());
    }
}
