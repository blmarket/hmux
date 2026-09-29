use hmux2::src::shared::pane::{window_pane, window_pane_resize, window_pane_resizes};
use std::ptr::null_mut;

fn resize(sx: u32, sy: u32, osx: u32, osy: u32) -> window_pane_resize {
    window_pane_resize { sx, sy, osx, osy }
}

#[test]
fn resize_queue_preserves_order_and_exception_removal() {
    let mut queue = window_pane_resizes::default();
    assert!(queue.is_empty());
    queue.clear_except(null_mut());
    assert!(queue.storage.is_empty());

    let first = queue.push_back(resize(100, 40, 0, 0));
    let second = queue.push_back(resize(110, 41, 100, 40));
    let last = queue.push_back(resize(120, 42, 110, 41));

    assert_eq!(
        queue
            .as_ref()
            .iter()
            .map(|resize| (resize.sx, resize.sy))
            .collect::<Vec<_>>(),
        [(100, 40), (110, 41), (120, 42)]
    );
    assert_eq!(
        (&**queue.as_ref().front().unwrap()) as *const window_pane_resize
            as *mut window_pane_resize,
        first
    );
    assert_eq!(
        (&**queue.as_ref().back().unwrap()) as *const window_pane_resize
            as *mut window_pane_resize,
        last
    );

    queue.clear_except(last);
    assert_eq!(
        queue
            .as_ref()
            .iter()
            .map(|resize| (resize.sx, resize.sy))
            .collect::<Vec<_>>(),
        [(120, 42)]
    );
    assert_eq!(
        (&**queue.as_ref().front().unwrap()) as *const window_pane_resize
            as *mut window_pane_resize,
        last
    );

    queue.clear_except(null_mut());
    assert!(queue.storage.is_empty());
    assert!(queue.is_empty());
    let _ = (first, second);
}

#[test]
fn pane_owner_drops_resize_storage_without_manual_cleanup() {
    let mut pane = Box::new(window_pane::empty());
    pane.resize_queue.push_back(resize(100, 40, 0, 0));
    assert!(!pane.resize_queue.storage.is_empty());
    drop(pane);
}
