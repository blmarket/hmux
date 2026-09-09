//! Stable value-state capabilities of a parsed command.

use crate::{Arguments, CommandGroupState, CommandParseFlagsState, CommandSourceState};

/// Shared access to a parsed command.
pub trait Command: CommandGroupState + CommandSourceState + CommandParseFlagsState {
    /// The parsed-argument implementation carried by the command.
    type Arguments: Arguments;

    /// The executable registration selected by the parser.
    type Entry: crate::CommandEntry;

    /// Returns the command definition selected by the parser.
    fn command_entry(&self) -> &'static Self::Entry;

    /// Returns the parsed arguments while the command still carries them.
    fn command_arguments(&self) -> Option<&Self::Arguments>;

    /// Returns mutable parsed arguments while the command still carries them.
    fn command_arguments_mut(&mut self) -> Option<&mut Self::Arguments>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_command<T: Command>() {}

    #[test]
    fn server_command_implements_the_aggregate_contract() {
        assert_command::<crate::cmd::cmd>();
    }
}
