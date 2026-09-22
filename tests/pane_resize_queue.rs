use hmux2::src::shared::pane::{window_pane_resize, window_pane_resize_entry, window_pane_resizes};
use std::ptr::null_mut;

fn resize(sx: u32, sy: u32, osx: u32, osy: u32) -> window_pane_resize {
    window_pane_resize {
        sx,
        sy,
        osx,
        osy,
        entry: window_pane_resize_entry {
            tqe_next: null_mut(),
            tqe_prev: null_mut(),
        },
    }
}

#[test]
fn resize_queue_preserves_order_and_exception_removal() {
    unsafe {
        let mut queue = window_pane_resizes {
            storage: null_mut(),
            reserved: null_mut(),
        };

        let first = queue.push_back(resize(100, 40, 0, 0));
        let second = queue.push_back(resize(110, 41, 100, 40));
        let last = queue.push_back(resize(120, 42, 110, 41));

        assert_eq!(
            queue
                .as_ref()
                .unwrap()
                .iter()
                .map(|resize| (resize.sx, resize.sy))
                .collect::<Vec<_>>(),
            [(100, 40), (110, 41), (120, 42)]
        );
        assert_eq!(
            (&**queue.as_ref().unwrap().front().unwrap()) as *const window_pane_resize
                as *mut window_pane_resize,
            first
        );
        assert_eq!(
            (&**queue.as_ref().unwrap().back().unwrap()) as *const window_pane_resize
                as *mut window_pane_resize,
            last
        );

        queue.clear_except(last);
        assert_eq!(
            queue
                .as_ref()
                .unwrap()
                .iter()
                .map(|resize| (resize.sx, resize.sy))
                .collect::<Vec<_>>(),
            [(120, 42)]
        );
        assert_eq!(
            (&**queue.as_ref().unwrap().front().unwrap()) as *const window_pane_resize
                as *mut window_pane_resize,
            last
        );

        queue.clear_except(null_mut());
        assert!(queue.storage.is_null());
        assert!(queue.is_empty());
        let _ = (first, second);
    }
}
