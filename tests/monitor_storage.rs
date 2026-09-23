use hmux2::src::monitor::*;
use std::{ffi::CStr, ptr::null_mut};

#[test]
fn byte_names_composite_keys_and_nested_cleanup() {
    unsafe {
        let set = monitor_create_client(null_mut(), None, null_mut());
        for name in [c"z", c"\xff", c"a"] {
            let item = Box::into_raw(Box::new(std::mem::zeroed::<monitor_item>()));
            (*item).name = libc::strdup(name.as_ptr());
            monitor_items_insert(&mut (*set).items, item);
            for (id, idx) in [(2, 1), (1, 9), (1, 0), (u32::MAX, 0)] {
                let pane =
                    libc::calloc(1, std::mem::size_of::<monitor_pane>()) as *mut monitor_pane;
                (*pane).pane = id;
                (*pane).idx = idx;
                monitor_panes_insert(&mut (*item).panes, pane);
                let window =
                    libc::calloc(1, std::mem::size_of::<monitor_window>()) as *mut monitor_window;
                (*window).window = id;
                (*window).idx = idx;
                monitor_windows_insert(&mut (*item).windows, window);
            }
            let mut pane = monitor_panes_minmax(&mut (*item).panes, -1);
            let mut window = monitor_windows_minmax(&mut (*item).windows, -1);
            for pair in [(1, 0), (1, 9), (2, 1), (u32::MAX, 0)] {
                assert_eq!(((*pane).pane, (*pane).idx), pair);
                assert_eq!(((*window).window, (*window).idx), pair);
                pane = monitor_panes_next(pane);
                window = monitor_windows_next(window);
            }
            assert!(pane.is_null() && window.is_null());
        }
        let mut item = monitor_items_minmax(&mut (*set).items, -1);
        for name in [c"a", c"z", c"\xff"] {
            assert_eq!(CStr::from_ptr((*item).name), name);
            item = monitor_items_next(item);
        }
        assert!(item.is_null());
        for name in [c"a", c"\xff", c"z"] {
            monitor_remove(set, name.as_ptr());
        }
        assert!((*set).items.storage.is_null());
        monitor_remove(set, c"missing".as_ptr());
        monitor_destroy(set);
    }
}
