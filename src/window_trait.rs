//! Stable value-state capabilities of a window.

use crate::{
    WindowAlertQueueState, WindowDimensionsState, WindowFillCharacterState,
    WindowLayoutSelectionState, WindowNameState, WindowSavedLayoutState, WindowScrollbarState,
    WindowTimestampState,
};

/// The stable, independently implementable state of a window.
///
/// Layout ownership, pane membership, options, event timers, and other server
/// graph resources remain context-specific engine concerns.
pub trait Window:
    WindowNameState
    + WindowTimestampState
    + WindowDimensionsState
    + WindowScrollbarState
    + WindowLayoutSelectionState
    + WindowSavedLayoutState
    + WindowFillCharacterState
    + WindowAlertQueueState
{
    /// Returns the window's stable numeric identity.
    fn window_id(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window_id<T: Window>(window: &T) -> u32 {
        window.window_id()
    }

    #[test]
    fn server_window_implements_the_aggregate_contract() {
        let mut window = crate::types::window::default();
        window.id = 42;
        assert_eq!(window_id(&window), 42);
    }
}
