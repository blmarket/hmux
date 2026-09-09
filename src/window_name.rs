//! Owned name retained by a window.

use core::ffi::CStr;
use std::ffi::CString;

/// Storage for a window's optional name.
pub trait WindowNameState: Default {
    /// Returns the current name, if present.
    fn window_name(&self) -> Option<&CStr>;

    /// Copies a replacement name or clears it.
    fn set_window_name(&mut self, name: Option<&CStr>);
}

/// The window name storage used by hmux.
#[derive(Default)]
pub struct RustWindowNameState {
    name: Option<CString>,
}

impl WindowNameState for RustWindowNameState {
    fn window_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }

    fn set_window_name(&mut self, name: Option<&CStr>) {
        self.name = name.map(CStr::to_owned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_can_be_replaced_and_cleared() {
        let mut state = RustWindowNameState::default();
        assert!(state.window_name().is_none());
        state.set_window_name(Some(c"editor"));
        assert_eq!(state.window_name(), Some(c"editor"));
        state.set_window_name(Some(c""));
        assert_eq!(state.window_name(), Some(c""));
        state.set_window_name(None);
        assert!(state.window_name().is_none());
    }
}
