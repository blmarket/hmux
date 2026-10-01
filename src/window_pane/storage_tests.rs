//! Storage assertions stay beside the pane implementation.
use super::*;

#[test]
fn directional_selection_without_a_source_returns_none() {
    unsafe {
        assert!(window_pane_find_right(None).is_none());
    }
}
