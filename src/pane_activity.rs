//! Activity ordering for panes.

use core::ffi::c_uint;

/// Storage for a pane's activity sequence number.
pub trait PaneActivityState {
    /// Returns the activity sequence number.
    fn activity_point(&self) -> c_uint;

    /// Records the pane as active at the given sequence number.
    fn mark_active_at(&mut self, point: c_uint);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_point_is_zero_then_replaceable() {
        let mut state = crate::types::window_pane::default();
        assert_eq!(state.activity_point(), 0);
        state.mark_active_at(42);
        assert_eq!(state.activity_point(), 42);
        state.mark_active_at(c_uint::MAX);
        assert_eq!(state.activity_point(), c_uint::MAX);
    }
}
