//! Stable value-state capabilities of a command list.

use crate::types::u_int;

/// Shared access to a parsed command list.
pub trait CommandList {
    /// The command implementation stored in the list.
    type Command: crate::Command;

    /// Returns the number of commands in execution order.
    fn command_count(&self) -> usize;

    /// Returns a command by its execution-order index.
    fn command_at(&self, index: usize) -> Option<&Self::Command>;

    /// Returns a mutable command by its execution-order index.
    fn command_at_mut(&mut self, index: usize) -> Option<&mut Self::Command>;

    /// Returns the list's current group.
    fn command_list_group(&self) -> u_int;

    /// Replaces the list's current group.
    fn set_command_list_group(&mut self, group: u_int);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_command_list<T: CommandList>() {}

    #[test]
    fn server_command_list_implements_the_aggregate_contract() {
        assert_command_list::<crate::cmd::cmd_list>();
    }
}
