#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{window_pane_find_left, window_pane_find_right};
use hmux2::src::window::{PaneOrder, Window};
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn directional_selection_preserves_order_activity_and_retained_result() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
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
            window
                .borrow_pane_order_mut(PaneOrder::Index)
                .push_back(Rc::downgrade(owner));
        }
        assert!(window_pane_find_right(None).is_none());
        assert!(Rc::ptr_eq(
            &window_pane_find_right(Some(&source)).unwrap(),
            &upper
        ));
        (*lower.get()).active_point = 1;
        let selected = window_pane_find_right(Some(&source)).unwrap();
        assert!(Rc::ptr_eq(&selected, &lower));
        assert!(Rc::ptr_eq(
            &window_pane_find_left(Some(&source)).unwrap(),
            &lower
        ));
        let observer = Rc::downgrade(&lower);
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .storage
            .clear();
        drop(lower);
        assert!(observer.upgrade().is_some());
        assert_eq!((*selected.get()).yoff, 5);
        drop(selected);
        assert!(observer.upgrade().is_none());
        drop(source);
        drop(upper);
        window.release(c"test owner");
        options.free();
    }
}
