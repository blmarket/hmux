//! Numeric identity retained by a pane.

/// Immutable numeric identity assigned when a pane is constructed.
pub trait PaneIdentity {
    /// Returns the pane id.
    fn pane_id(&self) -> u32;

}
