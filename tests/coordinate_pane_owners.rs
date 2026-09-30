#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::options::{options_create_owned, options_default, options_set_number};
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{window_find_string, window_get_active_at};
use hmux2::src::window::{window_lost_pane, PaneOrder, Window};
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn coordinate_results_retain_panes_and_modal_observer_does_not() {
    unsafe {
        let options = WindowOptions::new();
        let window = options.create(20, 10);
        let first = window_pane::new();
        let second = window_pane::new();
        for (owner, x) in [(&first, 0), (&second, 10)] {
            let pane = &mut *owner.get();
            pane.window = Rc::downgrade(&window);
            pane.xoff = x;
            pane.sx = 9;
            pane.sy = 9;
            window
                .borrow_pane_order_mut(PaneOrder::Stacking)
                .push_back(Rc::downgrade(owner));
        }
        window.begin_modal_pane(&first);
        hmux2::src::window::window_redraw_active_switch(&window, None);
        assert!(Rc::ptr_eq(
            &window_get_active_at(&window, 2, 2).unwrap(),
            &first
        ));
        assert!(window_get_active_at(&window, 12, 2).is_none());
        window_lost_pane(&window, &first);
        assert!(Rc::ptr_eq(
            &window_get_active_at(&window, 12, 2).unwrap(),
            &second
        ));
        assert!(window_get_active_at(&window, 30, 2).is_none());
        assert!(window_find_string(&window, c"unknown").is_none());
        let selected = window_find_string(&window, c"right").unwrap();
        assert!(Rc::ptr_eq(&selected, &second));
        let observer = Rc::downgrade(&second);
        window.begin_modal_pane(&second);
        window
            .borrow_pane_order_mut(PaneOrder::Stacking)
            .storage
            .clear();
        drop(second);
        assert!(observer.upgrade().is_some());
        drop(selected);
        assert!(observer.upgrade().is_none());
        assert!(window.modal_pane().is_none());
        assert!(window_get_active_at(&window, 12, 2).is_none());
        window.with_options_mut(|options| {
            options_set_number(options, c"pane-border-status".as_ptr(), 1);
        });
        (*first.get()).yoff = 3;
        (*first.get()).border_status_line.ranges.push(Box::new(
            hmux2::src::shared::style::style_range {
                type_0: hmux2::src::shared::style::STYLE_RANGE_CONTROL,
                argument: 4,
                string: [0; 16],
                start: 1,
                end: 3,
            },
        ));
        let range = hmux2::src::window::window_pane_status_get_range(&first, 3, 2).unwrap();
        assert!(hmux2::src::window::window_pane_status_get_range(&first, 2, 2).is_none());
        assert!(hmux2::src::window::window_pane_status_get_range(&first, 5, 2).is_none());
        assert!(hmux2::src::window::window_pane_status_get_range(&first, 3, 3).is_none());
        (*first.get()).border_status_line.ranges.clear();
        assert_eq!(range.argument, 4);
        assert!(hmux2::src::window::window_pane_status_get_range(&first, 3, 2).is_none());
        drop(first);
        window.release(c"test owner");
        options.free();
    }
}
