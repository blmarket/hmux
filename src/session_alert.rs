//! Alert-reporting state retained by a session.

/// Storage for whether a session alert has been reported.
pub trait SessionAlertState {
    /// Returns whether an alert has already been reported.
    fn session_alerted(&self) -> bool;

    /// Records or clears the reported-alert state.
    fn set_session_alerted(&mut self, alerted: bool);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_can_be_set_and_cleared() {
        let mut state = crate::session::session::default();
        assert!(!state.session_alerted());
        state.set_session_alerted(true);
        assert!(state.session_alerted());
        state.set_session_alerted(false);
        assert!(!state.session_alerted());
    }
}
