use hmux2::src::shared::sort::{sort_criteria, SORT_CREATION};
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::sort::sort_get_panes_window;
use std::rc::Rc;

#[test]
fn sorted_panes_survive_removal_of_ordering_and_source_handles() {
    unsafe {
        let window = window::new();
        let first = window_pane::new();
        let last = window_pane::new();
        (*first.get()).id = 1;
        (*last.get()).id = 2;
        let first_weak = Rc::downgrade(&first);
        let last_weak = Rc::downgrade(&last);
        (*window.get()).panes.push_back(last_weak.clone());
        (*window.get()).panes.push_back(first_weak.clone());
        let sorted = sort_get_panes_window(
            &*window.get(),
            &sort_criteria {
                order: SORT_CREATION,
                reversed: 0,
                order_seq: &[],
            },
        );
        assert!(Rc::ptr_eq(&sorted[0], &first));
        assert!(Rc::ptr_eq(&sorted[1], &last));
        (*window.get()).panes.storage.clear();
        drop(first);
        drop(last);
        hmux2::src::window::window_remove_ref(window, c"test owner".as_ptr());
        assert!(first_weak.upgrade().is_some());
        assert!(last_weak.upgrade().is_some());
        drop(sorted);
        assert!(first_weak.upgrade().is_none());
        assert!(last_weak.upgrade().is_none());
    }
}
