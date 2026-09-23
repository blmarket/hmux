use hmux2::src::control::*;

#[test]
fn window_resize_replacement_and_pane_reset_release_indexes() {
    unsafe {
        let mut state: control_state = std::mem::zeroed();
        let mut client: client = std::mem::zeroed();
        client.control_state = &mut state;
        let c = &mut client;
        for id in [u32::MAX, 0, 17] {
            control_set_window_size(c, id, 80, 24);
        }
        let original = control_windows_minmax(&mut state.windows, -1);
        control_set_window_size(c, 0, 120, 40);
        assert_eq!(control_windows_minmax(&mut state.windows, -1), original);
        let (mut sx, mut sy) = (0, 0);
        assert_eq!(control_get_window_size(c, 0, &mut sx, &mut sy), 1);
        assert_eq!((sx, sy), (120, 40));
        for id in [0, 17, u32::MAX] {
            control_clear_window_size(c, id);
            assert_eq!(control_get_window_size(c, id, &mut sx, &mut sy), 0);
        }
        assert!(state.windows.storage.is_null());
        for id in [u32::MAX, 2, 0] {
            let pane = Box::into_raw(Box::new(std::mem::zeroed::<control_pane>()));
            (*pane).pane = id;
            assert!(control_panes_insert(&mut state.panes, pane).is_null());
        }
        let first = control_panes_minmax(&mut state.panes, -1);
        assert_eq!((*first).pane, 0);
        assert_eq!((*control_panes_next(first)).pane, 2);
        control_reset_offsets(c);
        assert!(state.panes.storage.is_null());
        control_reset_offsets(c);
    }
}
