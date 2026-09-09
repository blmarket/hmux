//! Serialized layout retained for restoring a window's previous arrangement.

use core::ffi::CStr;
use std::ffi::CString;

/// Storage for a window's optional serialized saved layout.
pub trait WindowSavedLayoutState: Default {
    /// Returns the saved layout text, if present.
    fn saved_layout(&self) -> Option<&CStr>;

    /// Copies a replacement layout or clears the saved layout.
    fn set_saved_layout(&mut self, layout: Option<&CStr>);
}

/// The window saved-layout storage used by hmux.
#[derive(Default)]
pub struct RustWindowSavedLayoutState {
    layout: Option<CString>,
}

impl WindowSavedLayoutState for RustWindowSavedLayoutState {
    fn saved_layout(&self) -> Option<&CStr> {
        self.layout.as_deref()
    }

    fn set_saved_layout(&mut self, layout: Option<&CStr>) {
        self.layout = layout.map(CStr::to_owned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_can_be_replaced_and_cleared() {
        let mut state = RustWindowSavedLayoutState::default();
        assert!(state.saved_layout().is_none());
        state.set_saved_layout(Some(c"80x24,0,0"));
        assert_eq!(state.saved_layout(), Some(c"80x24,0,0"));
        state.set_saved_layout(Some(c""));
        assert_eq!(state.saved_layout(), Some(c""));
        state.set_saved_layout(None);
        assert!(state.saved_layout().is_none());
    }
}
