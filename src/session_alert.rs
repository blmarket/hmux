//! Alert-reporting state retained by a session.

/// Storage for whether a session alert has been reported.
pub trait SessionAlertState {
    /// Returns whether an alert has already been reported.
    fn session_alerted(&self) -> bool;

    /// Records or clears the reported-alert state.
    fn set_session_alerted(&mut self, alerted: bool);
}

/// The alert-reporting flag used by hmux.
#[derive(Default)]
pub struct RustSessionAlertState {
    alerted: bool,
}

impl SessionAlertState for RustSessionAlertState {
    fn session_alerted(&self) -> bool {
        self.alerted
    }

    fn set_session_alerted(&mut self, alerted: bool) {
        self.alerted = alerted;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_can_be_set_and_cleared() {
        let mut state = RustSessionAlertState::default();
        assert!(!state.session_alerted());
        state.set_session_alerted(true);
        assert!(state.session_alerted());
        state.set_session_alerted(false);
        assert!(!state.session_alerted());
    }
}
