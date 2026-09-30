//! Storage assertions stay beside the pane implementation.
use super::*;
use crate::src::shared::pane::{window_pane, window_pane_resize};
use std::ptr::null_mut;

fn resize(sx: u32, sy: u32, osx: u32, osy: u32) -> window_pane_resize {
    window_pane_resize { sx, sy, osx, osy }
}

#[test]
fn resize_queue_preserves_order_and_exception_removal() {
    let mut pane = window_pane::empty();
    assert!(pane.resize_queue.is_empty());
    pane.clear_resizes_except(null_mut());
    assert!(pane.resize_queue.is_empty());

    let first = pane.push_resize(resize(100, 40, 0, 0));
    let second = pane.push_resize(resize(110, 41, 100, 40));
    let last = pane.push_resize(resize(120, 42, 110, 41));

    assert_eq!(
        pane.resize_queue
            .iter()
            .map(|resize| (resize.sx, resize.sy))
            .collect::<Vec<_>>(),
        [(100, 40), (110, 41), (120, 42)]
    );
    assert_eq!(
        (&**pane.resize_queue.front().unwrap()) as *const window_pane_resize
            as *mut window_pane_resize,
        first
    );
    assert_eq!(
        (&**pane.resize_queue.back().unwrap()) as *const window_pane_resize
            as *mut window_pane_resize,
        last
    );

    pane.clear_resizes_except(last);
    assert_eq!(
        pane.resize_queue
            .iter()
            .map(|resize| (resize.sx, resize.sy))
            .collect::<Vec<_>>(),
        [(120, 42)]
    );
    assert_eq!(
        (&**pane.resize_queue.front().unwrap()) as *const window_pane_resize
            as *mut window_pane_resize,
        last
    );

    pane.clear_resizes_except(null_mut());
    assert!(pane.resize_queue.is_empty());
    let _ = (first, second);
}

#[test]
fn pane_owner_drops_resize_storage_without_manual_cleanup() {
    let mut pane = Box::new(window_pane::empty());
    pane.push_resize(resize(100, 40, 0, 0));
    assert!(!pane.resize_queue.is_empty());
    drop(pane);
}
