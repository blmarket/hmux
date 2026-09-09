//! Stable access to a terminal capability table entry.

use crate::types::tty_code_type;
use core::ffi::CStr;

/// The kind and terminfo name of one terminal capability.
pub trait TerminalCodeEntry {
    /// Builds a terminal capability table entry.
    fn from_terminal_code_entry(code_type: tty_code_type, name: &'static CStr) -> Self
    where
        Self: Sized;

    /// Returns whether the capability is a string, number, or flag.
    fn terminal_code_type(&self) -> tty_code_type;

    /// Returns the capability's terminfo name.
    fn terminal_code_name(&self) -> &CStr;
}

impl TerminalCodeEntry for crate::types::tty_term_code_entry {
    fn from_terminal_code_entry(code_type: tty_code_type, name: &'static CStr) -> Self {
        Self {
            type_0: code_type,
            name,
        }
    }

    fn terminal_code_type(&self) -> tty_code_type {
        self.type_0
    }

    fn terminal_code_name(&self) -> &CStr {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::tty_term_code_entry;

    #[test]
    fn const_entry_exposes_its_capability() {
        const ENTRY: tty_term_code_entry = tty_term_code_entry::new(2, c"cup");
        assert_eq!(ENTRY.terminal_code_type(), 2);
        assert_eq!(ENTRY.terminal_code_name(), c"cup");
    }
}
