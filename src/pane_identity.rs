//! Numeric identity retained by a pane.

/// Storage for a pane's stable numeric identity.
pub trait PaneIdentity {
    /// Returns the pane id.
    fn pane_id(&self) -> u32;

    /// Replaces the pane id during construction.
    fn set_pane_id(&mut self, id: u32);
}
