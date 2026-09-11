//! Portable state for one pane watched by a control client.

/// Pane identity, output cursors, and scheduling state for a control client.
pub trait ControlPaneState {
    /// Builds a pane state from all portable values.
    fn from_control_pane(
        pane_id: u32,
        written_position: usize,
        queued_position: usize,
        flags: i32,
        pending: bool,
    ) -> Self
    where
        Self: Sized;

    /// Returns the watched pane identifier.
    fn control_pane_id(&self) -> u32;

    /// Replaces the watched pane identifier.
    fn set_control_pane_id(&mut self, pane_id: u32);

    /// Returns the absolute output position already written to the client.
    fn control_pane_written_position(&self) -> usize;

    /// Replaces the absolute output position already written to the client.
    fn set_control_pane_written_position(&mut self, position: usize);

    /// Returns the absolute output position already represented by queued blocks.
    fn control_pane_queued_position(&self) -> usize;

    /// Replaces the absolute output position already represented by queued blocks.
    fn set_control_pane_queued_position(&mut self, position: usize);

    /// Returns the control-pane flags.
    fn control_pane_flags(&self) -> i32;

    /// Replaces the control-pane flags.
    fn set_control_pane_flags(&mut self, flags: i32);

    /// Returns whether this pane is on the pending-output list.
    fn control_pane_pending(&self) -> bool;

    /// Sets whether this pane is on the pending-output list.
    fn set_control_pane_pending(&mut self, pending: bool);
}

impl ControlPaneState for crate::control::control_pane {
    fn from_control_pane(
        pane_id: u32,
        written_position: usize,
        queued_position: usize,
        flags: i32,
        pending: bool,
    ) -> Self {
        Self {
            pane: pane_id,
            offset: crate::RustPaneOutputOffset::at(written_position),
            queued: crate::RustPaneOutputOffset::at(queued_position),
            flags,
            pending_flag: i32::from(pending),
            blocks: Vec::new(),
        }
    }
    fn control_pane_id(&self) -> u32 {
        self.pane
    }
    fn set_control_pane_id(&mut self, pane_id: u32) {
        self.pane = pane_id;
    }
    fn control_pane_written_position(&self) -> usize {
        self.offset.position()
    }
    fn set_control_pane_written_position(&mut self, position: usize) {
        self.offset.set_position(position);
    }
    fn control_pane_queued_position(&self) -> usize {
        self.queued.position()
    }
    fn set_control_pane_queued_position(&mut self, position: usize) {
        self.queued.set_position(position);
    }
    fn control_pane_flags(&self) -> i32 {
        self.flags
    }
    fn set_control_pane_flags(&mut self, flags: i32) {
        self.flags = flags;
    }
    fn control_pane_pending(&self) -> bool {
        self.pending_flag != 0
    }
    fn set_control_pane_pending(&mut self, pending: bool) {
        self.pending_flag = i32::from(pending);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::control_pane;

    #[test]
    fn pane_state_round_trips_and_updates() {
        let mut state = control_pane::from_control_pane(17, 23, 29, 3, false);
        assert_eq!(state.control_pane_id(), 17);
        assert_eq!(state.control_pane_written_position(), 23);
        assert_eq!(state.control_pane_queued_position(), 29);
        assert_eq!(state.control_pane_flags(), 3);
        assert!(!state.control_pane_pending());

        state.set_control_pane_id(31);
        state.set_control_pane_written_position(37);
        state.set_control_pane_queued_position(41);
        state.set_control_pane_flags(5);
        state.set_control_pane_pending(true);
        assert_eq!(state.control_pane_id(), 31);
        assert_eq!(state.control_pane_written_position(), 37);
        assert_eq!(state.control_pane_queued_position(), 41);
        assert_eq!(state.control_pane_flags(), 5);
        assert!(state.control_pane_pending());
    }
}
