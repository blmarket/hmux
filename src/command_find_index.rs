//! Numeric index retained while resolving a command target.

use core::ffi::c_int;

/// Storage for the index retained by command-target lookup.
pub trait CommandFindIndexState {
    /// Returns the retained index.
    fn command_find_index(&self) -> c_int;

    /// Replaces the retained index.
    fn set_command_find_index(&mut self, index: c_int);
}

/// The command-target index storage used by hmux.
#[derive(Clone, Default)]
pub struct RustCommandFindIndexState {
    index: c_int,
}

impl RustCommandFindIndexState {
    /// Builds index storage with the supplied value.
    pub const fn new(index: c_int) -> Self {
        Self { index }
    }
}

impl CommandFindIndexState for RustCommandFindIndexState {
    fn command_find_index(&self) -> c_int {
        self.index
    }

    fn set_command_find_index(&mut self, index: c_int) {
        self.index = index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_can_be_replaced() {
        let mut state = RustCommandFindIndexState::default();
        state.set_command_find_index(-1);
        assert_eq!(state.command_find_index(), -1);
        state.set_command_find_index(c_int::MAX);
        assert_eq!(state.command_find_index(), c_int::MAX);
    }
}
