//! The numbered layout most recently selected for a window.

use core::ffi::c_int;

/// Storage for a window's previous numbered layout.
pub trait WindowLayoutSelectionState: Default {
    /// Returns the previous numbered layout, if one has been selected.
    fn previous_layout(&self) -> Option<c_int>;

    /// Records a numbered layout as the previous selection.
    fn remember_layout(&mut self, layout: c_int);

    /// Forgets the previous numbered layout.
    fn clear_previous_layout(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_layout_can_be_replaced_and_cleared() {
        let mut state = crate::types::window::default();
        assert_eq!(state.previous_layout(), None);
        state.remember_layout(6);
        assert_eq!(state.previous_layout(), Some(6));
        state.remember_layout(17);
        assert_eq!(state.previous_layout(), Some(17));
        state.remember_layout(-1);
        assert_eq!(state.previous_layout(), None);
        state.remember_layout(3);
        state.clear_previous_layout();
        assert_eq!(state.previous_layout(), None);
    }
}
