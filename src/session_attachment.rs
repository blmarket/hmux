//! Attached-client count retained by a session.

use crate::types::u_int;

/// Storage for a session's attached-client count.
pub trait SessionAttachmentState {
    /// Returns the number of attached clients.
    fn session_attached(&self) -> u_int;

    /// Starts the attached-client count again.
    fn clear_session_attached(&mut self);

    /// Adds one attached client using tmux's wrapping counter semantics.
    fn add_session_attached(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attached_clients_can_be_counted_and_cleared() {
        let mut state = crate::session::session::default();
        assert_eq!(state.session_attached(), 0);
        state.add_session_attached();
        state.add_session_attached();
        assert_eq!(state.session_attached(), 2);
        state.clear_session_attached();
        assert_eq!(state.session_attached(), 0);
    }
}
