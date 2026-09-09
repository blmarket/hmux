//! Window-link selection held by command lookup state.

/// The optional winlink index in a resolved command target.
pub trait CommandFindTargets {
    /// Builds command state with a window-link index and no selected pane.
    fn from_command_find_targets(winlink_index: Option<i32>) -> Self
    where
        Self: Sized;

    /// Returns the resolved winlink index.
    fn command_find_winlink_index(&self) -> Option<i32>;

    /// Replaces the resolved winlink index.
    fn set_command_find_winlink_index(&mut self, winlink_index: Option<i32>);
}

impl CommandFindTargets for crate::types::cmd_find_state {
    fn from_command_find_targets(winlink_index: Option<i32>) -> Self {
        Self {
            wl_idx: winlink_index,
            ..Self::default()
        }
    }

    fn command_find_winlink_index(&self) -> Option<i32> {
        self.wl_idx
    }

    fn set_command_find_winlink_index(&mut self, winlink_index: Option<i32>) {
        self.wl_idx = winlink_index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::cmd_find_state;

    #[test]
    fn target_identities_round_trip_and_clear() {
        let mut state = cmd_find_state::from_command_find_targets(Some(4));
        assert_eq!(state.command_find_winlink_index(), Some(4));
        state.set_command_find_winlink_index(None);
        assert_eq!(state.command_find_winlink_index(), None);
    }
}
