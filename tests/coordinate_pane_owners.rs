use hmux2::src::options::{options_create_owned, options_default, options_set_number};
use hmux2::src::shared::{pane::window_pane, window::window};
use hmux2::src::window::{window_find_string, window_get_active_at};
use std::rc::Rc;

#[test]
fn coordinate_results_retain_panes_and_saved_zoom_does_not() {
    unsafe {
        let window = window::new();
        (*window.get()).sx = 20;
        (*window.get()).sy = 10;
        let mut options = options_create_owned(std::ptr::null_mut());
        let definition = hmux2::src::options_table::options_table.iter()
            .find(|entry| entry.name == Some(c"pane-border-status")).unwrap();
        options_default(&mut *options, definition);
        options_set_number(&mut *options, c"pane-border-status".as_ptr(), 0);
        (*window.get()).options = Some(options);
        let first = window_pane::new();
        let second = window_pane::new();
        for (owner, x) in [(&first, 0), (&second, 10)] {
            let pane = &mut *owner.get();
            pane.window = Rc::downgrade(&window);
            pane.xoff = x;
            pane.sx = 9;
            pane.sy = 9;
            (*window.get()).z_index.push_back(Rc::downgrade(owner));
        }
        (*window.get()).modal = Rc::downgrade(&first);
        hmux2::src::window::window_redraw_active_switch(&window, None);
        assert!(Rc::ptr_eq(&window_get_active_at(&window, 2, 2).unwrap(), &first));
        assert!(window_get_active_at(&window, 12, 2).is_none());
        (*window.get()).modal = std::rc::Weak::new();
        assert!(Rc::ptr_eq(&window_get_active_at(&window, 12, 2).unwrap(), &second));
        assert!(window_get_active_at(&window, 30, 2).is_none());
        assert!(window_find_string(&window, c"unknown").is_none());
        let selected = window_find_string(&window, c"right").unwrap();
        assert!(Rc::ptr_eq(&selected, &second));
        let observer = Rc::downgrade(&second);
        (*window.get()).was_zoomed = observer.clone();
        (*window.get()).modal = observer.clone();
        (*window.get()).z_index.storage = None;
        drop(second);
        assert!(observer.upgrade().is_some());
        drop(selected);
        assert!(observer.upgrade().is_none());
        assert!((*window.get()).was_zoomed.upgrade().is_none());
        assert!((*window.get()).modal.upgrade().is_none());
        assert!(window_get_active_at(&window, 12, 2).is_none());
        (*window.get()).flags |= hmux2::src::window::WINDOW_WASZOOMED;
        assert_eq!(hmux2::src::window::window_pop_zoom(&window), 0);
        assert_eq!((*window.get()).flags & hmux2::src::window::WINDOW_WASZOOMED, 0);
        options_set_number((*window.get()).options.as_deref_mut().unwrap(), c"pane-border-status".as_ptr(), 1);
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
        hmux2::src::window::window_remove_ref(window, c"test owner".as_ptr());
    }
}
