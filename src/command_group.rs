//! Group identity retained by a command.

use crate::types::u_int;

/// Storage for the group a command belongs to.
pub trait CommandGroupState {
    /// Returns the command's group.
    fn command_group(&self) -> u_int;

    /// Moves the command into a group.
    fn set_command_group(&mut self, group: u_int);
}

/// The command-group storage used by hmux.
#[derive(Default)]
pub struct RustCommandGroupState {
    group: u_int,
}

impl CommandGroupState for RustCommandGroupState {
    fn command_group(&self) -> u_int {
        self.group
    }

    fn set_command_group(&mut self, group: u_int) {
        self.group = group;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_can_be_replaced() {
        let mut state = RustCommandGroupState::default();
        assert_eq!(state.command_group(), 0);
        state.set_command_group(42);
        assert_eq!(state.command_group(), 42);
    }
}
