use hmux2::src::monitor::*;
use hmux2::src::shared::monitor::{monitor_item, monitor_set, MONITOR_SESSION};
use std::{ffi::CStr, ptr::null_mut};

unsafe fn find_item(set: *mut monitor_set, name: &CStr) -> *mut monitor_item {
    let mut key = monitor_item::empty();
    key.name = ::std::ffi::CStr::from_ptr(name.as_ptr().cast_mut()).to_owned();
    monitor_items_find(&(*set).items, &key)
}

#[test]
fn byte_names_composite_keys_and_nested_cleanup() {
    unsafe {
        let set = monitor_create_client(null_mut(), None, null_mut());
        for (name, format) in [(c"z", c"\xfeZ"), (c"\xff", c"\xffF"), (c"a", c"A\xfe")] {
            let mut input_name = name.to_bytes_with_nul().to_vec();
            let mut input_format = format.to_bytes_with_nul().to_vec();
            monitor_add(
                set,
                input_name.as_ptr().cast(),
                MONITOR_SESSION,
                -1,
                input_format.as_ptr().cast(),
                0,
            );
            input_name[0] = b'Q';
            input_format[0] = b'Q';

            let item = find_item(set, name);
            assert!(!item.is_null());
            assert_eq!(CStr::from_ptr(((*item).name).as_ptr().cast_mut()), name);
            assert_eq!(CStr::from_ptr(((*item).format).as_ptr().cast_mut()), format);
        }

        // A caller can replace a monitor using pointers it previously read
        // from that monitor. The old item is destroyed during replacement.
        let old = find_item(set, c"z");
        assert!(!old.is_null());
        monitor_add(
            set,
            ((*old).name).as_ptr().cast_mut(),
            MONITOR_SESSION,
            -1,
            ((*old).format).as_ptr().cast_mut(),
            0,
        );
        let replacement = find_item(set, c"z");
        assert!(!replacement.is_null());
        assert_eq!(
            CStr::from_ptr(((*replacement).name).as_ptr().cast_mut()),
            c"z"
        );
        assert_eq!(
            CStr::from_ptr(((*replacement).format).as_ptr().cast_mut()),
            c"\xfeZ"
        );

        for name in [c"z", c"\xff", c"a"] {
            let item = find_item(set, name);
            for (id, idx) in [(2, 1), (1, 9), (1, 0), (u32::MAX, 0)] {
                let pane = monitor_pane_new();
                (*pane).pane = id;
                (*pane).idx = idx;
                monitor_panes_insert(&raw mut (*item).panes, pane);
                let window = monitor_window_new();
                (*window).window = id;
                (*window).idx = idx;
                monitor_windows_insert(&raw mut (*item).windows, window);
            }
            let mut pane = monitor_panes_minmax(&(*item).panes, -1);
            let mut window = monitor_windows_minmax(&(*item).windows, -1);
            for pair in [(1, 0), (1, 9), (2, 1), (u32::MAX, 0)] {
                assert_eq!(((*pane).pane, (*pane).idx), pair);
                assert_eq!(((*window).window, (*window).idx), pair);
                pane = monitor_panes_next(&*pane);
                window = monitor_windows_next(&*window);
            }
            assert!(pane.is_null() && window.is_null());
        }
        let mut item = monitor_items_minmax(&(*set).items, -1);
        for name in [c"a", c"z", c"\xff"] {
            assert_eq!(CStr::from_ptr(((*item).name).as_ptr().cast_mut()), name);
            item = monitor_items_next(&*item);
        }
        assert!(item.is_null());
        for name in [c"a", c"\xff", c"z"] {
            monitor_remove(set, name.as_ptr());
        }
        assert!((*set).items.storage.is_none());
        monitor_remove(set, c"missing".as_ptr());
        monitor_destroy(set);
    }
}
