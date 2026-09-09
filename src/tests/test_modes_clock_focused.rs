use super::*;
use crate::WindowPane;
use crate::grid::grid_get_cell;
use crate::tests::test_fixtures::{Target, Window, globals};
use crate::window::{
    window_pane_current_mode_mut, window_pane_find_by_id, window_pane_set_mode,
    window_pane_set_window_ref, window_panes_insert_tail, window_panes_take,
};

fn assert_clock_colour(screen: &RustScreen, colour: i32) {
    let (sx, sy) = screen.size();
    let cells: Vec<_> = (0..sx)
        .map(|x| grid_get_cell(RustScreen::grid(screen), x, sy / 2))
        .filter(|cell| cell.data.data[0] != b' ')
        .collect();
    assert!(cells.iter().any(|cell| cell.data.data[0] == b':'));
    assert!(cells.iter().all(|cell| cell.fg == colour));
}

#[test]
fn clock_resize_and_timer_follow_the_panes_current_window_options() {
    let _guard = globals();
    let mut target = Target::new(20, 5);
    let fs = target.state();
    let mut pane = fs.pane_ref().unwrap();
    let id = pane.id();
    let destination = Window::new(100, "destination", 20, 5);
    unsafe {
        let options = pane.window().unwrap().options();
        options.set_number(c"clock-mode-colour", 1);
        options.set_number(c"clock-mode-style", 1);
        window_pane_set_mode(pane.get_mut().unwrap(), None, WindowMode::Clock, None, None);
        {
            let entry = window_pane_current_mode_mut(pane.get_mut().unwrap()).unwrap();
            let data = entry.state.clock().unwrap();
            assert_clock_colour(&data.screen, 1);
            assert!(data.timer.is_armed());
        }
        let mut original = pane.window().unwrap();
        let mut owned = window_panes_take(
            &mut original.as_window_mut(),
            &crate::window::window_pane_find_by_id(id).expect("the pane exists"),
        )
        .unwrap();
        let mut destination_owner = destination.reference();
        window_pane_set_window_ref(owned.as_pane_mut(), Some(&destination_owner));
        window_panes_insert_tail(&mut destination_owner.as_window_mut(), owned);
        assert!(pane.get().is_some());
        let mut moved = window_pane_find_by_id(id).unwrap();
        assert!(pane.ptr_eq(&moved));
        destination.options().set_number(c"clock-mode-colour", 2);
        destination.options().set_number(c"clock-mode-style", 1);
        {
            let entry = window_pane_current_mode_mut(moved.get_mut().unwrap()).unwrap();
            window_clock_resize(entry, 22, 5);
            assert_clock_colour(&entry.state.clock().unwrap().screen, 2);
        }
        destination.options().set_number(c"clock-mode-colour", 3);
        {
            let wp = moved.get_mut().unwrap();
            *wp.flags_mut() &= !PANE_REDRAW;
            let entry = window_pane_current_mode_mut(wp).unwrap();
            entry.state.clock().unwrap().tim = window_clock_now().as_secs() as time_t + 30;
        }
        window_clock_timer_callback(moved.clone());
        let timer = {
            let wp = moved.get_mut().unwrap();
            assert_ne!(*wp.flags() & PANE_REDRAW, 0);
            let entry = window_pane_current_mode_mut(wp).unwrap();
            let data = entry.state.clock().unwrap();
            assert_clock_colour(&data.screen, 3);
            assert!(data.timer.is_armed());
            data.timer
        };
        window_pane_reset_mode(moved.get_mut().unwrap());
        assert!(!timer.is_armed());
        *moved.get_mut().unwrap().flags_mut() &= !PANE_REDRAW;
        window_clock_timer_callback(moved.clone());
        assert_eq!(*moved.get().unwrap().flags() & PANE_REDRAW, 0);
        drop(destination);
        assert!(window_pane_find_by_id(id).is_none());
        window_clock_timer_callback(moved.clone());
    }
}
