//! Flags carried by a resolved command target.

use core::ffi::c_int;

/// Storage and mask operations for command-target lookup flags.
pub trait CommandFindFlagsState {
    /// Returns every lookup flag currently set.
    fn command_find_flags(&self) -> c_int;

    /// Replaces every lookup flag.
    fn set_command_find_flags(&mut self, flags: c_int);

    /// Reports whether any bit in `flags` is set.
    fn has_command_find_flags(&self, flags: c_int) -> bool {
        self.command_find_flags() & flags != 0
    }

    /// Adds every bit in `flags`.
    fn add_command_find_flags(&mut self, flags: c_int) {
        self.set_command_find_flags(self.command_find_flags() | flags);
    }

    /// Removes every bit in `flags`.
    fn remove_command_find_flags(&mut self, flags: c_int) {
        self.set_command_find_flags(self.command_find_flags() & !flags);
    }
}

/// The command-target flag storage used by hmux.
#[derive(Clone, Default)]
pub struct RustCommandFindFlagsState {
    flags: c_int,
}

impl RustCommandFindFlagsState {
    /// Builds flag storage with the supplied value.
    pub const fn new(flags: c_int) -> Self {
        Self { flags }
    }
}

impl CommandFindFlagsState for RustCommandFindFlagsState {
    fn command_find_flags(&self) -> c_int {
        self.flags
    }

    fn set_command_find_flags(&mut self, flags: c_int) {
        self.flags = flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_can_be_added_and_removed() {
        let mut state = RustCommandFindFlagsState::default();
        state.add_command_find_flags(0x3);
        assert!(state.has_command_find_flags(0x2));
        state.remove_command_find_flags(0x1);
        assert_eq!(state.command_find_flags(), 0x2);
    }
}
