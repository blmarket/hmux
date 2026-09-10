//! Serialized layout retained for restoring a window's previous arrangement.

use core::ffi::CStr;

/// Storage for a window's optional serialized saved layout.
pub trait WindowSavedLayoutState: Default {
    /// Returns the saved layout text, if present.
    fn saved_layout(&self) -> Option<&CStr>;

    /// Copies a replacement layout or clears the saved layout.
    fn set_saved_layout(&mut self, layout: Option<&CStr>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_can_be_replaced_and_cleared() {
        let mut state = crate::types::window::default();
        assert!(state.saved_layout().is_none());
        state.set_saved_layout(Some(c"80x24,0,0"));
        assert_eq!(state.saved_layout(), Some(c"80x24,0,0"));
        state.set_saved_layout(Some(c""));
        assert_eq!(state.saved_layout(), Some(c""));
        state.set_saved_layout(None);
        assert!(state.saved_layout().is_none());
    }
}
