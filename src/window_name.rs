//! Owned name retained by a window.

use core::ffi::CStr;

/// Storage for a window's optional name.
pub trait WindowNameState: Default {
    /// Returns the current name, if present.
    fn window_name(&self) -> Option<&CStr>;

    /// Copies a replacement name or clears it.
    fn set_window_name(&mut self, name: Option<&CStr>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_can_be_replaced_and_cleared() {
        let mut state = crate::types::window::default();
        assert!(state.window_name().is_none());
        state.set_window_name(Some(c"editor"));
        assert_eq!(state.window_name(), Some(c"editor"));
        state.set_window_name(Some(c""));
        assert_eq!(state.window_name(), Some(c""));
        state.set_window_name(None);
        assert!(state.window_name().is_none());
    }
}
