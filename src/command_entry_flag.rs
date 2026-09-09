//! Stable access to a command entry's source or target descriptor.

use crate::cmd::cmd_find_type;
use core::ffi::{c_char, c_int};

/// The option character and lookup rules for a command source or target.
pub trait CommandEntryFlag {
    /// Builds a command source or target descriptor.
    fn from_command_entry_flag(flag: c_char, find_type: cmd_find_type, find_flags: c_int) -> Self
    where
        Self: Sized;

    /// Returns the option character containing the target, or zero for the default target.
    fn command_entry_flag_character(&self) -> c_char;

    /// Returns the kind of object this descriptor finds.
    fn command_entry_flag_find_type(&self) -> cmd_find_type;

    /// Returns the flags controlling the lookup.
    fn command_entry_flag_find_flags(&self) -> c_int;
}

impl CommandEntryFlag for crate::cmd::cmd_entry_flag {
    fn from_command_entry_flag(flag: c_char, find_type: cmd_find_type, find_flags: c_int) -> Self {
        Self {
            flag,
            type_0: find_type,
            flags: find_flags,
        }
    }
    fn command_entry_flag_character(&self) -> c_char {
        self.flag
    }
    fn command_entry_flag_find_type(&self) -> cmd_find_type {
        self.type_0
    }
    fn command_entry_flag_find_flags(&self) -> c_int {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::CMD_FIND_WINDOW;
    use crate::cmd::cmd_entry_flag;

    #[test]
    fn const_descriptor_exposes_its_lookup_rules() {
        const FLAG: cmd_entry_flag = cmd_entry_flag::new(b't' as c_char, CMD_FIND_WINDOW, 7);
        assert_eq!(FLAG.command_entry_flag_character(), b't' as c_char);
        assert_eq!(FLAG.command_entry_flag_find_type(), CMD_FIND_WINDOW);
        assert_eq!(FLAG.command_entry_flag_find_flags(), 7);
    }
}
