//! Stable value-state capabilities of a resolved command target.

use crate::{CommandFindFlagsState, CommandFindIndexState, CommandFindTargets};

/// The stable, independently implementable state of command-target lookup.
///
/// Session, window, winlink, and pane observations remain server-graph
/// resources managed by the command finder.
pub trait CommandFind: CommandFindFlagsState + CommandFindIndexState + CommandFindTargets {}

impl<T> CommandFind for T where T: CommandFindFlagsState + CommandFindIndexState + CommandFindTargets
{}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_command_find<T: CommandFind>() {}

    #[test]
    fn resolved_target_implements_the_aggregate_contract() {
        assert_command_find::<crate::types::cmd_find_state>();
    }
}
