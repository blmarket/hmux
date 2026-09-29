use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{window_pane_find_left, window_pane_find_right};
use std::rc::Rc;

#[test]
fn directional_selection_preserves_order_activity_and_retained_result() {
    unsafe {
        let window = window::new();
        (*window.get()).sx = 20;
        (*window.get()).sy = 10;
        let source = window_pane::new();
        let upper = window_pane::new();
        let lower = window_pane::new();
        for (owner, x, y, width, height) in [
            (&source, 0, 0, 9, 9),
            (&upper, 10, 0, 10, 4),
            (&lower, 10, 5, 10, 4),
        ] {
            let pane = &mut *owner.get();
            pane.window = Rc::downgrade(&window);
            pane.xoff = x;
            pane.yoff = y;
            pane.sx = width;
            pane.sy = height;
            (*window.get()).panes.push_back(Rc::downgrade(owner));
        }
        assert!(window_pane_find_right(None).is_none());
        assert!(Rc::ptr_eq(&window_pane_find_right(Some(&source)).unwrap(), &upper));
        (*lower.get()).active_point = 1;
        let selected = window_pane_find_right(Some(&source)).unwrap();
        assert!(Rc::ptr_eq(&selected, &lower));
        assert!(Rc::ptr_eq(&window_pane_find_left(Some(&source)).unwrap(), &lower));
        let observer = Rc::downgrade(&lower);
        (*window.get()).panes.storage = None;
        drop(lower);
        assert!(observer.upgrade().is_some());
        assert_eq!((*selected.get()).yoff, 5);
        drop(selected);
        assert!(observer.upgrade().is_none());
        drop(source);
        drop(upper);
        hmux2::src::window::window_remove_ref(window, c"test owner".as_ptr());
    }
}
