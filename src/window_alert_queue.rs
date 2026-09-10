//! Membership state for the deferred window-alert queue.

/// Storage for whether a window is queued for deferred alert processing.
pub trait WindowAlertQueueState: Default {
    /// Returns whether the window is currently queued.
    fn alerts_are_queued(&self) -> bool;

    /// Marks the window queued and returns whether this was a new transition.
    fn queue_alerts(&mut self) -> bool;

    /// Marks the window no longer queued.
    fn clear_queued_alerts(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_transition_reports_only_new_membership() {
        let mut state = crate::types::window::default();
        assert!(!state.alerts_are_queued());
        assert!(state.queue_alerts());
        assert!(state.alerts_are_queued());
        assert!(!state.queue_alerts());
        state.clear_queued_alerts();
        assert!(!state.alerts_are_queued());
        assert!(state.queue_alerts());
    }
}
