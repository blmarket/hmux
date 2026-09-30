use hmux2::src::window::Window as _;
use hmux2::src::window_pane::WindowPane as _;
#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::shared::{pane::window_pane, window::window};

use hmux2::src::window::{PaneOrder, Window};
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn coordinate_results_retain_panes_and_modal_observer_does_not() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
        let first = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        let second = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        for (owner, x) in [(&first, 0), (&second, 10)] {
            owner.set_layout_offset(x, 0);
            window
                .borrow_pane_order_mut(PaneOrder::Stacking)
                .push_back(Rc::downgrade(owner));
        }
        window.begin_modal_pane(&first);
        window.redraw_active_switch(None);
        assert!(Rc::ptr_eq(&window.pane_at(2, 2).unwrap(), &first));
        assert!(window.pane_at(12, 2).is_none());
        window.forget_pane(&first);
        assert!(Rc::ptr_eq(&window.pane_at(12, 2).unwrap(), &second));
        assert!(window.pane_at(30, 2).is_none());
        assert!(window.find_pane(c"unknown").is_none());
        let selected = window.find_pane(c"right").unwrap();
        assert!(Rc::ptr_eq(&selected, &second));
        let observer = Rc::downgrade(&second);
        window.begin_modal_pane(&second);
        window
            .borrow_pane_order_mut(PaneOrder::Stacking)
            .storage
            .clear();
        second.destroy();
        second.release(c"coordinate source");
        assert!(observer.upgrade().is_some());
        drop(selected);
        assert!(observer.upgrade().is_none());
        assert!(window.modal_pane().is_none());
        assert!(window.pane_at(12, 2).is_none());
        first.destroy();
        first.release(c"coordinate source");
        window.release(c"test owner");
        options.free();
    }
}
