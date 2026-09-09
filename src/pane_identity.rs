//! Numeric identity retained by a pane.

/// Storage for a pane's stable numeric identity.
pub trait PaneIdentity {
    /// Returns the pane id.
    fn pane_id(&self) -> u32;

    /// Replaces the pane id during construction.
    fn set_pane_id(&mut self, id: u32);
}

/// The pane identity storage used by hmux.
#[derive(Default)]
pub struct RustPaneIdentity {
    id: u32,
}

impl PaneIdentity for RustPaneIdentity {
    fn pane_id(&self) -> u32 {
        self.id
    }

    fn set_pane_id(&mut self, id: u32) {
        self.id = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_can_be_assigned_and_replaced() {
        let mut identity = RustPaneIdentity::default();
        assert_eq!(identity.pane_id(), 0);
        identity.set_pane_id(42);
        assert_eq!(identity.pane_id(), 42);
        identity.set_pane_id(u32::MAX);
        assert_eq!(identity.pane_id(), u32::MAX);
    }
}
