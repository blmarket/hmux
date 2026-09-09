//! Foreground and background colours reported for a pane by control mode.

use core::ffi::c_int;

/// The optional foreground and background reported for a pane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaneControlColourPair {
    /// The reported foreground, if known.
    pub foreground: Option<c_int>,
    /// The reported background, if known.
    pub background: Option<c_int>,
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

/// The pane control-client colour storage used by hmux.
#[derive(Default)]
pub struct RustPaneControlColours {
    colours: PaneControlColourPair,
}

impl PaneControlColours for RustPaneControlColours {
    fn colours(&self) -> PaneControlColourPair {
        self.colours
    }

    fn set_colours(&mut self, colours: PaneControlColourPair) {
        self.colours = colours;
    }

    fn clear(&mut self) {
        self.colours = PaneControlColourPair::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_colours_are_replaced_and_cleared_together() {
        let mut state = RustPaneControlColours::default();
        assert_eq!(state.colours(), PaneControlColourPair::default());
        state.set_colours(PaneControlColourPair {
            foreground: Some(3),
            background: Some(4),
        });
        assert_eq!(
            state.colours(),
            PaneControlColourPair {
                foreground: Some(3),
                background: Some(4),
            }
        );
        state.clear();
        assert_eq!(state.colours(), PaneControlColourPair::default());
    }
}
