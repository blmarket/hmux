use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::window_lost_pane;
use std::rc::Rc;

#[test]
fn losing_modal_pane_tolerates_expired_previous_target() {
    unsafe {
        let window = window::new();
        let previous = window_pane::new();
        (*window.get()).modal_last = Rc::downgrade(&previous);
        drop(previous);
        assert!((*window.get()).modal_last.upgrade().is_none());
        let modal = window_pane::new();
        (*modal.get()).window = Rc::downgrade(&window);
        (*window.get()).set_active(modal.get());
        (*window.get()).modal = Rc::downgrade(&modal);
        (*window.get()).panes.push_back(Rc::downgrade(&modal));
        window_lost_pane(window.get(), modal.get());
        assert!((*window.get()).active_ptr().is_null());
        assert!((*window.get()).modal.upgrade().is_none());
        assert!((*window.get()).modal_last.upgrade().is_none());
        (*window.get()).panes.storage = None;
        drop(modal);
        drop(window);
    }
}

#[test]
fn losing_previous_target_clears_observer_without_retaining_it() {
    unsafe {
        let window = window::new();
        let previous = window_pane::new();
        let observer = Rc::downgrade(&previous);
        (*previous.get()).window = Rc::downgrade(&window);
        (*window.get()).modal_last = observer.clone();
        window_lost_pane(window.get(), previous.get());
        assert!(!(*window.get()).modal_last.ptr_eq(&observer));
        drop(previous);
        assert!(observer.upgrade().is_none());
        drop(window);
    }
}
