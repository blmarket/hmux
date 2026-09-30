use hmux2::src::window::Window as _;
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{PaneOrder, Window};
use std::rc::Rc;

#[test]
fn losing_modal_pane_tolerates_expired_previous_target() {
    unsafe {
        let window = hmux2::src::shared::window::WindowRef::empty();
        let previous = window_pane::new();
        let previous_observer = Rc::downgrade(&previous);
        window.initialize_pane(&previous, None);
        let modal = window_pane::new();
        (*modal.get()).window = Rc::downgrade(&window);
        window.begin_modal_pane(&modal);
        window.initialize_pane(&modal, None);
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            let mut panes = window.borrow_pane_order_mut(order);
            panes.storage.clear();
            panes.push_back(Rc::downgrade(&modal));
        }
        drop(previous);
        assert!(previous_observer.upgrade().is_none());
        window.forget_pane(&modal);
        assert!(window.active_pane().is_none());
        assert!(window.modal_pane().is_none());
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            window.borrow_pane_order_mut(order).storage.clear();
        }
        drop(modal);
        window.release(c"test owner");
    }
}

#[test]
fn losing_previous_target_clears_observer_without_retaining_it() {
    unsafe {
        let window = hmux2::src::shared::window::WindowRef::empty();
        let previous = window_pane::new();
        let observer = Rc::downgrade(&previous);
        (*previous.get()).window = Rc::downgrade(&window);
        window.initialize_pane(&previous, None);
        let modal = window_pane::new();
        window.begin_modal_pane(&modal);
        window.initialize_pane(&modal, None);
        let weak_count = Rc::weak_count(&previous);
        window.forget_pane(&previous);
        assert_eq!(Rc::weak_count(&previous), weak_count - 1);
        assert!(Rc::ptr_eq(&window.modal_pane().unwrap(), &modal));
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            window.borrow_pane_order_mut(order).storage.clear();
        }
        drop(previous);
        assert!(observer.upgrade().is_none());
        window.release(c"test owner");
    }
}
