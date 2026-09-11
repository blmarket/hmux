//! Stable value-state capabilities of a parsed command.

use crate::args::arguments_trait::Arguments;
use crate::command_source::CommandSource;
use crate::types::u_int;
use core::ffi::c_int;

/// Shared access to a parsed command.
pub trait Command {
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

    /// Returns the command's group.
    fn command_group(&self) -> u_int;

    /// Moves the command into a group.
    fn set_command_group(&mut self, group: u_int);

    /// Returns the current source location.
    fn command_source(&self) -> CommandSource<'_>;

    /// Copies a replacement source location.
    fn set_command_source(&mut self, source: CommandSource<'_>);

    /// Returns the parser flags.
    fn command_parse_flags(&self) -> c_int;

    /// Replaces the parser flags.
    fn set_command_parse_flags(&mut self, flags: c_int);
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
