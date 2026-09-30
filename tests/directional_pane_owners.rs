use hmux2::src::window_pane::WindowPane as _;
#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{PaneOrder, Window};
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn directional_selection_preserves_order_activity_and_retained_result() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
        let source = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 9, 9, 0);
        let upper = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 10, 4, 0);
        let lower = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::create(&window, 10, 4, 0);
        for (owner, x, y) in [(&source, 0, 0), (&upper, 10, 0), (&lower, 10, 5)] {
            owner.set_layout_offset(x, y);
            window
                .borrow_pane_order_mut(PaneOrder::Index)
                .push_back(Rc::downgrade(owner));
        }
        assert!(Rc::ptr_eq(&source.neighbor_right().unwrap(), &upper));
        lower.on_selected(true);
        lower.on_selected(true);
        let selected = source.neighbor_right().unwrap();
        assert!(Rc::ptr_eq(&selected, &lower));
        assert!(Rc::ptr_eq(&source.neighbor_left().unwrap(), &lower));
        let observer = Rc::downgrade(&lower);
        window
            .borrow_pane_order_mut(PaneOrder::Index)
            .storage
            .clear();
        lower.destroy();
        lower.release(c"directional source");
        assert!(observer.upgrade().is_some());
        assert_eq!(selected.geometry().3, 5);
        drop(selected);
        assert!(observer.upgrade().is_none());
        source.destroy();
        upper.destroy();
        source.release(c"directional source");
        upper.release(c"directional source");
        window.release(c"test owner");
        options.free();
    }
}
