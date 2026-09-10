//! Owned working directory retained by a session.

use core::ffi::CStr;

/// Storage for a session's optional working directory.
pub trait SessionDirectoryState {
    /// Returns the current working directory, if present.
    fn session_directory(&self) -> Option<&CStr>;

    /// Copies a replacement working directory or clears it.
    fn set_session_directory(&mut self, directory: Option<&CStr>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_can_be_replaced_and_cleared() {
        let mut state = crate::session::session::default();
        assert!(state.session_directory().is_none());
        state.set_session_directory(Some(c"/tmp/work"));
        assert_eq!(state.session_directory(), Some(c"/tmp/work"));
        state.set_session_directory(Some(c""));
        assert_eq!(state.session_directory(), Some(c""));
        state.set_session_directory(None);
        assert!(state.session_directory().is_none());
    }
}
