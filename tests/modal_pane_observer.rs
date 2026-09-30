#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::Window as _;
use hmux2::src::window::{PaneOrder, Window};
use hmux2::src::window_pane::WindowPane as _;
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn losing_modal_pane_tolerates_expired_previous_target() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
        let previous = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        let previous_observer = Rc::downgrade(&previous);
        window.initialize_pane(&previous, None);
        let modal = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        window.begin_modal_pane(&modal);
        window.initialize_pane(&modal, None);
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            let mut panes = window.borrow_pane_order_mut(order);
            panes.storage.clear();
            panes.push_back(Rc::downgrade(&modal));
        }
        previous.destroy();
        previous.release(c"modal previous source");
        assert!(previous_observer.upgrade().is_none());
        window.forget_pane(&modal);
        assert!(window.active_pane().is_none());
        assert!(window.modal_pane().is_none());
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            window.borrow_pane_order_mut(order).storage.clear();
        }
        modal.destroy();
        modal.release(c"modal source");
        window.release(c"test owner");
        options.free();
    }
}

#[test]
fn losing_previous_target_clears_observer_without_retaining_it() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
        let previous = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        let observer = Rc::downgrade(&previous);
        window.initialize_pane(&previous, None);
        let modal = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        window.begin_modal_pane(&modal);
        window.initialize_pane(&modal, None);
        let weak_count = Rc::weak_count(&previous);
        window.forget_pane(&previous);
        assert_eq!(Rc::weak_count(&previous), weak_count - 1);
        assert!(Rc::ptr_eq(&window.modal_pane().unwrap(), &modal));
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            window.borrow_pane_order_mut(order).storage.clear();
        }
        previous.destroy();
        previous.release(c"modal previous source");
        assert!(observer.upgrade().is_none());
        modal.destroy();
        modal.release(c"modal source");
        window.release(c"test owner");
        options.free();
    }
}
