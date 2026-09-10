//! Foreground and background colours reported for a pane by control mode.

use core::ffi::c_int;

/// The optional foreground and background reported for a pane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaneControlColourPair {
    /// The reported foreground, if known.
    pub control_fg: Option<c_int>,
    /// The reported background, if known.
    pub control_bg: Option<c_int>,
}

/// Storage for a pane's control-client colour report.
pub trait PaneControlColours {
    /// Returns both reported colours.
    fn colours(&self) -> PaneControlColourPair;

    /// Replaces both reported colours.
    fn set_colours(&mut self, colours: PaneControlColourPair);

    /// Forgets both reported colours.
    fn clear(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_colours_are_replaced_and_cleared_together() {
        let mut state = crate::tests::test_fixtures::PaneAllocation::default();
        assert_eq!(state.colours(), PaneControlColourPair::default());
        state.set_colours(PaneControlColourPair {
            control_fg: Some(3),
            control_bg: Some(4),
        });
        assert_eq!(
            state.colours(),
            PaneControlColourPair {
                control_fg: Some(3),
                control_bg: Some(4),
            }
        );
        PaneControlColours::clear(&mut *state);
        assert_eq!(state.colours(), PaneControlColourPair::default());
    }
}
