//! Numeric index retained while resolving a command target.

use core::ffi::c_int;

/// Storage for the index retained by command-target lookup.
pub trait CommandFindIndexState {
    /// Returns the retained index.
    fn command_find_index(&self) -> c_int;

    /// Replaces the retained index.
    fn set_command_find_index(&mut self, index: c_int);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_can_be_replaced() {
        let mut state = crate::types::cmd_find_state::default();
        state.set_command_find_index(-1);
        assert_eq!(state.command_find_index(), -1);
        state.set_command_find_index(c_int::MAX);
        assert_eq!(state.command_find_index(), c_int::MAX);
    }
}
