//! Group identity retained by a command.

use crate::types::u_int;

/// Storage for the group a command belongs to.
pub trait CommandGroupState {
    /// Returns the command's group.
    fn command_group(&self) -> u_int;

    /// Moves the command into a group.
    fn set_command_group(&mut self, group: u_int);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_can_be_replaced() {
        let mut state = *crate::tests::test_fixtures::empty_cmd();
        assert_eq!(state.command_group(), 0);
        state.set_command_group(42);
        assert_eq!(state.command_group(), 42);
    }
}
