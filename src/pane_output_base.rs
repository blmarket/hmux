//! Absolute position of the front of a pane's retained output.

/// Storage for a pane output buffer's absolute base position.
pub trait PaneOutputBaseState {
    /// Returns the absolute position at the front of retained output.
    fn output_base(&self) -> usize;

    /// Replaces the absolute base position.
    fn set_output_base(&mut self, position: usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_is_zero_then_replaceable() {
        let mut state = crate::tests::test_fixtures::PaneAllocation::default();
        assert_eq!(state.output_base(), 0);
        state.set_output_base(42);
        assert_eq!(state.output_base(), 42);
        state.set_output_base(usize::MAX);
        assert_eq!(state.output_base(), usize::MAX);
    }
}
