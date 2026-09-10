//! Cached status-line placement retained by a session.

use crate::types::u_int;
use core::ffi::c_int;

/// The cached placement and height of a session's status line.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SessionStatus {
    pub position: c_int,
    pub lines: u_int,
}

/// Storage for a session's cached status-line state.
pub trait SessionStatusState {
    /// Returns the cached status-line state.
    fn session_status(&self) -> SessionStatus;

    /// Replaces the cached status-line state together.
    fn set_session_status(&mut self, status: SessionStatus);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_and_height_are_replaced_together() {
        let mut state = crate::session::session::default();
        assert_eq!(state.session_status(), SessionStatus::default());
        state.set_session_status(SessionStatus {
            position: -1,
            lines: 2,
        });
        assert_eq!(
            state.session_status(),
            SessionStatus {
                position: -1,
                lines: 2,
            }
        );
    }
}
