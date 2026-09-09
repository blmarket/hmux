//! Parser flags retained by a command.

use core::ffi::c_int;

/// Storage for a command's parser flags.
pub trait CommandParseFlagsState {
    /// Returns the parser flags.
    fn command_parse_flags(&self) -> c_int;

    /// Replaces the parser flags.
    fn set_command_parse_flags(&mut self, flags: c_int);
}

/// The command parser-flag storage used by hmux.
#[derive(Default)]
pub struct RustCommandParseFlagsState {
    flags: c_int,
}

impl CommandParseFlagsState for RustCommandParseFlagsState {
    fn command_parse_flags(&self) -> c_int {
        self.flags
    }

    fn set_command_parse_flags(&mut self, flags: c_int) {
        self.flags = flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_can_be_replaced() {
        let mut state = RustCommandParseFlagsState::default();
        assert_eq!(state.command_parse_flags(), 0);
        state.set_command_parse_flags(0x1234);
        assert_eq!(state.command_parse_flags(), 0x1234);
    }
}
