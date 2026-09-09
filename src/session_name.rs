//! Owned name retained by a session.

use core::ffi::CStr;
use std::ffi::CString;

/// Storage for a session's optional name.
pub trait SessionNameState {
    /// Returns the current name, if present.
    fn session_name(&self) -> Option<&CStr>;

    /// Copies the current name for storage beyond this borrow.
    fn session_name_owned(&self) -> Option<CString> {
        self.session_name().map(CStr::to_owned)
    }

    /// Copies a replacement name or clears it.
    fn set_session_name(&mut self, name: Option<&CStr>);
}

/// The session name storage used by hmux.
#[derive(Default)]
pub struct RustSessionNameState {
    name: Option<CString>,
}

impl SessionNameState for RustSessionNameState {
    fn session_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }

    fn set_session_name(&mut self, name: Option<&CStr>) {
        self.name = name.map(CStr::to_owned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_can_be_replaced_and_cleared() {
        let mut state = RustSessionNameState::default();
        assert!(state.session_name().is_none());
        state.set_session_name(Some(c"work"));
        assert_eq!(state.session_name(), Some(c"work"));
        state.set_session_name(Some(c""));
        assert_eq!(state.session_name(), Some(c""));
        state.set_session_name(None);
        assert!(state.session_name().is_none());
    }
}
