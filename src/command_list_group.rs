//! Current group identity retained by a command list.

use crate::types::u_int;

/// Storage for the group assigned to commands appended to a list.
pub trait CommandListGroupState {
    /// Returns the list's current group.
    fn command_list_group(&self) -> u_int;

    /// Replaces the list's current group.
    fn set_command_list_group(&mut self, group: u_int);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_can_be_replaced() {
        let mut state = crate::cmd::cmd_list {
            group: 0,
            list: None,
        };
        assert_eq!(state.command_list_group(), 0);
        state.set_command_list_group(77);
        assert_eq!(state.command_list_group(), 77);
    }
}
