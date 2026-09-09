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

/// The window alert-queue storage used by hmux.
#[derive(Default)]
pub struct RustWindowAlertQueueState {
    queued: bool,
}

impl WindowAlertQueueState for RustWindowAlertQueueState {
    fn alerts_are_queued(&self) -> bool {
        self.queued
    }

    fn queue_alerts(&mut self) -> bool {
        let changed = !self.queued;
        self.queued = true;
        changed
    }

    fn clear_queued_alerts(&mut self) {
        self.queued = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_transition_reports_only_new_membership() {
        let mut state = RustWindowAlertQueueState::default();
        assert!(!state.alerts_are_queued());
        assert!(state.queue_alerts());
        assert!(state.alerts_are_queued());
        assert!(!state.queue_alerts());
        state.clear_queued_alerts();
        assert!(!state.alerts_are_queued());
        assert!(state.queue_alerts());
    }
}
