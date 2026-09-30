#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::shared::sort::{sort_criteria, SORT_CREATION};
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::sort::sort_get_panes_window;
use hmux2::src::window::Window as _;
use hmux2::src::window::{PaneOrder, Window};
use hmux2::src::window_pane::WindowPane as _;
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn sorted_panes_survive_removal_of_ordering_and_source_handles() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
        let first = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        let last = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        assert!(first.id() < last.id());
        let first_weak = Rc::downgrade(&first);
        let last_weak = Rc::downgrade(&last);
        {
            let mut panes = window.borrow_pane_order_mut(PaneOrder::Index);
            panes.push_back(last_weak.clone());
            panes.push_back(first_weak.clone());
        }
        let sorted = sort_get_panes_window(
            &window,
            &sort_criteria {
                order: SORT_CREATION,
                reversed: 0,
                order_seq: &[],
            },
        );
        assert!(Rc::ptr_eq(&sorted[0], &first));
        assert!(Rc::ptr_eq(&sorted[1], &last));
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .storage
            .clear();
        first.destroy();
        last.destroy();
        first.release(c"sort source");
        last.release(c"sort source");
        window.release(c"test owner");
        assert!(first_weak.upgrade().is_some());
        assert!(last_weak.upgrade().is_some());
        drop(sorted);
        assert!(first_weak.upgrade().is_none());
        assert!(last_weak.upgrade().is_none());
        options.free();
    }
}
